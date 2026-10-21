# 0003 — Explicit commands and editable profiles

**Status:** accepted direction, 2026-10-20; implementation tracked in [plan 0008](../plans/0008-explicit-commands-and-profiles.md).

## Context

The existing local-model routing experiments produced valid structures but unreliable capability choices and literal/default arguments. The user's immediate priority is invoking tools and receiving concise personalized replies in the terminal. Autonomous investigation using multiple tools is a later possibility, not a prerequisite for this experience.

## Decision and approval

Use explicit module-declared commands as the default terminal interface, with deterministic argument binding and independent schema validation before invocation. Keep natural-language routing behind explicit experimental opt-in. Both entry points use the same module execution, result and grounded phrasing/fallback boundaries. Core consumes opaque declarations rather than module-specific commands or imports.

Store characters as locally trusted, editable JSON files, one per profile, discovered at startup. Profiles own instructions, greetings, authored statuses/progress and minimal presentation settings. IDs support selection without adding source branches. Profile text cannot grant tool/action authority or weaken the result validation rules.

The product owner approved this direction with “Let's get that setup then … you have my go ahead” after discussion of deterministic commands and editable personalities. Preserve the current personality wording during extraction; refinement can happen independently afterward.

## Consequences

Tool invocation can be verified independently of model routing quality, and successful tools can return direct results even if local phrasing is unavailable. Natural-language convenience and eventual bounded multi-tool investigation can target the same validated invocation boundary later. Historical routing failures remain recorded and are not retroactively accepted. This decision does not authorize dashboards, telemetry, personality redesign or a model/default change.
