# 0004 — Phrasing and fallback polish

**Status:** implemented and independently reviewed; returned to discussion mode. Model variability and the 2B fixture-routing failure remain documented limits.

## Scope and authorization

The user approved a polishing pass after hands-on testing showed awkward cat phrasing, internal “sources in the evidence” wording, and a long clipped source fallback on the Zelda composer question. The user explicitly deferred external/editable profiles to a later discussion. Profiles remain built-in Rust definitions for this pass. No commits are authorized.

## Acceptance criteria

1. Add opt-in CLI diagnostics that distinguish accepted phrasing, inference failure/timeout, malformed output and specific deterministic rejection reasons. Normal answers remain uncluttered. Diagnostics contain safe categories/timing only, not keys, full prompts, retrieved text or raw transport errors. No persistent interaction logs.
2. Keep the existing fallback and security boundaries: one registered module call, unchanged sources, no automatic retries or extra model calls, no cloud fallback. Preserve contraction, Unicode-number/sign, argument/schema and injection regressions. If a guard changes, justify it with a concrete false-positive case and appropriate regression coverage; do not loosen checks merely to improve acceptance counts.
3. Make the deterministic web fallback readable as an attributed source excerpt, preferring a complete short sentence and otherwise a word-boundary excerpt with an honest ellipsis. Avoid repeated long titles and partial words. Do not invent a synthesized answer or claim an excerpt conclusively answers the question. Keep canonical factual text separate from presentation labels if needed for validation, without module-specific branches in core.
4. Refine the two built-in prompts for direct answers without internal evidence/process jargon. Cat voice becomes warm, playful and lightly confident; avoid obligatory reluctance, teasing that changes factual polarity, and forced “nya” on every answer. Butler remains brief. Character may appear more strongly in authored greetings/progress than factual replies. Do not add another inference call just for style.
5. Extend the controlled evaluation to cover the user-reported Rust package-manager and Ocarina composer cases alongside temperature, fare and negation. Use explicitly synthetic, fixed evidence, with output/timing/fallback-reason records sufficient for manual assessment. Compare the already-installed 2B and 4B Q4 models on the same cases; report facts, style, fallback frequency and latency. Keep 2B as default while evaluating; any recommendation to change it returns to discussion.
6. Run formatting, warnings-denied Clippy and meaningful workspace tests. Independent reviewer checks the focused changes and factual guard regressions. Live model comparison is separate from credential-free tests; reuse an existing user-owned Ollama service without stopping or reconfiguring it. No additional Tavily calls are necessary to reproduce these cases.

## Non-goals and delegation

External profile files or `/pirate`, profile discovery/loading, module expansion, dashboard, chat memory, changing the model default, new inference/search dependencies, runtime installation/downloads and commits are outside scope.

Developer: request Sol/medium for this normal multi-crate feature/debugging task. Reviewer: request Sol/medium for focused independent review. Use the explicit-role fallback if this interface lacks native role selection; report requests separately from observed settings. Keep handoffs concise and limit correction rechecks to affected behavior. Architect owns final model measurements and documentation of evidence/limits. Maximum two correction/recheck rounds before escalation.

## Execution and evidence

Implemented all six criteria within the stated limits: safe opt-in diagnostics; unchanged bounded inference/module flow; canonical text separate from source presentation; readable excerpt fallback; subtler built-in cat/direct prompts; and a five-case, two-profile evaluator with fixture-only candidate capture and per-case wall timing. `ModuleResult.factual_text` is optional and defaults absent, preserving fallback-based checks for existing envelopes. No dependencies, default-model change or external profiles were added.

Independent review identified abbreviation truncation, and actual inference identified the redundant-leading-“No,” false positive. Both were corrected in one round with focused regression coverage. Existing factual/security checks remain; no broad guard relaxation was made. Developer and independent reviewer each passed all 20 tests, formatting and warnings-denied Clippy on macOS. Root built the evaluator/CLI and checked CLI help, whitespace, artifact integrity and secret exclusion. Requests for developer/reviewer were Sol/medium; observed runtime selection was not separately verified.

The final fixed run produced accepted replies for all 9 valid 2B routes and all 10 valid 4B routes; successful warm medians were 1.123 and 2.226 seconds respectively. One 2B invalid-argument route failed before execution in both runs. The first run also had an unusable 2B composer candidate, safely rejected, which did not recur in the final run. Both runs are retained; success on the final small sample does not prove general reliability or semantic fidelity. See [report and raw evidence](../evaluations/phrasing-polish.md).

The existing user-owned Ollama service was reused and left running; evaluated models were unloaded afterward. No Tavily calls, installations, billing changes or commits were performed. Further implementation requires a new scoped approval; editable profiles remain the next discussion topic.
