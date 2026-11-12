# tetrapong

> 🚧 **WORK IN PROGRESS — this project is not finished.**
> Expect missing features (there is no game-over state yet), placeholder behaviour, and
> breaking changes at any time.

A four-paddle take on Pong, built in Rust with [Bevy](https://bevyengine.org/) 0.20.

Instead of the usual two paddles, the arena has one on each side — left, right, top, and
bottom. The ball bounces around the inside of the box, and each paddle defends its own
wall. Only the bottom paddle is currently controlled by a human; the other three are driven
by simple computer logic.

## Status

This is an early prototype. The core loop now includes scoring, a serve delay, and paddle
elimination, but there is still no win/lose condition.

**Working**

- Window-relative arena that resizes with the window
- Four paddles, each with its own lives counter and HUD label
- Human control of the bottom paddle; computer control of the other three
- Ball movement and reflection off walls and paddles
- Scoring: when the ball hits a wall, that side loses a life
- Ball reset and a 2-second serve pause before relaunching in a random direction
- Paddle elimination at 0 lives, with the HUD showing 0

**Not implemented yet**

- Game over / win conditions, or restarting after the game ends
- Behaviour for an eliminated side: its wall still emits `LoseLife`, but no paddle remains
  to lose a life, so the ball keeps bouncing off that wall
- Improved ball physics (speed limits, contact-angle influence, avoiding repetitive paths)
- Power-ups (`ColliderType::PowerUp` exists but is unused and currently panics via `todo!()`)
- Menus, pause menu, audio, and general polish
- Human control for paddles other than the bottom one
- Automated tests or CI

## How it plays

1. The ball starts at the centre with a random direction.
2. Each of the four paddles starts with 3 lives.
3. If the ball reaches a wall, that wall's side loses a life.
4. The ball resets to the centre, waits 2 seconds, then serves in a random direction.
5. At 0 lives, the paddle is removed and its HUD label shows 0.

There is no winner yet — the game keeps running after paddles are eliminated.

## Controls

| Action             | Key        |
| ------------------ | ---------- |
| Move bottom paddle | `←` / `→` |
| Move top paddle\*  | `A` / `D`  |

\* The top paddle is currently computer-controlled, so the `A`/`D` binding is not reachable
in normal play. It is kept in the input code for future local multiplayer.

Close the window to quit.

## Requirements

- Rust stable with edition 2024 support (Rust 1.85+)
- Linux, macOS, or Windows with a Vulkan/DX12/Metal-capable GPU
- On Linux, the usual Bevy system libraries (ALSA, X11/Wayland, Vulkan, udev)

The easiest way to get a working toolchain and system libraries is the included Nix flake.

## Running

### With Nix (recommended)

```sh
nix develop
cargo run
```

If you use [direnv](https://direnv.net/), the provided `.envrc` loads the dev shell
automatically:

```sh
direnv allow
cargo run
```

### Without Nix

Install the Rust toolchain and Bevy's [Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
for your distribution, then:

```sh
cargo run
```

For a faster build of the game itself, use the release profile:

```sh
cargo run --release
```

## Project layout

```text
.
├── Cargo.toml            # Crate metadata and dependencies
├── LICENSE               # MIT license
├── flake.nix             # Nix dev shell with Rust and Bevy system libraries
└── src
    ├── main.rs           # App setup: plugins, camera, shared constants
    └── plugins
        ├── ball.rs       # Ball entity, sizing, serve-pause timer and relaunch
        ├── paddle.rs     # Paddles, input/AI, lives, life-loss and elimination
        ├── physics.rs    # Velocity, colliders, collisions, emits LoseLife on wall hits
        ├── walls.rs      # Arena walls and their PaddleLocation ownership
        └── hud.rs        # On-screen lives displays and eliminated state
```

The game is organised as one Bevy plugin per concern, registered in `src/main.rs`. Arena
dimensions are expressed as percentages of the window's smaller dimension so the game
scales with the window.

Gameplay flows through messages: a wall collision emits `LoseLife`, `paddle.rs` decrements
lives and resets the ball, and `PaddleEliminated` tells the HUD when a paddle has been
removed.

## Roadmap

Rough order of intended work:

1. Add game-over/winner detection and a way to restart.
2. Decide what happens when a side is eliminated (stop scoring it, remove its goal, or end
   the game).
3. Improve ball physics (speed limits, angles based on paddle contact).
4. Let players choose which side(s) are human-controlled.
5. Add power-ups.
6. Menus, audio, and visual polish.

## License

Released under the [MIT License](LICENSE).

Copyright (c) 2026 Juan Vásquez.
