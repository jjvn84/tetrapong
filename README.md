# tetrapong

> 🚧 **WORK IN PROGRESS — this project is not finished.**
> Expect missing features (menus, audio, and local multiplayer are not there yet), placeholder
> behaviour, and breaking changes at any time.

A four-paddle take on Pong, built in Rust with [Bevy](https://bevyengine.org/) 0.20.

Instead of the usual two paddles, the arena has one on each side — left, right, top, and
bottom. The ball bounces around the inside of the box, and each paddle defends its own
wall. Only the bottom paddle is currently controlled by a human; the other three are driven
by simple computer logic.

## Status

This is an early prototype. The core loop includes scoring, a serve delay, paddle
elimination, and a game-over screen with a restart, but the game is still far from finished.

**Working**

- Window-relative arena that resizes with the window
- Four paddles, each with its own lives counter and HUD label
- Human control of the bottom paddle; computer control of the other three
- Ball movement and reflection off walls and paddles
- Scoring: when the ball hits a wall, that side loses a life
- Ball reset and a 2-second serve pause before relaunching in a random direction
- Paddle elimination at 0 lives, with the HUD showing 0
- Power-ups that spawn in the arena and apply to the last paddle to hit the ball
- Dark neon theme with a colour per side and a distinct glyph for every power-up
- Game over when every human paddle is eliminated or only one paddle is left, with a
  result screen and `R` to start a new round

**Not implemented yet**

- Behaviour for an eliminated side: its wall still emits `LoseLife`, but no paddle remains
  to lose a life, so the ball keeps bouncing off that wall
- Menus, pause menu, audio, and general polish
- Human control for paddles other than the bottom one
- Broader automated test coverage — only the ball substepping and the game-over rule are
  unit-tested

## How it plays

1. The ball starts at the centre with a random direction.
2. Each of the four paddles starts with 3 lives.
3. If the ball reaches a wall, that wall's side loses a life.
4. The ball resets to the centre, waits 2 seconds, then serves in a random direction.
5. At 0 lives, the paddle is removed and its HUD label shows 0.
6. The round ends when only one paddle is left — it wins, even if it is computer-controlled —
   or when every human-controlled paddle has been eliminated. The game freezes and a result
   screen appears; press `R` to start a new round with fresh lives, a new ball, and no
   leftover power-ups.

## Power-ups

Power-ups appear at random spots inside the arena every couple of seconds (up to four at a
time). When the ball touches one, the power-up is collected and applied to the paddle that
hit the ball last.

Each power-up is drawn as a dark circular badge with a coloured ring and a unique white
glyph, so they can be told apart by shape and by colour:

| Glyph                   | Ring colour | Power-up            | Effect                                             |
| ----------------------- | ----------- | ------------------- | -------------------------------------------------- |
| Wide horizontal bar     | Blue        | `PaddleDouble`      | Doubles paddle width (up to 4× the original)       |
| Short horizontal bar    | Blue        | `PaddleHalf`        | Halves paddle width (down to ¼ of the original)    |
| Triangle pointing right | Violet      | `PaddleSpeedDouble` | Doubles paddle movement speed (up to 4×)           |
| Small triangle right    | Violet      | `PaddleSpeedHalf`   | Halves paddle movement speed (down to ¼)           |
| Filled circle           | Orange      | `BallSpeedDouble`   | Doubles the ball speed (capped at 4×)              |
| Ring                    | Orange      | `BallSpeedHalf`     | Halves the ball speed (never below the base speed) |
| Diamond                 | Green       | `EarnOneLife`       | Grants the paddle one extra life                   |
| Triangle pointing down  | Red         | `LoseOneLife`       | Removes one life; at 0 the paddle is eliminated    |

## Visual style

The arena uses a dark navy background with a uniform wall colour, while each paddle has its
own accent colour so the four sides stay easy to tell apart. All colours live in
`src/theme.rs`; change them there to retheme the whole game.

| Element               | Colour                            |
| --------------------- | --------------------------------- |
| Left paddle           | Cyan                              |
| Right paddle          | Magenta                           |
| Bottom paddle (human) | Lime                              |
| Top paddle            | Amber                             |
| Walls                 | Slate (uniform on all four sides) |
| Ball                  | Warm white                        |
| HUD panels            | Dark slate, with per-side text    |
| Game-over overlay     | Translucent navy, winner text in that side's colour |

## Controls

| Action                  | Key                                |
| ----------------------- | ---------------------------------- |
| Move bottom paddle      | `←` / `→`                          |
| Move top paddle\*       | `A` / `D`                          |
| Restart after game over | `R`                                |

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

### Prebuilt binaries

Pushing a `v*` tag publishes Linux and Windows packages (`.tar.gz` and `.zip`) to the
[Releases page](https://github.com/jjvn84/tetrapong/releases). Builds for every push and
pull request are also available as workflow artifacts on the Actions tab.

## Project layout

```text
.
├── Cargo.toml            # Crate metadata and dependencies
├── LICENSE               # MIT license
├── flake.nix             # Nix dev shell with Rust and Bevy system libraries
└── src
    ├── main.rs           # Entry point: adds GamePlugin, shared constants
    ├── game_state.rs     # GameState: Game / GameOver
    ├── theme.rs          # Central colour palette used by every plugin
    └── plugins
        ├── game.rs       # App wiring, state gating, game-over detection, result screen
        ├── ball.rs       # Ball entity, sizing, serve-pause timer and relaunch
        ├── paddle.rs     # Paddles, input/AI, lives, life-loss and elimination
        ├── physics.rs    # Velocity, colliders, collisions, emits LoseLife on wall hits
        ├── walls.rs      # Arena walls and their PaddleLocation ownership
        ├── hud.rs        # On-screen lives panels and eliminated state
        └── powerups.rs   # Power-up spawning, badge/glyph visuals and effects
```

The game is organised as one Bevy plugin per concern, all registered by `GamePlugin` in
`src/plugins/game.rs`. Arena dimensions are expressed as percentages of the window's
smaller dimension so the game scales with the window.

Gameplay flows through messages: a wall collision emits `LoseLife`, `paddle.rs` decrements
lives and resets the ball, and `PaddleEliminated` tells the HUD when a paddle has been
removed. Gameplay systems are grouped in per-plugin system sets, and `GamePlugin` runs
those sets only in `GameState::Game`; the ball, paddles and power-ups of a round are
rebuilt whenever a new round starts.

## Roadmap

Rough order of intended work:

1. Decide what happens when a side is eliminated (stop scoring it, remove its goal, or end
   the game).
2. Improve ball physics (angles based on paddle contact, spin).
3. Let players choose which side(s) are human-controlled.
4. Menus, audio, and further visual polish (glow, trails, hit effects).

## License

Released under the [MIT License](LICENSE).

Copyright (c) 2026 Juan Vásquez.
