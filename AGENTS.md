# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

`tetrapong` is a **work-in-progress** four-paddle Pong clone written in Rust with Bevy
0.20. There are four paddles (left, right, top, bottom), one per wall of a square arena.
The bottom paddle is human-controlled; the others use simple AI. The project is an early
prototype: collision and movement work, but lives/scoring/game-over are not wired up yet.

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

- `ball.rs` — spawns the ball, sets its size from the window, resets it on resize.
- `paddle.rs` — spawns the four paddles, defines `PaddleLocation`, `PlayerType`, `Lives`,
  handles human input and computer movement.
- `physics.rs` — `Velocity`, `Collider`/`ColliderType`, `apply_velocity`, and
  `check_for_collisions` (AABB vs. bounding circle reflection).
- `walls.rs` — spawns the four arena walls and keeps them aligned on resize.
- `hud.rs` — spawns and updates the per-side lives text.

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

### Bevy 0.20 API notes

This code targets Bevy 0.20, which differs from older examples online:

- Use `MessageReader<T>` (not `EventReader`) for buffered messages such as `WindowResized`.
- `TextFont` takes `font_size: FontSize::Px(..)` in this version; check existing HUD code.
- Required components (`#[require(...)]`) are used, e.g. `Wall` requires `Sprite`,
  `Transform`, and `Collider`.
- Prefer `Single<...>` queries when exactly one entity is expected.
- Use `rand` 0.10 APIs (`rand::random_range`, `rand::rng().random_range`, `RngExt`).

## Known gaps and gotchas

- **Lives never change.** `Lives` and the HUD update system exist, but nothing decrements
  lives when the ball passes a paddle. This is the main unfinished mechanic.
- **Walls are solid.** Because the ball reflects off all four walls, it can never reach a
  paddle's "goal". Implementing scoring will likely require removing or disabling wall
  collision for the ball, or adding a separate goal zone.
- **Resize assumes entity order.** `walls.rs::on_window_resize` maps query iteration order
  to wall locations. This is fragile; preserve spawn order or refactor to store the
  location on the entity if you touch it.
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
- Update `README.md` when you change controls, status, or the project layout.
- The project is a learning/prototype effort: prefer clear, idiomatic Bevy code over
  over-engineering or premature abstraction.
