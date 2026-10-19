# 0003 — First useful terminal milestone

**Status:** proposed; application implementation awaits explicit user authorization.

## Objective and authorization

Open HEART, enter a short natural-language request, and receive a concise, sourced web answer in the selected character's voice. The user has established the product direction, Tavily as the initial provider, and a generic core/module boundary. Rust is the preferred CLI-core language. This proposal makes the implementation scope concrete; it is not a record of approval to start application code.

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

## Setup and open items

The planning-session PATH did not expose `cargo`, `rustc`, or `ollama`; this is not proof they are uninstalled. Check existing installations before proposing setup. Necessary installations and model downloads use the environment's approval mechanisms. Do not enroll in paid services or enable paid Tavily usage. A Tavily API key will be needed for live lookup verification; it must be supplied through local configuration, not committed.

The exact model/quantization, contract fields, dependency versions, and measured latency/resource limits remain to be resolved within the approved milestone. No application directories, code, downloads, or installations have been created for this proposal.
