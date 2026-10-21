# 0006 — Terminal home and supplied character portraits

**Status:** accepted, 2026-10-21; implementation tracked in [plan 0013](../plans/0013-terminal-home.md).

## Context and approval

The product owner describes HEART as a cozy personal terminal workspace with a companion, existing tools and custom modules. They explicitly approved this visual milestone through the architect/developer/reviewer workflow. Pastel's supplied pixel-art portrait and existing character JSON are the visual/behavioral sources; generated character approximations are no longer wanted.

## Decision

Remove the large generated character environment. Give dialogue a substantial JRPG frame with actual portrait artwork and separate input, retaining useful System/Music mocks and keyboard controls. Use character-aware semantic colors and a small bundled asset mapping, without a general theme/animation engine. Preserve pixel-art proportions and nearest-neighbor sampling. Prefer terminal graphics supported by Ghostty on Linux/macOS, with a readable labelled fallback where unavailable.

The layout must use today's content naturally and leave a clear composition boundary for a later workspace. Do not reserve a large empty shell panel. Embedded shell/PTY management, real persistent widget integrations and advanced workspace management remain separately scoped future milestones.

## Consequences

Real portraits depend on terminal graphics support and cell metrics. Styled-cell previews can evaluate layout but cannot establish emulator protocol behavior; actual terminal checks and their limitations must be reported separately. Artwork stays in the CLI presentation layer, while profile instructions and module/core boundaries remain intact. New expressions can later add assets without implementing expression selection or animation now.
