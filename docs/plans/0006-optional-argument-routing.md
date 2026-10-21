# 0006 — Preserve explicitly supplied optional arguments

**Status:** implemented and independently source-reviewed; acceptance remains open. Two correction rounds exhausted; return to discussion before further implementation.

The user subsequently authorized the separately scoped [two-step experiment](0007-two-step-routing.md). This plan preserves its single-step implementation/results; [latest observations](../evaluations/two-step-routing.md) describe the current experimental code.

## Scope and authorization

The user authorized this focused follow-up with “You can proceed” after plan 0005 exhausted its correction rounds. Diagnose and correct the generic routing behavior that silently omits explicitly supplied disk paths. Preserve prior failed observations and the existing default model. No commits.

Developer first isolates actual structured-generation behavior with a bounded local-model probe, then implements the simplest generic fix. An inference-facing representation may differ from module arguments if conversion is explicit, validated and independent of module names. Preserve declared optional/default semantics at the module boundary. Do not introduce capability-specific parsing, guessed paths, extra inference calls, new dependencies, cloud use, model downloads, dashboard, telemetry or profile work.

## Acceptance and verification

- Both installed 2B and 4B models faithfully route an omitted path, explicit root, an existing absolute path, a relative path, a path containing spaces, and a nonexistent path. Test literal values and actual module outcomes separately: omitted means cwd; nonexistent means a safe failure rather than a cwd reading. Use temporary fixtures where necessary; never inspect user file contents.
- Empty-input OS/uptime, required web query and unsupported requests retain validated behavior. Count invocations using the credential-free web stub; no real Tavily calls.
- Keep capability/argument binding and discriminator-first serialization, independent module-boundary validation, safe fallback and phrasing guards. Regression tests exercise the actual routing schema/conversion and invalid representations.
- Run formatting, warnings-denied Clippy and all-target workspace tests, plus Linux container checks where practical. Preserve historical results; record new live observations with arguments and outcomes, not just selected IDs.
- Independently review implementation and evidence; at most two correction/recheck rounds for unresolved findings, then escalate.

## Delegation

Developer requested as GPT-6.1 Sol/high: difficult debugging demonstrated by two unsuccessful corrections and reproduction on both models. Reviewer normally Sol/medium, increased only for a concrete need. Use the explicit-role fallback when the current tool lacks a native role selector; requested settings are not verified execution telemetry. Architect owns final acceptance and documentation status. Developer reads project instructions and relevant lessons, implements within scope and reports evidence; reviewer does not edit application code and may record substantial lessons only.

## Execution and unresolved acceptance

Implemented a generic required-nullable inference projection, aligned metadata and strict inverse conversion before original module validation. Required/empty-input shapes, discriminator-first order and module string-or-absent inputs remain unchanged. Expanded the credential-free smoke example to check twenty cases per model, with literal dispatch arguments, independent canonical-path checks and outcomes. Historical failed observations remain intact.

All 33 all-target workspace tests passed on macOS and Linux ARM64. Formatting and warnings-denied Clippy passed; independent source review found no conversion or boundary defect. The architect's actual-model acceptance still fails with the unchanged default 2B model after two prompt-only corrections: a relative `.` became `/`, and both weather searches invoked native modules. Schema correctness and successful native reads do not satisfy the routing criteria. The reviewer confirmed these as blocking acceptance findings.

See [full current results and preserved runs](../evaluations/optional-argument-routing.md): final 2B passes 17/20 and final 4B passes 20/20. The 4B comparison cannot substitute for default-model acceptance. No further fix is authorized under this plan after the two correction rounds. Recommend discussing a separately scoped routing design/evaluation rather than further prompt tuning. A two-step selection/extraction experiment would add a local inference call and needs its own approval and latency/correctness criteria. Current follow-up code remains unaccepted, including the 2B web regression. No model default changes, real Tavily calls, model downloads, dashboard/telemetry work or commits. Developer/reviewer requested Sol/high and Sol/medium respectively via explicit-role fallback; execution telemetry was not inspected.
