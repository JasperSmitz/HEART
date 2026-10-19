# Architecture and working configuration

## Current state

HEART currently contains project documentation and agent workflow configuration. There is no application implementation. The application language, LLM model/runtime, module contract, and complete runtime structure remain undecided; the planned source boundary and initial search provider are recorded below.

Rust is the user's preferred candidate for the CLI core. Tavily is selected as the initial web-search provider, starting within its free allowance. Fully self-hosted search is a future goal. The application stack remains open pending the first implementation plan; no application implementation is authorized yet.

Linux and macOS portability, a terminal-first experience, safe permission-aware operating-system interactions, simplicity, and modular capabilities guide future decisions. Local/cloud model interchangeability is an exploratory direction. A KDE desktop companion is a long-term possibility. These directions do not authorize application implementation.

## Planned capability boundary

Capability implementation code will live separately from the core under `modules/*`, with web search under `modules/websearch/`. Modules will return an established, shared result structure. Core code will use declared capability metadata and input requirements to select and invoke a module, then handle its result generically; module internals and provider details stay behind that boundary. This is an MCP-like separation of responsibilities, not a decision to implement MCP or require separate server processes. Exact fields, transport, and package structure remain to be designed. This is a planned source layout; application directories have not been created. See [decision 0001](decisions/0001-module-separation-and-search.md).

## Confirmed interaction direction

The chat interface is for brief task requests and answers. The user's expectation is that roughly 98% of interactions can be answered in one sentence; this is an experience target, not a hard output limit. Extended conversation is not the central feature. Personality should fit that brevity and responsiveness. The temperature example is a short lookup status followed by a concise factual answer in the selected style. Module-specific structured results alongside an answer are a possible later addition. Conversation storage, context handling, routing, and presentation mechanisms remain undecided.

Progress text will be authored, with character- and module-specific variants. Final replies should take account of how the request was phrased, with a direct factual answer available as a fallback. Local reply generation is preferred if it meets quality and responsiveness needs; the model, runtime, fallback conditions, and any cloud use remain undecided.

## Hardware and resource expectations

The working baseline is the user's M1 MacBook with approximately 16 GB of memory, subject to confirming its specification. The user's main PC has 32 GB RAM and a GPU with 24 GB memory, but HEART must coexist with games and other applications. Its default should be modest resource use rather than filling available capacity. Local models should load on demand and unload after a configurable idle period; the duration remains open. Different model choices per machine are an option to evaluate; no automatic switching, resource scheduler, or concrete memory/latency budget has been selected. Candidate models need measurements on the baseline machine before performance claims or a default selection.

## Questions for the next discussion

- Which user tasks should HEART own, and which responsibilities should remain outside its scope?
- What is the smallest useful task that can be completed end to end, with observable acceptance criteria?
- What user control and permission boundaries does that first task require?
- What evidence from that task should guide later model, platform, and integration choices?

Record significant approved decisions in `docs/decisions/`; keep execution scope and acceptance criteria in `docs/plans/`.

A [first-milestone proposal](plans/0003-first-milestone.md) now records the recommended implementation scope and remaining approval. It proposes Rust and Ollama; neither the proposal nor discussion of it authorizes application implementation.

## Agent configuration

`.codex/config.toml` requests `gpt-6.1-sol` with high reasoning for the architect and enables agents. `max_concurrent_threads_per_session = 2` caps concurrent spawned-agent threads, excluding the primary thread: up to three total threads. Its subagent defaults request `gpt-6.1-sol` with medium reasoning. Only two custom roles are defined: `.codex/agents/developer.toml` and `.codex/agents/reviewer.toml`. Their instructions describe responsibilities; model and effort fields are deliberately omitted so selection can match the task.

Task selection policy:

- Architect: normally `gpt-6.1-sol`, high reasoning.
- Developer: normally `gpt-6.1-sol`, medium reasoning.
- Reviewer: normally `gpt-6.1-sol`, high reasoning.
- Trivial tasks may use `gpt-6-luna` with low or medium reasoning. Challenging tasks may use `gpt-6-astra` with medium or high reasoning. Use Extra High (`xhigh`) only when task complexity genuinely warrants it. Prioritize responsiveness and implementation quality over minimizing tokens or maximizing reasoning effort.

Reviewer high reasoning is an explicit task selection policy, not enforced by the role file; the shared subagent default remains medium. Revisit these preferences against the supported catalog when the environment changes. Never silently substitute an unsupported model or effort.

When a custom-role selector is exposed, select the role through that interface and explicitly request the appropriate model and effort. Custom role-file model/effort fields would override both spawn selections and shared defaults, which is why these files omit them. Explicit spawn selections override shared defaults.

The user reports successful native project role selection in a fresh session through `agent_type`, with the project instructions loaded natively. Prefer that mechanism when exposed and explicitly select the task's model and effort. Inspect the current session's tools rather than assuming every client exposes the same interface.

The original setup session exposed `collaboration.spawn_agent` without a custom-role selector. For interfaces with that limitation, use a fresh agent (`fork_turns="none"`), explicitly request the model and effort, and supply the relevant role TOML instructions together with a bounded task and instructions to read the repository sources. In that interface, full-history forks inherit the parent's model and effort and cannot accept overrides. Do not assume a role name in a task name loads its instructions.

## Verification and portability

On 2026-10-19 the inspected environment reported `codex-cli 0.163.0`, multi-agent support enabled, and an already trusted project. The local catalog and native agent interface exposed the preferred models and low, medium, high, and xhigh reasoning. These observations are machine-specific; repository configuration does not set trust or authentication.

Current verification: local `codex app-server --stdio --strict-config` and `config/read` with project `cwd` and `includeLayers` resolved the enabled project layer to Sol/high, enabled agents, two concurrent spawned-agent slots, and Sol/medium subagent defaults. No diagnostic errors occurred. Starting the reader required approved sandbox escalation; verification sent no model requests. Details and historical evidence are in the plans.

The spawn response does not expose model metadata, but the installed app-server supports metadata-only `thread/read` and `thread/list`. Using the current thread ID and a direct-parent/project filter, these reported Sol/high for the architect and Luna/medium for the newly delegated policy update. The schema defines `model` and `reasoningEffort` as current configured values when loaded, or latest persisted values otherwise; they are not per-turn execution telemetry. Report these as verified client thread settings, not proof of backend model execution. Avoid reading unrelated threads or full conversation content for this check.

Fresh-session verification, reported by the user on 2026-10-19: developer and reviewer were selected sequentially through `agent_type`, received their project instructions natively, and had matching assigned-role and model/effort metadata: developer `gpt-6.1-sol`/medium, reviewer `gpt-6.1-sol`/high. Setup checks passed and all 14 repository files remained unchanged. This closes the previously untested native-role selection check for that environment. The architect recorded the supplied result without rerunning it in the original session; it verifies client settings, not backend execution telemetry.

For a new environment, the same no-write check can be run in a fresh CLI session from the repository with `codex --strict-config`. Select the project developer and reviewer roles sequentially with explicit Sol/medium and Sol/high requests. Confirm native role assignment and project instructions, using `/subagents`, `/status`, and available thread metadata. If a particular session lacks the selector, use the documented fallback; that does not imply native role support is absent elsewhere.

On a new machine or session:

1. Check the installed Codex version, supported configuration schema, custom-role discovery, agent availability, and model/effort catalog.
2. Validate TOML and use Codex strict configuration validation or local configuration resolution where available. Do not install dependencies or run paid model sessions merely to validate this foundation.
3. Reload or start a new session when needed to load project configuration. Verify effective settings through available runtime metadata; a request or configuration value is not proof it was applied. If metadata is unavailable, report the selection as requested and unverified.
4. Recheck project trust and authentication locally through the user's normal setup. Do not put trust grants, credentials, or user/global settings in this repository.

The foundation plan records checks actually performed and remaining limitations. This configuration does not change the model of an already running architect session.

Configuration references: [Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents), [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference), [developer commands](https://learn.chatgpt.com/docs/developer-commands), and [app-server](https://learn.chatgpt.com/docs/app-server). Recheck current documentation when behavior or schema differs.
