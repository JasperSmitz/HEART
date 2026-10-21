# Explicit commands and editable profiles

Status: implemented, verified and independently reviewed. Return to discussion mode.

## Scope and decision

The user approved deterministic tool invocation and editable character files after the routing experiments. Explicit commands are the default terminal interface. The existing natural-language router remains an explicitly enabled experiment; its historical acceptance failures are preserved, not declared fixed. Module results still pass through the existing grounded character phrasing and exact fallback. No default model change is authorized.

Modules declare their own command name and simple argument binding alongside their capabilities. Core discovers and validates declarations without importing modules or branching on their IDs. Initial commands are `/disk [path]`, `/system`, `/uptime`, and `/search <query>`. The remainder after the command is a single literal string argument, preserving internal spaces; optional matching outer quotes may be supported if clearly documented and tested. No shell expansion, subprocess execution, guessed argument values or LLM routing occurs for explicit commands. Empty/no-argument validation follows the module schema. Unknown/malformed commands never invoke modules or inference. Help is generated from declarations.

Store one JSON file per character under `profiles/`, initially butler and cat. Files contain stable ID, instructions, greeting, generic status and capability-ID progress overrides, plus any small presentation metadata needed to preserve current authored behavior without checking character IDs in code. Do not rewrite character content as part of extraction. Load/discover at startup; documented restart reload is sufficient. Support `--profiles-dir` and `--personality ID`, dynamic `/ID` switches and `/profile ID` selection. Reserve built-in/module command names and reject ambiguous collisions or duplicate IDs clearly. New `/pirate` must work by adding one valid JSON file without changing source. No Markdown parser, hot reload, profile editor or configuration framework.

## Acceptance criteria

- Explicit commands select the declared capability and preserve literal/default argument semantics, including `/`, `.`, absolute paths, spaces, omitted optional input and nonexistent paths. Unsupported commands invoke nothing. Module errors stay safe.
- Deterministic fixture tests prove zero selection/extraction calls for explicit invocation, at most one successful-result phrasing call, identical original sources and exact fallback on failed/rejected phrasing. Idle retention still works, and unavailable inference does not prevent a completed tool result from being returned.
- Module declaration/registration rejects invalid bindings and command collisions. Core remains module-agnostic. Existing examples compile; the experimental router stays opt-in in CLI, with diagnostics accurately reporting absent routing work.
- JSON profiles drive instructions, greetings, statuses and switching. Tests cover custom discovery, duplicate/reserved IDs, malformed files and prompt/data separation. Factual guards remain independent of profile instructions. Trusted local profiles are presentation configuration, never action authority. Avoid new profile-ID-dependent behavior or weakening guards.
- README explains commands, experimental routing, profile fields, profile directory resolution, creating a pirate profile, restart behavior and retained fallback limits. Architecture/roadmap reflect actual new state without retroactively accepting prior routing experiments.
- Focused tests, all-target workspace tests, formatting and warnings-denied Clippy pass. Run a bounded local native-command smoke with the existing installed model (no downloads, Tavily calls or credentials); record phrasing/fallback separately from deterministic invocation. Linux verification if existing tooling is readily available; otherwise state limits.

## Non-goals

Dashboard, telemetry, autonomous multi-tool investigations, routing prompt repair, model/default changes, personality redesign, cloud inference, real web-provider calls, dependencies without need, commits and unrelated cleanup.

## Workflow

Developer: Sol/medium requested through the available delegation interface, with repository developer-role instructions supplied because this interface has no role selector. Reviewer: separate Sol/medium requested after implementation, following repository reviewer instructions. These are requested selections, not verified backend execution telemetry. Architect coordinates at most two focused correction rounds.

## Verification and review

Module declarations now drive `/disk [path]`, `/system`, `/uptime` and `/search <query>`, their argument bindings and generated help. CLI defaults to deterministic invocation; `--experimental-nl` enables the retained, historically unaccepted router. Remainders preserve internal spaces and literal quote characters while trimming outer whitespace; no shell quoting/expansion is performed. Original schemas validate every invocation. There are no core branches for installed capability IDs.

`profiles/butler.json` and `profiles/cat.json` preserve existing authored text and progress. Runtime discovery supports custom IDs, `--profiles-dir`, `--personality ID`, `/ID` and `/profile ID`; restart reloads files. Invalid files, duplicates and command collisions fail clearly. Complete presented answers, including any authored suffix, pass the unchanged factual guard. README documents fields and a pirate example. Existing router evaluators compile with the data-driven profile API; fixture convenience constructors use the same bundled JSON.

Final verification: **47 workspace/all-target tests passed on macOS and Linux ARM64**, using locked dependencies. Linux used the existing `rust:1.99-slim` image with read-only source/registry mounts, offline dependencies and no container network. Formatting and warnings-denied workspace/all-target Clippy passed on macOS; whitespace checks passed. No dependencies were added.

The architect's [bounded native CLI smoke](../evaluations/explicit-commands.md) passed, using the existing default local model with a counted localhost proxy and no Tavily requests or credentials. It independently confirmed canonical paths and exactly seven successful-result phrasing calls with zero routing calls. Four generated replies were accepted and three used the exact fallback; character quality remains refinement work, not a correctness claim. Native errors and unsupported input returned safely.

Independent reviewer inspected source, final docs and focused tests, reporting no actionable defects. The architect requested one bounded resource refinement: skip idle retention for explicit module Error/NeedsInput outcomes when no inference was attempted, leaving previous warmth/idle state untouched. Developer added a cold/warm regression; reviewer independently rechecked it with no findings. [Postcorrection CLI evidence](../evaluations/explicit-command-errors.json) confirms zero model requests for nonexistent-path and missing-key results. Final full tests include this correction. No reviewer lesson was warranted; no application changes were made by the reviewer.

Developer and reviewer were separate fresh agents with Sol/medium explicitly requested through the available fallback interface; backend execution settings were not independently verified. No additional agents, live provider calls, model downloads/default changes or commits occurred. Prior user work was preserved. The scoped command/profile criteria are met; this does not accept the separate natural-language experiments or establish performance on 16 GiB hardware. Dashboard, telemetry and autonomous investigation remain deferred.
