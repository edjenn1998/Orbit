# Changelog

## v0.1.0 — 2026-10-30

First public release of Orbit, a Rust desktop orbital-survival game.

### Features

- Indestructible central primary with mutual gravity between all launched objects.
- Three payload classes: light (24 M), medium (72 M), and heavy (180 M).
- Movable launcher, mouse aiming, adjustable launch speed, and circular-orbit assist.
- Destructive collisions, orbital trails, impact particles, and a trajectory guide.
- Scoring for full, bound revolutions and a session-best score.
- Pause, restart, help, and 1x / 2x / 4x simulation speed.
- Fixed 120 Hz gravity simulation with swept collision detection.

### Linux download

Extract `orbit-v0.1.0-linux-x86_64.tar.gz` and run `./orbit`.
The archive includes the executable, README, and license. Rust is not needed to
run the prebuilt executable. Requires Linux x86-64 with a compatible glibc,
X11 (or XWayland), and OpenGL.

For other systems, build from source with a current stable Rust toolchain using
`cargo run --release`.

Quick start: press **T** for circular-orbit assist, then click to launch.
Use **WASD** to move the launcher, **1–3** to select mass, and **H** for help.

### Verification

Six physics tests cover stable orbits, mass-dependent mutual attraction,
close-encounter path deflection, collisions, and escapes. The release also passes
formatting and Clippy checks.
