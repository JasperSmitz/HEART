# 0001 — Project and agent workflow foundation

Historical record: its model policy and configuration evidence are superseded by [plan 0002](0002-model-policy.md); the original checks below are preserved as evidence of the foundation work.

**Status:** complete; configuration checks passed and independent reviewer recheck cleared. Returned to discussion mode.

## Approval and scope

The user explicitly approved project agent workflow, configuration, and documentation. This plan records that authorized scope. Establish persistent architect/developer/reviewer responsibilities, project direction, honest current-state architecture, a discussion-first roadmap, and lightweight plan/decision/lesson records.

**Non-goals:** application code, technology-stack or provider selection, dependencies, tooling scaffolds, product implementation plans, global/user configuration changes, and commits.

## Acceptance criteria

- `AGENTS.md` defines startup reading, a plan recording explicitly authorized scope without duplicate confirmation, role boundaries, independent review, no reviewer application edits, reviewer-only lesson entries, and architect coordination with at most two correction rounds before escalation. Missing authority or materially changed scope requires approval.
- Vision captures Linux/macOS terminal-first natural-language assistance, modular capabilities, exploratory local/cloud interchangeability, and a possible long-term KDE companion without implementation promises.
- Architecture describes the actual documentation-only state, focused open questions, configuration semantics, task model/effort policy, fallback role loading, and verification limitations without inventing a stack.
- Roadmap leaves the next core-responsibilities and thin-milestone discussion open; no application implementation is approved.
- Lessons start empty; decision and plan conventions remain lightweight. README links the foundation documents and configuration requirements.
- Project config requests `gpt-6-astra`/high for architect and enabled agents with `max_concurrent_threads_per_session = 2`, capping concurrent spawned-agent threads excluding the primary thread (up to three total), and `gpt-6.1-sol`/medium defaults. Exactly two role TOMLs contain only name, description, and developer instructions; reviewer high is selection policy, not a fixed override.
- Available configuration checks and independent review complete, with actual results and limitations recorded. No product code, stack choice, dependency installation, tooling scaffold, user/global settings change, or commit occurs.

## Verification and risks

Inspect the complete diff and scope, parse TOML if an existing parser is available, and check strict Codex configuration/local resolution where practical. Independently review the acceptance criteria. Do not install validation tooling or launch nested paid model sessions. Environment-specific trust, schema support, and model availability must be rechecked elsewhere. Configuration requests do not prove runtime application, especially for the current session.

## Execution and review record

- Native fallback developer requested: `gpt-6.1-sol`, medium reasoning. Observed runtime model/effort metadata: unavailable; selection unverified.
- Native fallback reviewer requested: `gpt-6.1-sol`, high reasoning. Observed runtime model/effort metadata: unavailable; selection unverified.
- Documentation and project configuration: authored.
- Validation evidence (architect, 2026-10-19): `codex-cli 0.163.0` ran `app-server --stdio --strict-config`; `config/read` with project `cwd` and `includeLayers` succeeded without diagnostic errors. The non-disabled project `.codex` layer resolved to Astra/high, enabled agents, a two-spawned-thread cap, and Sol/medium defaults. Approved sandbox escalation was required to start the local server. No model task was run; resolved configuration is not evidence of runtime model selection.
- `codex debug prompt-input` succeeded after approved escalation and confirmed `AGENTS.md` inclusion. Its strict flag is unsupported; the plain command was used. Output did not expose custom role descriptions, so standalone role discovery and named-role invocation remain unverified.
- Additional architect checks passed using already installed `pip._vendor.tomli`: all three TOMLs parse; exactly two role files contain only nonempty `name`, `description`, and `developer_instructions`. Whitespace and relative Markdown link checks passed for all nine Markdown files, including untracked files; `git diff --check` passed. No dependency was installed.
- Independent review found two documentation issues: duplicate-approval wording and ambiguous thread-cap semantics. Correction round 1 clarified existing authorization suffices for recorded scope and the cap excludes the primary thread; aligned developer instructions and acceptance criteria.
- Correction rounds: 1.
- Final review: independent reviewer recheck cleared both findings; no further defects or substantial lessons found.
- Completion: foundation acceptance criteria met within the documented verification limits. No application implementation, stack decision, dependency installation, or commit. Operating mode is discussion.
