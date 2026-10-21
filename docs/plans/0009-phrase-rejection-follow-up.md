# Phrase rejection follow-up

Status: source fixes implemented and independently verified; numerical personality experience remains unresolved. Return to discussion.

## Scope and authorization

The user reported repeated `Rejected(NumberChanged)` for direct `/disk /` requests in both voices and an Earth-size `/search` result after the command/profile implementation. Investigate and correct demonstrable phrasing/validation defects within that delivered behavior. Inspect original generated candidates in an explicit local evaluator, not normal CLI logs. Preserve the user's current profiles and all previous work.

## Acceptance and limits

- Establish whether reproduced rejections arise from changed values, harmless formatting, use of raw structured quantities instead of canonical text, or another evidenced cause. Preserve initial observations. The CLI's fallback text is not the rejected candidate.
- Correct proven formatting false positives conservatively, or improve the generic phrasing input/instructions for exact fact preservation when evidence supports that change. Keep module/core separation. Do not accept changed values through numeric tolerance, guessed unit conversions, deleted signs, substring matches or blanket guard removal. Do not rewrite profiles or change models/defaults.
- Keep final presented-answer validation, exact fallback, original sources, bounded one-call phrasing and zero routing on explicit commands. Existing Unicode signs/digits, negation, unit and injection regressions must remain protected. New tests cover each demonstrated defect plus meaningful counterexamples.
- Run relevant fixtures and all-target workspace tests, formatting and warnings-denied Clippy. Independently review implementation. Use a bounded actual-model check with native disk and fixed synthetic Earth evidence only; no Tavily calls/keys/downloads. Report acceptance and genuine remaining model mistakes separately; do not promise every answer will pass.
- No new feature architecture, automatic retries, multi-call correction loops, dependencies, broad cleanup or commits. If diagnosis requires a material change beyond these boundaries, report evidence and discuss scope with the user. At most two focused correction/review rounds.

## Roles

Developer and independent reviewer: Sol/medium requested through fresh-agent fallback because this session exposes no native role selector. Requested settings are not backend execution telemetry. Architect owns actual-model evidence and final acceptance.

## Results

Initial [local-only observations](../evaluations/phrase-rejection-initial.json) reproduced two bad disk phrases in three native requests: the module said `747.66 GiB`, while the model transformed raw `802792296448`-style byte counters into `802.79 GiB`, confusing decimal scaling with the provided binary-unit result. The validator correctly rejected those candidates. Both fixed Earth excerpt probes copied the comma-formatted values correctly, so the user's specific Earth rejection was not reproduced and must not be attributed to a known cause without evidence.

Implemented the canonical-only phrasing input, preserving the full module envelope for consumers. Deterministic regressions proved and fixed comma-component matching and missing GiB/GB/byte unit protection. Independent review identified a P2 interaction with lexical grounding for concise equivalent spellings; one focused correction removed validated numeric fragments from word scoring without adding numeric votes or lowering the threshold. Concise numeric/unit answers require canonical text grounding for accompanying short words as well. The generic prompt requests light edits and retention of factual vocabulary; profiles remain untouched.

All **51 workspace/all-target tests** passed on macOS and offline Linux ARM64 with locked dependencies, plus formatting, warnings-denied Clippy and whitespace checks. Independent reviewer cleared the correction and ran 34 compiled validator probes plus 30 non-socket core tests. Full authorized suites passed the socket fixtures too. Reviewer added one substantial LESSONS entry. Developer/reviewer model/effort were requested, not independently verified as backend execution telemetry.

The architect preserved [initial](../evaluations/phrase-rejection-initial.json), [canonical-only](../evaluations/phrase-rejection-canonical-only.json) and [final](../evaluations/phrase-rejection-final.json) actual-model observations. See [evaluation and limits](../evaluations/phrase-rejection.md). Final default-model run accepted 3/8 phrasings and 0/4 disk phrasings: two disk replies failed grounding on harmless synonyms; two invented totals and correctly failed number checks. Both Earth replies and museum negation passed; fare used the exact fallback after grounding rejected its paraphrase. No Tavily calls/credentials, downloads, model/default changes, dependencies or commits occurred.

The bounded source-fix and regression criteria are met, but the reported numerical-character experience is not solved. Stop prompt loops and return to the user with a recommendation for authored numeric templates or immutable facts with character framing. Such a presentation change requires discussion/approval; it was not silently implemented. Existing generated replies and exact fallback remain available, with limitations explicit.
