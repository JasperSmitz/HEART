# 0002 — Revised agent model policy

**Status:** complete; configuration verified and independent review cleared. Returned to discussion mode.

The user initially requested Sol/high for the architect, Sol/medium for the developer, and Sol/high for the reviewer. The original execution evidence below is preserved; the lighter-default follow-up at the end supersedes that selection policy.

## Scope and acceptance

- Update project defaults and current policy documentation; preserve the original foundation plan as historical evidence.
- Keep custom role model/effort fields unpinned so explicit task selections remain effective. Reviewer high remains explicit selection policy because the shared subagent default is medium.
- Verify listed models and supported efforts before configuration; validate TOML and effective project settings where practical.
- Explain practical next steps for named-role selection and model verification. Distinguish client capabilities, requested settings, resolved configuration, and observed execution. Do not add unsupported switches or claim unobserved runtime settings.
- Independently review the focused changes. No application code, stack decisions, dependencies, global settings, or commits.

## Verification and execution

- Before configuration: CLI 0.163.0; local catalog fetched 2026-10-19 lists `gpt-6.1-sol`, `gpt-6-luna`, and `gpt-6-astra` with low, medium, high, and xhigh support. The current native spawn tool exposes model/effort controls for fresh agents, with no custom-role selector. Official configuration and subagent documentation checked.
- Developer selection requested: `gpt-6-luna`, medium reasoning. The spawn response did not expose model/effort metadata; a subsequent project/direct-parent-filtered app-server metadata read reported Luna/medium for the newly delegated child, corroborating configured thread settings. It also reported Sol/high for the architect. The installed schema explicitly says these fields are not per-turn execution telemetry.
- Effective project configuration verified using CLI 0.163.0 `app-server --stdio --strict-config` and `config/read`: enabled project layer, Sol/high architect defaults, Sol/medium subagent defaults, two concurrent spawned-agent slots; no diagnostics. Approved sandbox escalation was needed to start the reader. These verification requests ran no model tasks.
- Developer checks: all three TOMLs parse with the existing `pip._vendor.tomli`; `git diff --check` passed. No dependency installed.
- Architect documented a fresh-CLI native-role check using `/subagents`, `/status`, and available thread metadata. This session's explicit-role fallback was exercised; native role discovery/selection in a new CLI session remains untested. No unsupported metadata switch was added.
- Architect checks passed for expected policy values, unpinned role fields, Markdown whitespace and local links, and `git diff --check`.
- Reviewer requested: `gpt-6.1-sol`, high reasoning; this review thread's configured metadata was not separately inspected. Independent review reported no findings; no corrections or lessons were needed.
- Complete within the stated verification limits. No application code, stack decision, dependency, global setting, or commit changed. Discussion mode resumes.

## Follow-up — native-role verification

The user supplied a successful fresh-session test on 2026-10-19: developer and reviewer were selected sequentially through `agent_type`, received their project instructions natively, and showed matching assigned roles and model/effort metadata (developer Sol/medium; reviewer Sol/high). Setup checks passed; all 14 repository files remained unchanged. This supersedes the earlier untested status above. It is user-supplied verification of client settings, not backend execution telemetry; the original architect session did not independently rerun it.

Architecture documentation was updated to prefer the reported native role mechanism where exposed and scope the original tool-interface limitation to that session. No configuration change was needed. Independent documentation review found one provenance wording ambiguity; the opening policy paragraph was corrected to explicitly attribute the result to the user. No other findings; Markdown whitespace and local link checks passed.

## Follow-up — lighter defaults

**Status:** complete; lighter defaults verified and focused independent review cleared. Returned to discussion mode.

After observing high usage, the user approved the architect's recommendation with “Let's go with that.” Scope: Sol/medium by default for architect, developer and reviewer; Luna/medium for small, well-defined developer tasks; Luna/low for straightforward documentation and formatting. Escalate Sol to high for significant architecture, difficult debugging, complex logic or sensitive reviews. Reserve Astra for exceptional problems where Sol is struggling, and Extra High for demonstrated benefit. Keep concise handoffs/output, focused correction reviews and independent implementation review; avoid extra agents for routine bookkeeping.

Acceptance: update project config and both role instructions without pinning model/effort overrides, align AGENTS and architecture, preserve historical checks, validate TOML and effective project defaults where practical, and obtain one focused independent review. This routine policy bookkeeping is handled directly by the architect, followed by the reviewer; no separate developer agent, application changes, dependencies, global settings or commits are needed.

Preflight: installed CLI remains 0.163.0. The local model catalog fetched on 2026-10-19 lists Sol, Luna and Astra with the selected low/medium/high/xhigh efforts; the native spawn interface supports explicit model/effort for fresh agents. Official OpenAI [configuration](https://learn.chatgpt.com/docs/config-file/config-reference) and [subagent](https://learn.chatgpt.com/docs/agent-configuration/subagents) documentation confirm shared defaults and explicit task selection, with role overrides taking precedence. Leave role overrides absent. Updating repository defaults does not change the already-running architect's effort or establish backend execution telemetry.

Verification: all three TOMLs parsed with the installed Python `tomllib`; both role files remain free of model/effort overrides. A metadata-only `codex app-server --stdio --strict-config` / `config/read` request resolved the enabled project layer to Sol/medium for the architect and subagents, with agents enabled and the two-spawned-thread cap preserved. No model request was sent by this check; the temporary reader was terminated. `git diff --check` passed. The single fresh reviewer requested Sol/medium and reported no findings; its runtime selection was not independently observed. No application code, dependency, global setting, lesson or commit changed.
