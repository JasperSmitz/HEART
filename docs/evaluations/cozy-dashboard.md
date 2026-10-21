# Cozy dashboard prototype evaluation

2026-10-20. [Approved plan](../plans/0012-cozy-dashboard.md) · [presentation decision](../decisions/0005-terminal-dashboard-presentation.md).

## Scope and visual evidence

Opt-in `--dashboard` preserves the ordinary console. Cat starts in a plum/rose room; Butler uses navy/cream/gold. Character and framed dialogue take priority, with supporting fixed system and local-state music mocks. No real media, background telemetry, image protocol or new capability is implemented. Typed commands retain their existing on-demand behavior.

These PNGs are exported from the **production Ratatui renderer** using its deterministic `dashboard_preview` example, then rasterized with native macOS AppKit. They are styled-cell previews, not terminal screenshots: Menlo and 9×18 cells approximate a terminal; font metrics/color approximation vary with the user's terminal.

- [Cat, 120×40](dashboard/cat-wide.png)
- [Butler, 120×40](dashboard/butler-wide.png)
- [Cat, 80×24](dashboard/cat-compact.png)
- [Cat, 48×18](dashboard/cat-small.png)

Reproduce the renderer export without services or credentials:

```sh
cargo run -p heart --example dashboard_preview -- cat 120 40 > /tmp/heart-cat.svg
```

The architect and independent reviewer inspected rendered output. Initial compact portrait stretching/overlapping scenery was corrected through contain-fit art and reserved scenery regions. Compact header content and input caret visibility were independently reviewed and corrected. Small portrait features were refined to survive limited cell resolution. Visual acceptance remains with the product owner; this prototype is a starting point for look/feel iteration.

## Verification

- **macOS ARM64:** developer's final `cargo test --workspace --all-targets --locked --offline` passed 73 tests. `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` passed.
- **Linux ARM64:** architect ran the same 73-test all-target suite in the existing `rust:1.99-slim` container, `--network none`, repository/cache mounted read-only and target output under container `/tmp`; all passed. This verifies Linux compilation/tests, not a Linux terminal emulator.
- **Independent review:** all five dashboard tests passed separately; reviewer inspected source and final four previews, verified header/caret corrections, and reported no unresolved findings. No application edits by reviewer.
- **macOS real PTY:** [recorded checks and logical screen snapshots](dashboard-pty.json), all 16 checks passed, clean exit code 0. Verified startup without requests, raw mode, real native `/uptime` reply without inference, F2 theme switch, Tab-focused mock play toggle, resize 120×40 → 80×24, help and editable draft during a blocked request, prompt Ctrl-Q exit, exact terminal mode restoration, alternate-screen exit and visible cursor.

The PTY launched the real binary from a temporary directory with explicit profile path and no Tavily key. `/system` triggered only a loopback fake Ollama endpoint, which deliberately held its response; no provider/service forwarding occurred. A supervising process kept the macOS controlling PTY alive long enough to compare terminal attributes after HEART exited. A minimal VT decoder recorded logical text, not colored screenshots. Actual-renderer PNGs supply separate visual evidence.

73 tests comprise 5 dashboard, 2 CLI, 3 smoke, 5 contract, 37 core, 8 system and 13 web tests. Dependencies resolved to Ratatui 0.30.2 / Crossterm 0.29.0; TUI dependencies remain in CLI.

## Boundaries

Opening/rendering makes no module/model/provider requests. System widget values are STATIC DEMO; music is LOCAL MOCK with fixed position and interactive play/track state. Existing real `/disk` and `/uptime` require no model. Other explicitly invoked commands retain their prior model/provider requirements and existing search/phrasing limitations.

A responsive UI does not cancel a remote request: quitting drops the worker channels and exits without joining its blocking deadline. There is no persisted conversation or session history. Profile appearance is optional validated plain data; old/custom profiles receive a neutral lantern theme. Portrait assets remain builtin; editable arbitrary sprites, full expressions and rich images are future work. Very small buffers are handled without panic; practical artwork and side widgets require more room. Font/cell geometry, Unicode and true-color support remain terminal-dependent.

No live Ollama/Tavily request was needed for this milestone. Panic/error cleanup is inspected against Ratatui lifecycle and the scope guard; normal and pending-request exit are verified in a real PTY. No claim is made of an injected runtime panic or interactive testing on every terminal/Linux desktop. No commit created.
