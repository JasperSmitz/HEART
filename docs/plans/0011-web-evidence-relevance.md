# Web evidence relevance and bounded provider comparison

Status: bounded mitigation implemented, tested and reviewed; general semantic answer reliability remains open. Return to discussion.

## Scope and authorization

The user approved a web-module refinement after `/search When did Deltarune release?` returned DID medical pages: reject clearly irrelevant or insufficient evidence, select a relevant answer-bearing excerpt rather than the first sentence, and compare search options on a fixed everyday task set. Keep provider logic and relevance handling in `modules/websearch`; core, module contract, model defaults and characters remain unchanged. Do not start autonomous browsing or redesign HEART.

## Frozen comparison matrix

Before live probing, freeze these exact queries:

1. `When did Deltarune release?`
2. `What is Rust's package manager called?`
3. `Who composed The Legend of Zelda: Skyward Sword?`
4. `How large is Earth?`
5. `When was Python 3.13 released?`
6. `How do I list disk usage on Linux?`
7. `What's the weather in Amsterdam today?`
8. `What is the capital of the Netherlands?`
9. `When did the Flarnox Pebble Orchestra release its Moonspoon album?`

The last query tests insufficient evidence, not a claim that no similarly named entity can exist. Assess subject relevance and explicit answer-bearing text separately from truth/currentness; dates/composer lists and weather require qualifications. Preserve all outcomes, including unavailable/blocked alternatives. No query rewriting or cherry-picked reruns to improve scores.

## Provider check boundaries

At most nine Tavily basic and nine advanced calls (27 documented credits at current 1/2-credit rates), no retries, using the existing locally stored key without printing/persisting it or provider error bodies. Record literal sent/echoed queries, request mode, response status, bounded source titles/URLs/excerpts and observed latency/usage. Keep auth out of captures. No billing changes, paid upgrades or accounts.

Try a credential-free independent search reference, initially DuckDuckGo's public non-JavaScript search, with at most nine requests; stop if unavailable/challenged and do not bypass access restrictions. Its HTML interface is a comparison reference, not an adopted production API. No Brave key or self-hosted SearXNG URL is currently configured; do not claim an API-provider comparison was completed if credentials/access are absent. Document limitations and make an evidence-based recommendation; do not silently replace production Tavily/basic or add unsupported scraping to the application.

## Implementation and acceptance

- Trace/test literal query forwarding through the current module request. A user-facing result does not establish the sent HTTP payload; preserve a credential-free fixture regression for the exact Deltarune query and capture the real query/response echo in the bounded comparison.
- Inside the web module, conservatively filter/rank sanitized bounded results and choose a query-relevant excerpt, including later answer-bearing sentences. A DID-only response for the Deltarune query must return a clear no-relevant-evidence non-success result, with no irrelevant fallback or generated claim. A top unrelated result must not hide a later relevant result.
- Avoid claiming heuristic checks prove semantic relevance or factual correctness. Prefer a small transparent lexical/answer-signal heuristic over an LLM classifier, domain/entity hardcoding, embeddings, large NLP frameworks or a search-engine rewrite. Generic dates/releases, named subjects and simple factual intent may be handled where supported by tests. Weak/ambiguous snippets should decline rather than force a confident answer. Record limitations, including valid paraphrases/short forms and incomplete evidence.
- Canonical factual text, exact fallback and displayed source indices must identify the selected actual evidence; keep data/source ordering consistent. Do not promote arbitrary statements from model memory. Preserve one production request/no retries, basic mode/default, URL/credential filtering, safe outcomes, response/excerpt budgets, sanitization and source integrity. No page fetching or automatic follow-up calls.
- Meaningful local fixtures cover reported wrong-topic results, reordered candidates, later answer sentences, topic-only/no answer, generic-word-only matches, punctuation/case/Unicode names and misleading high provider scores. Existing credential/quota/timeout, malformed/empty, safe-source and abbreviation tests remain relevant; adjust fixtures only where new query relevance makes their previous assertions invalid, not to conceal failures.
- Run workspace/all-target tests, formatting, warnings-denied Clippy, and independent review. Architect replays preserved provider snapshots through the final module selection path without spending more credits; root owns live comparison evidence. Linux checks use existing offline tooling where practical.
- Update README, architecture, module contract discussion and roadmap to reflect actual behavior and limitations. No core/application contract changes, provider/default changes, new dependencies, commits, model downloads or personality edits.

## Roles and results

Developer and independent reviewer used the fresh-agent fallback with Sol/medium explicitly requested; requests were not independently verified execution telemetry. Architect owned live captures, replay, final assessment and the two permitted correction rounds. No extra correction loops were spent.

Implemented private web-module response selection with bounded sanitized snippets, whole subject tokens, generic declarative eligibility plus stricter factual signals, later-sentence/candidate selection and coherent source/data ordering. Candidates failing checks return Error, no canonical text/sources and therefore no generated answer. Production remains one basic Tavily request, no retries, no automatic mode/provider fallback. Core, contract, profiles, model/defaults and dependencies were unchanged.

[Live comparison](../evaluations/web-provider-comparison.json) completed all 18 Tavily requests with exact sent/echoed queries and **27 reported credits**. Both modes returned Deltarune sources, so the original DID failure was not reproduced. The credential-free DuckDuckGo reference returned a challenge and stopped after one attempt; no third-provider API comparison could be completed. No keys, auth headers or provider error bodies were captured; no billing/account changes occurred. See [assessment and references](../evaluations/web-evidence-relevance.md).

Initial [snapshot replay](../evaluations/web-relevance-replay-initial.json) exposed a planned Python date selected over the actual release and incomplete/heading eligibility issues. Correction round 1 fixed those and preserved generic questions outside specific intent checks. Independent reviewer found period-terminated headings and unrelated page-number/measurement matches; correction round 2 fixed both with positive counterexamples. Reviewer rechecked the focused cases with no further blocking findings in that recheck. [Final production-selector replay](../evaluations/web-relevance-replay.json) verifies all 18 snapshots plus asserted DID-only, later-relevant-source and topic-only fixtures without additional provider credits or model calls.

All **66 workspace/all-target tests** passed on macOS and offline Linux ARM64, plus formatting, warnings-denied Clippy, whitespace and documentation links. Existing local HTTP fixtures used authorized loopback permissions; those tests passed. Linux used existing offline tooling/read-only mounts, no network or dependency downloads. All source/example fixtures compile. Docs explain lexical limits; no commits were created.

Both modes produced five selected excerpts and four declines in the final nine-query matrix. These counts are not factual accuracy scores: composer/instruction snippets contained useful material but were declined for context/format constraints. General semantic selection remains unresolved, including reviewer counterexamples where a same-subject predicate does not answer a location question or a composer statement names no musician. The reported obvious wrong-topic class and confirmed defects are mitigated; do not claim all irrelevant/insufficient answers are now recognized. Keep basic mode/default and return to discussion before further heuristic growth or a provider replacement.
