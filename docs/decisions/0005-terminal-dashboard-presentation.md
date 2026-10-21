# 0005 — Cozy terminal dashboard frontend

**Status:** accepted direction, 2026-10-20; [plan 0012](../plans/0012-cozy-dashboard.md).

## Context and approval

The user explicitly authorized an interactive terminal dashboard inspired by [their reference](../design/dashboard-reference.png): a cozy JRPG-like home with character and dialogue central, supporting system/music widgets and reusable Cat/Butler styles. They specifically excluded real music playback, background system polling and additional modules until visual iteration establishes the experience.

## Decision

Add an opt-in Ratatui/Crossterm CLI frontend with portable terminal-cell portraits and restrained animation, preserving the plain console and shared command/session behavior. Keep UI state/rendering/mock widgets in the CLI, with no TUI dependencies in core or capabilities. A single on-demand worker prevents existing blocking tools from freezing terminal interaction.

Profiles may carry optional frontend-neutral appearance metadata, while portrait rendering lives in the frontend. Builtin Cat and Butler art/themes demonstrate reuse; older custom profiles use a fallback appearance. Widgets display labelled demo data; typed commands retain actual existing capabilities. Startup/rendering never implies system refresh, real media control or provider connectivity.

## Consequences and alternatives

Ratatui supports predictable cell-based rendering and test buffers without requiring a browser, image protocol or icon font. Crossterm covers Linux/macOS terminal interaction. The reference's raster detail is adapted into small original pixel portraits/decorations; rich terminal images are postponed. Font selection remains the user's preference. Mock widgets can later consume capabilities without putting telemetry/music integration into this milestone. Visual acceptance remains iterative and belongs to the product owner.
