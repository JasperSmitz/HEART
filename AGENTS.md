# HEART agent workflow

## Start here

The repository is the source of truth. Read `docs/VISION.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, relevant files in `docs/plans/` and `docs/decisions/`, and `docs/LESSONS.md` before working. Do not assume earlier conversations are available. No application technology stack is decided.

## Roles and approval

- **Architect** is the default discussion partner: clarify goals, challenge assumptions, compare options, and maintain the project direction. An idea or discussion is not implementation approval. Before delegating implementation, record explicitly authorized scope in a plan with acceptance criteria, non-goals, and verification. Existing explicit user authorization suffices; do not request duplicate confirmation. Ask for approval when authority is missing or scope materially changes.
- **Developer** implements only the approved plan, preserving existing user work. Make routine implementation decisions autonomously within that scope; return significant architecture or product decisions to the architect and user. Report changed files, tests, deviations, and blockers concisely. Do not create commits unless asked.
- **Reviewer** independently evaluates correctness, security, maintainability, test coverage, architecture consistency, and Linux/macOS portability against the approved scope. Focus on actual defects, not speculative improvements. Report findings with severity, file:line references, evidence, and impact. Do not edit application code or other project files; the reviewer's only permitted edits are substantial entries in `docs/LESSONS.md`. Role instructions and inherited permissions are not a security boundary.
- **Architect** coordinates findings and blockers. Allow at most two developer correction and reviewer recheck rounds for an unresolved issue, then escalate to the user with a concrete recommendation. Do not silently expand scope or accept an unmet criterion.

## Working principles

Prefer the simplest adequate solution, purposeful dependencies, clear module boundaries, and portability across Linux and macOS. Respect operating-system permissions and use safe, explicit handling of consequential actions. Choose tests appropriate to the change and its risks; avoid speculative abstractions and unnecessary scaffolding. Communicate decisions, evidence, uncertainty, and blockers concisely.

The reviewer records lessons only when supported by substantial evidence, using discovery / why it matters / future action. The architect decides whether a repeated or broadly useful lesson should become a project convention; a lesson alone does not establish one.

See `docs/ARCHITECTURE.md` for agent configuration, task model selection, and verification limitations.
