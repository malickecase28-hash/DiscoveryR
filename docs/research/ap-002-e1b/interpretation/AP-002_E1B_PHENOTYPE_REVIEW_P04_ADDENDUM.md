# AP-002 E1B SEALED ADDENDUM - Interpreter P-04 (CONTRACT_COMPLETION)

- Generated (UTC): 2026-09-06T14:19:30Z
- Interpreter: P-04. Short sealed addendum ordered by the director after E1B reconciliation (PASS_TO_SYNTHESIS). No full reinterpretation; no peer interpretation or peer addendum read; no pipeline code touched.
- Program: AP-002 FVG | Stage: E1B lifecycle-stage phenotype | Instrument: XAUUSD | Exposure: E1_PHENOTYPE
- Confirmation: LOCKED | E2/E3: NOT STARTED | Cross-scale object identity: PROHIBITED (respected)
- Basis: `E1B_FILL_STRATIFICATION.json`, `E1B_INTERPRETER_FACT_CHECK.json`, own prior review `AP-002_E1B_PHENOTYPE_REVIEW_P04.json` (reference only).
- Cohort identity verified: stratification `anchors_logical_rows_sha256 = 911301ee…285c2` equals the frozen E1B anchor logical rows hash; totals reconcile (379,132 fills = 378,122 touched + 1,010 gap-through).
- Citation notation: `E1B_FILL_STRATIFICATION.json:cells[tf][FILL_TYPE].horizons[h].<field>`; horizons = 1/3/5 lawful native closes after the FILL boundary tick; all return figures are p50 in bps unless stated.

**Predeclared support rule** (E1B_FILL_STRATIFICATION.json:predeclared_support_rule; rule fixed before inspecting per-cell aggregates; labels carry no significance meaning; P-04 did not choose or re-derive it): DESCRIPTIVE_ADEQUATE anchor_n >= 200 | DESCRIPTIVE_THIN 30-199 | TOO_SPARSE_FOR_COMPARISON 1-29 | EMPTY 0.

## 1. Touched-fill vs gap-through: any descriptive difference?

Paired comparison made ONLY where both cells are predeclared DESCRIPTIVE_ADEQUATE: **15s and 30s**.

### 15s (touched n=226,210: 115,315 bull / 110,895 bear; gap-through n=682: 350 bull / 332 bear)

| anchor | h | adj p50 bps | raw p50 bps |
|---|---|---|---|
| TOUCHED_FILL | 1 | +0.0217 | +0.0211 |
| TOUCHED_FILL | 3 | +0.0661 | +0.0387 |
| TOUCHED_FILL | 5 | +0.0848 | +0.0544 |
| GAP_THROUGH_FILL | 1 | 0.0000 | -0.0296 |
| GAP_THROUGH_FILL | 3 | +0.0716 | +0.0679 |
| GAP_THROUGH_FILL | 5 | +0.1902 | +0.2626 |

DIRECT OBSERVATION: h1 IQRs nearly identical (touched -0.8392..+0.9104, width 1.750; gap -0.8496..+0.8642, width 1.714). Difference (gap minus touched, adj p50): h1 -0.0217, h3 +0.0055, h5 +0.1054 - sign NOT consistent across horizons; |differences| <= 0.105 bps against IQR widths 1.7-5.2 bps. Gap-through has a heavier right tail (h3 p95 +25.50 vs +5.98 bps). End-of-window missing 0 in all cells.

### 30s (touched n=99,011: 51,116 bull / 47,895 bear; gap-through n=213: 119 bull / 94 bear)

| anchor | h | adj p50 bps | raw p50 bps |
|---|---|---|---|
| TOUCHED_FILL | 1 | +0.0298 | +0.0388 |
| TOUCHED_FILL | 3 | +0.1077 | +0.0752 |
| TOUCHED_FILL | 5 | +0.1607 | +0.1102 |
| GAP_THROUGH_FILL | 1 | +0.8894 | +0.9279 |
| GAP_THROUGH_FILL | 3 | +1.3700 | +1.3700 |
| GAP_THROUGH_FILL | 5 | +0.8009 | -0.2248 |

DIRECT OBSERVATION: gap-through adj medians higher at ALL THREE horizons (differences +0.8596 / +1.2623 / +0.6402 bps); h1 IQR ~2.3x wider (gap -0.8005..+5.2041, width 6.005 vs touched width 2.620); |median| = 14.8% of h1 IQR width vs 1.1% for touched; h1 p95 +15.14 vs +5.14 bps. Caveats: n=213 (bearish arm 94), and raw-vs-adjusted sign divergence at h5 (raw -0.2248 vs adj +0.8009). Touched h5 has 1 end-of-window missing (outcome_n 99,010); all gap cells 0.

**Answer 1 (INTERPRETATION).** At 15s: NO descriptive difference in the distribution body (difference sign flips across horizons). At 30s: a single-stratum contrast - gap-through fills show larger, more dispersed direction-adjusted medians at all three horizons - descriptively visible but small-N and sign-sensitive to the direction adjustment at h5. **No scale-consistent descriptive difference; the 30s contrast is registered as new descriptive information, not a finding.** HYPOTHESIS FOR LATER TESTING (refines QG2): does gap-through fill carry systematically stronger zone-direction post-fill drift than ordinary touch->fill at strata with adequate N, under a drift-aware null (15s n=682 as replication stratum, 30s n=213 as motivating observation)?

Cells explicitly excluded from comparison: 1m gap-through n=81 DESCRIPTIVE_THIN (single-cell description only: adj p50 0.0000 / -0.9421 / +2.4689 bps at h1/h3/h5); 5m n=20, 15m n=10, 1h n=4 TOO_SPARSE_FOR_COMPARISON (not interpreted); 4h n=0 EMPTY.

## 2. Which native strata have enough N for even descriptive comparison?

Using the predeclared eligibility labels (thresholds fixed before per-cell inspection):

| stratum | touched n | gap n | labels | paired comparison |
|---|---|---|---|---|
| 15s | 226,210 | 682 | ADEQUATE / ADEQUATE | **ELIGIBLE** |
| 30s | 99,011 | 213 | ADEQUATE / ADEQUATE | **ELIGIBLE** |
| 1m | 43,227 | 81 | ADEQUATE / DESCRIPTIVE_THIN | not eligible (single-cell description only) |
| 5m | 6,961 | 20 | ADEQUATE / TOO_SPARSE | not eligible |
| 15m | 2,094 | 10 | ADEQUATE / TOO_SPARSE | not eligible |
| 1h | 478 | 4 | ADEQUATE / TOO_SPARSE | not eligible |
| 4h | 141 | 0 | DESCRIPTIVE_THIN / EMPTY | not eligible |

**Answer 2.** Only **15s (n=682)** and **30s (n=213)** support an eligible paired touched-vs-gap-through descriptive comparison. 1m is DESCRIPTIVE_THIN (single-cell description only); 5m/15m/1h TOO_SPARSE; 4h EMPTY. The 4h TOUCHED_FILL cell itself is DESCRIPTIVE_THIN (n=141), reinforcing the small-N caution already carried for 4h in P-04.

## 3. Does the new evidence change any prior P-04 conclusion?

| prior P-04 conclusion | status |
|---|---|
| Touched-vs-gap-through comparison requires a later frozen measurement (QG2) | COMPLETED by this addendum (FILL anchor only); not contradicted |
| Pooled FILL medians = weak positive central tendency dominated by touched-fill population | CONFIRMED: stratified touched-fill adj p50 match pooled FILL medians to <= 0.008 bps (15s h1 +0.0217 vs +0.0214; 30s h1 +0.0298 vs +0.0299; 1m h1 +0.1041 = +0.1041; 4h h1 +8.3089 = +8.3089) |
| Gap-through adequacy: 15s adequate; 30s/1m marginal; 5m-4h inadequate | CONFIRMED and formalized by predeclared labels (1m now formally DESCRIPTIVE_THIN - marginally more conservative, consistent) |
| AN2 4h FILL h1 raw -0.1451 vs adj +8.3089 bps sign sensitivity (N=141) | REINFORCED (cell confirmed; whole 4h touched population labeled DESCRIPTIVE_THIN) |
| AN1 exact-zero median mass at fine strata | REINFORCED (15s gap h1 adj p50 exactly 0.0, n=682; 1m gap h1 exactly 0.0, n=81) |
| Bearish cohort not sign-consistent | CONFIRMED; one tally corrected (below) |
| Overall verdict PASS_TO_SYNTHESIS; must-not-claim list | UNCHANGED |

**Answer 3. NO major P-04 conclusion changes.** The addendum completes the deferred FILL-anchor stratification measurement and registers one new small-N descriptive observation (30s gap-through contrast). FIRST_TOUCH-stage material is untouched; QG1 and QG3 remain unmeasured.

## Fact-check acknowledgement (correction accepted in one line)

**CORRECTION ACCEPTED: P-04's bearish FILL direction-adjusted p50 sign tally is 10 positive / 1 zero / 10 negative (mechanical count, E1B_INTERPRETER_FACT_CHECK.json:claims_verified.P04_bearish_fill_10_pos_2_zero_9_neg = false), not the 10 / 2 / 9 P-04 reported - the disputed cell is 30s FILL h1 bearish p50 = -2.1e-12 bps, mechanically classified negative under strict sign where P-04 classified |x| < 1e-9 as zero; both agree the value is zero to within float epsilon, and the substantive conclusion (bearish FILL behavior not sign-consistent) is unchanged.**

All other P-04 claims checked were verified TRUE: raw p50 positive 41/42; bullish positive 42/42; bearish FIRST_TOUCH 5 pos / 5 zero / 11 neg; fill-adjusted positive 21/21; first-touch adjusted positive 21/21.

## Must-not-claim (addendum scope)

- The 30s gap-through contrast is a single-stratum, n=213 descriptive observation - not a confirmed difference, threshold, or trading effect.
- No comparison outside the two predeclared-eligible pairs; TOO_SPARSE/EMPTY cells are not interpreted.
- Eligibility labels carry no significance meaning; the rule was fixed before per-cell inspection and was not chosen by P-04.
- No alpha/edge/profitability/strategy/signal claims; no significance or confirmation claims; confirmation LOCKED; E2/E3 NOT STARTED.
- No cross-scale identity or pooling; all statements within-stratum.
- Raw-vs-adjusted sign divergence in small cells (30s gap h5; 4h touched h1) must not be quoted as either sign alone.
- The fact-check is mechanical verification only; TRUE labels on other interpreters' claims are not P-04 evidence.

## Recommended next scientific action

Carry this addendum to E1B synthesis as a contract-completion record: QG2 now has eligible-stratum motivating evidence (15s null result, 30s small-N contrast); QG1 and QG3 remain unmeasured; no further E1B measurement recommended; any test remains gated behind a frozen E2 contract.
