# Explicit commands and editable profiles

Scope: [plan 0008](../plans/0008-explicit-commands-and-profiles.md). This validates the deterministic command interface, not the historical experimental natural-language router or general personality quality.

## Native command smoke

[Recorded observations](explicit-commands.json) use the installed `qwen3.5:2b-q4_K_M` on the M1 Pro/32 GiB Mac. A temporary working directory avoided `.env.local`; the child inherited no Tavily key. A localhost counting proxy forwarded only to the existing local Ollama and recorded request types and native structured readings, not prompts or generated candidates. No Tavily calls, downloads or service reconfiguration occurred. Unload was requested after the run; the user-owned service remained running.

- Seven successful native requests covered disk default cwd, `/`, `.`, an absolute path containing spaces, OS/architecture, uptime, and disk `/` after switching to cat. All returned the expected native structured data; canonical paths matched independently computed paths. Sources remained empty.
- A nonexistent path returned the safe module error without a phrasing call. An unknown command and plain natural-language input were rejected before module/inference dispatch.
- Exactly seven chat requests occurred, all phrasing. Every native reply reported `routing_performed=false`, zero selection/extraction/combined routing timings, and no extraction call.
- Four of seven generated replies were accepted. Three used direct factual fallback after grounding, negation or number checks rejected the candidate. This is evidence of continued model/presentation limitations, not a reason to weaken the guards. Profiles were extracted without rewriting their instructions.

These observations are one bounded functional run, not a latency benchmark, 16 GiB hardware check, semantic proof of every accepted paraphrase, or a real search-provider test. Deterministic fixtures separately verify query binding, module invocation counts, original sources, error and exact fallback behavior, custom-profile prompt separation, collisions and startup discovery.

## Resource follow-up and final verification

The initial trace above includes an idle-retention request after the nonexistent-path error. The final implementation skips retention and preserves existing warmth/idle bookkeeping when explicit module execution performs no inference. This avoids loading an unused model through an empty retention request. A separate [postcorrection CLI check](explicit-command-errors.json) confirmed zero requests to its counting model endpoint for both nonexistent-path Error and missing-key NeedsInput outcomes, with no retention warning. It did not forward any request to Ollama or a web provider.

All 47 final workspace/all-target tests passed on macOS and offline Linux ARM64, with formatting and warnings-denied Clippy passing on macOS. Independent source/doc review and focused resource-correction recheck reported no actionable findings. See the plan for role/settings limitations and verification details.
