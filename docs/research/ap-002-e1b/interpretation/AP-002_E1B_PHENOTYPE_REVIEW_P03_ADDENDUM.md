# AP-002 E1B PHENOTYPE REVIEW — P-03 — SEALED ADDENDUM (CONTRACT_COMPLETION)

- **Interpreter:** P-03 | **Generated (UTC):** 2026-09-06T14:18:57Z | **Addendum type:** CONTRACT_COMPLETION
- **New evidence (only):** `E1B_FILL_STRATIFICATION.json`, `E1B_INTERPRETER_FACT_CHECK.json` (+ my own prior review for reference). No peer interpretation or addendum inspected.
- **Reconciliation:** touched-fill 378,122 + gap-through 1,010 = 379,132 fill anchors; anchors sha256 `911301ee...` matches the prior review basis. Every touched-fill cell equals the prior per-stratum fill count minus gap-through (e.g. 30s: 99,224 − 213 = 99,011); the 4h touched-fill cell (n=141) is the entire 4h FILL cohort (gap-through = 0).
- **Fact-check acknowledgment (one line):** The mechanical fact check verified all three P-03 sign-count claims TRUE — FIRST_TOUCH direction-adjusted p50 positive 21/21, FILL 21/21, FILL > FIRST_TOUCH in 19/21 — and issued no corrections against P-03.

**Predeclared support rule** (fixed before per-cell inspection; labels carry no significance meaning): DESCRIPTIVE_ADEQUATE anchor_n ≥ 200; DESCRIPTIVE_THIN 30–199; TOO_SPARSE_FOR_COMPARISON 1–29; EMPTY 0.

---

## Q1. Does touched-fill vs gap-through behavior support any descriptive difference?

**Answer: No robust descriptive difference is supported** (interpretation). Direct observations:

- **15s** (touched n=226,210; gap-through n=682; both DESCRIPTIVE_ADEQUATE): direction-adjusted p50 h1/h3/h5 = **+0.0217 / +0.0661 / +0.0848** (touched) vs **0.0 / +0.0716 / +0.1902** (gap-through) bps. Ordering flips across horizons; every value is a small fraction of the respective h1 IQRs (touched [−0.839, +0.910]; gap-through [−0.850, +0.864]). Descriptively indistinguishable.
- **30s** (touched n=99,011; gap-through n=213; both DESCRIPTIVE_ADEQUATE): touched **+0.0298 / +0.1077 / +0.1607** vs gap-through **+0.8894 / +1.3700 / +0.8009** bps — gap-through uniformly higher (5–13× at h1/h3) with sign-consistent positive medians. But: n=213, the h1 median gap (0.86 bps) sits inside a wide gap-through IQR [−0.800, +5.204], and the gap-through cell is modestly bullish-overweight (119/213 = 55.9% vs 51.1% touched) while the established post-fill lean is bullish-carried — composition-confounded.
- The two comparable strata disagree (15s: no difference; 30s: gap-through higher). A stratum-inconsistent, single-stratum pattern is **not** a descriptive difference; it is a registered hypothesis.
- **Mixture check (closes a P-03 unverified expectation):** pooled FILL medians (E1B_RESULTS.json) minus touched-fill-only medians (new artifact) differ by at most ~0.002 bps across shared cells (e.g. 15s h1 0.0214 vs 0.0217; 30s h3 0.1092 vs 0.1077; 1m h1 identical to 6 decimals) — the 0.19–0.83% gap-through mixture negligibly moved pooled quantiles, as P-03 anticipated.
- The 1m gap-through cell (n=81, THIN) already shows sign-flipping, non-monotone medians (0.0/−0.942/+2.469 bps), illustrating the predeclared labels' purpose. The 5m (20), 15m (10), 1h (4) cells are TOO_SPARSE and the 4h cell EMPTY (0); their quantiles are deliberately not quoted and carry zero interpretive weight.

## Q2. Which native strata have enough N?

**Answer: a touched-fill vs gap-through descriptive comparison is adequately supported ONLY at 15s (682) and 30s (213)** — the only strata where both populations carry DESCRIPTIVE_ADEQUATE.

| TF | Touched-fill n (label) | Gap-through n (label) |
|----|------------------------|------------------------|
| 15s | 226,210 (ADEQUATE) | 682 (ADEQUATE) |
| 30s | 99,011 (ADEQUATE) | 213 (ADEQUATE) |
| 1m | 43,227 (ADEQUATE) | 81 (THIN) |
| 5m | 6,961 (ADEQUATE) | 20 (TOO_SPARSE) |
| 15m | 2,094 (ADEQUATE) | 10 (TOO_SPARSE) |
| 1h | 478 (ADEQUATE) | 4 (TOO_SPARSE) |
| 4h | 141 (THIN) | 0 (EMPTY) |

Touched-fill alone is adequate at 15s–1h and thin at 4h, consistent with the P-03 small-N caveats.

## Q3. Does the new evidence change any prior P-03 conclusion?

**Answer: No major conclusion changes.**

- Unchanged: the stage sign flip (formation 18-negative-of-21 → FT/FILL 21/21 positive; fact-check TRUE); the weak-central-tendency, bullish-carried characterization (stratified touched-fill medians reproduce pooled medians within ~0.002 bps); the tail/censoring ceilings and no-unconditional-p99-beyond-1m rule; the split assessment and must-not-claim list (extended below).
- Resolved: P-03 AN-5 (gap-through outcomes unmeasured) is closed by the stratification artifact; P-03 QG-2 now has a partial descriptive answer — no robust touched-fill vs gap-through difference at the two adequately powered strata.
- New registered hypothesis (only): 30s gap-through post-fill medians uniformly exceed touched-fill (n=213 vs 99,011) — hypothesis-grade, stratum-inconsistent, composition-confounded.
- Superseded sentence: P-03's "even at 15s (n=682) no per-population outcomes are committed" — they are now committed, and 15s shows descriptive indistinguishability.

**Claim corrections from the fact-check: none** (all three P-03 claims TRUE; fact-check same-bar rates reproduce P-03's computed shares exactly at all seven strata).

## Must-not-claim (addendum-scoped)

No alpha/edge/strategy claims (nothing tested; confirmation LOCKED; E2/E3 NOT STARTED); the 30s gap-through pattern is not an established effect; no interpretation of the TOO_SPARSE (5m/15m/1h) or EMPTY (4h) cells; eligibility labels are support labels, not significance statements; no cross-scale identity or pooling (all comparisons within-stratum, same anchor, same horizon); the unquoted sparse-cell quantiles must never be quoted later as findings.

**Addendum status: NO_REWORK — prior P-03 review stands.**
