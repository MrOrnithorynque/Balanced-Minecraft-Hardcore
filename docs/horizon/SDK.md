# Horizon SDK package

The SDK package uses native Horizon hooks and the same item IDs and resources as the original port.
It targets Minecraft 26.3 and Horizon Nightly `2b77287de099c9316278ba7ac4d83590fdb7ef30`.
The manifest in `horizon/sdk/horizon-mod.toml` contains the build pins.

## Build the package

Use `cargo-horizon-mod` from the pinned Horizon build and Rust 1.99.1.

```sh
python3 horizon/tools/package-sdk.py --checker /path/to/cargo-horizon-mod
```

This writes `horizon/build/bmh-1.9.0-horizon-26.3.hmod`.
It copies the shared Rust logic, assets, and data into a temporary project.
The Horizon tool checks the package before the script installs it in the output folder.

For a development build, stage the same project in a new folder and build it with the launcher.

```sh
python3 horizon/tools/package-sdk.py --stage /path/to/bmh-sdk-project
horizon-launcher native-build /path/to/bmh-sdk-project \
  --version nightly-2026-11-22-2b77287 --trust-native-code
```

Prepare the pinned Nightly's resources before the native build.
The launcher builds the mod against the published SDK and installs a separate version.
Use this version for BMH on the dedicated server and on each client.

## Test on a dedicated server

Give the test script the launcher's staged version folder and the prepared resources of its base Nightly.

```sh
python3 horizon/tools/test-dedicated.py --multiplayer \
  --launcher /path/to/horizon-launcher \
  --bin-dir /path/to/launcher/versions/native-2b77287-BUILD \
  --resources /path/to/prepared/resources/26.3 \
  --package horizon/build/bmh-1.9.0-horizon-26.3.hmod \
  --out horizon/build/test-results
```

The test starts a dedicated server on loopback with two offscreen Horizon clients.
It checks permissions, a totem in a pocket, per-player amulet state, and PvP.
It writes the server and client logs and the result of each check.

## Current differences

- The SDK checks container clicks and quick moves. Hopper and dropper transfers can bypass the menu slot rule.
- A pocket save uses Horizon's vanilla totem sprite during the activation animation.
- The journey draws over the death screen and the HUD. Horizon's file loading and resource reload overlays use a different hook.
- The compass uses the local player's view. The constellation uses one color per triangle.
- The port reads the Horizon world's existing BMH rules file. Direct import of Java's saved rule tags and the world creation toggle still need a port.

## Publish

Follow the [Horizon publishing guide](https://playhorizon.dev/docs/publish-a-mod) for the SDK package.
Attach the checked `.hmod` to a GitHub release from the public BMH source repository.
Open a pull request to `horizon-rust/horizon-mods` with the asset link.
Add the version, filename, and SHA-256 to `mods/bmh.toml` as a `native-mod`.
