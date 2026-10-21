# Shell-first layout evaluation

2026-10-21. [Approved plan 0015](../plans/0015-shell-first-layout.md).

Status: implemented, verified on Linux/Ghostty and independently reviewed. No
unresolved findings; ready for product-owner hands-on iteration.

The shell is given the full content height between header and footer. Companion
dialogue and HEART input move to the right sidebar below the existing mocked
System and Music widgets. Theme, portrait assets, shell lifecycle and F12 remain
unchanged. A new shell retains normal cursor placement; commands that fill the
screen naturally scroll at the bottom. No artificial prompt offset or shell
initialization command is introduced.

## Layout and responsive behavior

At 100+ columns and 16+ rows, the sidebar is 34 columns wide with a one-column
gutter. The shell extends directly from header to footer. System and Music each
use five compact rows at 100+ columns and 30+ rows; shorter windows hide them.
The sidebar portrait is centered above dialogue, followed by HEART input at the
bottom. System values and Music playback/track controls remain local mocks.

Below 100 columns or 16 rows, a small bottom dialogue/input strip preserves shell
width instead of squeezing it into a narrow column. This fallback does not offer
the full-height side-by-side layout. Portraits disappear when necessary to retain
text; fewer than four rows cannot show an input row. No invalid geometry or hidden
Music/dialogue focus is retained. F12 is unchanged.

## Verification evidence

- Developer: 31 CLI all-target tests, including 13 dashboard tests, passed.
  Added coverage checks nonoverlap and containment across breakpoint/offset
  frames, full-height wide shell, visible portrait/reply/input, focus normalization
  and exact VT cursor coordinates through focus changes. CLI formatting and
  warnings-denied Clippy passed. Full workspace formatting's protected pre-existing
  core discrepancy remains untouched.
- Architect: built the locked CLI; the existing [outer-PTY regression script](../../scripts/check-embedded-terminal.py)
  passed all [25 checks](shell-first-layout/pty-checks.json), including resize,
  normal input/completion/history/Ctrl+C, separate HEART input, shell-state
  persistence, background responsiveness and shutdown safety. CLI formatting and
  whitespace checks passed. No model/provider call was needed.
- Inspected seven deterministic production-renderer previews: Pastel 120×40,
  100×30, 100×16, 80×24, 45×16; Butler 120×40; help at 80×24. The
  [100×30 preview](shell-first-layout/preview-100x30.png) labels its exact-PNG
  composition as simulated graphics, rather than claiming emulator evidence.
- Actual Linux/Ghostty inspection used the user's zsh/Starship configuration:
  [wide workspace](shell-first-layout/ghostty-wide.png), [fullscreen btop](shell-first-layout/ghostty-btop.png)
  and [compact fallback](shell-first-layout/ghostty-compact.png). These are
  unmodified screenshots. At 140×46 outer cells the inner shell is 103×40;
  resizing to 81×24 produces a 79×11 shell. The portrait rendered, `/uptime`
  replied in the sidebar, F12 returned to the shell, and an exported marker
  survived focus changes, btop and resize. Prompt placement reached the last
  shell row after ordinary output. [Recorded checks](shell-first-layout/ghostty-checks.json)
  include clean exit, restored outer modes and unchanged protected hashes.
- The temporary GUI window and KDE focus/resize helpers were removed. btop used
  temporary configuration. No shell configuration, font, profile, artwork, core,
  terminal lifecycle or dependency change was made for this refinement.

The independent reviewer inspected the implementation against the approved plan,
previews and actual screenshots, and independently ran all 13 dashboard tests.
Final review cleared with no findings or blockers; no correction round was needed
and the reviewer edited no project files. Both native developer and reviewer
agents completed. Medium effort was requested with inherited model selection;
backend model/effort telemetry was not available.

macOS runtime remains untested here; the layout introduces no platform-specific
code. User visual preference remains subject to hands-on iteration. No commit
was created.
