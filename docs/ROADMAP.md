# Roadmap

## Foundation — complete

Established the project vision, honest current-state documentation, architect/developer/reviewer workflow, agent configuration, and lightweight records for plans, decisions, and lessons. Configuration checks passed and independent review cleared after one correction round. See `plans/0001-foundation.md` for evidence and verification limits.

## Milestone 1 — implemented and reviewed; target-hardware validation remains

Implemented the terminal flow in Rust with Ollama, Tavily-backed web search under `modules/websearch/`, two text personalities, authored progress, bounded generated replies, factual fallback, and configurable idle unloading. Tests pass on macOS and Linux ARM64; live Tavily, personality switching and idle unload/reload passed on the M1 Pro/32 GiB Mac. Independent review cleared after one correction round. The 2B Q4 model is the measured default; factual fallback remains frequent and 16 GB performance has not been validated.

See the [plan and execution record](plans/0003-first-milestone.md) and [acceptance evidence](evaluations/milestone-1.md). Return to discussion before further implementation. Later milestones and new Git commits require their own authorization.

## Phrasing polish — implemented and reviewed

Added opt-in safe phrasing diagnostics, clearer source-excerpt fallback and subtler built-in character prompts. Canonical facts are separate from fallback labels; targeted fixes handle abbreviations and redundant leading “No,” without dropping existing factual guards. All 20 tests, formatting and Clippy pass on macOS; independent review cleared after one correction round. The final fixed comparison completed 9/10 routes with 2B and 10/10 with 4B; earlier model failures are preserved in the evidence. Default remains 2B, and editable profile files were explicitly deferred.

See [plan 0004](plans/0004-phrasing-polish.md) and [comparison/limits](evaluations/phrasing-polish.md). Discussion mode resumes.

## Read-only system information — implemented; explicit-path acceptance unresolved

Implemented disk space, OS/version/architecture and uptime in `modules/system/`, using the common capability contract. All 38 tests pass on macOS/Linux ARM64, with independent source review and native comparisons. The optional-argument follow-up and subsequently authorized two-step routing experiment did not meet actual-model correctness criteria. Current experimental code selects an ID, then extracts only that capability's inputs; 2B passed 16/24 and 4B 22/24. Extra median paired warm routing cost was modest (169/121.5 ms), but both models still made semantic argument/selection errors. Acceptance remains open. Return to discussion about evaluating models on the frozen task set before further architecture changes; no downloads or default-model change is authorized. See [plan 0005](plans/0005-system-information.md), [optional-argument plan](plans/0006-optional-argument-routing.md), [two-step plan](plans/0007-two-step-routing.md) and [latest evidence](evaluations/two-step-routing.md).

The subsequent dashboard prototype is authorized under plan 0012 below; live telemetry remains a separate future discussion.

## Explicit commands and editable profiles — implemented and reviewed

Authorized [plan 0008](plans/0008-explicit-commands-and-profiles.md) adds module-declared `/disk [path]`, `/system`, `/uptime` and `/search <query>` as the default interface. Dispatch preserves literal/default input semantics without model selection or extraction; successful results retain grounded character phrasing and exact fallback. The existing natural-language experiment requires `--experimental-nl`, and its historical acceptance failures remain open.

One JSON file per profile now drives instructions, greetings, generic/capability statuses and optional presentation metadata. Startup discovery, custom directories, initial ID selection and dynamic `/ID` or `/profile ID` switching support new profiles without source changes. Final presentation remains subject to unchanged factual guards. Automated all-target tests pass on macOS and offline Linux ARM64; formatting and warnings-denied Clippy pass on macOS; independent review and bounded native smoke evidence are recorded in the plan. No model/default change, dashboard or telemetry implementation is included.

## Authored numerical replies — implemented and reviewed

Authorized [plan 0010](plans/0010-authored-numeric-replies.md) implements deterministic profile templates for disk and uptime through optional common display-ready fields. Explicit replies, including missing/invalid-template fallback, require no model or idle-retention requests. Core remains module-agnostic; native quantities, structured data and canonical fallback remain intact. Existing profiles without templates stay compatible. Web/OS generation and its conservative guards remain unchanged. See [template syntax, fields and pirate example](../README.md). Independent review and final acceptance evidence are tracked in the plan; no multi-tool composition, dashboard, telemetry or routing repair is included.

All 57 tests passed on macOS and were covered on Linux ARM64 (56-test all-target run plus the later-added compatibility test), with formatting, warnings-denied Clippy and independent review clear. The [native terminal check](evaluations/authored-numeric.md) verified thirteen tool results with zero model/retention requests, including both characters and custom-template fallback. Return to discussion; future combined-answer generation remains open.

## Web evidence relevance — bounded mitigation implemented and reviewed

Authorized [plan 0011](plans/0011-web-evidence-relevance.md) adds conservative subject/answer-signal filtering inside `modules/websearch`, selects later eligible sources/sentences, and declines insufficient evidence before generation. The bounded provider comparison and captured-response replay are tracked in the plan. Production remains one Tavily basic request without retries or page fetching; no provider/model/default, core, shared-contract or profile change is included. General declarative evidence remains eligible alongside stricter factual-intent signals. Lexical selection is not semantic verification and can reject useful paraphrases or weakly evidenced question forms.

All 66 all-target tests passed on macOS and Linux ARM64, with formatting, warnings-denied Clippy and independent review clear after two focused correction rounds. Eighteen live Tavily requests used 27 credits; both modes returned Deltarune sources, so the reported DID failure was not reproduced. DuckDuckGo's reference endpoint was unavailable. Final replay selected five excerpts and declined four per mode, including valid material split across sentences; remaining relationship/completeness false positives are documented. See [evaluation and limitations](evaluations/web-evidence-relevance.md). Return to discussion before further heuristic growth or a provider replacement.

## Cozy dashboard — prototype implemented; visual acceptance pending

Authorized [plan 0012](plans/0012-cozy-dashboard.md) adds opt-in `--dashboard` with Cat by default, original cell-pixel character rooms, JRPG dialogue, editable input, profile themes, responsive layout and a single request worker. Existing console/commands remain available; system **STATIC DEMO** and music **LOCAL MOCK** widgets do not poll or integrate services. Optional frontend-neutral appearance keeps older profile files compatible. Deterministic actual-renderer SVG previews support visual iteration. Independent review, real terminal/Linux checks and the user's visual evaluation are tracked in the plan/evaluation; live telemetry, audio and desktop integration remain outside this scope.


## Possible later exploration

Useful terminal tasks, additional modular capabilities, fully self-hosted web search, local/cloud model interchangeability, and a desktop companion may inform future milestones. Their order, scope, feasibility, and delivery are open; this list is not a commitment.
