# Authored numerical replies

Scope: [plan 0010](../plans/0010-authored-numeric-replies.md). This verifies deterministic character presentation for disk and uptime; web/OS generation and experimental routing retain their separate limitations.

## Native terminal check

[Raw observations](authored-numeric.json) were recorded on the M1 Pro/32 GiB Mac using native readings, temporary working directories and a localhost counting endpoint. That endpoint returned an error to any attempted model request and never forwarded traffic to Ollama or providers. No `.env.local` or Tavily key was used. There was no inference, model download/loading/unloading, service reconfiguration or real web request.

All checks passed:

- Eight native authored replies across both bundled profiles: omitted disk path, root, relative `.`, an absolute path with spaces, a path containing literal `{available}`, uptime, then cat root disk/uptime. Path values with braces stayed literal, not recursively expanded.
- One nonexistent path returned the existing safe module error.
- A custom pirate profile without templates returned direct disk and uptime facts. A template with an unknown field returned the exact direct fallback, while its valid custom uptime template was used.
- All thirteen tool results made **zero model or retention requests**. Explicit diagnostics showed zero routing/phrasing timings, authored success/fallback statuses, and no spurious model warnings or generated rejections.

One butler reply was `You have 746.75 GiB available on the filesystem containing "/", sir.`; the cat's uptime reply was `Still purring after 89 days, 22 hours, 2 minutes and 59 seconds of uptime.` Values are live samples, not expected constants. Native tests independently verify GiB/byte accounting and duration formatting against the original canonical result.

## Automated checks and review

The final macOS all-target workspace suite passed **57 tests**, plus formatting and warnings-denied Clippy. Linux ARM64 passed the 56-test all-target suite and the subsequently added contract compatibility regression separately, using the existing offline Rust image, read-only source/registry mounts and no container network. This covers the same final implementation without repeating unaffected tests for a test-only addition.

Independent review found no application defects, checked the renderer/native/idle cases, and rechecked the late compatibility regression. One stale architecture paragraph was corrected and rechecked. No new reviewer lesson was needed. Full structured data, canonical text and source lists remain intact for future consumers; composed/generated multi-tool answers are not implemented.

Templates are trusted authored prose. The renderer verifies syntax/output safety and copies fields verbatim; it cannot establish that arbitrary custom template wording describes a value honestly. Missing or invalid templates return exact fallback without trying generation. Tests of the generated path and its guards remain in the passing suite; no new live-model or Tavily acceptance is claimed here.
