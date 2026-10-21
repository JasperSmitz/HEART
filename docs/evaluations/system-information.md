# System-information acceptance evidence

**Status: native implementation and source review complete; milestone acceptance remains open.** Explicit disk paths are being omitted by both tested local models. Do not equate a correct capability ID or successful native read with a correctly fulfilled request.

This report preserves plan 0005's original results. The subsequently authorized [optional-argument follow-up](optional-argument-routing.md) improved extraction but still leaves default-model routing acceptance open; consult that report for current behavior.

## Implemented and verified

`modules/system/` provides disk space, OS/version/architecture and uptime through the existing common contract. CLI registers these alongside web search; core contains no system/provider branches. Native responses contain structured data, deterministic factual text and empty sources. Collection uses safe `fs4` 1.1.0 filesystem snapshots and static `sysinfo` 0.39.6 functions, with scoped features. No file contents, processes, CPU/memory/temperature data or network services are queried by the module. No product elevation, shell execution, dashboard, polling or external profiles were added.

All **32 all-target workspace tests** passed on macOS and Linux ARM64 (`rust:1.99-slim`, Rust 1.99.1, locked dependencies, source/manifests mounted read-only, no API key). Formatting and warnings-denied all-target Clippy passed on macOS. The independent reviewer checked native semantics, path safety, boundaries, schema changes and focused regressions; no remaining source-code finding was reported. The unresolved finding is actual natural-language argument fidelity.

Native tests cover optional/default/relative/file/symlink paths, missing and unsafe paths, literal command-like filenames without execution, byte arithmetic/reserved capacity/saturation, missing metadata, boot-time failure/future clocks, uptime formatting, empty sources, invalid argument schemas and counted local/web dispatch. Core's factual guards remain unchanged.

## Actual model checks

Used the existing Ollama 0.40.6 service on an Apple M1 Pro/32 GiB Mac. Each model received five requests in both profiles: default disk space, explicit root path, system basics, uptime and a web-weather request. The web stub matches the production declaration and cannot call Tavily; no credentials were read. The source fixture records model-generated routes, module calls and envelopes, displayed replies and timing. A Linux container build ran concurrently; these are correctness checks, not isolated latency benchmarks. Models were unloaded after each run; the user-owned service remains running.

| Final observed behavior | 2B Q4_K_M | 4B Q4_K_M |
|---|---:|---:|
| Expected capability ID | 10/10 | 10/10 |
| Exactly one expected module invoked | 10/10 | 10/10 |
| Module returned success | 10/10 | 10/10 |
| Request arguments matched tested intent | 8/10 | 8/10 |
| Omitted path correctly used cwd | 2/2 | 2/2 |
| Explicit `/` correctly passed | **0/2** | **0/2** |
| Native results checked against independent readings | 8/8 | 8/8 |
| Exact factual fallback among native replies | 5/8 | 3/8 |
| Tavily calls | 0 | 0 |

The explicit-root request produced `system.disk_space` with `{}` in both profiles and models. The module consequently queried the process working directory, correctly following its input default but incorrectly fulfilling the user's request. The returned canonical path identifies what was actually checked; on another machine this may be a different filesystem.

Final full observations: [2B](system-information.json), [4B](system-information-4b.json). A bounded actual-CLI check with Tavily credentials explicitly disabled additionally tried:

1. `Check disk space for the path /private/tmp.`
2. `How much space is available on "/private/tmp"?`
3. `How much space is available on "/heart-no-such-volume-12345"?`

All three returned working-directory disk readings instead of honoring the supplied path. Quoting does not provide a verified workaround. In particular, an invalid requested path can incorrectly appear successful because it never reaches the module. Increasing to 4B did not resolve the observed argument omission. No model default was changed.

## Native comparison semantics

Actual returned disk counters were compared with Python's independent `os.statvfs` wrapper for each executed path. Total capacity matched exactly; maximum later-snapshot free/available/used drift was 3,026,944 bytes for the 2B run and 16,957,440 bytes for 4B. The evaluator records the differences and allows 1 GiB sampling drift during concurrent disk activity. Every captured result satisfies used = total − free and reserved = free − available.

On this Mac, `statvfs` reported shared APFS pool capacity/free space for both root and the Data volume. `df` agreed on total and available within sampling drift, but its per-volume Used count differed. HEART reports `statvfs` total-minus-free; this is not a guarantee of exclusive usage by a particular APFS volume or a physical-device inventory. Available/free/reserved are OS API readings and may reflect quotas or shared capacity. Do not compare the Used field to Finder/`df` as though they had identical accounting semantics.

Product version 26.5.2 matched `sw_vers`; architecture `arm64` matched `uname -m`. `sysinfo::System::name` returned `Darwin` on this Mac, so that is the reported OS-name string; the product version was not replaced by a kernel version. Both uptime readings per run fell within independently sampled wall-clock-minus-`kern.boottime` bounds. macOS boot-query failures are rejected before subtraction; Linux's static uptime zero remains ambiguous as documented in the module contract. Linux execution here covers container tests, not a Linux desktop/PC or model-runtime benchmark.

## Corrections and handoff

The [initial run](system-information-initial.json) had 9/10 correct IDs but only 5/10 expected module calls: independent ID/argument schema constraints allowed mismatched pairs, which runtime validation rejected. Correction round 1 bound each ID to its own complete route object. That [union run](system-information-union.json) produced valid pairs but selected the first disk branch in all ten cases.

Correction round 2 enabled JSON insertion-order preservation and a serialized-schema regression, placing the capability discriminator before arguments for constrained decoding. This improved observed capability selection to 10/10 for both models while preserving fail-closed validation. It did not establish optional-argument fidelity. The reviewer recorded the substantive lesson in [LESSONS](../LESSONS.md).

Developer/reviewer settings were requested as Sol/medium via explicit-role fallback; runtime metadata was not separately inspected. Source checks are distinct from the architect's actual-model/native/container evidence.

The two-round correction limit is reached. Recommendation: scope a focused follow-up on optional-argument generation/schema handling before accepting arbitrary-path natural-language disk queries. Retain the current model default and avoid masking the failure with a larger model or silent argument guessing. Dashboard, telemetry, model/personality refinements and commits remain deferred. No further code changes or claims of milestone completion are authorized by these results alone.
