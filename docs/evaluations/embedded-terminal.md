# Embedded terminal workspace evaluation

2026-10-21. [Plan 0014](../plans/0014-embedded-terminal.md) · [decision 0007](../decisions/0007-single-embedded-shell.md).

Status: implemented, verified on Linux/Ghostty and independently reviewed. No unresolved findings; ready for product-owner hands-on iteration.

The approved direction is one manually controlled shell in a native PTY, a dominant terminal workspace and compact companion dialogue. F12 switches between shell and HEART; normal shell keys must not invoke assistant actions. System/Music remain mocks. No AI shell API, multiplexing, new module, shell-configuration change or commit is included.

## Environment and preservation

Linux/Wayland with Ghostty 1.3.1-arch2, Rust 1.96.0, zsh 5.9.2, Starship 1.26.0 and btop 1.4.7 are available. Neovim is absent and will not be installed solely for this milestone. macOS compatibility must be source-reviewed; actual macOS runtime evidence is not yet available.

Existing visual work is uncommitted and remains the baseline. Protected SHA-256 values: core `profiles.rs` user edit `e9a0cdb9a1cefb7b55d47df874a166fc329f235cab47185486f7baddaa5fc140`; `profiles/cat.json` `b9e3a0d2689333515cbb9abf8d9b2168551fc979f93febe854ea9749fa5162b3`; Pastel PNG `8e462a36024c876d856cd5388e14238506c0bd9bad4fc6aab25ece6639c29108`. Baseline workspace formatting discrepancy in the user core edit remains protected.

## Implementation

The CLI owns one native controlling PTY and interactive configured `$SHELL`, using portable-pty 0.9.0, tui-term 0.3.4/vt100 0.16.2 and small Unix descriptor/signal handling. Shell rc files, cwd, PATH, aliases and Starship configuration are retained. Inner TERM is `xterm-256color`; outer emulator identity and size hints are removed. The assistant engine/worker has no terminal writer. Shell I/O is a separate component from layout, personality and module execution; the no-flag CLI is unchanged.

Shell focus is the default. F12 toggles shell/HEART input; Ctrl+C/Q, Tab, arrows and function keys otherwise belong to the child. Focus is visible in header, border and footer. Shift+PgUp/PgDn use bounded 2,000-row scrollback; typing returns live. The renderer and PTY/parser use the same calculated dimensions. Companion artwork/dialogue are compact beneath the shell; utility mocks hide at narrow sizes. Latest answer text stays visible, and focusing dialogue exposes bounded assistant history.

Background nonblocking I/O, bounded input/output queues and a 64 KiB-per-frame drain budget preserve responsiveness. Shell paste is limited to 64 KiB per event and brackets are delivered according to the child's mode. Unsupported OSC/DCS/APC/PM strings are dropped incrementally, never passed to the host or buffered without bounds. The existing parser identifies a small HVP-to-CUP cursor-position alias; no separate emulator was introduced.

## Automated and independent evidence

- Developer's final locked workspace/all-target suite passed **91 tests**, including real PTY, mode/escape, layout/input, lifecycle and regression checks. [Recorded counts](embedded-terminal/automated-checks.json). CLI formatting and warnings-denied Clippy passed; full workspace formatting retains only the protected pre-existing core-file discrepancy.
- Architect's reproducible [outer-PTY verifier](../../scripts/check-embedded-terminal.py) passed **all 25 assertions** against the built dashboard: [results](embedded-terminal/pty-checks.json). It uses temporary zsh rc/aliases/history settings and files, preserving the user's configuration. Assertions cover configured shell startup, aliases/environment/TERM, history, completion, ANSI color, F12/assistant input, cwd/export state, Ctrl+C, resize, scrollback controls, response during a foreground job, ordinary cleanup, leaderless pipeline cleanup, unsubmitted input, EOF/start-failure controls and exact terminal restoration.
- The verifier inspects command-created fixture files and owned processes, as well as whitespace-normalized incremental renderer output. It is not a screenshot emulator. Temporary inference points to an unused loopback endpoint and provider credentials are absent; `/uptime` requires no inference. No model or search service was needed.
- Independent reviewer inspected source, six previews and actual Ghostty screenshots, exercised focused tests and independently reproduced the two shutdown cases described below. Final recheck cleared P1 after correction 1: all ten terminal tests, independent unsubmitted-input cases with HUP and HUP/TERM ignored, Clippy, CLI formatting and protected hashes passed. [Independent recheck evidence](embedded-terminal/reviewer-pending-input-checks.json). No unresolved findings; reviewer recorded one substantial lesson in [LESSONS](../LESSONS.md).
- Architect CLI formatting, whitespace and protected-file hashes passed. No shell configuration, core, module or personality behavior was rewritten; no commit was created.

Reproduce after building, on Linux/macOS with Python 3 and zsh:

```sh
cargo build -p heart --locked
python3 -B scripts/check-embedded-terminal.py
```

The optional `--output-dir` must be new or empty. Test files and raw captures remain in a temporary directory; failed cleanup checks only kill a recorded fixture child after independently checking its session identity.

## Actual Ghostty inspection

These are unmodified screenshots of the real Linux Ghostty window, captured with Spectacle and inspected by the architect:

- [Configured zsh/Starship startup](embedded-terminal/ghostty-startup.png), at 139×45 cells.
- [HEART focus and Pastel's authored uptime alongside shell commands/ANSI output](embedded-terminal/ghostty-heart-input.png). The actual user rc loaded seven aliases and its Starship prompt.
- [Live compact layout](embedded-terminal/ghostty-compact.png), 81×24 outer cells and 79×11 inner shell cells. A manually exported `cozy` marker and cwd survived switching contexts and the fullscreen experiment.
- [Fullscreen btop after the compatibility fix](embedded-terminal/ghostty-btop.png).
- [Pastel replying while btop remains active](embedded-terminal/ghostty-btop-companion.png), with btop arrow navigation also exercised.

[Recorded Ghostty checks](embedded-terminal/ghostty-checks.json) include real user rc/prompt, ANSI output, F12/assistant replies, retained shell state, live resize, btop alternate-screen return, clean exit and outer mode restoration. The verification harness forwarded real terminal input/replies through an outer PTY; it is not a production AI shell API. btop's config was isolated with a temporary XDG_CONFIG_HOME; the System/Music sidebar remained STATIC DEMO/LOCAL MOCK. No application/font/shell config was installed or changed. The test windows and temporary KDE screenshot helpers were removed afterward.

Deterministic production-renderer [wide](embedded-terminal/preview-wide.png) and [80×24](embedded-terminal/preview-80x24.png) previews use mock terminal cells and label composed PNG portraits as simulated. They open no shell or services; they do not establish emulator protocol behavior. Actual screenshots above supply separate evidence.

## Discoveries and corrections

1. **Fullscreen cursor positioning:** initial btop output was garbled. An isolated raw-PTY capture counted 397 `CSI row;col f` HVP sequences and zero `H` CUP sequences. vt100 0.16.2 supports CUP but omits HVP. The narrow callback alias applies CUP at each parsed HVP boundary, including fragmented reads, without interpreting string payloads. The refreshed real screenshot shows coherent borders, graphs, colors and layout; F12 and HEART remained responsive. This resolved the observed btop issue without expanding into a general emulator.
2. **Leaderless foreground pipeline:** architect [initial evidence](embedded-terminal/pipeline-initial-failure.json) showed `true | sh -c 'trap "" HUP TERM; …; exec sleep 30'` surviving HEART exit when the original group leader had exited. Checking `getsid(group_id)` incorrectly required that leader PID to exist. Correction 1 validates foreground attachment through the owned PTY before signaling and stops the foreground group before the shell, keeping bounded escalation and own/unrelated-group exclusions. The independent reviewer and final outer-PTY regression confirmed cleanup.
3. **Unsubmitted text executed on exit (P1):** reviewer [initial evidence](embedded-terminal/pending-input-initial-failure.json) showed pending `printf unexpected > shutdown-executed.txt` executing during quit after the shell ignored HUP. portable-pty's Unix writer destructor sends newline/EOF. Correction 1 for this separate issue replaces it on Linux/macOS with an owned duplicated descriptor wrapped in `File`, whose destructor closes only. A documented temporary `BorrowedFd` borrow is limited to duplication while the master owns the live descriptor. The final independent 25-check run and reviewer rechecks confirmed pending input is not executed and both lifecycle regressions still pass. The reviewer cleared the finding; no second correction round was needed for either shutdown issue.

## Remaining limits

This is a bounded VT widget, not Ghostty's complete emulator: child images, OSC clipboard/title passthrough, mouse reporting and extended keyboard protocols are unsupported. Existing scrollback cells do not promise Ghostty-style resize reflow; programs should redraw for new dimensions. Fullscreen programs have their own minimum screen sizes; use a sufficiently large outer window. Only btop was exercised here, not Neovim. F12 must reach HEART from the host terminal and is reserved inside child programs.

Linux/Ghostty and zsh were tested directly; macOS was source-reviewed but not run here. The cleanup guarantee covers the owned shell and attached foreground job, with bounded HUP/TERM/KILL escalation and reaping; deliberately detached daemons are outside that guarantee. No multiplexing, automatic restart, AI shell access, real music/telemetry, new module or animation is implemented. Product-owner hands-on iteration remains the next step.
