# HEART

A cozy personal terminal workspace around a Rust assistant: work in your configured shell and use HEART commands for brief factual answers in an editable character voice. Authored profile templates present disk/uptime results; a local Ollama model phrases other successful results; Tavily performs one basic web search for `/search`; separate native modules read filesystem space, OS/architecture and uptime. No chat history is stored. Multiple terminals and expanded module workspaces remain future milestones.

Install a current Rust toolchain and [Ollama](https://ollama.com). Start `ollama serve` in a separate terminal (or use your existing local Ollama service). Download a model explicitly; HEART never installs one:

```sh
ollama pull qwen3.5:2b-q4_K_M
cp .env.example .env.local
# Edit .env.local locally to set TAVILY_API_KEY. Never commit it.
cargo run -p heart -- --model qwen3.5:2b-q4_K_M
```

The default is `qwen3.5:2b-q4_K_M`, selected from a small local fixture evaluation against the 4B Q4 candidate ([original results](docs/evaluations/milestone-1.md)); this does not establish accuracy for arbitrary questions or performance on the 16 GB target. The [polishing-pass comparison](docs/evaluations/phrasing-polish.md) records improved phrasing checks, model variability and a remaining 2B fixture-routing failure. Use `--model` or `HEART_MODEL` to select an installed tag. Missing runtime/model returns the completed module result verbatim; missing search credentials produces a clear module message; there is no automatic cloud fallback. Tavily usage is subject to your account allowance; HEART makes one basic request with no retries and never enrolls or changes billing.

Use `/disk [path]`, `/system`, `/uptime`, and `/search <query>`. `/help` lists commands from installed module declarations, available profiles, and built-ins. The remainder after a command is one literal string: leading/trailing whitespace is trimmed and internal spaces are preserved. Quotes are literal characters, so write `/disk /path/with spaces` without wrapping quotes. Omitted optional input stays absent (disk uses the working directory); there is no guessed input, expansion, shell execution or inference for selection/extraction. Unknown commands, missing required arguments and arguments to no-input commands invoke nothing. Disk/uptime replies use authored templates and contact no model, including when templates are missing or invalid. Other successful results receive at most one local phrasing call; module errors receive none.

Natural-language routing is an experiment enabled only with `--experimental-nl`. It selects a capability first, then extracts only its arguments when needed; empty-input capabilities skip extraction. Historical routing failures below remain unresolved. Explicit commands work independently of those stages.

Use `/cat`, `/butler`, `/profile ID`, or `/ID` for any discovered profile, and `/exit` to leave. `--personality ID` selects the initial voice (default `butler`). Add `--diagnostics` for safe phrasing categories and stage timings (stderr in console, dialogue in dashboard); explicit commands report `routing_performed=false`, zero routing timings and no extraction. Diagnostics add no persistent logs or inference calls. `--endpoint` or `OLLAMA_HOST` selects the inference HTTP endpoint (default `http://127.0.0.1:11434`). Environment variables take precedence over optional project `.env.local`. `HEART_IDLE_SECONDS` controls model unload after the last request that attempted inference (default 300; 0 unloads immediately). Startup does not contact Ollama or wait for model loading. Inference calls hold the model while active, then reset the idle timer. Explicit module errors and authored replies (including template fallback) contact no inference endpoint and preserve the existing idle timer.

## Cozy dashboard prototype

See the [real Pastel/Ghostty screenshot](docs/evaluations/terminal-home/pastel-ghostty.png), [Butler screenshot](docs/evaluations/terminal-home/butler-ghostty.png), and [verification/compact screenshots](docs/evaluations/terminal-home.md).

Run `cargo run -p heart -- --dashboard` in an interactive Linux/macOS terminal. Dashboard starts with Pastel (profile ID `cat`); `--personality butler` or a discovered custom ID selects another initial character. The no-flag console still defaults to Butler. Portrait expressions/typewriter effects remain future work; `--no-animation` remains accepted. Opening the dashboard invokes no assistant module/model/provider; no Ollama service/key is needed to view it. The configured shell runs its usual startup commands. In HEART input, `/disk` and `/uptime` use real native data and authored replies without model requests; `/system`, `/search` and opt-in natural-language requests retain their existing inference behavior.

The dashboard opens one interactive `$SHELL` in a real PTY and starts with **shell focus**. Its normal rc files, environment, aliases and prompt run unchanged. **F12 switches between the shell and HEART input**. With shell focus, Ctrl+C interrupts the shell's foreground command, Tab completes, arrows recall/edit shell history, and F1/F2/Ctrl+Q go to the child. They do not invoke HEART controls. The focused border, header and footer identify the active context. The outer terminal must deliver F12 to HEART.

To talk to Pastel, press F12, enter a HEART command such as `/uptime`, then press F12 to return to the same shell cwd and state. In HEART context, the controls below retain their existing meaning. Shift+PgUp/PgDn while shell-focused scroll a bounded 2,000-line shell history; typing returns to live output. Paste goes to the focused context and respects the child's bracketed-paste mode, with a 64 KiB shell paste limit. Exiting HEART closes its shell and attached foreground job; deliberately detached processes are outside that cleanup guarantee. Shell exit/start failure leaves HEART controls available and does not automatically restart a shell.

Manual shell commands are never assistant tool invocations: no model, capability or assistant response can write to the terminal. The no-flag console retains its existing command interface and starts no embedded shell. Shell history files follow your shell configuration; HEART's assistant history remains in memory only. See [embedded-terminal verification and limitations](docs/evaluations/embedded-terminal.md).

System **STATIC DEMO** values are fixed illustrations. Music **LOCAL MOCK** playback/track controls only change local UI state; they play no audio. READY/WORKING describes HEART's request state, not service connectivity. The shell fills the left workspace vertically beneath the header. The right sidebar stacks System, Music, compact character dialogue and HEART input; smaller terminals hide utilities and shrink/omit companion artwork before sacrificing shell space. The shell preserves normal cursor placement: a fresh prompt can start at the top, and output scrolls at the bottom once it fills the pane. There is no widget polling, real music or new capability.

The sidebar layout needs at least 100 columns and 16 rows; System/Music appear
at 30+ rows. Narrower or shorter windows keep a compact bottom companion/input
strip to preserve shell width. [Layout screenshots and verification](docs/evaluations/shell-first-layout.md)
show both arrangements.

| Keys | Action |
| --- | --- |
| F12 | Globally toggle shell / HEART input |
| Tab / Shift-Tab in HEART | Cycle visible HEART controls and shell; shell Tab always completes |
| Enter in HEART | Submit input / toggle focused music |
| ← / →, Home / End | Edit input; music arrows change mock track |
| ↑ / ↓ in input | Recall command history, returning to the current draft |
| ↑ / ↓, PgUp / PgDn, Home / End in dialogue/help | Scroll |
| F1 / Esc | Toggle help / close help and focus input |
| F2 | Cycle discovered characters and themes |
| Space in music | Toggle mock playback |
| Ctrl-Q / Ctrl-C / `/exit` in HEART | Quit promptly, including during an assistant request |

Input supports Unicode character editing and a horizontal viewport, with a 2,000-character draft limit. A single worker executes requests; submitting another capability while busy preserves the draft. Help and profile selection remain available. Each reply keeps the character selected when submitted. The dashboard keeps a bounded in-memory display (80 entries) and input history (50), with no persistence or model conversation context. Safe `--diagnostics` appear in dialogue rather than over the alternate screen. Normal exit, initialization errors and panics restore raw/alternate-screen mode and cursor through Ratatui and a scope guard.

Pastel's supplied portrait uses real Kitty graphics on Ghostty/Kitty with available terminal pixel metrics, keeping sharp nearest-neighbor scaling and proportions. Unknown terminals, multiplexers, missing metrics and nonempty `NO_COLOR` use a labelled text fallback; `HEART_PORTRAIT=text cargo run -p heart -- --dashboard` forces it. Butler/custom assets currently use text fallback until artwork is added. No generated character art, Nerd Font or font changes are used. True-color RGB, Unicode widths and cell metrics depend on your terminal; choose fonts and line height in its settings. See the [asset notes](crates/cli/assets/README.md).

A deterministic preview uses the actual renderer with mock state, without reading credentials/configuration or constructing services:

```sh
cargo run -p heart --example dashboard_preview -- cat 120 40 > /tmp/heart-cat.svg
cargo run -p heart --example dashboard_preview -- butler 80 24 help > /tmp/heart-help.svg
# Last argument: home (default), help, or long (long source-list fixture).
```


Profiles are one JSON file each under `profiles/` relative to the process working directory. `--profiles-dir PATH` overrides it (relative paths also use that working directory). All `.json` files are loaded at startup; filenames need not match IDs. Restart HEART after edits. Invalid JSON/unknown fields, empty or control-containing required profile text, invalid/duplicate IDs and collisions with installed commands or built-ins fail startup clearly. IDs use lowercase ASCII letters, digits and hyphens. The chosen initial profile must exist; a custom-only directory needs `--personality ID`.

Each profile has `id`, `instructions` (trusted character system text), `greeting`, `status` (generic experimental-routing status), optional `progress` (map of opaque capability IDs to authored status text), optional `templates` (map of opaque capability IDs to literal answer templates), optional `presentation` (`suffix` and `markers`), and optional `appearance` for dashboard styling. A module's default progress is used when no override exists. The bundled butler uses presentation metadata for its existing honorific; cat has none. Generated answers, including any suffix, pass the existing factual guard. Its fixed style-word allowlist may reject new vocabulary. Authored templates use trusted local prose directly, without this generated-word guard or suffix processing. Neither profile path authorizes actions.

An optional `appearance` contains `portrait` (`cat`, `butler`, or `neutral` builtin asset), `display_name`, and semantic `background`, `surface`, `text`, `accent`, `gold` colors as `#RRGGBB`. All fields are required when the section is present; unsupported assets, malformed colors and empty/control-containing names fail startup clearly. Omitting the section gives a neutral palette and lantern portrait, preserving older custom profiles. Rendering selects art through the asset key, so a custom profile can reuse Cat/Butler independently of its ID. Appearance has no effect on voice, templates, capability authority or model prompts.

For example, add `profiles/pirate.json`, restart, then use `/pirate` or `--personality pirate`:

```json
{
  "id": "pirate",
  "instructions": "A concise, friendly pirate. Preserve facts and answer directly.",
  "greeting": "Ahoy! What shall we find?",
  "status": "Checking the charts.",
  "progress": { "system.uptime": "Checking the ship's clock." },
  "templates": {
    "system.disk_space": "Arrr, {available} available on the filesystem containing {path}.",
    "system.uptime": "The ship's computer has been running for {duration}, matey."
  }
}
```

A successful module result with `template_fields` selects authored presentation. Core looks up `templates` by its opaque capability ID; absent fields retain generated phrasing. Existing custom profiles without templates stay valid and receive exact direct fallback for opted-in modules. Missing/invalid templates never trigger model generation. Restart reloads edits.

Disk (`system.disk_space`) provides `{available}`, `{used}`, `{total}`, `{free}`, `{reserved}` as two-decimal quantities already including `GiB`, plus `{path}` as a safe JSON-quoted canonical path. Uptime (`system.uptime`) provides `{duration}` including day/hour/minute/second units. These reuse native answer formatting; structured byte/second data and canonical text remain unchanged for consumers. Web search and OS info retain generated phrasing.

Syntax is literal named substitution, `{name}`, with ASCII letters/digits/underscores and no leading digit. Repeated names work; field values are copied once, including any braces. There is no brace escape, recursive expansion, expression, calculation, conversion, loop, conditional or shell expansion. Unknown/missing fields, malformed braces, empty output, terminal controls, line separators, bidirectional controls or output over 8192 UTF-8 bytes return the exact module fallback; templates themselves are limited to 8192 bytes. Authored prose is trusted configuration: interpolation cannot verify arbitrary claims written by a profile author. Diagnostics report `Authored` versus `AuthoredFallback`, separately from generated `Accepted`; authored replies have no phrasing timing.

Trusted local profile files are presentation configuration. The phrasing payload contains the request and canonical module text as separate data fields; structured data and source metadata stay outside model input. Canonical text uses `factual_text` when present, otherwise `fallback`; neither gains action authority. Character instructions cannot bypass independent runtime argument checks or factual guards.

Search selects a short sentence that passes lexical evidence checks from at most three sanitized snippets (900 characters each), including later candidates/sentences. It requires the query’s subject words and a substantive declarative predicate or stricter factual answer signals in that sentence; candidates that fail those checks return a non-success result without phrasing. Scores and titles cannot substitute for answer evidence. This conservative lexical heuristic accepts general subject-and-predicate declarations and applies stricter signals to release dates, composers, capitals, package managers, physical size, explicitly current temperatures and inline-code instructions. Selected sentences must end with punctuation; planned, conditional, future and negative statements decline. It can miss valid paraphrases, short names, cross-sentence context and other question forms. Matching words/signals do not establish relationships, correctness, completeness or freshness; a dated or qualified excerpt remains only that evidence. No page fetching or follow-up searches occur. The [bounded evaluation](docs/evaluations/web-evidence-relevance.md) records both missed valid answers and same-subject statements that still fail to answer the question. Tavily basic remains the default; the third-provider comparison was unavailable.

Search snippets are evidence, not verified truth. The model can paraphrase canonical text, with instructions to minimally edit factual vocabulary, add brief character wording, copy numbers and units exactly, and avoid calculations, conversions, rounding or outside knowledge. Conservative number/unit/negation and grounding checks cannot guarantee semantic accuracy. Numeric checks allow exact ASCII thousands grouping and trailing decimal zeros, while preserving signs and scale; disk units distinguish GiB, GB and bytes. Trusted system instructions select the character; neutral butler wording gets a short authored honorific; cat answers use subtle warmth without a forced “nya”. When phrasing fails, HEART returns an attributed factual excerpt with the original source links. Insufficient evidence and provider errors are reported honestly. Review sources for consequential decisions.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
# Optional local model evaluation; no Tavily calls or credentials:
cargo run -p heart-core --example evaluate -- qwen3.5:2b-q4_K_M
# Real native readings + counted web stub, both voices; never calls Tavily:
cargo run -p heart --example system_smoke -- qwen3.5:2b-q4_K_M
```

Tests use loopback HTTP fixtures and need permission to bind local sockets in restricted sandboxes. The `evaluate` example exercises the experimental router and emits JSON timing, phrasing status/rejection reasons, raw structured candidates and result records for both profiles across five questions: temperature, fare, negation, Rust/Cargo and Ocarina of Time composer. Evidence is fixed and synthetic, with an injection fixture; it is not a recorded provider response. Candidate capture is confined to this evaluator. Review factual correctness/style, inspect Ollama memory separately, and distinguish cold from warm runs. See [module contract](docs/MODULE_CONTRACT.md), [architecture](docs/ARCHITECTURE.md), [milestone plan](docs/plans/0003-first-milestone.md), and [workflow](AGENTS.md). The original milestone fixture tests passed on macOS and Linux ARM64; the explicit command/profile pass has all-target automated coverage on macOS and offline Linux ARM64. Live inference was tested only on the M1 Pro/32 GiB Mac.

The `/disk`, `/system` and `/uptime` commands use read-only native capabilities and need no Tavily key. Disk space refers to the filesystem containing a path, not directory contents. Omitted paths use the process working directory; an existing file or directory selects its filesystem. Relative paths use that working directory and symlinks resolve to the reported canonical path. Paths are literal: no `~`, environment-variable, wildcard or shell expansion. Missing/inaccessible paths, control characters and paths that cannot be displayed as UTF-8 fail safely. Native readings never request elevated permissions.

**Experimental-routing limitation:** `--experimental-nl` enables an unaccepted two-step routing experiment. The explicit command default bypasses this router. The default 2B model passed 16/24 live checks: it dropped some explicit paths, substituted disk readings for unsupported operations, and answered a public factual question with OS information. The 4B model passed 22/24 but mishandled both omitted paths. Direct module validation works; it cannot verify that a valid route matches the user's intent. These historical natural-language acceptance failures remain open; inspect the reported canonical path when using experimental routing. See [current two-step results](docs/evaluations/two-step-routing.md), [prior routing evidence](docs/evaluations/optional-argument-routing.md) and [original system-information evidence](docs/evaluations/system-information.md).

Disk data contains exact `total_bytes`, `free_bytes`, `available_bytes`, `used_bytes` (total minus free) and `reserved_bytes` (free minus available); human text uses GiB (2³⁰ bytes), rounded to two decimals. Available is what non-privileged users can use, so available plus used may be less than total. A single `fs4::statvfs` snapshot supplies all counters. Saturated/overflow or inconsistent counters return an error. Reads are a point-in-time sample and can change immediately afterward.

On APFS, these API counters may describe shared pool capacity; total-minus-free can differ from `df`'s per-volume Used. They do not promise exclusive physical-drive or per-volume accounting. System OS-name text is what the API reports (on this Mac, `Darwin`); the product version remains the macOS version.

System information uses static `sysinfo` APIs, with no process/CPU/memory refresh. Missing OS name or product version appears as unavailable/null, never as a substituted kernel version. Architecture comes from `System::cpu_arch`: native machine architecture where supported, or the HEART build architecture if the underlying native query fails; this API does not expose which source it used. macOS may call it `arm64`, while Rust's build target calls it `aarch64`. No host/user names or serial numbers are collected.

Uptime is whole seconds since system boot. Linux `sysinfo::System::uptime` reads `/proc/uptime`, falling back to the native sysinfo call; it includes suspended time, and zero cannot distinguish an unsupported/failed native read from a just-booted system. On macOS HEART uses `System::boot_time` and checked wall-clock subtraction, matching sysinfo's uptime semantics while rejecting a missing/zero or future boot timestamp. It includes sleep and can be affected by wall-clock adjustments. No permission escalation or shell command is attempted.

The `system_smoke` example intentionally exercises the experimental natural-language API and emits the unchanged twenty regression JSON lines (ten requests in both profiles): omitted disk path, explicit `/`, existing absolute path, relative `.`, path containing spaces, nonexistent path, OS/architecture, uptime, external weather lookup and an unsupported request. Four additional butler-only holdouts are tagged separately: unquoted root and existing fixture paths, an unsupported folder-open request, and a Rust package-manager public lookup. These are predefined wording checks, not statistical independent validation. It creates temporary empty directories for path fixtures and removes them after the run. Each line contains the raw selection and extraction, a reconstructed combined route for comparison, per-stage inference calls/timings/errors, expected literal module arguments, actual module invocations, expected outcome/canonical path, counted calls, captured module envelopes, final reply/status/timing, individual acceptance checks and `tavily_calls: 0`. A failed check produces a nonzero exit status after all observations are printed. Its only web capability is a fixed local stub even if the model misroutes a local question; it neither constructs the Tavily module nor reads credentials. Native results are real local readings, not synthetic fixtures. Review route accuracy and factual preservation separately; the existing conservative phrasing guard may return the exact native fallback.

The explicit command/profile implementation is scoped by [plan 0008](docs/plans/0008-explicit-commands-and-profiles.md); see its verification record for current acceptance evidence.

The [numerical-phrasing follow-up](docs/evaluations/phrase-rejection.md) fixed canonical-input and numeric-validation defects but found frequent disk fallback. [Authored numerical replies](docs/plans/0010-authored-numeric-replies.md) now give disk and uptime deterministic character presentation. Generated web/OS phrasing retains conservative validation and its documented limitations.
