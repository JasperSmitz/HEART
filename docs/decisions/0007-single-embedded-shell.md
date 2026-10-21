# 0007 — Single manual shell inside the dashboard

Status: accepted direction, 2026-10-21; [approved plan 0014](../plans/0014-embedded-terminal.md).

The product owner explicitly authorizes a real terminal workspace as the dashboard's primary content. Use a native PTY plus an existing VT parser/Ratatui terminal widget, with the user's interactive shell configuration. `portable-pty` and `tui-term` fit the current Ratatui 0.30 architecture; custom PTY syscalls/emulation and multiplexer frameworks add unnecessary scope.

Shell I/O belongs to a separate CLI component. Manual shell input has no connection to the assistant's capability registry, model prompts or request worker; no AI-facing shell invocation is added. Ordinary child output is parsed into screen cells, never forwarded as host terminal control sequences.

The shell starts focused. F12 is the single reserved context escape; ordinary shell shortcuts, including Ctrl+C, Tab and function keys, go to the child. HEART shortcuts are contextual outside shell focus. The layout prioritizes shell area, keeps existing mock utilities in the wide sidebar, and places compact character dialogue/input beneath the shell. One shell session survives focus/profile changes.

Layout follow-up: [plan 0015](../plans/0015-shell-first-layout.md) moves dialogue
and input into the right sidebar, giving the shell the full content height. It
preserves the input and shell-ownership decisions above.

Bounded scrollback, responsive background I/O, shared resize dimensions and explicit owned-child cleanup are part of the milestone, not deferred polish. ANSI/alternate-screen compatibility is bounded by the chosen parser and widget; do not advertise arbitrary Ghostty extensions to the child. User-spawned detached daemons are outside guaranteed cleanup. No shell configuration, theme/personality behavior or no-flag CLI behavior changes are authorized.

Primary references checked: [portable-pty API](https://docs.rs/portable-pty/0.9.0/portable_pty/), [Unix controlling-terminal/session implementation](https://docs.rs/portable-pty/0.9.0/src/portable_pty/unix.rs.html), and [tui-term widget/API](https://docs.rs/tui-term/latest/tui_term/). In particular, `process_group_leader()` reports the foreground PTY process group, not necessarily the spawned shell's group; cleanup must distinguish them.
