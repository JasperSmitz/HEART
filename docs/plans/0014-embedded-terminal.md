# 0014 — Single embedded terminal workspace

Status: implemented, verified on Linux/Ghostty and independently reviewed, 2026-10-21. Ready for product-owner hands-on iteration.

## Approved scope and decisions

Implement one real interactive shell in the dashboard, using a PTY and an existing VT parser/renderer rather than writing a terminal emulator. Investigated `portable-pty` 0.9.0 and `tui-term` 0.3.4: their published APIs support native PTYs, resize, VT100 parsing and the Ratatui 0.30 component versions already in HEART. Prefer these compatible versions with only necessary features; justify any substitution. Keep the component in CLI, separate from dashboard layout, assistant execution, personalities and capability modules.

Spawn the configured `$SHELL` as an interactive shell, inheriting the environment/cwd and ordinary rc files without injecting shell commands, replacing prompts or disabling user configuration. Use an honest supported inner TERM (normally `xterm-256color`) rather than advertising Ghostty-only capabilities; remove outer-terminal identity hints where necessary. Preserve PATH/ZDOTDIR/Starship configuration and aliases. Shell process startup is a deliberate dashboard feature; it is never an assistant tool. Neither core, model, module results nor assistant requests receive a terminal writer or shell execution API.

The terminal gets the majority of the central height. Header stays above it; System/Music retain existing mocks in the wide sidebar. A compact double-framed companion box below the terminal retains Pastel's actual artwork, profile identity and latest greeting/response, with separate HEART input. Preserve accessible bounded dialogue history if useful, but do not let its sizing dominate the shell. Shared layout calculations must drive both terminal rendering and PTY/parser dimensions. Narrow/tiny layouts hide utilities/portrait and shrink optional companion content first, keeping the shell usable. Missing shell/start failures and shell exit must leave HEART controls usable and show a clear state, without implicit shell restart.

## Focus and input contract

Start with shell focus. Reserve **F12** as the one global escape/toggle between terminal and HEART input. It avoids normal editing/completion/job-control chords and default Neovim/btop bindings; the outer terminal must deliver it. Shell focus sends ordinary keys directly to the child: Ctrl+C, Ctrl+D/Z/Q, Tab, arrows, Escape, F1/F2 and modified navigation included. HEART's quit/help/profile shortcuts run only outside shell focus. Show the active context in both border/header and footer; F12 always leaves fullscreen child programs. Tab/Shift-Tab in HEART cycle visible HEART controls and shell; do not steal shell Tab. Document Shift+PgUp/PgDn (shell focus) as bounded host scrollback and how to return to live output. Correctly encode application-cursor modes and bracketed paste when the child enables them; do not send arbitrary host events as assistant requests.

## Reliability and lifecycle

Use a bounded scrollback and bounded output/input handling. A blocked child or heavy output cannot hold the render/input thread indefinitely: background I/O and a bounded frame-drain budget, with orderly backpressure and explicit error/EOF status. Preserve ANSI colors/styles and cursor visibility/position; handle alternate-screen applications and ordinary device/cursor-report requests through the parser where needed. Never pass unparsed child escape sequences to the host terminal (including OSC clipboard/title and graphics). Refresh parser/PTY dimensions on resize; do not treat output as instructions to HEART.

Own the child and PTY lifecycle explicitly. Closing HEART must stop/reap the shell and terminate its attached foreground job, with bounded waiting/escalation, close writer/master handles and avoid unbounded thread joins. Inspect process-group/session behavior on Linux/macOS; do not claim arbitrary daemon cleanup. Preserve prompt recovery after Ctrl+C and interactive state across focus changes. Use safe dependency APIs, or small justified Unix-specific process signaling behind platform gates if necessary. No cleanup may signal HEART's own process group or unrelated processes.

## Acceptance and verification

- Launch dashboard in Ghostty, use configured zsh/rc aliases/environment/Starship when installed, run shell commands, switch to HEART input for `/uptime` and return with shell cwd/state intact.
- Real PTY integration checks exercise interactive input/history, Tab completion, Ctrl+C during a foreground job, ANSI attributes, resize, scrollback, shell EOF/exit, process cleanup and responsiveness under long-running/high-output commands.
- Focus tests demonstrate Ctrl+C/Tab/F1/F2 do not invoke HEART when the shell owns input; F12 safely returns control, existing HEART controls remain available, paste is routed to exactly one context and AI has no shell writer.
- Functional layout checks cover wide/minimum-wide, 80×24, small/tiny buffers, actual supplied portrait containment and hidden-control focus normalization. Preview construction spawns no shell, engine or service.
- Run workspace/all-target tests, CLI formatting and warnings-denied Clippy. Preserve pre-existing core formatting difference and user files. Independent reviewer evaluates correctness, portability, process lifecycle, input boundaries and maintainability, with at most two correction/recheck rounds per unresolved issue.
- Architect performs independent end-to-end PTY and, where practical, real Ghostty screenshot checks; try an installed fullscreen program (btop is available, Neovim currently is not) without installing applications solely for this milestone. Record source/tests versus actual macOS runtime evidence honestly.

## Non-goals and preservation

No multiple shells/tabs, multiplexing, full emulator implementation, assistant shell automation, new module/integration, telemetry/audio, character animation, new artwork, font or user-shell configuration changes, model/provider changes or commits. Existing uncommitted visual milestone files remain the baseline. Preserve `profiles/cat.json`, portrait bytes and the user's existing `crates/core/src/profiles.rs` edit exactly. The architect owns direction/decision/evaluation documentation; developer owns CLI component/integration/tests/dependencies/preview.

## Results

Implemented one interactive `$SHELL` via portable-pty 0.9.0, tui-term 0.3.4/vt100 0.16.2 and small justified Unix fd/signal handling. Shell state/I/O lives in its own CLI component, with no engine/module/request-worker writer access. The main terminal and kernel/parser use shared layout dimensions; Pastel's compact portrait/latest answer and separate HEART input remain below it, with existing wide-layout widget mocks. F12 switches contexts while shell keys retain their normal meaning. The no-flag CLI, user profile configuration and existing core edit remain unchanged.

Native developer and separate reviewer used Sol/high for the PTY/lifecycle work and review; selection requests are not backend execution telemetry. Developer's final **91 workspace/all-target tests**, warnings-denied Clippy and CLI formatting passed. Workspace formatting preserves the known pre-existing core edit. Architect's independent reproducible [outer-PTY verifier](../../scripts/check-embedded-terminal.py) passed **25 assertions**; actual Ghostty inspection verified user zsh rc/aliases/Starship, ANSI output, F12/HEART uptime, persistent shell cwd/export, live resize and real btop running alongside Pastel. Both controlled fixtures and actual user configuration are identified separately in the evidence. Preview construction starts no shell/services.

Three bounded discoveries were resolved:

- vt100 omitted the standard HVP cursor-position command used by btop. A narrow callback maps parsed HVP to supported CUP at the correct input boundary; regressions cover fragmented reads and control-string payloads. Refreshed real btop rendering is coherent.
- Architect found a leaderless foreground pipeline could survive cleanup. Correction 1 validates the foreground group through the owned PTY rather than requiring its leader PID to exist, and finishes that job before stopping the shell. The reviewer and independent final verifier cleared the regression.
- Reviewer found P1: portable-pty's Unix writer destructor could submit unentered shell input by sending newline/EOF during quit. Correction 1 for this separate issue uses an owned duplicate wrapped in `File`, whose destructor only closes. The temporary unsafe fd borrow is documented and limited to duplication while the master is live. Real-PTY pending-input checks with ignored HUP/TERM passed alongside cleanup and reaping.

The reviewer cleared the final correction after ten terminal tests, its own pending-input rechecks, Clippy, CLI formatting and protected hashes; inspected six previews and actual screenshots, and found no unresolved issues. One substantial destructor-side-effect lesson was recorded by the reviewer. No issue required a second correction round. Final acceptance is within the documented compatibility limits; product-owner visual/workflow preference remains the next iteration, not a claim of a complete terminal emulator.

Source portability covers Linux/macOS, but actual macOS runtime remains untested here. Child graphics, mouse/OSC clipboard passthrough, extended keyboard protocols and full resize reflow are outside this bounded integration; fullscreen programs require adequate pane dimensions. Only btop was available for the fullscreen experiment. No multiplexing, shell automation, live widget integration, new module, animation, font/config change or commit was added. See [evaluation, screenshots and preserved failure/recheck evidence](../evaluations/embedded-terminal.md).
