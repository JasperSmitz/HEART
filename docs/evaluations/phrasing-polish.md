# Phrasing-polish evidence

Recorded 2026-10-19 on the same Apple M1 Pro/32 GiB Mac as milestone 1, using the existing Ollama 0.40.6 service. No new model downloads, runtime reconfiguration or Tavily requests were needed. The service remains running; evaluated models were unloaded after the comparison. The 2B application default is unchanged.

## Fixed comparison

Five requests per profile cover temperature, adult fare, museum closure, Rust/Cargo and Ocarina's composer. The last case uses a fixed reproduction-shaped excerpt with the accented name Kōji Kondō, rather than an idealized answer sentence. All evidence is fixed fixture data, not fresh web results or a recorded Tavily response. An injection note asks for an unsupported 99 °F claim. Each candidate is explicitly unloaded before its run; only the first request per candidate is cold. Context remains 4096 tokens, temperature 0.2, thinking disabled, with 384 output tokens. Machine activity was not isolated.

The evaluator explicitly captures structured phrasing candidates for these fixed questions. Normal CLI diagnostics never capture candidates or prompts. Model digests and complete results are in [final raw records](phrasing-polish-models.json).

| Final-run measurement | 2B Q4_K_M | 4B Q4_K_M |
|---|---:|---:|
| Valid routed results | 9/10 | 10/10 |
| Accepted generated replies among valid results | 9/9 | 10/10 |
| Factual fallback among valid results | 0/9 | 0/10 |
| Cold request, core timing | 3.460 s | 5.217 s |
| Successful warm core-request median | 1.123 s (8 cases) | 2.226 s (9 cases) |
| Successful warm routing median | 0.526 s | 0.997 s |
| Successful warm phrasing median | 0.590 s | 1.223 s |

Core request timing ends before idle reset and terminal rendering. `case_wall_ms` in the final records also covers retention and failed requests. The one 2B error took 341 ms: invalid arguments on the butler adult-fare fixture were rejected before module execution. This is a failed request, not a factual fallback, and is excluded from successful-request medians. The same failure occurred in both runs, despite the successful cat request; routing itself does not use the persona. No specific routing root cause is established by these records.

Manual inspection found the displayed requested facts, numbers, units, closure polarity and fixture source links preserved across valid results. Both profiles answered the composer request in the final run; 4B's cat wording was “Kōji Kondō composed the music for The Legend of Zelda: Ocarina of Time.” Cat factual replies were mostly neutral, while authored greeting/progress conveyed more of the character. This pass removes forced reluctance/“nya”; it does not demonstrate rich roleplay. No displayed result repeated the injection's unsupported temperature.

## Preserve variability and explain corrections

[Initial raw records](phrasing-polish-initial-models.json) are retained rather than replaced by the better final run. They used the same inference prompts/cases before the final guard correction. 2B had 9/10 valid routes, seven accepted replies and two factual fallbacks; 4B had 10/10 valid routes, eight accepted replies and two fallbacks. The initial 2B composer candidate copied a long system-instruction passage into `answer`; the length/shape check rejected it and displayed the factual excerpt. It did not recur in the final run, so improved final counts do not establish that the underlying model behavior is fixed.

The remaining initial fallbacks were a concrete false positive: correct answers beginning “No, the museum is not open…” failed because canonical text contained `not` but lacked a redundant leading `no`. A narrow correction recognizes only the leading “No,” marker when the remaining clause and canonical text independently retain negative polarity; original negation/number/unit checks remain. Tests cover the observed correct strings and reject marker-only polarity masking, changed no/not/never, positive clauses, numbers and units. This is still a practical rejection heuristic, not semantic verification.

Source display labels and titles are now separate from optional canonical `factual_text`, preventing label numbers or title negation from being treated as factual constraints. Web fallback presents `Source [1] excerpt:` and a short sentence or word-boundary excerpt with an ellipsis, retaining attribution and source links. Independent review also found abbreviation truncation (`Dr. Koji Kondo…` becoming `Dr.`); conservative abbreviation/initial/acronym handling and regressions resolved that finding.

## Verification and limits

All 20 workspace tests passed on macOS, including existing Unicode, contraction, injection, argument and schema regressions. Formatting and warnings-denied all-target Clippy passed. The independent reviewer reran those checks and cleared the changes after one correction round. CLI help exposes `--diagnostics`; categories and timing are closed/safe values printed on stderr only when requested. This pass did not rerun Linux tests; the original milestone's Linux ARM64 evidence remains historical.

Developer and reviewer requests selected Sol/medium via explicit-role fallback. Their runtime metadata was not independently inspected. The reviewer inspected code and regressions; model runs and manual response inspection were performed by the architect.

The comparison is small, uses fixed evidence and one order, and does not validate live search relevance, semantic equivalence, 16 GB hardware or simultaneous gaming. 4B is worth trying explicitly when quality matters more than roughly doubled inference latency; 2B remains the modest default pending hands-on discussion. The routing failure and initial instruction echo remain known limits, not silently repaired by retrying. Editable profiles are deferred. No commit was created.
