# Numerical phrasing follow-up

Scope: [plan 0009](../plans/0009-phrase-rejection-follow-up.md). Source/guard fixes are verified; reliable personalized disk phrasing remains unresolved.

## Diagnosis

[Initial observations](phrase-rejection-initial.json) captured the actual generated candidates, unlike normal CLI diagnostics which intentionally show only safe statuses. Two of three native disk candidates changed canonical `747.66 GiB` into `802.79 GiB` by using the structured byte counter as a decimal quantity. Those NumberChanged rejections were correct. Fixed Earth excerpts from the user report were phrased faithfully in both voices; the exact original live search rejection was not reproduced, and no Tavily requests or keys were used.

Deterministic probes found separate guard defects: `24,901` split into components and allowed an answer of `24`; same-valued GiB/GB substitutions were not checked. Exact grouping/decimal spelling normalization initially exposed another interaction: numeric lexical fragments caused concise equivalent answers to fail grounding. Independent review caught that case and verified the correction.

## Implemented correction

Phrasing receives one canonical text string (`factual_text`, otherwise `fallback`) alongside the separate request. Raw structured data and source metadata stay in the original module result, outside the phrasing input. The generic prompt requests minimal factual-word editing, exact numeric/unit copying and no calculation, conversions or rounding. No profiles, models, module inputs, retry behavior or defaults changed.

The validator uses exact string normalization for supported ASCII grouping/trailing fractional zeros without floating-point arithmetic or tolerances. It preserves signs, magnitudes and unsupported forms, rejects changed disk units, and excludes already-validated numeric spelling fragments from real-word grounding. Concise numeric answers require their accompanying text/units to match canonical words. Unicode, negation, injection, final presentation, original source and exact fallback protections remain.

## Bounded model checks

The installed `qwen3.5:2b-q4_K_M` was evaluated on the M1 Pro/32 GiB Mac using real root-filesystem readings and fixed synthetic Earth, negation and fare fixtures. Both voices were tested for disk and Earth; museum was butler-only and fare cat-only. A temporary Rust evaluator used the real core and local Ollama boundary, capturing candidates solely for evaluation. Fixture source URLs were never fetched. Unload was requested afterward; the user-owned service was left running.

| Observation | Canonical-only input | Final light-edit prompt and guard correction |
|---|---:|---:|
| Accepted generated replies | 4/8 | 3/8 |
| Accepted disk replies | 0/4 | 0/4 |
| Disk NumberChanged fallbacks | 1/4 | 2/4 |
| Disk Ungrounded fallbacks | 3/4 | 2/4 |
| Earth replies accepted | 2/2 | 2/2 |

Raw [canonical-only observations](phrase-rejection-canonical-only.json) and [final observations](phrase-rejection-final.json) preserve every case. In the final run, the butler's valid disk paraphrases used words such as `contains` and `space` that failed the conservative lexical threshold. Both cat disk candidates invented a `1,788.88 GiB` total despite canonical `926.35 GiB`; those rejections were correct. The museum answer preserved negation and was accepted; the fare answer spelled `four` and changed `adult` to `Adults`, then failed grounding and returned the exact fallback. No unsupported candidate was displayed as an accepted numerical answer in these checks.

These are small, non-randomized, single-run observations, not an isolated prompt comparison or a claim about general model accuracy, other model tags, real web search or 16 GiB performance. The final prompt did not establish a numerical-personality improvement. No more prompt iterations were spent.

## Verification and next decision

All **51 all-target workspace tests** passed on macOS and Linux ARM64. Linux used the existing offline Rust container, read-only source/registry mounts and no network. Formatting and warnings-denied Clippy passed. Independent review cleared after one focused correction, including 34 compiled public-validator probes and 30 non-socket core tests; the full authorized suites also passed the two HTTP fixtures. The reviewer recorded the substantial discovery in LESSONS.

Keep the verified source fixes and the exact fallback, but do not claim the user's disk-personality experience is repaired. Return to discussion about authored character templates or an immutable factual answer with authored character framing for numerical tools. These are recommendations, not implemented or authorized feature scope. Do not lower the grounding threshold or accept changed quantities to manufacture a better pass rate.
