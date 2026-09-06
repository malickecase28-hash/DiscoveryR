# AP-002 E2 Contract V1

Frozen by the Rust `ap002_qg_freeze` tool. Machine record: `AP-002_E2_CONTRACT_V1.json`.

## Data identity

Execution binds to the corrected 16:15 DEVELOPMENT scope and the scope-repair sidecar used by active E1A/E1B. Authority V1 supplies semantics only.

## Lifecycle state model

Market states: FORMED_UNTOUCHED, TOUCHED_UNFILLED, FILLED. Lawful terminal paths: FORMED_UNTOUCHED -> TOUCHED_UNFILLED -> FILLED; FORMED_UNTOUCHED -> FILLED (gap-through, producer first_touch_observed=false); TOUCHED_UNFILLED -> FILLED. Observation status is separate: OBSERVED_TERMINAL, RIGHT_CENSORED_UNTOUCHED, RIGHT_CENSORED_TOUCHED. Same-bar resolution is TOUCH_AND_FILL_SAME_COMPLETED_BAR (no intrabar inference). No separate producer invalidation event exists.

## Competing-risk rule

For formation->first-touch: FIRST_TOUCH is the event of interest; FILL_WITHOUT_RECORDED_TOUCH is a competing terminal event (never non-informative censoring); END_OF_DEVELOPMENT is right censoring. Touched->fill: end-of-development active touched zones are right censored.

## Graduated primary instruments

| Instrument | Registry ids | Cells | Multiplicity family |
| --- | --- | --- | --- |
| E2-PANEL | ["DIR-01","PD-A","PD-B","PD-C"] | 42 | MF_STAGE_MATCHED_PANEL = 2 matched stage contrasts (formation->first-touch, pre-fill->fill) x 7 strata x 3 horizons |
| E2-DIRNULL | ["PD-D"] | 84 | MF_DIRECTION_ASYMMETRY_NULL = 2 directions x 2 stage anchors x 3 horizons x 7 strata |
| E2-CORE-01 | ["QG1-CORE-01","PD-F","SF-2-attribute-aspect"] | 1 | declared per instrument record |
| E2-CORE-03 | ["QG1-CORE-03"] | 1 | declared per instrument record |
| E2-CORE-04 | ["QG1-CORE-04"] | 1 | declared per instrument record |
| E2-CORE-05 | ["QG1-CORE-05"] | 1 | declared per instrument record |
| E2-CORE-09 | ["QG1-CORE-09"] | 1 | declared per instrument record |
| E2-CORE-10 | ["QG1-CORE-10","PD-E","PD-G"] | 1 | declared per instrument record |
| E2-CORE-12 | ["QG1-CORE-12"] | 3 | declared per instrument record |
| E2-SF-1 | ["SF-1"] | 98 | declared per instrument record |
| E2-SF-2 | ["SF-2"] | 126 | declared per instrument record |
| E2-SF-5 | ["SF-5"] | 63 | declared per instrument record |

## Robustness lane

CORE-13 (1 cell), OPT-02 (21), OPT-04 (28): method robustness reported separately from behavioral inference.

## Multiplicity

E2 primary 135 (blind core 9 + stage-matched panel 42 + direction-asymmetry null family 84); systematic 287 (SF-1 98, SF-2 126, SF-5 63); robustness 50; total 472. V1's 346/412/758 accounting is historical QG-V1 bookkeeping only.

## Wording rules

STAGE-CONDITIONED INFORMATIONAL RELATIONSHIP, never 'causal stage effect'. Decision rule: SURVIVES_STAGE_CONDITIONING iff within-zone stage contrast survives AND matched temporal/drift-aware null survives AND selection diagnostics survive. PD-D temporal blocks used for null construction do not become a temporal-context finding.

## Execution rules

One Rust research pass materializes the reusable same-domain features once and evaluates the entire frozen E2 universe from them. No outcome-based pruning. Behavioral and robustness lanes never share multiplicity accounting. HISTORY_INCOMPLETE anchor counts reported under the warm-state rule. No E3 execution authorized by this contract. Confirmation LOCKED.
