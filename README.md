# tetrapong

> 🚧 **WORK IN PROGRESS — this project is not finished.**
> Expect missing features (pause menu, audio, and networked play are not there yet), placeholder
> behaviour, and breaking changes at any time.

A four-paddle take on Pong, built in Rust with [Bevy](https://bevyengine.org/) 0.20.

Instead of the usual two paddles, the arena has one on each side — left, right, top, and
bottom. The ball bounces around the inside of the box, and each paddle defends its own
wall. Every paddle can be set to human or computer control in the Options menu; by default
the bottom paddle is human and the other three are driven by simple computer logic.

## Status

This is an early prototype. The core loop includes menus, configurable players and
difficulty, scoring, a serve delay, paddle elimination, and a game-over screen, but the game
is still far from finished.

**Working**

- Main menu with Play, How to Play, Options and Quit
- Options for setting each of the four paddles to Human or Computer
- Easy / Medium / Hard difficulty for the computer paddles
- How to Play screen explaining the rules, controls and every power-up
- Window-relative arena that resizes with the window
- Four paddles, each with its own lives counter and HUD label
- Human control for any paddle assigned to a player, using dedicated keys per side
- Ball movement and reflection off walls and paddles
- Scoring: when the ball hits a wall owned by a side that is still alive, that side loses a life
- Ball reset and a 2-second serve pause before relaunching in a random direction
- Paddle elimination at 0 lives, with the HUD showing 0
- Power-ups that spawn in the arena and apply to the last paddle to hit the ball
- Dark neon theme with a colour per side and a distinct glyph for every power-up
- Game over when every human paddle is eliminated or only one paddle is left, with a
  result screen offering Play again, Main menu and Quit

**Not implemented yet**

- Pause menu, audio, networked play, and general polish
- Broader automated test coverage — ball substepping, the game-over rule, and the settings
  resource are unit-tested; the UI is verified by playing the game

## Menus and options

The game boots into the main menu:

- **Play** starts a round with the current settings.
- **How to Play** explains the rules, the per-side controls, and every power-up.
- **Options** lets you choose Human or Computer for each paddle, and the difficulty.
- **Quit** closes the game.

`Esc` returns to the main menu from any other screen, including a running round and the
game-over screen. The game-over screen also has buttons for **Play again**, **Main menu**
and **Quit**, and `R` still restarts a finished round.

The difficulty only changes how often a computer paddle skips its movement for a frame.
`fail_probability` is drawn from `0.0..10.0`; when it reaches the threshold below, the
paddle does not move that frame:

| Difficulty | `fail_probability` threshold |
| ---------- | ---------------------------- |
| Easy       | `>= 5.0`                     |
| Medium     | `>= 6.0`                     |
| Hard       | `>= 7.0`                     |

Any combination is allowed, including an all-computer round. If at least one human paddle
is configured, the round ends as soon as every human paddle is eliminated; an all-computer
round instead runs until only one paddle remains.

## How it plays

1. The ball starts at the centre with a random direction.
2. Each of the four paddles starts with 3 lives.
3. If the ball reaches a wall owned by a side that is still alive, that side loses a life.
   An eliminated side's wall still bounces the ball but concedes no point.
4. The ball resets to the centre, waits 2 seconds, then serves in a random direction at
   its base speed. Resizing the window does the same: the ball returns to the centre and
   serves again, losing any speed power-up it had.
5. At 0 lives, the paddle is removed and its HUD label shows 0.
6. The round ends when only one paddle is left — it wins, even if it is computer-controlled —
   or, when at least one human is configured, when every human-controlled paddle has been
   eliminated. The game freezes and a result screen appears; **Play again** or `R` starts a
   new round with fresh lives, a new ball, and no leftover power-ups, while **Main menu** or
   `Esc` returns to the title screen.

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
| Bottom paddle         | Lime                              |
| Top paddle            | Amber                             |
| Walls                 | Slate (uniform on all four sides) |
| Ball                  | Warm white                        |
| HUD panels            | Dark slate, with per-side text    |
| Menus                 | Dark navy with slate buttons and a cyan accent for the active option |
| Game-over overlay     | Translucent navy, winner text in that side's colour |

## Controls

Paddle keys only apply to paddles set to **Human** in Options.

| Action                    | Key                                |
| ------------------------- | ---------------------------------- |
| Move bottom paddle        | `←` / `→`                          |
| Move left paddle          | `Q` / `A`                          |
| Move right paddle         | `I` / `K`                          |
| Move top paddle           | `F` / `G`                          |
| Use the menus             | Mouse; click a button              |
| Go back to the main menu  | `Esc`                              |
| Restart after game over   | `R`                                |
| Quit                      | **Quit** button, or close the window |

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
    ├── game_state.rs     # GameState: MainMenu / Options / HowToPlay / Game / GameOver
    ├── settings.rs       # Settings: per-paddle Human/Computer and AI difficulty
    ├── theme.rs          # Central colour palette used by every plugin
    └── plugins
        ├── game.rs       # App wiring, state gating, game-over detection, result screen
        ├── menu.rs       # Main menu, options and How to Play screens
        ├── ui.rs         # Shared menu buttons, actions and hover visuals
        ├── ball.rs       # Ball entity, sizing, BallReset, serve-pause timer and relaunch
        ├── paddle.rs     # Paddles, input/AI, lives, AlivePaddles, elimination
        ├── physics.rs    # Velocity, colliders, collisions, emits LoseLife on wall hits
        ├── walls.rs      # Arena walls and their PaddleLocation ownership
        ├── hud.rs        # On-screen lives panels and eliminated state
        └── powerups.rs   # Power-up spawning, badge/glyph visuals and effects
```

The game is organised as one Bevy plugin per concern, all registered by `GamePlugin` in
`src/plugins/game.rs`. Arena dimensions are expressed as percentages of the window's
smaller dimension so the game scales with the window.

`Settings` is the single source of truth for player ownership and difficulty. The Options
screen edits it through menu button actions, `paddle.rs` reads it when spawning the round
and when deciding how often a computer paddle skips a frame, and `game.rs` reads it to
decide whether the "all humans eliminated" rule applies.

Gameplay flows through messages: a wall collision on a live side emits `LoseLife`,
`paddle.rs` decrements lives and resets the ball, and `PaddleEliminated` tells the HUD when
a paddle has been removed. The `AlivePaddles` resource tracks which sides are still in the
round; `physics.rs` uses it to ignore dead walls and `game.rs` uses it to decide the outcome.
Gameplay systems are grouped in per-plugin system sets, and `GamePlugin` runs
those sets only in `GameState::Game`; the ball, paddles, HUD and power-ups of a round are
rebuilt whenever a new round starts, while the menu screens are spawned when their state is
entered.

## Roadmap

Rough order of intended work:

1. Add a pause menu, audio, and further visual polish (glow, trails, hit effects).
2. Improve ball physics (angles based on paddle contact, spin).
3. Consider networked or AI-vs-AI spectator modes.

## License

Released under the [MIT License](LICENSE).

Copyright (c) 2026 Juan Vásquez.
