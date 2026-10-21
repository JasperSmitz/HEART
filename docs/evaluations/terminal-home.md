# Terminal-home visual milestone

2026-10-21. [Approved plan](../plans/0013-terminal-home.md) · [decision](../decisions/0006-terminal-home-and-portraits.md).

Status: implemented, verified on Linux and independently reviewed; no unresolved findings. Product-owner visual iteration is expected.

## Sources and preservation

The user supplied Pastel's PNG portrait. Keep the asset unchanged; original SHA-256: `8e462a36024c876d856cd5388e14238506c0bd9bad4fc6aab25ece6639c29108`.

Pastel's existing `profiles/cat.json` is behavioral configuration and is not rewritten. Its pre-implementation SHA-256 is `b9e3a0d2689333515cbb9abf8d9b2168551fc979f93febe854ea9749fa5162b3`. The pre-existing user edit in `crates/core/src/profiles.rs` is preserved; starting SHA-256 is `e9a0cdb9a1cefb7b55d47df874a166fc329f235cab47185486f7baddaa5fc140`.

## Verification evidence

The generated room and character art are removed. The layout uses a double-framed dialogue area, contained portrait on the left, speaker/history on the right, a continuation/scroll marker, separate message input and supporting mock widgets at wide sizes. Pastel retains the existing `cat` profile ID and all character behavior. Display names are used in dashboard messages; asynchronous replies retain the submitting profile's name.

Dependencies are Ratatui 0.30.2 / Crossterm 0.29.0, `ratatui-image` 11.1.0 with only its Crossterm feature, and `image` 0.25.10 with PNG decoding only. No Chafa/native image dependency or stdin protocol probe is required. Known Ghostty/Kitty terminals with reported pixel metrics use Kitty virtual placements; unknown terminals, multiplexers, missing metrics, nonempty `NO_COLOR` and `HEART_PORTRAIT=text` use text fallback. The bundled source is decoded once; nearest-neighbor contain encoding is cached by dimensions, with one owned image ID reused and released on quit.

### Automated and independent checks

- Developer: **78 workspace/all-target tests passed** with locked dependencies. Existing loopback HTTP fixtures required execution outside the restricted sandbox. CLI formatting and warnings-denied workspace/all-target Clippy passed.
- Reviewer: ten focused dashboard/portrait tests passed; independently inspected source, all seven composed previews and actual Ghostty portrait/help/Butler screenshots. The only finding was P2: at 100×30 the System panel's inner height was eight rows and its old nine-row guard hid all readings.
- Correction round 1: developer accepts the eight rows needed for three label/gauge pairs and adds actual CPU/memory/disk assertions for both profiles at 100×30, 100×31 and 120×30. Ten focused tests, CLI formatting and warnings-denied Clippy passed; binary rebuilt. Reviewer independently reran all ten, inspected both final minimum-wide previews and the actual compact screenshot, and cleared the finding. No unresolved findings.
- Architect: CLI formatting, whitespace checks and all three preservation hashes passed. Independent [fallback PTY](terminal-home/fallback-pty.json) and [no-color PTY](terminal-home/no-color-pty.json) each passed eight checks: ready startup, labelled fallback, first `/uptime` request, help, profile switching, no Kitty output, normal exit and exact terminal mode restoration. No live model/provider was needed.

### Actual Ghostty screenshots

These are unmodified screenshots of the real Linux Ghostty window, captured with Spectacle and inspected by the architect, rather than SVG composites:

- [Pastel and Sunset colors](terminal-home/pastel-ghostty.png), initially 119×39 cells.
- [Authored `/uptime` reply and changed mock music state](terminal-home/command-ghostty.png).
- [Help overlay clears portrait](terminal-home/help-ghostty.png).
- [Butler theme and labelled portrait fallback](terminal-home/butler-ghostty.png).
- [Live resize to 81×23 with contained portrait and scroll controls](terminal-home/pastel-compact-ghostty.png).
- [Small layout hides portrait and keeps dialogue/input](terminal-home/pastel-small-ghostty.png). Some resize captures include compositor focus hints outside the application content.

[Recorded terminal checks](terminal-home/ghostty-checks.json) cover help, F2 changes, restoration of Pastel's image, native `/uptime`, Tab-focused mock play/track controls, live resize and clean exit with owned-image deletion. The harness forwarded real Ghostty responses through a PTY, used `/tmp` as the working directory, removed provider credentials, and set inference to an unused loopback endpoint; no live inference/search request was made. The test window and temporary KDE scripts were removed afterward.

The initial tool-launched run inherited `NO_COLOR=1`: its suppressed RGB sequences broke Kitty image-ID encoding. The implementation now falls back in that case, verified separately. Color screenshots removed the tool-injected flag for the test application; HEART does not force colors against the user's preference. A single transient missing-image warning occurred during later repaint; inspected images showed no retained overlay or missing portrait after settling.

### Deterministic previews

The production `dashboard_preview` example renders cells without services. Its `image` mode composes the exact bundled PNG using the same contain bounds and marks the result **SIMULATED GRAPHICS**. This verifies composition/layout, not terminal protocol behavior. Omit `image` for the real text-fallback layout. Reviewer/architect inspected wide, 80×24, small, Butler, help, fallback and long-history previews. Final [Pastel 100×30](terminal-home/pastel-100x30-preview.png) and [Butler 100×30](terminal-home/butler-100x30-preview.png) capture the corrected utility threshold.

```sh
cargo run -p heart --example dashboard_preview -- cat 120 40 image > /tmp/heart-home.svg
HEART_PORTRAIT=text cargo run -p heart -- --dashboard
```

The available Linux environment has Ghostty 1.3.1-arch2, Rust 1.96.0 and a Wayland desktop. macOS runtime verification is not established by their availability.

Baseline `cargo fmt --all -- --check` fails only on the pre-existing user edit in `crates/core/src/profiles.rs:130`. Preserve that edit byte-for-byte; validate formatting of the changed CLI package separately and report the workspace formatting limitation rather than silently formatting user work.

## Boundaries

The milestone changes presentation and bundled artwork. System remains a static demo and Music a local mock. No shell embedding, additional module, live telemetry, real audio, model/provider change or elaborate animation system is implemented. macOS runtime, iTerm2/Sixel graphics and multiplexed graphics were not exercised; unsupported paths retain text fallback. Font/cell geometry and terminal RGB/Unicode behavior remain environment-dependent. No commit was created. Product-owner visual iteration remains expected after this implementation.
