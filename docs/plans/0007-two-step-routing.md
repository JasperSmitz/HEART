# 0007 — Two-step routing experiment

**Status:** experiment concluded and independently reviewed; correctness criteria unmet. Current implementation remains unaccepted. Return to discussion before further work.

## Authorization and scope

The user approved the proposed experiment with “Let's go for that” after plan 0006 left default-model routing unaccepted. Separate capability selection from extraction of that capability's arguments, and measure the extra local inference cost. This supersedes plan 0006's prohibition on extra routing calls for this experiment only. Preserve historical evidence, module interfaces, existing default 2B model, phrasing guards and user changes. No commits.

First select only an opaque registered capability ID or null from IDs/descriptions and the complete request. Validate this selection independently of constrained generation. Then extract arguments using only the selected declaration and original request; extraction cannot select another capability. Keep generic optional-null projection and strict normalization to original module inputs. Empty-input capabilities need no extraction call. Unsupported selection must stop without extraction or invocation. No module-specific parsing, guessed paths, extra retries, cloud calls, model downloads, dependencies, dashboard, telemetry or profile work.

## Acceptance and verification

- Pass the unchanged twenty-case system smoke matrix on both installed 2B and 4B models: defaults, root, existing absolute/relative/spaced/missing paths, OS, uptime, required web query and unsupported operations, both profiles. Compare literal arguments, capability, invocation counts, native outcomes and canonical paths separately.
- Before live evaluation, define a small additional set of wording variations covering unquoted paths, another unsupported operation and public factual lookup. Report these separately; do not tune only to the original matrix or weaken it.
- Deterministic tests cover exact call order/count, selected-only extraction payload/schema, empty-input shortcut, null/unknown/missing/malformed selection, extraction failure/timeout/invalid arguments and optional normalization. Invalid stages must never invoke a module; safe errors, lease reset/unload and no-retry behavior remain intact. Keep at most two routing calls plus existing optional phrasing call.
- Record selection and extraction timings separately and preserve a combined routing measurement for diagnostics/comparison. Compare warm medians with preserved plan 0006 final results, matching cases and excluding first-request cold load. Target no more than 1 second additional median warm routing time for 2B on the current Mac. Report sample size, cold behavior and measurement limitations; a larger cost needs an explicit recommendation rather than silent acceptance.
- Formatting, warnings-denied all-target Clippy, workspace/all-target tests on macOS and practical Linux container tests; independent implementation review and architect-owned actual-model checks. Use only native modules and the credential-free counted web stub. No actual Tavily calls or key access.
- Preserve failed observations and allow at most two correction/recheck rounds, then return unresolved issues to the user.

## Delegation and completion

Developer requested as Sol/high for the demonstrated routing/debugging complexity; independent reviewer Sol/medium with proportional scrutiny of changed inference boundaries. Use explicit-role fallback when native role selection is unavailable and distinguish requested settings from runtime telemetry. Developer reads relevant project docs/lessons, implements and reports tests/deviations; reviewer edits only substantial lessons when warranted. Architect owns final live evidence, plan/architecture/roadmap status and the acceptance recommendation. Successful bounded samples establish this experiment's acceptance, not universal intent reliability or 16 GB/PC performance. Return to discussion after the result; dashboard remains unapproved.

## Execution and result

Implemented strict ID/null selection followed by selected-only extraction, skipping extraction for empty-input declarations. Module interfaces, default model, optional-null normalization, runtime validation, fallback guards and idle cleanup remain intact. Added separate/combined routing timings and the predefined 24-case evaluator with exact call-stage assertions. No retries, dependencies, real Tavily calls, key access, downloads, global changes or commits.

All 38 all-target tests passed on macOS and Linux ARM64, plus formatting and warnings-denied Clippy. Independent review found no deterministic implementation defect but confirmed unmet model-routing criteria. The 2B run passed 14/20 original plus 2/4 variation cases; 4B passed 18/20 plus 4/4. The additional median paired warm routing cost was 169 ms and 121.5 ms respectively, satisfying the 2B cost target but not correctness. See [preserved evidence, failures and sampling limits](../evaluations/two-step-routing.md).

No correction round was initiated: after the prior prompt loops, the experiment exposed failures in both simpler stages without an evidence-supported code correction. Stop rather than spend the two-round maximum automatically. Recommendation is to discuss a bounded lightweight-model comparison on the frozen cases before further architecture changes. This authorizes neither downloads nor a default-model switch. Parent milestone acceptance remains open; current checkout contains this unaccepted experiment. Developer/reviewer settings were requested via explicit-role fallback, not verified execution telemetry.
