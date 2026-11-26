# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

`tetrapong` is a **work-in-progress** four-paddle Pong clone written in Rust with Bevy
0.20. There are four paddles (left, right, top, bottom), one per wall of a square arena.
Each side can be set to human or computer control in the Options menu; by default the bottom
paddle is human and the rest use simple AI. The game boots into a main menu and has a
How to Play screen, an Options screen, a running round, and a game-over screen.

The project is an early prototype: movement, collisions, scoring, the serve delay, paddle
elimination, menus, per-side players, difficulty, and the game-over/restart flow work, but
the game is far from finished.

Do not assume the game is feature-complete. Check the "Status" section of `README.md`
before implementing or "fixing" gameplay behaviour.

## Toolchain and commands

The repository uses Nix (see `flake.nix`). `cargo` may not be on the host `PATH`; run
commands inside the dev shell:

```sh
nix develop --command cargo check
nix develop --command cargo run
```

If the shell is already loaded (for example via direnv), plain `cargo ...` works.

Common commands:

| Task              | Command                              |
| ----------------- | ------------------------------------ |
| Type-check        | `cargo check`                        |
| Build             | `cargo build`                        |
| Run               | `cargo run`                          |
| Format            | `cargo fmt`                          |
| Lint              | `cargo clippy --all-targets`         |
| Tests             | `cargo test`                         |

Run `cargo fmt` and `cargo clippy` before finishing changes. Do not commit `target/` or
`.direnv/`; they are gitignored.

## Architecture

The game follows Bevy's ECS and plugin pattern. Each concern is a plugin in
`src/plugins/`:

- `game.rs` — `GamePlugin`, which owns the app-level wiring: it adds `DefaultPlugins`, the
  camera, every sub-plugin, `GameState`, the state gating, game-over detection, and the
  result screen.
- `menu.rs` — `MenuPlugin`, which spawns the main menu, Options and How to Play screens and
  handles `Esc` back to the main menu.
- `ui.rs` — shared menu building blocks: `MenuAction`, `MenuButton`, `spawn_button`,
  `spawn_choice_button`, title/text helpers, selected/hovered/pressed visuals, and the single
  `Activate` observer that drives every menu button.
- `ball.rs` — spawns the ball at the start of each round, sets its size from the window,
  and owns `BallReset` and `ServePause`: every reset (after a point or a resize) parks the
  ball at the centre and starts the 2-second timer that relaunches it in a random direction.
- `paddle.rs` — spawns the four paddles at the start of each round from `Settings`, defines
  `PaddleLocation`, `PlayerType`, `Lives`, `LoseLife`, and `PaddleEliminated`; handles
  human input, computer movement, life loss, and despawning eliminated paddles.
- `physics.rs` — `Velocity`, `Collider`/`ColliderType`, and `move_ball`, which
  integrates the ball in distance-bounded substeps and resolves AABB vs. bounding circle
  collisions. Emits `LoseLife` when the ball hits a wall and `PowerUpHit` when it touches a
  power-up.
- `walls.rs` — spawns the four arena walls, each carrying a `WallLocation` component, and
  keeps them aligned on resize. `WallLocation::owner` maps a wall to its `PaddleLocation`.
- `hud.rs` — spawns and updates the per-side lives panels for the current round, including
  the 0 state when a paddle is eliminated.
- `powerups.rs` — `PowerUpKind`, spawning of collectible badges at random positions, their
  preloaded meshes/materials, and applying each effect to the paddle that last hit the ball.

`src/game_state.rs` defines `GameState` (`MainMenu`, `Options`, `HowToPlay`, `Game`,
`GameOver`; `MainMenu` is the default). `src/settings.rs` defines the `Settings` resource
(per-paddle `PlayerType` and `Difficulty`). `src/theme.rs` holds the palette. Plugins import
colours and the state from those modules rather than defining their own constants.

Plugins are declared in `src/plugins.rs` and registered in `src/plugins/game.rs`.
`src/main.rs` only adds `GamePlugin`; `STARTING_LIVES` lives there too.

### Game states and round lifecycle

- `GameState` has five variants: `MainMenu` (the default), `Options`, `HowToPlay`, `Game`,
  and `GameOver`. `GamePlugin` calls `init_state::<GameState>()`, so the app boots into
  `MainMenu`.
- Each menu screen is spawned in `OnEnter(its state)` and carries `DespawnOnExit(that
  state)`; entering another state removes the old screen before building the new one.
- Each gameplay plugin declares a `SystemSet` (`BallSet`, `PaddleSet`, `PhysicsSet`,
  `PowerUpsSet`) and tags its gameplay systems with `.in_set(..)`. `GamePlugin` applies
  `run_if(in_state(GameState::Game))` to those sets once, so plugins never import the state
  for gating.
- Systems that must keep running in all states (all `on_window_resize` systems, the menu
  button visuals, the `Esc` handler) are not put in those sets.
- Round entities (ball, paddles, HUD panels, power-up badges) are spawned in
  `OnEnter(GameState::Game)` and carry `DespawnOnExit(GameState::Game)`. Leaving `Game`
  clears the arena and HUD; re-entering it rebuilds a fresh round, which is what `R` and the
  **Play again** button trigger. `OnEnter(Game)` also resets `GameResult` and the power-up
  `SpawnTimer`.
- Arena-level entities (camera, walls, `PowerUpAssets`) stay on `Startup` and survive state
  changes.
- `check_game_over` runs in `Update` after `PaddleSet`/`PowerUpsSet` and delegates the
  decision to `decide_outcome`. The last paddle standing wins, whoever controls it. When
  `Settings::has_human_players()` is true, the round also ends as soon as every
  `PlayerType::Human` paddle is gone (loss); an all-computer round instead runs until one
  paddle is left. The decision is unit-tested.
- The game-over overlay is spawned in `OnEnter(GameOver)` with buttons for Play again, Main
  menu and Quit. `R` restarts; `Esc` returns to the main menu from every non-menu state
  (`menu.rs::escape_to_menu`).
- `Settings` is the source of truth for player ownership and difficulty. `paddle.rs::setup`
  reads `settings.player_type(location)` for each of `PaddleLocation::ALL`, and
  `move_paddle` uses `settings.difficulty.fail_threshold()` for both computer branches.
  Options changes therefore apply to the next round; they are not applied mid-round.

### Key conventions

- **Window-relative sizing.** Most dimensions are percentages of
  `min(window.width, window.height)`, applied via `Transform::scale`. Keep this approach
  when adding entities so the arena scales with the window.
- **Z ordering.** 2D entities keep `z` scale at `1.0`; use the translation `z` for
  layering (the ball uses `z = 1.0`).
- **Resizing.** Entities that depend on window size implement an `on_window_resize` system
  reading `WindowResized` messages and recomputing position/scale. The ball is the exception:
  `ball.rs::ball_reset_on_resize` turns a resize into `BallReset` so the ball is parked and
  served like after a point.
- **Collisions.** Anything the ball should bounce off gets a `Collider` component. The
  ball is a `BoundingCircle`; colliders are treated as `Aabb2d`.
- **Paddle ownership.** `PaddleLocation` identifies a side and computes its position and
  size. `PaddleLocation::ALL`, `index()` and `label()` keep settings, spawning, menus and the
  HUD in the same side order. Walls map to `PaddleLocation` via `WallLocation::owner`.
- **Settings drive the round.** Read `Settings` for player ownership and difficulty instead
  of hard-coding `PlayerType` or the `fail_probability` threshold. Default settings are
  Bottom = Human, the other three = Computer, Difficulty = Medium.
- **Menu UI goes through `ui.rs`.** Add screens in `menu.rs`, build them with `spawn_screen` /
  `spawn_title` / `spawn_text` / `spawn_button`, and give buttons a `MenuAction`. The single
  `Activate` observer in `ui.rs` handles navigation, settings changes and quitting; don't add
  per-button observers or duplicate state-transition logic.
- **Button labels are not pickable.** Every text node inside a button (and every layout
  container) carries `Pickable::IGNORE`; otherwise the text sits on top of the button and
  swallows clicks. Buttons keep `Pickable::default()`.
- **Scoring flow.** `physics.rs` emits `LoseLife(PaddleLocation)` on a wall hit;
  `paddle.rs::handle_lose_life` is the single place that decrements lives, and it emits
  `BallReset` for the live paddle that conceded. Keep game rules out of the collision system.
- **Ball resets.** `ball.rs::reset_ball` is the only place that moves the ball back to the
  centre, restores its size, clears its per-rally state (`last_hit_by`, `speed_multiplier`)
  and inserts `ServePause`. Its only triggers are `BallReset` from a conceded point and from
  a window resize. `reset_ball` is ordered after `PaddleSet` and before `PhysicsSet` so a
  parked ball is never advanced with the reflected velocity from the wall hit.
- **Serve pause.** `ServePause(Timer)` is inserted on the ball by `reset_ball`;
  `tick_serve_pause` relaunches the ball when the timer finishes. The ball's velocity is
  zeroed during the pause, so it stays parked at the centre.
- **HUD follows messages, not entity lifetime.** Eliminated paddles are despawned, so the
  HUD cannot read their final `Lives` value. `PaddleEliminated` is what updates the label
  to 0; don't couple the HUD back to the paddle entity.
- **One point per frame.** A corner hit can emit multiple `LoseLife` messages in one frame;
  `handle_lose_life` guards with `point_already_scored`.
- **Substepped ball movement.** `physics.rs::move_ball` splits each frame into substeps so
  the ball never travels more than half its radius between collision checks, and caps the
  frame delta. Don't go back to a single move-then-check: fast balls (stacked ball-speed
  power-ups) tunnel through the thin walls when the frame rate drops.
- **Colours come from `theme.rs`.** Don't hard-code `Color::srgb` values in plugins; add a
  constant (or helper such as `theme::paddle_color`) instead. Per-side identity is carried by
  paddle colour; walls and the arena stay uniform.
- **Power-ups are preloaded.** `PowerUpsPlugin` builds a `PowerUpAssets` resource once in
  `Startup` (badge, ring, glyph meshes and materials). Reuse those handles when spawning;
  don't build meshes in the spawn loop.
- **Power-up entity layout.** The root entity holds `Transform` (sized from the window),
  `Collider`, and `PowerUp`, so collisions stay tied to the root. Badge, coloured ring, and
  white glyph are children and may be restyled without changing collision size.

### Bevy 0.20 API notes

This code targets Bevy 0.20, which differs from older examples online:

- Buffered events are `Message`s: use `#[derive(Message)]`, `app.add_message::<T>()`, and
  `MessageReader<T>` / `MessageWriter<T>`. This is what `WindowResized`, `LoseLife`, and
  `PaddleEliminated` use. `Event`/`On` are the separate observer API — don't mix them up.
- UI buttons use the headless `bevy::ui_widgets::Button`, **not** the deprecated
  `bevy::ui::Button` that the prelude also exports. Import the widget explicitly
  (`use bevy::ui_widgets::{Activate, Button};`); an explicit import shadows the prelude glob.
- A button reports activation through the `Activate` entity event. `UiPlugin` registers one
  global observer with `app.add_observer(..)` and reads the entity's `MenuAction`; observers
  can use `Res`/`ResMut`/`MessageWriter` normally, including `MessageWriter<AppExit>`.
- Hover state is not automatic: insert `Hovered::default()` from
  `bevy::picking::hover::Hovered` on the button. Press state is `bevy::ui::Pressed` (inserted
  and removed by the widget). `Pickable` (from `bevy::picking`) controls hit-testing;
  non-button UI nodes must use `Pickable::IGNORE`.
- UI node borders use `BorderColor` (`BorderColor::all(color)`) plus `border` on `Node`.
- `TextFont` takes `font_size: FontSize::Px(..)` in this version; check existing HUD code.
- `BorderRadius` is a field on `Node` (`node.border_radius = BorderRadius::all(..)`), not a
  standalone UI component.
- Required components (`#[require(...)]`) are used, e.g. `Wall` requires `Sprite`,
  `Transform`, and `Collider`.
- `Single<...>` validates that the query matches exactly one entity and skips the system
  otherwise. Use a plain `Query` when a component is conditional (e.g. `ServePause`), or
  `Single` with an `Option<&T>` field when the entity always exists but the component may
  not.
- Use `Timer::from_seconds(secs, TimerMode::Once)`, tick it with `time.delta()`, and check
  `just_finished()`.
- Use `rand` 0.10 APIs (`rand::random_range`, `rand::rng().random_range`, `RngExt`).

## Known gaps and gotchas

- **Game over is decided by number of players, not by side.** `game.rs::decide_outcome`
  awards the win to the last paddle standing, whatever `PlayerType` it has. With two or more
  survivors, the round ends as soon as every `PlayerType::Human` paddle is gone — but only
  when the settings started with at least one human (`humans_playing`). If you add more
  human players, keep the "all humans eliminated" check rather than checking a hard-coded
  side.
- **Eliminated sides still score.** Their wall keeps emitting `LoseLife`, but no paddle
  matches, so nothing is decremented or reset and the ball bounces off that wall. Decide
  the intended rule before changing this.
- **Walls are solid.** The ball reflects off a wall in the same frame the point is scored;
  `handle_lose_life` then resets it. A wall hit is the scoring trigger, not a pass-through.
- **Power-ups need a `last_hit_by`.** A power-up is only awarded when `ball.last_hit_by` is
  set; a ball that has never touched a paddle collects nothing and the badge stays in place.
- **Power-ups don't bounce the ball.** `ColliderType::PowerUp` despawns the badge and emits
  `PowerUpHit` without reflecting the ball's velocity, so the ball passes through it.
- **Resize still assumes entity order.** `walls.rs::on_window_resize` maps query iteration
  order to `WallLocation` even though walls now carry a `WallLocation` component. Prefer
  querying `(&WallLocation, &mut Transform)` if you touch it.
- **AI is intentionally imperfect.** Computer paddles skip movement randomly
  (`fail_probability`), and this is deliberate. The Easy/Medium/Hard thresholds are defined
  in `settings.rs::Difficulty::fail_threshold`; don't "fix" the misses without being asked.
- **Menus are mouse-driven.** Buttons respond to clicks; `Esc` is the only menu keyboard
  shortcut. There is no Tab focus or gamepad navigation yet.
- **Menu sizes are fixed pixels.** Menu layouts use `Val::Px` rather than the
  window-relative sizing used by the arena, so very small windows can clip content.
- **Few tests.** `.github/workflows/build.yml` packages release builds for Linux and Windows
  and publishes them on `v*` tags. Unit tests cover ball substepping (`physics.rs`), the
  game-over rule (`game.rs::decide_outcome`), and settings defaults/thresholds
  (`settings.rs`). If you add logic that is hard to verify by running the game, consider
  adding unit tests.

## Working style

- Keep changes small and focused; match the existing plugin/module structure and naming.
- Avoid adding dependencies unless clearly necessary; the project intentionally has only
  `bevy`, `rand` and `strum`.
- When changing scoring or lives, keep `LoseLife` and `PaddleEliminated` as the interface
  between physics, paddle logic, and the HUD.
- Update `README.md` when you change controls, status, game rules, or the project layout.
- The project is a learning/prototype effort: prefer clear, idiomatic Bevy code over
  over-engineering or premature abstraction.
