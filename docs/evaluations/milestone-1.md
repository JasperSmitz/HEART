# Milestone 1 acceptance evidence

Recorded 2026-10-19. The implemented terminal flow passed independent review after one correction round. The 16 GB compatibility target remains untested.

## Local model comparison

Actual hardware: Apple M1 Pro, 32 GiB unified memory, macOS. Runtime: Ollama 0.40.6, one loaded model, one parallel request, 4096-token context. Rust/Cargo 1.99.1. Both candidates are explicit GGUF Q4_K_M tags; full model digests, sizes, responses and timing records are in [raw measurements](milestone-1-models.json).

Each model ran six requests: temperature, adult train fare and museum opening, each in the butler and cat profiles. A deterministic module supplied fixed evidence plus an instruction-injection note. No Tavily requests were made in this comparison. Each candidate was unloaded before its run; the first request is cold and the next five are warm. A Linux container build limited to two CPUs ran concurrently, so these are indicative development-machine measurements rather than isolated benchmarks.

| Measurement | `qwen3.5:2b-q4_K_M` | `qwen3.5:4b-q4_K_M` |
|---|---:|---:|
| Download | 1.81 GiB | 3.10 GiB |
| Cold routing load reported by Ollama | 2.06 s | 3.34 s |
| Cold full fixture request | 3.34 s | 5.71 s |
| Warm routing median | 0.560 s | 1.060 s |
| Warm phrasing median | 0.608 s | 1.231 s |
| Warm full fixture request median | 1.147 s | 2.291 s |
| Peak sampled Ollama process RSS | 2.35 GiB | 3.87 GiB |
| Peak runtime-reported model size | 1.52 GiB | 2.94 GiB |
| Valid routed results | 6/6 | 6/6 |
| Exact factual fallback used | 3/6 | 2/6 |

Full fixture request timing covers routing, module execution and phrasing, ending before idle-timer reset and terminal rendering. Per-stage warm medians need not sum to the full-request median. Memory was sampled every 0.5 seconds; process RSS sums the Ollama service and runner, while runtime-reported size is a separate metric. Neither measures exclusive GPU usage, all system memory, or pressure while gaming.

Manual inspection and independent review found the displayed requested facts, numbers, units, negation and source links preserved across these twelve results. Accepted generated replies distinguished `sir`/`certainly` from `nya`; rejected phrasing retained the exact neutral fallback. The injected 99 °F claim did not appear. This small sample cannot establish general accuracy. The rejection filter does not verify semantic relationships or completeness, and character wording remains inconsistent because of fallback frequency.

Select the 2B Q4 variant as the modest default: it gave lower latency and observed memory use here. The 4B variant did not show a sufficient improvement in this sample to justify doubling warm latency. Users may explicitly select another installed tag; there is no automatic switching. A base M1/16 GB machine, Linux model runtime, and simultaneous gaming have not been measured.

## Live application checks

- With the ignored local Tavily key and the 2B model, the latest CLI answered “What is Rust's package manager called?” correctly with source links. Butler: “Certainly, the reported package manager for Rust is called Cargo.” Switching with `/cat` produced “Rust's package manager is called Cargo, as reported by the sources, nya.” Both authored routing and module progress strings changed with the profile.
- A separate cold invocation of the built CLI with the same question and `/exit` took **3.67 seconds wall time**, including startup, local inference, the real basic Tavily request, idle reset, output and exit. This is one repeated stable-fact query, not a live-search latency guarantee.
- With `HEART_IDLE_SECONDS=2`, a metadata-only `/api/ps` check after idle returned no loaded models. The next request successfully reloaded the model. Production default remains 300 seconds; the five-minute interval was not separately timed.
- `/exit` completed cleanly. `.env.local` is Git-ignored; no credentials appear in these artifacts. No paid enrollment or billing changes were performed. The test service was temporary; models remain explicitly installed for future use.

## Automated checks and review

- macOS: all 15 workspace tests passed; formatting and Clippy with warnings denied passed.
- Linux ARM64: all 15 tests passed with the locked dependencies in official `rust:1.99-slim` (image digest `sha256:4df2ae8cfdfebdfc13a65da7f253b73d68983dc0339f2719bc17a279c913b13d`). Only manifests and Rust source directories were mounted read-only; no API key was mounted. This verifies the build and fixture tests, not a Linux desktop or live inference deployment.
- Tests exercise routing and input validation, bounded provider requests, safe credential/quota/error handling, timeout and exact fallback, source preservation, trusted profiles/progress and retention behavior. They use local HTTP fixtures, without live credentials.
- Independent review found and cleared four issues: contraction polarity, Unicode numeric/sign changes, raw argument lengths, and unsupported schema constraints. The reviewer independently reran focused regressions and original compiled counterexamples; full-platform and live evidence was supplied by the architect. The reviewer recorded the substantive lesson in [LESSONS](../LESSONS.md).
- Developer delegation requested Sol/medium and reviewer delegation requested Sol/high through fresh explicit-role agents, because this session exposes no native role selector. Those spawn requests were made; current task settings/backend execution telemetry were not independently verified. Earlier native-role metadata verification remains a separate historical check.

No application commit was created. Further features require a new approved scope.
