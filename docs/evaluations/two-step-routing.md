# Two-step routing experiment

**Status:** experiment concluded; implementation source-reviewed but correctness acceptance unmet. Current checkout contains the unaccepted experimental implementation. Scope: [plan 0007](../plans/0007-two-step-routing.md). Return to discussion; no additional correction loop or model change was started.

## Design and checks

The experiment separates selecting an opaque capability ID (or null) from extracting only that capability's arguments. Selection receives IDs/descriptions and the original request; extraction receives the original request and selected declaration. Core validates both stages independently and still normalizes optional nulls before validating the unchanged module input schema. Empty-input capabilities skip extraction; unsupported or invalid selection stops before module invocation. There are no capability-specific parsing rules, retries or model changes. Phrasing and factual fallback remain separate.

The existing twenty regression cases remain unchanged. Four predefined butler-only wording cases are reported separately: unquoted root, an unquoted existing absolute fixture path, opening Downloads (unsupported), and Rust's package-manager question (public web lookup). These variations were selected before live probing; they are additional coverage, not a statistical generalization set. All web calls use the credential-free counted stub with the production capability declaration. Temporary fixture directories are cleaned up after evaluation.

## Comparison method

Compare combined selection/extraction routing time with [plan 0006's preserved final 2B](optional-argument-acceptance-2b-round2.json) and [4B](optional-argument-acceptance-4b-round2.json) runs. Exclude the first request in each run, then pair regression cases that passed both runs and have reply timings. Exclude prior wrong routes and unsupported cases with no reply timing. Report stage times and disk-specific overhead as well as the overall median, since empty-input capabilities skip extraction. The target is at most 1 second additional median warm routing time on 2B.

The runs are not interleaved or randomized, use a local web stub, and include only a small sample on the current M1 Pro/32 GiB Mac. Cold load is excluded from warm comparisons and recorded in raw observations. This cannot establish 16 GiB/PC performance, real Tavily latency, universal intent accuracy or general phrasing quality. Preserve any failed runs rather than rerunning to replace them.

## Observed results

Raw observations: [2B](two-step-routing-2b.json), [4B](two-step-routing-4b.json), using the installed Q4_K_M models with Ollama 0.40.6. Model digests, stage timings, literal arguments, invocation counts, native envelopes, checks and independent comparisons are recorded. Zero Tavily calls or key access; unload requested after each run and the existing user-owned service left running.

| Check | Qwen 2B | Qwen 4B | Ministral 3B |
|---|---:|---:|---:|
| Original regression requests fulfilled | 14/20 | 18/20 | 15/20 |
| Additional wording requests fulfilled | 2/4 | 4/4 | 3/4 |
| Total fulfilled | **16/24** | **22/24** | **18/24** |
| Matched passing warm timing pairs | 11 | 16 | 10 vs Qwen 2B; 13 vs Qwen 4B |
| Median paired routing increase | +169 ms | +121.5 ms | +432.5 ms vs Qwen 2B; −305 ms vs Qwen 4B |
| Median paired disk-routing increase | +258 ms (7 pairs) | +254 ms (10 pairs) | Not compared |

The 2B model converted explicit root `/` and relative `.` to null in both profiles. For root, that queried cwd rather than the requested root. For relative `.`, the canonical result happened to match cwd but literal argument extraction still failed. It also selected disk readings for both deletion requests and the Downloads-open variation, and OS info for the Rust package-manager question. No deletion/open operation was performed: the substituted module only read filesystem counters. Original weather web queries worked, but this did not offset the other regressions.

The 4B model selected the right capabilities and passed every explicit-path/wording case, but both omitted-path cases failed: butler invented `./` instead of omitting the argument, and cat returned the string `"null"` rather than JSON null, which the module treated as a literal nonexistent path. Equivalent canonical cwd does not establish faithful omitted/default semantics.

The additional [Ministral 3B Q4_K_M run](two-step-routing-ministral-3b.json) fulfilled 18/24 checks. It handled explicit root paths for both profiles, but both profiles failed the omitted-path case and nonexistent-path case; butler also passed quote characters as part of the relative `.` path and returned an unsupported route for the public Rust package-manager question. Its paired warm routing median was about 433 ms slower than Qwen 2B (10 matched passing regression requests) and about 305 ms faster than Qwen 4B (13 matched requests). These are small, sequential, non-randomized samples on the same two-step code path; they do not establish a general speed ranking or performance on 16 GiB hardware. The Ollama model tag is about 3.0 GB and was downloaded for this test; this does not change HEART's configured default.

The warm target was met for 2B, but correctness was not. Raw matched-set routing medians were 1503→1783 ms for 2B and 1552.5→1685.5 ms for 4B; these differences are not the same statistic as the median paired increases above. Matched-set total-request medians were 1621→1944 ms and 3273.5→3458 ms respectively, using a web stub. The Linux container check ran concurrently with the 2B evaluation; treat these timings as approximate experiment observations, not an isolated benchmark. Failing cases are excluded from latency pairing, not hidden from correctness counts.

All **38 all-target workspace tests** passed on macOS and Linux ARM64 with locked dependencies. Formatting and warnings-denied all-target Clippy passed. Independent review found no actionable deterministic stage-validation, normalization, call-boundary or module-contract defect, and confirmed the semantic acceptance failures in both models. No extra prompt correction rounds were spent: simpler stages still failed requirements already stated in their instructions, with no demonstrated source defect to fix. Requested developer Sol/high and reviewer Sol/medium settings were not independently verified as runtime execution telemetry.

## Recommendation

Do not mark this routing implementation or the parent system milestone accepted. The model comparison shows that Ministral 3B improves the sample score over Qwen 2B but still misses required routing cases; Qwen 4B remains the strongest on this small matrix. No candidate has established a reliable small-model routing approach. Preserve the current code as an unaccepted experiment, unchanged model default and all observations; discuss the next scope before further implementation. Dashboard, telemetry, profiles and commits remain deferred.
