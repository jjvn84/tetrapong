# tetrapong

> 🚧 **WORK IN PROGRESS — this project is not finished and is not playable end-to-end yet.**
> Expect missing features, placeholder behaviour, and breaking changes at any time.

A four-paddle take on Pong, built in Rust with [Bevy](https://bevyengine.org/) 0.20.

Instead of the usual two paddles, the arena has one on each side — left, right, top, and
bottom. The ball bounces around the inside of the box, and each paddle is meant to defend
its own wall. Only the bottom paddle is currently controlled by a human; the other three
are driven by simple computer logic.

## Status

This is an early prototype. The core loop (rendering, movement, collisions) works, but the
game rules are incomplete.

**Working**

- Window-relative arena that resizes with the window
- Four paddles, each with its own lives counter and HUD label
- Human control of the bottom paddle; computer control of the other three
- Ball movement and reflection off walls and paddles

**Not implemented yet**

- Losing a life when the ball passes a paddle — the ball currently just bounces off the
  walls, so lives never change
- Game over / win conditions, scoring, or restarting
- Serving the ball after a point, and preventing the ball from getting stuck or too fast
- Power-ups (the `ColliderType::PowerUp` variant exists but is unused)
- Menus, pause, audio, and any polish
- Human control for paddles other than the bottom one

## Controls

| Action            | Key           |
| ----------------- | ------------- |
| Move bottom paddle | `←` / `→`    |
| Move top paddle\*  | `A` / `D`     |

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
├── flake.nix             # Nix dev shell with Rust and Bevy system libraries
└── src
    ├── main.rs           # App setup: plugins, camera, shared constants
    └── plugins
        ├── ball.rs       # Ball entity, size/position, window resize handling
        ├── paddle.rs     # Paddles, player/computer control, lives
        ├── physics.rs    # Velocity, colliders, collision detection and reflection
        ├── walls.rs      # Arena walls
        └── hud.rs        # On-screen lives displays
```

The game is organised as one Bevy plugin per concern, registered in `src/main.rs`. Arena
dimensions are expressed as percentages of the window's smaller dimension so the game
scales with the window.

## Roadmap

Rough order of intended work:

1. Detect when the ball crosses a wall and decrement that side's lives.
2. Reset/serve the ball after a point, and add a game-over state.
3. Improve ball physics (speed limits, angles based on paddle contact).
4. Let players choose which side(s) are human-controlled.
5. Add power-ups.
6. Audio and visual polish.

## License

Released under the [MIT License](LICENSE).

Copyright (c) 2026 Juan Vásquez.
