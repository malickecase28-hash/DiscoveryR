# Decision Record: AP-002 E2 Execution Design

**Status:** PROPOSED — requires explicit approval before AP-002 E2 resumes

## Decision

Do not treat approval of the AP-001 Drift-Burst plan as an implicit amendment to AP-002.

AP-002 E2 currently has a frozen design that called for exhaustive execution of the 472-cell grid with 4,999 replicates before interpretation.

The proposed change is to replace that execution pattern with:

1. deterministic discovery across the approved E2 question space;
2. interpretation of the frozen discovery output;
3. promotion of only supported and scientifically coherent candidates;
4. expensive resampling and method challenge only for promoted candidates;
5. untouched confirmation only after challenge survival.

## Reason

The change separates inexpensive phenotype discovery from expensive inferential challenge.

It is intended to reduce compute and governance cost without weakening confirmation discipline.

## Constraints

- The approved E2 question universe is not silently expanded.
- Discovery and challenge remain separate.
- Candidate-promotion criteria must be frozen before discovery outcomes are used for selection.
- Existing confirmation custody remains unchanged.
- No candidate reaches confirmation without the required challenge.
- The change does not retroactively alter completed AP-002 E1 evidence.

## Approval choices

- `APPROVED` — amend AP-002 E2 to the two-pass discovery/challenge design.
- `REJECTED` — retain the frozen exhaustive 472-cell / 4,999-replicate design.
- `DEFERRED` — keep AP-002 E2 stopped until a later decision.
