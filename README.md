# Orbit

A Rust desktop gravity game. Build a busy orbital system around an indestructible
primary, then see how many of your objects survive each other's gravity.

![Orbit gameplay showing the primary, launched objects, orbital trails, and launcher](Screenshots/Screenshot%20From%202026-10-30%2018-31-39.png)

*Gameplay screenshot; the current version uses stronger projectile masses.*

## Run

### Download the Linux release

Download `orbit-v0.1.0-linux-x86_64.tar.gz` from the
[releases page](https://github.com/edjenn1998/Orbit/releases/latest), then:

```sh
tar -xzf orbit-v0.1.0-linux-x86_64.tar.gz
./orbit
```

The prebuilt executable is for Linux x86-64 with a compatible glibc runtime,
X11 (or XWayland), and OpenGL. It does not require Rust to run.

### Build from source

With a current stable Rust toolchain installed:

```sh
git clone https://github.com/edjenn1998/Orbit.git
cd Orbit
cargo run --release
```

Or use `./run.sh`, which also finds Rust installed under `~/.cargo/bin`.
The first build downloads Macroquad and its dependencies. Later launches can run
`./target/release/orbit` directly. A graphical desktop and OpenGL are required;
there are no external assets or audio-library dependencies. On Debian/Ubuntu,
if the compiler or X11/OpenGL runtime libraries are missing:

```sh
sudo apt install build-essential libx11-6 libxi6 libgl1 libxcursor1
```

## Play

The launcher starts below the primary. **Press T and click** to launch your first
object with circular-orbit assist. Move the launcher to a different radius before
launching again. Try 4x time with Tab to watch complete revolutions sooner.
Disable assist to experiment with your own directions and launch speeds.

| Control | Action |
| --- | --- |
| WASD / arrow keys | Move launcher |
| Shift + movement | Move faster |
| Right-click in the playfield | Place launcher |
| Mouse | Aim toward cursor in manual mode |
| Left-click / Space | Launch one object |
| 1 / 2 / 3 | Select light / medium / heavy mass |
| Wheel / + / - | Adjust manual speed |
| T | Toggle circular-orbit assist |
| C | Reverse assisted orbit direction |
| P / Esc | Pause / resume |
| Tab | Cycle 1x / 2x / 4x simulation speed |
| V | Show / hide trails |
| R | Restart; retain the best score for this session |
| H / F1 | Open / close help (suspends simulation) |

### Rules and scoring

- The stationary primary has **2,000 mass units** and cannot be destroyed.
- Player objects have **24, 72, or 180 mass units**, with increasing physical radii.
  Even light objects visibly perturb nearby neighbors; heavy objects can strongly
  disrupt an orbital lane. At a 250-unit orbit, a light object 50 units away adds
  roughly one third as much acceleration as the primary.
- All launched objects attract each other as well as falling toward the primary.
  A heavier payload pulls more strongly on its neighbors. The launcher is a
  placement tool and does not participate in gravity or collisions.
- Touching the primary destroys a player object. Touching another player object
  destroys both; collision detection also checks movement between physics steps.
- An object counts as **in orbit** after completing a net full revolution while
  gravitationally bound to the primary. A thin ring marks scored objects. Losing
  bound status resets its orbital progress. Destroyed and escaped objects no
  longer contribute to the score.
- **Session best** is the highest number of simultaneously orbiting objects;
  it persists through restarts, until the application closes.
- Objects farther than **1,800 world units** from the primary are removed as
  escaped. There may be up to **180 live objects** at once. Offscreen objects
  continue simulating until they collide or escape.
- There is no ammunition limit or timer. The challenge is surviving a crowded sky.

### Physics

The game uses pairwise Newtonian attraction, a small Plummer softening radius,
and fixed 120 Hz kick-drift-kick integration (velocity Verlet). The primary is
fixed at the origin as a gameplay constraint. The dotted preview and circular
assist account **only for the primary**, so they are useful guides, not promises
once the sky is crowded. Increasing time speed runs more fixed simulation steps.
Pausing allows launcher placement and settings changes, but not firing.

## Development

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

`src/physics.rs` contains the simulation and headless physics tests;
`src/main.rs` contains input, rendering, and the interface. Built with
[Macroquad](https://macroquad.rs/). No assets need to be downloaded separately.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
