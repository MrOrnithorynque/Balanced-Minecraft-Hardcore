use crate::logic::moon as logic;
use mod_api::{Hand, ParticleOptions, PayloadId, PlayerRef, Registrar, ServerHost, ToClient};
use std::sync::Mutex;

#[derive(mod_api::PayloadCodec)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    fn new([x, y, z]: [f64; 3]) -> Self {
        Self { x, y, z }
    }

    fn coordinates(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(mod_api::PayloadCodec)]
enum Particle {
    Rune { at: Point, offset: Point },
    Puff { at: Point },
}

static CHANNEL: Mutex<Option<ToClient<Particle>>> = Mutex::new(None);

pub(super) fn register(registrar: &mut dyn Registrar) {
    let enchant = registrar.particle_key("minecraft:enchant");
    let poof = registrar.particle_key("minecraft:poof");
    let token = registrar
        .payload_handlers()
        .registrar("1")
        .play_to_client_typed(
            PayloadId("bmh:ritual_particles".into()),
            move |particle, context| {
                let client = context
                    .client_host()
                    .map_err(|error| format!("{error:?}"))?;
                match particle {
                    Particle::Rune { at, offset } => {
                        client.add_particle(
                            &ParticleOptions::simple(enchant),
                            at.coordinates(),
                            offset.coordinates(),
                        );
                    }
                    Particle::Puff { at } => {
                        let options = ParticleOptions::simple(poof);
                        let at = at.coordinates();
                        let mut state = super::server();
                        let particles: Vec<_> = (0..logic::PUFF_COUNT)
                            .map(|_| {
                                let start =
                                    at.map(|axis| axis + state.rng.gaussian() * logic::PUFF_SPREAD);
                                let velocity =
                                    [0.0; 3].map(|_| state.rng.gaussian() * logic::PUFF_SPEED);
                                (start, velocity)
                            })
                            .collect();
                        drop(state);
                        for (start, velocity) in particles {
                            client.add_particle(&options, start, velocity);
                        }
                    }
                }
                Ok(())
            },
        );
    *CHANNEL.lock().unwrap_or_else(|error| error.into_inner()) = Some(token);
}

pub(super) fn particles(host: &mut dyn ServerHost, owner: PlayerRef, hand: Hand, puff: bool) {
    let Some(channel) = CHANNEL
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
    else {
        return;
    };
    let (Some(eye), Some([yaw, pitch]), Some(level)) = (
        owner.entity().eye_position(host),
        owner.entity().rotation(host),
        owner.level(host),
    ) else {
        return;
    };
    let raised = logic::raised_totem(eye, yaw, pitch, hand == Hand::Main);
    for viewer in host.players().server_players() {
        if viewer.level(host) != Some(level) {
            continue;
        }
        let (Some(position), Some(camera)) = (
            viewer.entity().position(host),
            viewer.entity().eye_position(host),
        ) else {
            continue;
        };
        let at = if viewer == owner {
            if puff {
                logic::puff_position(eye, yaw, pitch)
            } else {
                logic::first_person_totem(eye, yaw, pitch)
            }
        } else {
            raised
        };
        if logic::distance_squared(position, at) > logic::PARTICLE_RANGE.powi(2) {
            continue;
        }
        let particle = if puff {
            Particle::Puff { at: Point::new(at) }
        } else {
            let rune = logic::rune(at, camera, &mut super::server().rng);
            Particle::Rune {
                at: Point::new(rune.at),
                offset: Point::new(rune.offset),
            }
        };
        let _ = channel.send(host, viewer, &particle);
    }
}
