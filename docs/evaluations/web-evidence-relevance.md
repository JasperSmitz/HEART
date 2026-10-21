# Web evidence relevance and provider-mode comparison

Scope and frozen queries: [plan 0011](../plans/0011-web-evidence-relevance.md). The reported DID result class now has rejecting regressions; the original live response was not reproduced. This is a bounded lexical mitigation, not a general answer-verification system.

## Live comparison

[Captured responses](web-provider-comparison.json) preserve nine exact queries in Tavily basic and advanced modes, top-three bounded source snippets, query echoes, safe status/usage and observed latency. All 18 echoes matched the sent query. Tavily reported **27 credits** total: nine basic credits plus eighteen advanced credits. No retries, billing changes, account creation or key disclosure occurred.

Both modes returned Deltarune pages for the original question. Basic's first source described the first chapter; advanced's first source described the paid release and a later chapter. Those are different qualified answers to an ambiguous question, not evidence that one date simply replaces the other. The earlier DID response's cause remains unproven.

Both modes returned relevant-looking sources for the eight ordinary queries. Neither found the fabricated album subject: both returned other orchestras, demonstrating why nonempty results and provider rank are insufficient. Source/topic relevance is not truth, completeness or freshness verification. Weather snippets mixed climatology, forecasts and station readings.

Observed median request time was 1,718 ms for basic and 3,579 ms for advanced. These small sequential runs are not a randomized performance benchmark; they cannot establish a general provider ranking. Advanced sometimes provided useful extraction but also returned boilerplate, a clipped composer list and tables. It costs two credits versus one according to the [current Tavily search reference](https://docs.tavily.com/documentation/api-reference/endpoint/search).

A credential-free DuckDuckGo HTML reference returned HTTP 202 with an access challenge on the first query. Evaluation stopped without bypassing it; the remaining eight were skipped. This public HTML interface was only a reference candidate, not an adopted API dependency. No alternative API credentials or self-hosted endpoint were configured. Consequently, **a third-provider comparison was unavailable**, not a win or loss for that provider. No provider or search-depth default changed.

## Final module replay

The architect compiled a temporary wrapper around the actual production module's private response selector and replayed preserved snapshots without network, credentials or inference. [Final results](web-relevance-replay.json) include all 18 responses and three asserted synthetic regressions. [Initial replay](web-relevance-replay-initial.json) preserves defects exposed before correction.

| Query | Basic selector | Advanced selector | Interpretation |
|---|---|---|---|
| Deltarune release | Selected | Selected | Preserves first-chapter/paid-release qualifications respectively |
| Rust package manager | Selected | Selected | Explicit Cargo relationship |
| Skyward Sword composer | Declined | Declined | Useful cross-sentence context or incomplete sentence cannot qualify |
| Earth size | Selected | Selected | Explicit measurement |
| Python 3.13 release | Selected | Selected | Actual release sentence; planned date rejected |
| Linux disk-usage command | Declined | Declined | Useful instructions split across context or formatted outside eligibility rules |
| Amsterdam weather today | Declined | Declined | Available snippets do not satisfy the narrow sentence/current-signal rules |
| Netherlands capital | Selected | Selected | Explicit capital/seat distinction |
| Fabricated album | Declined | Declined | No subject-matching evidence |

Each mode yields five selected excerpts and four declines. This is selector behavior, **not a 5/9 factual accuracy score**. The composer and command cases are documented false negatives, not evidence that the providers lack useful material. No actual local-model paraphrasing was tested in this scope.

Synthetic checks confirm: DID-only results return Error with no canonical text/sources; a later relevant Deltarune result becomes source `[1]` with aligned evidence; a topic-only Deltarune definition cannot answer its release-date query. Both successful and declined paths preserve literal query forwarding and the single-request budget in transport fixtures.

## Review, verification and limits

All **66 all-target workspace tests** passed on macOS and offline Linux ARM64, plus formatting, warnings-denied Clippy and whitespace checks. Linux used the existing Rust image with read-only source/registry mounts and no container network. Independent review cleared after the two allowed focused correction rounds. Corrections addressed a planned-versus-actual Python release, incomplete excerpts, period-terminated question/how-to headings, and page numbers mistaken for measurements.

The module can select later eligible sentences/results and consistently order source URLs and canonical excerpts. Its lexical checks still do not establish semantic relationships, identity completeness, freshness or truth. Reviewer probes showed that a location question can accept a statement about the same mountain appearing in paintings, and a composer question can accept an unnamed group of musicians. Strong topic matches can still be insufficient answers. Do not add indefinite special-case patches or describe these checks as full relevance verification. Further general-answer reliability needs a separately scoped discussion.

Keep Tavily basic for now: this sample does not justify a production provider/depth change, especially with no working third-provider API comparison. Preserve the one-call implementation, exact fallback and evidence limits. Core, common result contract, models, personalities and other modules are unchanged.
