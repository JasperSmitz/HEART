# Authored numerical tool replies

Status: implemented, verified and independently reviewed; return to discussion.

## Authorization and scope

The user approved authored character templates for numerical tool results after the bounded phrasing follow-up, with future composition of canonical answers remaining a possibility. Implement this for disk space and uptime. Preserve structured module results and canonical text for consumers; do not implement multi-tool summaries or autonomous investigation now. Web search and OS-info retain the existing canonical-only generated phrasing path. Models/defaults, character instruction prompts and unrelated behavior stay unchanged.

## Design

Add optional `template_fields` to the common result envelope: a map of display-ready strings, supplied by modules when they request authored presentation. Absent fields retain the current generated path. Present fields select deterministic presentation: core looks up a profile's `templates` map by opaque capability ID and interpolates the supplied fields; no matching/valid template returns the exact module fallback without inference. This is a generic presentation contract, not core knowledge of disk or uptime.

Disk supplies already-formatted quantities including units (available/used/total/free/reserved as appropriate) and safe quoted canonical path, derived by the same native formatting code as its canonical answer. Uptime supplies its already-formatted duration. Raw byte/second data remains unchanged alongside these strings. Templates perform literal named substitution only: no expressions, number formatting, shell expansion, conditionals, loops or recursive interpretation of field contents. Template prose is trusted local authored configuration like greetings; do not use the LLM's lexical style guard to reject authored character vocabulary. Keep malformed/unknown/missing substitutions and unsafe output conservative, using exact fallback. Do not claim interpolation proves the truth of arbitrary user-authored prose.

Profiles gain optional `templates` keyed by capability ID. Supply brief distinct butler and cat disk/uptime wording, e.g. `You have {available} available, sir.`; field strings include their units, so templates do no conversions. Existing instructions/greetings/progress and legacy generated presentation metadata remain intact. A custom profile without templates remains compatible and receives direct facts for opted-in modules. Restart remains the reload mechanism.

## Acceptance criteria

- `/disk` and `/uptime` produce authored replies in both bundled profiles without any inference or retention requests, regardless of model availability. Literal/default path behavior and native byte/uptime semantics remain unchanged.
- Core is module-agnostic. Deterministic fixture tests cover an unrelated opaque capability, literal placeholder substitution, repeated fields, values containing braces, missing/unknown fields and malformed templates, custom/missing profiles, safe outputs, exact fallback and original sources/data.
- Only successful opted-in results use templates. Errors/NeedsInput stay safe and skip inference as before. Opt-in experimental routing may still use inference before execution; authored presentation does not add more calls, and routing cleanup still occurs.
- Diagnostics explicitly distinguish successful authored presentation from accepted generated phrasing and fallback. Zero routing/phrasing timings and no spurious retention warnings for explicit template replies. Preserve idle state when no inference occurred, including template fallback.
- Generated web/OS replies retain canonical-only inputs, existing numeric/lexical/unit guards, one-call limits and original fallback. No automatic generation when authored templates are missing/invalid.
- Tests independently verify supplied formatted fields match the existing canonical/native quantities (GiB, checked byte counters and uptime duration), all existing examples compile, profile JSON backward compatibility is exercised, and documentation covers syntax/fields/selection/limits with a pirate example.
- Run workspace/all-target tests, formatting, warnings-denied Clippy and independent source review. Architect verifies native CLI replies through a localhost counting fixture that would detect any attempted model request, with no Tavily keys/calls or downloads. Practical Linux checks reuse the existing offline container.

## Non-goals

Template framework dependencies, hot reload/editor, model changes/downloads, weakening generated-response guards, fixing experimental routing, dashboards/telemetry, multi-tool composition, arbitrary calculations inside templates, provider calls or commits.

## Workflow and results

Developer and independent reviewer used the fresh-agent fallback with Sol/medium explicitly requested; model/effort requests are not verified backend execution telemetry. No implementation correction round was required. Reviewer identified one stale architecture paragraph, which the developer corrected and reviewer rechecked. The reviewer otherwise found no actionable defects and made no project edits or new LESSONS entry.

Implemented common optional `template_fields`, profile `templates`, generic single-pass literal rendering, `Authored`/`AuthoredFallback` statuses and no-inference idle preservation. Disk supplies quoted canonical `path` and formatted `available`, `used`, `total`, `free`, `reserved` quantities including GiB units; uptime supplies `duration`. Original instructions/greetings/progress, native structured data, canonical text and source envelopes remain intact. Missing/invalid presentation returns exact fallback; generated web/OS behavior is unchanged. Existing examples compile, with smoke stage expectations updated for authored native replies rather than generation.

All **57 workspace/all-target tests** passed on macOS, plus formatting and warnings-denied Clippy. Linux ARM64 passed the 56-test all-target matrix followed by the subsequently added old-envelope/optional-field contract compatibility regression separately. Only a test was added between those runs; implementation was unchanged. Checks used locked/offline dependencies in the existing Rust container with read-only source/cache mounts and no network. Whitespace and documentation-link checks passed.

The architect's [counting-endpoint terminal evaluation](../evaluations/authored-numeric.md) passed: eight native authored replies in both voices, one safe native error, two missing-template direct fallbacks, one unknown-field direct fallback and one custom pirate uptime template. All thirteen tool results made zero inference or retention requests, with zero routing/phrasing diagnostics, safe literal paths and no model warnings. No Ollama/model or real provider was contacted; no Tavily credentials were used.

The scoped criteria are met. There were no dependencies, model/default changes, downloads, commits, template expression engine, autonomous investigation or combined-answer generation. Shared canonical results remain available for future composition; that is discussion-only future scope. Preserve prior routing/model-phrasing evidence rather than retroactively declaring those experiments repaired.
