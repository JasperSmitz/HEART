# 0013 — Pastel's cozy terminal home

Status: implemented, verified on Linux/Ghostty, and independently reviewed, 2026-10-21. Ready for product-owner visual iteration.

## Scope and design

The user explicitly approves architect → developer → reviewer implementation of a focused dashboard redesign. HEART's direction is a personal terminal workspace with a character companion; embedded shell and expanded module workspaces are future milestones.

Remove the generated character/environment panel. Use the available space for a substantial JRPG dialogue area beside compact System/Music widgets on wide terminals; prioritize dialogue/input on compact screens. Do not leave an empty future-shell placeholder. Separate utility borders from the dialogue's bold frame, left portrait, speaker heading, comfortably padded text, restrained continuation detail, and separate input. Keep a simple layout boundary where a future workspace can be added.

Use the supplied `/tmp/codex-clipboard-aWY4Cl.png` unchanged as a bundled Pastel portrait, with aspect-preserving nearest-neighbor scaling. Investigate/use a maintained Ratatui image integration compatible with the existing backend, preferring Kitty graphics on Ghostty/Linux and retaining macOS compatibility. Unsupported protocols, missing assets, small screens and failed detection must retain usable text with a simple labelled fallback, not generated character art. Terminal capability probing must be bounded and isolated to interactive startup. Cache decoded/protocol state, avoid repeated image encoding on idle ticks, and ensure help/profile changes/resize/quit do not leave images over text.

Preserve `profiles/cat.json` behavior and configuration, including its `cat` ID and Pastel instructions. The uncommitted user change in `crates/core/src/profiles.rs` must be preserved without rewriting it. Keep TUI/image dependencies in CLI and optional presentation data backward compatible. Use Sunset MK. III (#f6c9d2, #f5bad9, #f5b0ec, #daaae4, #bfa4dc, #a49dd4, #848fdb, #708bd4) on dark plum; honor profile semantic colors and keep Butler/custom profiles usable. Prefer a small asset-key mapping and presentation helpers over a general asset/animation framework. No font changes.

## Acceptance criteria

- Oversized generated portrait/room removed; actual supplied portrait appears inside dialogue on supported terminals, preserving proportions/sharpness.
- Recognizable JRPG frame, portrait left, speaker/text right, separated editable input and tasteful continuation/scroll indication.
- Cohesive Pastel palette; System/Music mocks and existing commands, focus, help, profile switch, scrolling, resize, busy worker and quit remain functional.
- Compact/tiny screens and unsupported graphics remain readable without panic or hidden focus traps. Portrait lifecycle handles help/profile/resize without stale overlays.
- No startup module/model/provider requests; no personality behavior changes, and existing user edits preserved.
- Independently reviewed implementation and visual previews; honest distinction between styled previews, protocol fixtures and actual emulator screenshots.

## Non-goals

Embedded shell/PTY features, new modules/integrations, telemetry/audio, multiplexing/workspace management, generated art, complex animation/typewriter systems, model/provider changes, unrelated cleanup and commits.

## Verification

Run formatting, warnings-denied Clippy and workspace/all-target tests appropriate to dependency changes. Add meaningful coverage for image fallback/contain sizing, layout and lifecycle transitions. Produce deterministic production-renderer previews at wide, 80×24 and small sizes, including Butler and help; preview composition may embed the exact PNG and must identify simulated graphics. Inspect images. Exercise interactive controls/protocol lifecycle in a PTY fixture where practical; do not claim Ghostty/macOS runtime validation without running it. Record dependencies, checks, portability limits and any unmet acceptance criteria. Developer implements; separate reviewer checks correctness, appearance, portability and maintainability. At most two correction/recheck rounds before escalation.

## Execution and review

Native developer used Sol/high for graphics/lifecycle integration; native independent reviewer used Sol/medium. An initial reviewer launch failed model capacity before reviewing; a fresh reviewer completed inspection and recheck. Selection requests are not backend execution telemetry.

Implemented the bounded scope: generated art/room removed, supplied PNG bundled byte-for-byte, a substantial double dialogue frame with left portrait and display-name speaker text, separate input, restrained continuation/scroll details and profile-aware colors. Widgets, commands, keyboard controls and request-profile snapshots are retained. Image state stays in CLI: known Ghostty/Kitty plus OS pixel metrics, cached nearest-neighbor contain encoding and a single owned Kitty ID. No stdin probe, native Chafa dependency, font change or new application service. Unknown terminals, multiplexers, missing metrics, nonempty `NO_COLOR` and forced text mode fall back. Butler/custom asset keys currently use labelled fallback.

Developer's 78-test workspace/all-target suite, CLI formatting and warnings-denied Clippy passed. Workspace formatting retains the pre-existing user core-file difference. Architect independently verified protected core/profile hashes and original portrait hash, formatting/whitespace and two eight-check fallback/no-color PTY runs. Actual Ghostty screenshots verify portrait/colors, help cleanup, profile switching/restoration, native uptime, mock music controls, live resizing, small-screen image hiding and clean image deletion/quit. Deterministic composed previews are labelled separately.

Independent review found one P2: the System panel hid readings at 100×30 because an eight-row interior failed a nine-row guard. Correction round 1 adjusts the guard and adds actual-reading assertions for both profiles at 100×30, 100×31 and 120×30. Ten focused tests and Clippy passed afterward; reviewer independently reran all ten, inspected corrected previews and cleared the finding. **No unresolved findings; one correction round.**

The scoped implementation criteria are met within the recorded portability limits. macOS runtime remains untested here; Ghostty/Linux was exercised directly. Visual preference remains the product owner's next iteration, not a claim of a finished workspace. No embedded shell, live telemetry, real music, new module, animation framework or commit. See [evaluation and actual screenshots](../evaluations/terminal-home.md).
