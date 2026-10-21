# 0003 — First useful terminal milestone

**Status:** implemented and independently reviewed; validation on the 16 GB target remains open. Returned to discussion mode; no further implementation authorized by this record.

## Objective and authorization

Open HEART, enter a short natural-language request, and receive a concise, sourced web answer in the selected character's voice. The user explicitly authorized this milestone with “Alright let's go ahead on the first milestone. Let's rock!” This approves the scoped Rust/Ollama implementation below, with Tavily as the initial search provider and the generic core/module boundary. The earlier setup commit does not authorize any new commits.

## Recommended stack and sequence

- Rust for the CLI, core, shared module contract, and web-search module. Use ordinary workspace/package boundaries and purposeful dependencies.
- Ollama as a separately installed local inference service accessed through HTTP. Keep runtime-specific details behind a small replaceable interface. Automatic runtime installation and a bundled inference engine are outside this milestone.
- Compare quantized 2B and 4B candidates on the M1 before selecting a default. Qwen3.5-2B and Qwen3.5-4B are starting candidates, not validated selections. Pin the actual evaluated model variants and record their sizes/settings.
- First define the common contract and validate local-model behavior with controlled results; then connect the real Tavily module and complete the terminal flow. Share one local model for interpretation and final phrasing initially to avoid loading two models just for separate roles.

The architect resolves routine crate and implementation choices. Escalate changes to the language, inference architecture, scope, or user-facing behavior when they materially alter this proposal.

## Scope and acceptance criteria

1. **Usable terminal:** the input interface becomes usable without waiting for model loading. It handles requests sequentially, supports exiting and choosing between two text-only personalities, and reports failures clearly.
2. **Generic module contract:** capability IDs, descriptions, and validated inputs let the core select and invoke modules without provider-specific branches. A common result envelope represents success, missing information, or failure, carries a direct factual fallback, and can include structured data and sources. Module-specific internals remain under `modules/*`; Tavily integration lives under `modules/websearch/`. Exact types are designed before implementation and documented with examples.
3. **Bounded lookup:** natural-language requests can select the declared web-search capability and supply valid arguments. The initial path performs one basic Tavily search per lookup, returns source links with the answer, and handles missing credentials, quota exhaustion, timeouts, and insufficient evidence honestly. Iterative autonomous research is outside scope. Requests needing another unavailable capability receive a clear response.
4. **Character presentation:** two contrasting text profiles, initially a butler and playful tsundere cat character, provide authored greetings and character/module-specific progress strings. Final phrasing uses the user's request, returned facts, and selected style, normally producing one concise sentence. Switching personalities changes presentation while preserving facts, units, sources, and permissions, and does not load another model. No artwork or extended conversation is needed.
5. **Local inference and fallback:** load on demand and keep the model loaded while requests are active. A configurable idle timeout unloads it; five minutes is the proposed initial value. Bound context, output length, and request duration. A failed or timed-out final phrasing call returns the module's factual fallback and sources. If interpretation is unavailable, explain the limitation rather than guessing an operation. No automatic cloud fallback.
6. **Resource evidence:** record model load time, warm interpretation and phrasing time, end-to-end latency, and observed memory for a small fixed set of representative questions on the M1/16 GB baseline, confirming the machine specification first. Report cold and warm behavior separately. Do not claim acceptable speed or choose a default on parameter count alone. If neither candidate meets factual/style expectations with responsive behavior, return measurements and a recommendation before expanding the design.
7. **Safe boundary:** module results and retrieved text are data, not instructions granting new permissions or initiating more actions. No arbitrary shell execution, machine modifications, secret exposure, or paid search enrollment. Credentials come from local configuration/environment and are excluded from logs and version control. Search provider errors never become fabricated successful answers.

## Non-goals

Dashboard/widgets, system-information or music modules, visual characters, persistent chat history or personal memory, dynamic plugin loading, MCP transport, process isolation, automatic model switching under load, self-hosted search, background polling, cloud-model integration, and Git commits.

## Verification and handoff

- Use meaningful Rust tests and local HTTP fixtures for contract/argument validation, routing failures, provider responses, quota/errors, source handling, inference timeout/fallback, and personality isolation. Routine tests must not require credentials or live services.
- Run formatting, appropriate static checks, and tests. Verify live Ollama behavior and Tavily lookup separately when the local tools, model files, and API key are available; report any unverified acceptance criteria explicitly.
- Review Linux/macOS portability and execute platform checks where available. A local macOS run alone does not establish Linux compatibility.
- Developer: request `gpt-6.1-sol`/medium; raise effort for a demonstrated difficult subtask as needed. Reviewer: request `gpt-6.1-sol`/high. Prefer native roles when the active client exposes them; otherwise use the documented explicit-role fallback. Record requested and verified client settings separately from execution telemetry.
- Independent reviewer checks correctness, boundaries, security, portability, tests, and actual model measurements. The architect coordinates at most two correction rounds for unresolved findings before returning to the user.

## Initial setup observations

The planning-session PATH did not expose `cargo`, `rustc`, or `ollama`. Existing installations were checked, then Rust/Cargo 1.99.1 and Ollama 0.40.6 were installed through approved Homebrew commands. Explicit Qwen3.5 2B/4B Q4_K_M model files were downloaded with approval. The user supplied a Tavily key through ignored `.env.local`; no paid enrollment or billing change was made.

## Execution and handoff

Implemented Rust workspace packages for the generic contract, core, CLI and separate web-search module. See [module contract](../MODULE_CONTRACT.md) for actual types and validation limits. Selected `qwen3.5:2b-q4_K_M` from the measured candidates; dependency versions are locked in `Cargo.lock`. No dashboard, extra capability, cloud fallback or application commit was added.

Acceptance criteria 1–5 and 7 passed the implemented tests/live checks within the documented limitations. Criterion 6 has measured evidence on the actual M1 Pro/32 GiB machine, which supersedes the assumed baseline hardware; the intended 16 GB target is **not validated**. Warm fixture requests had a 1.147-second median for 2B versus 2.291 seconds for 4B. A full cold CLI/Tavily lookup took 3.67 seconds once. Exact factual fallback occurred in 3/6 2B cases and 2/6 4B cases; final character phrasing remains inconsistent. These small samples and practical rejection checks do not establish general semantic accuracy.

All 15 tests passed on macOS and Linux ARM64 in an official Rust container. Formatting and warnings-denied Clippy passed. Latest-binary real Tavily checks passed in both personalities; an accelerated two-second idle check verified unload and subsequent reload. The five-minute default was not separately timed. No live Linux inference deployment or gaming coexistence was tested.

Independent review cleared four concrete defects after one correction round, including a reverse-direction Unicode regression before closing the round. Reviewer maintained the substantive learning entry. Developer and reviewer requested Sol/medium and Sol/high respectively through the explicit-role fallback; task execution metadata was not independently verified. See [full evidence and limitations](../evaluations/milestone-1.md).

Further work requires discussion and scoped approval. Priorities to discuss after hands-on use: phrasing reliability, a small read-only system module, and testing the 16 GB target.
