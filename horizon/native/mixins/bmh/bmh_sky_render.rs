// Area sky, the part that lives in horizon-client-render: the hooks there have no `Game`.
// The module is `pub`, so the game side can reach it as `horizon_client_render::bmh_sky_render`.
// Owner: the sky agent. Log: docs/horizon/log/sky.md.
//
// The game side (`bmh_sky::on_frame`, once a frame, before the drawing) computes the triangles of the star
// reading with `bmh_logic::stars` and hands them to `show`. This module keeps them in `FRAME` and draws them in
// `world_drawn`, after the world: one pass of its own on the frame's colour, with the blend of vanilla's stars
// (source alpha, one: light adds up), no depth write, back faces culled (StarReadingRenderer.java:76-88). A pixel
// draws only where the world's depth is 0, the sky, so terrain, entities and the like hide the lines as they
// hide the stars. With the mod off, nothing calls `show`, so `FRAME` stays empty and nothing draws.
//
// The pipeline builds inside wgpu error scopes, as Horizon builds its own (gfx/animate.rs:399-428). An error there
// would otherwise be uncaptured and stop the game. On an error, the module logs one line and draws nothing for the
// rest of the session (`BROKEN`).

use crate::gfx::native_render::{RenderNeeds, WorldDrawn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Floats per vertex: x, y, z, red, green, blue, alpha (`bmh_logic::stars::STRIDE`, StarTrace.java:20). This
/// crate cannot use `bmh_logic`.
const STRIDE: usize = 7;

/// One frame of the star reading, from the game side.
pub struct SkyFrame {
    /// The triangles in the star field's own frame (before the sky turns it), STRIDE floats a vertex.
    pub vertices: Vec<f32>,
    /// The star angle in degrees: the sky turns the field by `Ry(-90) * Rx(angle)`, as Horizon turns its own
    /// stars (horizon-client game.rs:12232-12235).
    pub star_angle: f32,
}

/// The frame to draw. `show` puts it, `world_drawn` takes it.
static FRAME: Mutex<Option<SkyFrame>> = Mutex::new(None);
/// The pipeline and the buffers, built once for a device and a colour format.
static GPU: Mutex<Option<Gpu>> = Mutex::new(None);
/// A build or a bind group failed: the reading draws no more in this session.
static BROKEN: AtomicBool = AtomicBool::new(false);

struct Gpu {
    device: wgpu::Device,
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    vertices: wgpu::Buffer,
}

/// The uniform of the pass: the sky's clip matrix, then the frame size and the depth size in pixels.
const UNIFORM_SIZE: u64 = 80;

const SHADER: &str = "
struct Sky {
    clip_from_sky: mat4x4<f32>,
    // x, y: the size of the frame; z, w: the size of the depth.
    sizes: vec4<f32>,
}
@group(0) @binding(0) var<uniform> sky: Sky;
// The depth through an unfilterable float binding, as Horizon reads it (gfx/world_post.rs:158-160,
// gfx/pack_post.wgsl:14): the texel is the same, and the OpenGL backend can read it.
@group(0) @binding(1) var depth: texture_2d<f32>;

struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> Out {
    var out: Out;
    out.position = sky.clip_from_sky * vec4(position, 1.0);
    out.color = color;
    return out;
}

// Only the sky: the world's depth is reversed, 0 where nothing was drawn. The colour is premultiplied, so the
// blend (one, one) is vanilla's (source alpha, one). The alpha of the frame is HDR output's mark (hdr.wgsl): a
// line pixel writes 0.175, the bright tier, as Horizon's stars do (gfx/stars.wgsl:16-19). A pixel that adds less
// than half a step of an 8-bit frame changes nothing there, so it draws nothing and keeps the sky's mark.
@fragment fn fs(frag: Out) -> @location(0) vec4<f32> {
    let limit = vec2<i32>(sky.sizes.zw) - vec2(1);
    let texel = clamp(vec2<i32>(frag.position.xy * sky.sizes.zw / sky.sizes.xy), vec2(0), limit);
    if textureLoad(depth, texel, 0).r > 0.0 {
        discard;
    }
    let light = frag.color.rgb * frag.color.a;
    if max(light.r, max(light.g, light.b)) < 0.5 / 255.0 {
        discard;
    }
    return vec4(light, 0.175);
}
";

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The game side gives the frame to draw, or None to draw nothing. Called once a frame.
pub fn show(frame: Option<SkyFrame>) {
    *lock(&FRAME) = frame;
}

// V1: ask for the world depth while a reading shows.
pub(crate) fn on_render_needs(needs: &mut RenderNeeds) {
    if lock(&FRAME).is_some() {
        needs.world_depth = true;
    }
}

// V1: draw the star reading in the sky with the world depth.
pub(crate) fn on_world_drawn(world: &mut WorldDrawn<'_>) {
    let frame = lock(&FRAME).take();
    let mut gpu = lock(&GPU);
    // Horizon made a new device: release the old one, its pipeline and its buffers now.
    if gpu.as_ref().is_some_and(|g| g.device != *world.device) {
        *gpu = None;
    }
    let Some(frame) = frame else {
        return;
    };
    let count = frame.vertices.len() / STRIDE;
    if count < 3 || world.depth_size[0] == 0 || world.depth_size[1] == 0 || BROKEN.load(Ordering::Relaxed) {
        return;
    }
    if gpu.as_ref().is_none_or(|g| g.format != world.color_format) {
        *gpu = build(world.device, world.color_format);
    }
    let Some(gpu) = gpu.as_mut() else {
        return;
    };
    // Whole vertices only: three to a triangle.
    let floats = (count - count % 3) * STRIDE;
    let Some(data) = frame.vertices.get(..floats) else {
        return;
    };
    let bytes: &[u8] = bytemuck::cast_slice(data);
    let needed = bytes.len() as u64;
    if gpu.vertices.size() < needed {
        gpu.vertices = vertex_buffer(world.device, needed.next_power_of_two());
    }
    world.queue.write_buffer(&gpu.vertices, 0, bytes);

    let turn = glam::Mat4::from_rotation_y((-90.0_f32).to_radians())
        * glam::Mat4::from_rotation_x(frame.star_angle.to_radians());
    let mut uniform = [0.0_f32; 20];
    uniform[..16].copy_from_slice(&(world.view_proj * turn).to_cols_array());
    uniform[16..].copy_from_slice(&[
        world.size[0] as f32,
        world.size[1] as f32,
        world.depth_size[0] as f32,
        world.depth_size[1] as f32,
    ]);
    world.queue.write_buffer(&gpu.uniform, 0, bytemuck::cast_slice(&uniform));

    // The depth view comes from Horizon each frame: a view that does not fit the layout fails here, in the scope.
    let checks = Checks::push(world.device);
    let group = world.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("BMH star reading"),
        layout: &gpu.layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: gpu.uniform.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(world.depth) },
        ],
    });
    if checks.failed("bind group") {
        return;
    }
    let mut pass = world.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("BMH star reading"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: world.color_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
        })],
        ..Default::default()
    });
    pass.set_pipeline(&gpu.pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.set_vertex_buffer(0, gpu.vertices.slice(..needed));
    pass.draw(0..(floats / STRIDE) as u32, 0..1);
}

fn vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("BMH star reading vertices"),
        size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The Validation and Internal error scopes around a build, as gfx/animate.rs:408-422 pushes and pops them.
struct Checks {
    validation: wgpu::ErrorScopeGuard,
    internal: wgpu::ErrorScopeGuard,
}

impl Checks {
    fn push(device: &wgpu::Device) -> Self {
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        Self { validation, internal }
    }

    /// Pops both scopes, the last pushed first. On an error, logs one line and stops the drawing for the session.
    fn failed(self, what: &str) -> bool {
        let internal = pollster::block_on(self.internal.pop());
        let Some(error) = internal.or(pollster::block_on(self.validation.pop())) else {
            return false;
        };
        if !BROKEN.swap(true, Ordering::Relaxed) {
            log::warn!(
                "[BalancedMcHardcoreMain] The star reading does not draw in this session: {what} failed ({error})"
            );
        }
        true
    }
}

/// The pipeline: vanilla's star blend, no depth attachment (the shader tests the depth), back faces culled. None
/// when the device refuses the shader or the pipeline.
fn build(device: &wgpu::Device, format: wgpu::TextureFormat) -> Option<Gpu> {
    let checks = Checks::push(device);
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("BMH star reading"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("BMH star reading"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("BMH star reading"),
        bind_group_layouts: &[Some(&layout)],
        ..Default::default()
    });
    let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("BMH star reading"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: (STRIDE * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })],
        },
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    // The HDR mark of a line pixel replaces the sky's.
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("BMH star reading sky"),
        size: UNIFORM_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let vertices = vertex_buffer(device, 64 * 1024);
    if checks.failed("pipeline build") {
        return None;
    }
    Some(Gpu { device: device.clone(), format, pipeline, layout, uniform, vertices })
}
