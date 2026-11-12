# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

`tetrapong` is a **work-in-progress** four-paddle Pong clone written in Rust with Bevy
0.20. There are four paddles (left, right, top, bottom), one per wall of a square arena.
The bottom paddle is human-controlled; the others use simple AI. The project is an early
prototype: movement, collisions, scoring, the serve delay, and paddle elimination work, but
there is no game-over/win condition yet.

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
| Tests             | `cargo test` (no tests exist yet)    |

Run `cargo fmt` and `cargo clippy` before finishing changes. Do not commit `target/` or
`.direnv/`; they are gitignored.

## Architecture

The game follows Bevy's ECS and plugin pattern. Each concern is a plugin in
`src/plugins/`:

- `ball.rs` — spawns the ball, sets its size from the window, resets it on resize, and
  owns `ServePause` (the 2-second timer that relaunches the ball after a point).
- `paddle.rs` — spawns the four paddles, defines `PaddleLocation`, `PlayerType`, `Lives`,
  `LoseLife`, and `PaddleEliminated`; handles human input, computer movement, life loss,
  and despawning eliminated paddles.
- `physics.rs` — `Velocity`, `Collider`/`ColliderType`, `apply_velocity`, and
  `check_for_collisions` (AABB vs. bounding circle reflection). Emits `LoseLife` when the
  ball hits a wall.
- `walls.rs` — spawns the four arena walls, each carrying a `WallLocation` component, and
  keeps them aligned on resize. `WallLocation::owner` maps a wall to its `PaddleLocation`.
- `hud.rs` — spawns and updates the per-side lives text, including the 0 state when a
  paddle is eliminated.

Plugins are declared in `src/plugins.rs` and registered in `src/main.rs`, along with the
camera and shared constants such as `STARTING_LIVES`.

### Key conventions

- **Window-relative sizing.** Most dimensions are percentages of
  `min(window.width, window.height)`, applied via `Transform::scale`. Keep this approach
  when adding entities so the arena scales with the window.
- **Z ordering.** 2D entities keep `z` scale at `1.0`; use the translation `z` for
  layering (the ball uses `z = 1.0`).
- **Resizing.** Entities that depend on window size implement an `on_window_resize` system
  reading `WindowResized` messages and recomputing position/scale.
- **Collisions.** Anything the ball should bounce off gets a `Collider` component. The
  ball is a `BoundingCircle`; colliders are treated as `Aabb2d`.
- **Paddle ownership.** `PaddleLocation` identifies a side and computes its position and
  size. Walls map to `PaddleLocation` via `WallLocation::owner`.
- **Scoring flow.** `physics.rs` emits `LoseLife(PaddleLocation)` on a wall hit;
  `paddle.rs::handle_lose_life` is the single place that decrements lives, resets the ball,
  and starts the serve pause. Keep game rules out of the collision system.
- **Serve pause.** `ServePause(Timer)` is inserted on the ball when a point is scored;
  `tick_serve_pause` relaunches the ball when the timer finishes. The ball's velocity is
  zeroed during the pause, so it stays parked at the centre.
- **HUD follows messages, not entity lifetime.** Eliminated paddles are despawned, so the
  HUD cannot read their final `Lives` value. `PaddleEliminated` is what updates the label
  to 0; don't couple the HUD back to the paddle entity.
- **One point per frame.** A corner hit can emit multiple `LoseLife` messages in one frame;
  `handle_lose_life` guards with `point_already_scored`.

### Bevy 0.20 API notes

This code targets Bevy 0.20, which differs from older examples online:

- Buffered events are `Message`s: use `#[derive(Message)]`, `app.add_message::<T>()`, and
  `MessageReader<T>` / `MessageWriter<T>`. This is what `WindowResized`, `LoseLife`, and
  `PaddleEliminated` use. `Event`/`On` are the separate observer API — don't mix them up.
- `TextFont` takes `font_size: FontSize::Px(..)` in this version; check existing HUD code.
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

- **No game over.** Paddles are despawned at 0 lives, but the game continues indefinitely.
  Winner detection and restart are the main missing pieces.
- **Eliminated sides still score.** Their wall keeps emitting `LoseLife`, but no paddle
  matches, so nothing is decremented or reset and the ball bounces off that wall. Decide
  the intended rule before changing this.
- **Walls are solid.** The ball reflects off a wall in the same frame the point is scored;
  `handle_lose_life` then resets it. A wall hit is the scoring trigger, not a pass-through.
- **`ColliderType::PowerUp => todo!()`** will panic if a power-up collider is ever hit.
  Power-ups are unimplemented.
- **Resize still assumes entity order.** `walls.rs::on_window_resize` maps query iteration
  order to `WallLocation` even though walls now carry a `WallLocation` component. Prefer
  querying `(&WallLocation, &mut Transform)` if you touch it.
- **Top paddle input is dead code.** The human branch for `PaddleLocation::Top` exists but
  the paddle is spawned as `PlayerType::Computer`.
- **AI is intentionally imperfect.** Computer paddles skip movement randomly
  (`fail_probability`), and this is deliberate. Don't "fix" it without being asked.
- **No tests or CI.** `cargo test` passes trivially. If you add logic that is hard to
  verify by running the game, consider adding unit tests.

## Working style

- Keep changes small and focused; match the existing plugin/module structure and naming.
- Avoid adding dependencies unless clearly necessary; the project intentionally has only
  `bevy` and `rand`.
- When changing scoring or lives, keep `LoseLife` and `PaddleEliminated` as the interface
  between physics, paddle logic, and the HUD.
- Update `README.md` when you change controls, status, game rules, or the project layout.
- The project is a learning/prototype effort: prefer clear, idiomatic Bevy code over
  over-engineering or premature abstraction.
