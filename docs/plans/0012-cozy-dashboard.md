# Cozy terminal dashboard prototype

Status: prototype implemented and independently reviewed; product-owner visual evaluation pending.

## Approval, reference and scope

The product owner explicitly approved planning/implementation through the architect → developer → reviewer workflow. Primary inspiration is [dashboard-reference.png](../design/dashboard-reference.png), inspected by the architect: rose highlights, warm dark room, prominent cat character, framed JRPG dialogue and supporting system/music panels. Adapt that hierarchy to terminal cells rather than reproduce the raster image. Build a tangible prototype for visual iteration, not a finished desktop or monitoring application.

The existing plain console remains the no-flag default. Add `--dashboard` for the new TUI; default to Cat there unless `--personality ID` is explicit. Preserve all existing commands/options and their real on-demand behavior. System/music *widgets* show visibly marked demo data and never invoke modules or poll the OS. A user-entered `/disk`, `/uptime`, `/system` or `/search` still uses the existing registered capability; no requests occur merely from opening/rendering the dashboard.

## UI direction

- Wide layout: compact HEART brand/status/personality header; left supporting System and Music panels; substantially larger right character-room scene and JRPG dialogue; full-width input and short key hints. Character and dialogue occupy the majority of the useful area.
- Cat: midnight plum/cocoa surfaces, warm cream text, rose borders and pink accents, small gold highlights. Butler: navy/charcoal, cream, soft gold and restrained teal. Ready/busy describes HEART's UI/request state, never an unverified claim of provider connectivity.
- Render original stylized terminal-cell pixel portraits and small cozy decorations (window/stars/mug/books/plants), with no mandatory image protocol or special icon font. Cat should read as a character with hair/ears/face, not just an ASCII cat emoticon in an otherwise empty panel. Provide a distinct simple Butler portrait. A small dialogue portrait reinforces the speaker. Simple blink/speaking expressions are welcome if inexpensive; allow animations to be disabled.
- Music shows demo album-art/record placeholder, track/artist, position and playback controls. Focused keyboard controls alter local mock state only; no audio or integrations. System shows fixed demo CPU, memory, disk and temperature values with clear units and restrained gauges, not live readings.
- Responsive hierarchy: at common 100–120-column terminals show the full room and side widgets; around 80×24 prioritize character/dialogue/input and compact or hide supporting widgets; smaller sizes retain usable dialogue/input/help rather than overflowing or trapping focus. Use saturating/bounded layout math. Exact breakpoints may follow actual rendering evidence.
- Input is functional and editable, not a decoration. Tab/Shift-Tab focus visible interactive areas; Enter submits/activates; F2 cycles discovered personalities/themes; F1 shows help; Ctrl-Q/Ctrl-C and `/exit` leave. Focused music controls toggle playback/track mock state. Dialogue supports scrolling long replies/source lists. Preserve ordinary typing (letters like q and space are not global quit/play shortcuts while editing), UTF-8-safe editing, and visible input cursor/viewport. Document final key choices.

## Architecture and assets

Use Ratatui with its portable Crossterm backend inside the CLI frontend; no TUI types or dependencies in core/modules. Current official docs specify Ratatui 0.30.2 and matching Crossterm 0.29, compatible with installed Rust 1.99.1; verify resolved versions and lock them normally. Dependencies are purpose-bound; no UI framework, music/image engine, async runtime or telemetry library is needed.

Share command/session handling between console and dashboard rather than maintaining divergent command parsers. Keep rendering, UI state/input, theme/portraits and mock widgets in small focused modules, without inventing a dynamic widget/plugin framework. On-demand engine work runs in a single worker behind channels so network/model latency never blocks drawing, input, help or quitting. Bound submissions to one in-flight request; preserve typed input while busy. Capture the selected profile per request and label any eventual result consistently even if the UI theme changes meanwhile. Preserve safe diagnostics without printing over the alternate screen. Main-thread UI owns widget state; demo animation ticks are UI state updates, not system polling.

Extend profiles backward-compatibly with a small optional presentation/appearance section: semantic palette colors and a builtin portrait asset key, plus a display name if useful. Plain profile data must not import Ratatui; validate/fallback clearly on unsupported assets/colors. Existing custom profiles remain loadable, selectable and render with a neutral/default appearance. Cat/Butler demonstrate reusable styling. Keep existing instructions, greetings, progress and authored templates intact; do not tie rendering to profile IDs or add a large asset schema. Builtin portraits may be stored as small code/data assets behind the character widget, with expression states; richer image protocols remain a later option.

Do not change terminal fonts or settings. Recommend Departure Mono for pixel flavor and Cascadia Mono for a softer readable alternative, linking official sources. Terminal font/line-height is user-controlled. Document true-color/Unicode expectations and any fallback; no Nerd Font requirement.

## Acceptance and verification

- Runnable `cargo run -p heart -- --dashboard` starts a cozy Cat home; Butler and custom profiles can be selected by startup option and existing/dedicated shortcuts. Header, system/music mocks, character, dialogue/portrait and input are visible at the primary size. Mock labels are unambiguous.
- Existing plain-console tests/commands pass. Dashboard help/profile/exit commands work, on-demand native module replies and source lists reach dialogue, and opt-in NL retains its documented limitations. No automatic model/provider/native module calls, real music, system refresh or new capabilities.
- Input editing/history/focus, mock playback controls, theme changes, dialogue scrolling, viewport resizing and responsive hidden-widget focus work. Long input/results and Unicode don't panic or corrupt buffer positions. Mock-worker tests prove UI remains responsive during slow work and profile-tagged replies are consistent.
- Raw mode/alternate screen/cursor are restored on normal exit, startup errors and panic paths using the supported terminal lifecycle. Detect non-TTY interactive launch and fail clearly. Quitting during a pending request returns promptly rather than waiting for the worker's network deadline.
- Meaningful deterministic Ratatui TestBackend checks cover Cat/Butler, wide/80×24/compact/tiny sizes, long content and help/focus state. Inspect rendered output visually, not just geometry assertions. Provide a small deterministic preview example using the actual renderer and mock state (SVG or styled-cell export is sufficient); it must not initialize services/read credentials or make requests. Architect produces a standalone preview artifact for the user's visual evaluation.
- Run all-target workspace tests, formatting and warnings-denied Clippy; use practical Linux checks. Independently review usability, portability, maintainability, input/worker behavior and visual hierarchy. Architect runs a PTY smoke for focus, profile switch, native command response, resize and terminal restoration without Tavily/model calls. Preserve real evidence and report limits of simulated visuals versus an actual terminal.
- README documents startup, controls, demo boundaries, profile appearance and fonts. Architecture/roadmap reflect the implemented state; no claims of telemetry/music integration or universal terminal behavior. Record resolved dependency versions and visual evidence in the plan/evaluation.

## Non-goals

Real music playback, background system polling/telemetry, additional modules, dashboard networking on startup, automatic model downloads, provider/model changes, fixing existing search/phrasing limitations, GUI/desktop integration, image protocol support, font installation/control, full animation/asset framework, commits or unrelated cleanup.

## Roles and results

Developer: Sol/high requested because this change combines event handling, terminal lifecycle, blocking-worker coordination and shared command flow. Reviewer: separate Sol/medium requested for focused independent implementation/visual review. Used fresh-agent fallback in this session; requested settings are not verified backend execution telemetry. At most two focused correction/review rounds, then return architectural uncertainties to the user.

Implemented with Ratatui 0.30.2 / Crossterm 0.29.0. Final all-target workspace suites passed **73 tests on macOS ARM64 and offline Linux ARM64**; macOS formatting and warnings-denied Clippy passed. A real macOS PTY verified native command output, theme/focus/mock controls, resize, responsiveness during a deliberately blocked local fixture request, prompt quit and terminal restoration. No live model/provider requests were used. Independent reviewer cleared compact-header and input-caret fixes and inspected final previews; no unresolved findings. Portrait containment/miniature assets preserve the responsive hierarchy. No commit created.

See [evaluation and visual previews](../evaluations/cozy-dashboard.md) for evidence, commands and limitations. Product-owner look/feel acceptance remains open; telemetry and audio remain deferred.

References checked during planning: [Ratatui installation/version compatibility](https://ratatui.rs/installation/), [initialization/restoration](https://docs.rs/ratatui/latest/ratatui/fn.init.html), [Departure Mono](https://departuremono.com/), [Cascadia family](https://github.com/microsoft/cascadia-code).
