# 0015 — Shell-first layout refinement

Status: implemented, verified and independently reviewed, 2026-10-21. Scope was
explicitly approved by the user's layout refinement request.

## Scope and decisions

Keep the header and existing theme. Give the left shell the entire content height
between header and footer. Stack System, Music, compact character dialogue, and
HEART input in a wider right sidebar. Preserve the real portrait, double dialogue
border, latest response, music controls, and F12 focus switching.

Use one geometry calculation for drawing and PTY dimensions. Compact widget
rendering should reserve useful space for dialogue. At smaller sizes prioritize
shell dimensions, collapse utility widgets first, and keep HEART input reachable
with the existing focus control. Document any necessary tiny-screen fallback.

Retain actual PTY cursor positioning: a fresh shell starts where its own prompt
places the cursor; after output fills the terminal it scrolls naturally at the
bottom. Do not inject newlines, modify shell configuration, or offset the VT
screen to make an initial prompt appear at the bottom. This preserves interactive
and fullscreen application semantics while removing the former mid-screen limit.

## Acceptance criteria

- At ordinary dashboard sizes the shell spans all content rows below the header;
  companion and input never reserve rows beneath it.
- Sidebar order is System, Music, character dialogue, HEART input. Pastel's real
  artwork and latest message remain visible in the ordinary wide layout.
- Existing shell input, resizing, scrollback, focus switching, assistant commands,
  profile switching, and widget mock behavior are preserved.
- Narrow/short layouts avoid invalid rectangles and retain usable shell and
  reachable HEART interaction; hidden widgets cannot retain focus.
- No changes to personality instructions, artwork, core behavior, PTY process
  lifecycle, dependencies, or unrelated existing user edits.

## Non-goals

No new workspace features, integrations, terminal protocols, input shortcuts,
animation system, shell initialization changes, or visual identity redesign.

## Verification

Developer: geometry/focus/rendering regressions, targeted CLI tests and formatting.
Architect: inspect production-renderer previews at wide and compact sizes; run
the existing outer-PTY regression script and practical Ghostty checks/screenshots
where available. Reviewer: independent correctness, usability, layout balance,
portability, and maintainability review against this plan. No commits requested.

## Roles

Architect coordinates design, documentation and external verification. Native
developer and reviewer definitions are used with Medium reasoning and inherited
model selection (no model override). Agent completion evidence and final review
status will be recorded with the results.

## Results

Developer Chandrasekhar (`01a15fc3-a8bc-7271-b7d1-70da911e972a`) implemented the
layout in `dashboard/render.rs` and geometry/focus regressions in `dashboard/mod.rs`.
Reviewer Kant (`01a15fc5-48c5-75e3-a4a8-a1419be6338b`) independently inspected
source, previews and actual screenshots and ran all 13 dashboard tests. Final
review: clear, no findings or blockers; no correction rounds needed. Both native
agents completed; Medium effort was requested with inherited model selection,
whose backend execution telemetry was not available.

The sidebar is 34 columns at 100+ columns/16+ rows, with compact utilities at
30+ rows. Below that side-by-side threshold, bottom dialogue/input preserves
shell width. This documented responsive exception retains the prior practical
narrow-screen behavior rather than forcing an unusable narrow shell. At normal
dashboard sizes the shell reaches from header to footer; actual Ghostty output
confirmed the prompt reaches its bottom normally after scrolling.

31 CLI all-target tests, CLI formatting and warnings-denied Clippy passed.
Architect's existing outer-PTY verifier passed all 25 checks. Actual Linux/Ghostty
inspection verified real portrait, zsh/Starship, F12/uptime, full-height shell,
btop, retained export after resize and clean exit/terminal restoration. No macOS
runtime claim is made. Protected user core/profile/artwork and terminal component
hashes are unchanged. No dependency, shell configuration or font changes, or
commits. See [evaluation and screenshots](../evaluations/shell-first-layout.md).
