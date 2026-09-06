# AP-002 E1 FINAL METHOD CHALLENGE

- **artifact_type**: AP-002_E1_FINAL_METHOD_CHALLENGE
- **challenger_id**: AP-002-E1-FINAL-INDEPENDENT-SKEPTIC
- **generated_utc**: 2026-09-06T14:47:19Z
- **Program / instrument / exposure**: AP-002 FVG / XAUUSD / E1_PHENOTYPE
- **Scope**: XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only
- **E1A active logical result**: `1b701c016bd1effa300742ee74ce73d6e0e96313d62847d231391061dc94dca6`
- **E1B active logical result**: `2a7a142cadb36449d14d4128361d11ce99a3a4d7d8a004dd539bc4dc32fb823c`
- **Confirmation**: **LOCKED**. E2/E3 NOT STARTED. Cross-scale object identity PROHIBITED (respected).

Role: final independent scientific skeptic. Did not build the scanners, did not produce
P-01/P-02/P-03/P-04, did not write the reconciliations. No scanner code read-modified or
re-run; no row-level tables (`.runs/**`) consumed; evidence base limited to the frozen
artifacts; all claims re-checked by direct arithmetic against committed tables and by
citation of the committed mechanical audits
(`AP-002_E1A_SCOPE_REPAIR_AUDIT.json`, `E1B_INTERPRETER_FACT_CHECK.json`,
`FILL_TOUCH_CONSISTENCY_AUDIT.json`). Hash registries re-verified file-by-file.

---

## 1. Explicit decisions

| Decision | Verdict |
| --- | --- |
| **E1A_FORMATION** | **PASS** |
| **E1B_FIRST_TOUCH** | **PASS** |
| **E1B_FILL** | **PASS** |
| **COMPLETE_AP002_E1** | **PASS** (with mandatory annotations, section 8) |
| **QUESTION_GENERATOR_ELIGIBILITY** | **YES** |

**Sign-reversal classification: `SURVIVES_AS_DESCRIPTIVE_COHORT_PATTERN`.**

All 18 canonical claims survive **as descriptive phenotype statements**. No scanner
defect was demonstrated. Nothing in E1 licenses a stage-effect, drift, or directional
structure claim; those are gated behind E2 (section 3).

---

## 2. Claim-by-claim classifications (1–18)

| # | Claim | Verdict | Key verification |
| --- | --- | --- | --- |
| 1 | Formation→touch median = 2 native bars, all 7 strata | **SURVIVED** | p50/bar-width = 2.0 at every stratum in E1A and E1B; p25 = 1 bar; touched-population quantity (touch incidence 0.9401–0.9946) |
| 2 | Formation→fill body rapid vs full observed tail | **SURVIVED** | p50 = 7/7/7/7/8/8/5 bars vs observed p95 = 717/695/670/701/562/403/170 bars; observed tail itself a censored lower bound |
| 3 | First-touch→fill median = 1 native bar, all 7 strata | **SURVIVED** | `touch_to_fill_market_ms.p50` = 1.0 everywhere; p25 = 0; conditional on touched-fill (censor reported separately) |
| 4 | ~34–40% of touched-and-filled fill on the first-touch bar | **SURVIVED** | same_bar/touched_fill = 0.3531/0.3541/0.3536/0.3443/0.3419/0.3661/0.4043 (15s..4h); reproduced by fact-check |
| 5 | Persistent touched-but-unfilled population; censoring worsens with scale | **SURVIVED** | 476/294/201/78/42/19/16 zones; touch→fill censor 0.210%→10.191% monotone; never-filled 0.446%→15.569% |
| 6 | Formation-anchor adjusted medians 18 neg / 1 zero / 2 pos | **SURVIVED** | recounted and mechanically verified (scope-repair audit); zero = 15s h1 (`-0.0` sign-bit artifact); weak central tendency (magnitude audit) |
| 7 | FIRST_TOUCH adjusted medians positive 21/21 | **SURVIVED** | +0.0136..+13.2671 bps; fact-check TRUE; interpretation REQUIRES_E2 (raw also 21/21 positive = drift signature) |
| 8 | FILL adjusted medians positive 21/21 | **SURVIVED** | +0.0214..+18.1627 bps; fact-check TRUE; raw only 20/21 (4h h1 −0.145, n=141); interpretation REQUIRES_E2 |
| 9 | FILL > FIRST_TOUCH medians in 19/21 cells | **SURVIVED** | exceptions 1h h3 and 4h h1; fact-check TRUE; fine-stratum differences are quantile noise (15m h1 diff = +0.0011 bps vs IQR ~16.5 bps) |
| 10 | Sign reversal is descriptive across selected cohorts, NOT a causal stage effect | **SURVIVED** | classification `SURVIVES_AS_DESCRIPTIVE_COHORT_PATTERN`; cohorts are nested by outcome selection; anchor price and clock differ; no matched evidence |
| 11 | Pooled post-touch/fill positive tendency is bullish-carried | **SURVIVED** | bullish 42/42 positive; bearish 15/42 positive (FT 5/5/11, FILL 10/1/10 pos/zero/neg); drift attribution REQUIRES_E2 |
| 12 | Bearish stage-anchor medians not sign-consistent | **SURVIVED** | mechanically verified; P-04's FILL tally (10/2/9) FAILED the fact-check and was corrected to 10/1/10 — substance unchanged |
| 13 | Fine-scale central tendencies small vs dispersion | **SURVIVED** | \|p50\|/IQR-width 0.56–2.64% at 15s–15m (h1); \|p50\|/p05–p95 span 0.15–0.70% |
| 14 | Gap-through fill is a legitimate producer-semantic case | **SURVIVED** | A1 audit 1,010/1,010 `first_touch_observed=false`, zero touch events, fill bars beyond far edge; fill-touch audit 0/379,132 disagreements |
| 15 | No robust touched-fill vs gap-through difference; 30s hypothesis-grade only | **SURVIVED** | counts 682/213/81/20/10/4/0 confirmed; paired-eligible only 15s (null) and 30s (n=213, composition-confounded contrast); support labels ≠ significance |
| 16 | Left-truncated/orphan events correctly excluded | **SURVIVED** | 178 touch / 358 fill orphans; `all_orphans_left_truncated=true` per stratum; reconciled with scanner counters; E1B reconciles identical |
| 17 | Raw observed fill fractions are not lifetime probabilities | **SURVIVED** | fill_rate + censor = 1 by construction; window-truncated lower bounds; ceilings committed per stratum |
| 18 | Cross-scale object identity must not be inferred | **SURVIVED** | authority prohibition; both integrity gates verify native identities only; no pooled analytic statistic in any frozen artifact |

---

## 3. Mandatory selection audit

**Cohorts.** FORMATION = all lawful in-window formations (N = 381,498; 227,909 / 99,866 /
43,701 / 7,140 / 2,192 / 523 / 167). FIRST_TOUCH = only zones that emit a touch
(379,248; 94.0–99.5% of formations). FILL = only zones that fill (379,132; 99.6–84.4%),
itself split touched-fill 378,122 / gap-through 1,010. Touch and fill are **outcomes**,
so the later cohorts are outcome-selected subsamples of the formation cohort, and the
anchor price changes across stages (formation availability tick vs boundary-crossing
tick inside/at the zone) with a shifted horizon clock.

**Finding.** Any stage-level difference confounds (i) outcome selection, (ii) anchor
re-definition, (iii) horizon-clock shift, and (iv) drift interacting with cohort
direction composition. E1 contains no matched, within-zone, or time-matched evidence
separating these.

**Frozen-wording check.** P-03 labels the stage comparison INTERPRETATION and states
"anchor populations are selected (touched/filled zones)" / "not a tested effect"; P-04
registers cohort conditioning and anchor-price definition among confounds and prohibits
"incremental predictive value" language. **No frozen text converts the cohort pattern
into a within-object stage effect.** Classification:
**SURVIVES_AS_DESCRIPTIVE_COHORT_PATTERN**. Any stage-effect wording (approach-phase
spending, recentering, continuation, reversion) is REQUIRES_E2.

**Recommended E2/QG experiment — "E2 STAGE-MATCHED LIFECYCLE PANEL"** (recommended only;
not executed):

1. **Within-zone panel**: per stratum, for every zone touching within a pre-registered
   window, direction-adjusted returns at matched calendar offsets from BOTH the
   formation anchor and the first-touch anchor (and the fill anchor where filled).
   The within-zone touch-vs-formation contrast on identical clock offsets holds zone,
   direction, calendar time and drift exposure fixed.
2. **Selection model**: censor-aware comparison of touched vs never-touched and filled
   vs touched-unfilled zones on gap_atr, fvg_quality, direction, session/time-of-day —
   so selection into touch/fill is measured, not assumed away.
3. **Drift-aware null**: per stratum/horizon, a time-matched unconditional no-FVG
   control (same calendar grid, same direction mix) giving the null distribution of
   direction-adjusted medians under pure window drift.
4. **Pre-registered decision rule**: stage-conditioned = within-zone sign change that
   also beats the time-matched null; selection-only = sign change vanishes under
   matching. Cohorts, denominators, censoring treatment and nulls frozen in an E2
   contract before any computation. No strategy rules; E3 untouched.

---

## 4. Mandatory drift / direction audit

- **Raw returns are drift-contaminated**: raw medians positive 21/21 (formation),
  21/21 (FIRST_TOUCH), 20/21 (FILL; sole negative 4h h1 = −0.145 bps, n=141). The
  raw-return interpretation must not be used for directional claims.
- **Direction adjustment does not remove calendar drift**: sign(formation)×return is
  biased positive for bullish zones and negative for bearish zones under upward drift,
  and pooling leaves residue proportional to the direction imbalance — while the
  formation population is bullish-skewed, increasingly with scale (51.0%→60.5%).
- **The observed asymmetry matches the drift signature exactly**: bullish adjusted
  medians positive 42/42; bearish mostly zero/negative (FT 5 pos / 5 zero / 11 neg;
  FILL 10 pos / 1 zero / 10 neg). The bullish-carried pattern is therefore not
  distinguishable from drift-carried without controls.

**Required before promotion** (claims 7, 8, 10, 11, 12 as directional structure):
- **Drift-aware null** — time-matched unconditional return distribution per
  stratum/horizon.
- **Direction-matched control** — bullish-only and bearish-only vs same-direction
  unconditional windows.
- **Time-matched control** — same calendar grid, no FVG requirement, to separate
  FVG-specific from calendar/session effects (including session-gap concentration of
  gap-through fills).

---

## 5. Mandatory censoring audit

Observed-event quantiles and censored population quantities are kept distinct in the
committed artifacts; `fill_rate + right_censored_rate = 1` is an accounting complement,
never two findings. Committed ceilings (max identifiable unconditional quantile):

| Stratum | formation→fill | touch→fill | formation→touch (= touch incidence) |
| --- | --- | --- | --- |
| 15s | 0.9955 | 0.9979 | 0.9946 |
| 30s | 0.9936 | 0.9970 | 0.9944 |
| 1m | 0.9910 | 0.9954 | 0.9938 |
| 5m | 0.9777 | 0.9889 | 0.9859 |
| 15m | 0.9599 | 0.9803 | 0.9745 |
| 1h | 0.9216 | 0.9618 | 0.9503 |
| 4h | 0.8443 | 0.8981 | 0.9401 |

**p95/p99/max language that must be corrected or labeled before admission:**

1. Touch→fill p99 at 5m/15m/1h/4h (2,388 / 1,648 / 699 / 175 bars) is an **observed-event
   (filled-only) quantile**; unconditional p99 is identifiable only at 15s/30s/1m.
2. Formation→fill p99 at 5m/15m/1h/4h (4,654 / 2,835 / 1,249 / 462 bars) is observed-event
   only.
3. Formation→fill p95 is unconditionally identifiable through 15m but **not at 1h/4h**.
4. Formation→touch p95 at 4h is **not identifiable unconditionally** (ceiling = touch
   incidence p94.01 < 95). The committed 89-bar value is touched-only. (Subtler than the
   fill cases; recorded here as a required annotation — P-03/P-04 cover the fill and
   touch→fill ceilings but not this one.)
5. All "max" durations (~118.5 days, recurring across 15s–1h at the window end) are
   administrative window-boundary artifacts, never natural lifetimes.
6. No unconditional quantile statement at 4h beyond p84.4 (fill) / p89.8 (touch→fill) /
   p94.0 (touch) is licensed.

No frozen text misquotes an observed-event quantile as unconditional; the ceilings are
used correctly in P-03/P-04 with the single annotation above.

---

## 6. Mandatory same-bar audit

**Denominator verified**: `same_bar_touch_fill_n / touched_fill_n` =
0.3531 / 0.3541 / 0.3536 / 0.3443 / 0.3419 / 0.3661 / 0.4043 (15s..4h) — the canonical
denominator in the frozen artifacts and in `E1B_INTERPRETER_FACT_CHECK.json`. This is
correct: a same-bar event requires both touch and fill, so its natural cohort is
touched-and-filled zones (alternatives give 33.5–40.4% — same range).

**Distinct phenotype mode vs natural point mass: do not force either interpretation.**
For a distinct mode: stable ~34–41% share at every stratum, p25 = 0 with p50 = 1
(median sits exactly on the zero-mass boundary). For a geometric point mass: touch is
range overlap and fill is far-edge reach, so any bar whose range both overlaps a
~0.5-ATR zone and reaches the far edge resolves it same-bar — common by construction
for narrow zones — and **no post-fill outcome comparison between same-bar and multi-bar
fills exists anywhere in E1** (P-04 QG3 registered, unmeasured). Verdict: the
descriptive fact SURVIVES; "distinct phenotype" is INCONCLUSIVE, gated behind QG
candidate G; the point-mass reading is the default until matched evidence says
otherwise.

---

## 7. Mandatory gap-through audit

`E1B_FILL_STRATIFICATION.json` confirms **GAP_THROUGH_FILL n = 682 / 213 / 81 / 20 /
10 / 4 / 0** (15s..4h), total 1,010; touched_fill_total 378,122; fill_anchors_total
379,132 — exactly as required.

**"No robust scale-consistent descriptive difference" is justified and correctly
bounded**: paired comparison is lawful only at 15s and 30s under the predeclared size
rule. 15s (n=682): ordering flips across horizons, |diff| ≤ 0.105 bps against IQR
widths ~1.7–5.2 bps — a null. 30s (n=213): gap-through medians uniformly higher
(+0.889/+1.370/+0.801 vs +0.030/+0.108/+0.161 bps) but wider IQR (~2.3x), bullish
composition 55.9% vs 51.1%, and contradicted by the 15s null — single-stratum,
hypothesis-grade. 1m (n=81, THIN) already sign-flips; 5m/15m/1h TOO_SPARSE; 4h EMPTY.
Mixture-negligibility: pooled vs touched-only medians differ ≤ ~0.002–0.008 bps.

**Label discipline**: DESCRIPTIVE_ADEQUATE / THIN / TOO_SPARSE / EMPTY are
predeclared denominator-based support labels fixed before inspection; they are **not**
statistical significance and must never be read as such.

---

## 8. Mandatory magnitude audit

|p50|/IQR-width at h1 (FIRST_TOUCH / FILL): 15s 0.81%/1.22%; 30s 0.56%/1.14%;
1m 1.74%/2.64%; 5m 1.68%/1.87%; 15m 0.76%/0.71%; 1h 3.98%/4.81%; 4h 15.16%/10.77%.
|p50|/p05–p95 span: 0.15–3.24% (FT), 0.19–2.49% (FILL). Coarse-cell spans: 4h FT h1
329.7 bps (n=157); 4h FILL h3 534.2 bps (n=141).

**Claims that reduce to weak central tendency despite sign consistency:**

- **Claim 6** — formation 18/1/2 pattern: median magnitudes −0.000..−2.094 bps (+0.852/
  +12.508 exceptions) against p05–p95 spans 6.4..561 bps; the two positive cells are
  small-sample medians (n=523, n=167).
- **Claim 7** — FIRST_TOUCH 21/21 positive: medians 0.15–3.24% of the p05–p95 span at
  15s–15m.
- **Claim 8** — FILL 21/21 positive: same reduction; raw-vs-adjusted sign divergence at
  4h h1 (n=141) shows sign fragility at small N.
- **Claim 9** — 19/21 count verified, but fine-stratum differences are quantile noise
  (+0.0011 bps at 15m h1 vs IQR ~16.5 bps; +0.0078 bps at 15s h1); only 1m–4h h3/h5
  differences are descriptively non-trivial.
- **Claims 11/12** — the composition facts (bullish-carried; bearish inconsistent) are
  robust as censuses; per-direction magnitudes are weak at fine strata, small-N at
  coarse strata.

Claims 1, 3, 4, 5 (frequency/count claims) and 14/16 (resolved anomaly audits) are
large-mass, large-N facts, not median-vs-dispersion cases.

---

## 9. Question Generator handoff (registered; none tested)

| ID | Candidate question | Priority | One-line justification |
| --- | --- | --- | --- |
| **A** | Is the formation→touch sign change a genuine stage-conditioned effect after matching the same eligible population (within-zone matched anchors, matched clock offsets)? | HIGHEST | 18-neg/1-zero/2-pos vs 21/21-positive across cohorts differing by selection, anchor price and clock; no within-zone evidence exists; E2 STAGE-MATCHED LIFECYCLE PANEL is the designated instrument |
| **B** | Does post-touch behavior differ from matched formation-stage and time-matched controls? | HIGH | Complements A with a between-zone matched control; rules out drift and calendar/session artifacts behind the 21/21 post-touch positives |
| **C** | Does post-fill behavior differ from matched pre-fill controls? | HIGH | FILL 21/21 positive but no pre-fill baseline anchor exists; continuation/neutralization/reversion labels are indistinguishable without it |
| **D** | Does the direction asymmetry (bullish 42/42; bearish mostly zero/negative) survive direction-, time- and drift-aware nulls? | HIGHEST | Upward-drifted window (raw 41/42 positive), drift residue under direction imbalance, bullish-skewed population (51.0%→60.5%): attribution unresolved |
| **E** | Does gap-through fill differ from ordinary touch→fill after proper matching? | MEDIUM | 15s null vs 30s contrast (n=213, composition-confounded); ineligible at 5m–4h; no matched test exists |
| **F** | Do gap_atr or fvg_quality modify lifecycle duration or stage response? | MEDIUM | Split orderings sign-unstable and largest at smallest N; gap_atr p05 floor-pinned at 0.2205–0.2294 vs min 0.2 (threshold-selected population); censor-aware estimation required |
| **G** | Does same-bar fill represent a distinct lifecycle phenotype (vs multi-bar touch→fill)? | MEDIUM | 34.2–40.4% same-bar mass, p25 = 0 everywhere; geometric point-mass alternative unresolved; no committed outcome comparison (P-04 QG3) |

Each candidate requires its own frozen E2 measurement contract before any test.

---

## 10. Surviving E1 knowledge (admissible as descriptive phenotype)

1. 381,498 lawful FVG formations across 7 native strata in the frozen DEVELOPMENT
   window; deterministic two-run reproduction; integrity gates 9/9 (E1A) and 5/5 (E1B);
   cross-scale identity prohibited and respected.
2. Median formation→touch = 2 native bars; median touch→fill = 1 native bar (p25 = 0);
   median formation→fill = 7/7/7/7/8/8/5 bars — at every stratum.
3. 34.2–40.4% of touched-and-filled zones fill on the first-touch bar itself
   (denominator touched_fill_n).
4. Rapid lifecycle body with a materially long-lived right tail; the observed tail is a
   censored lower bound, increasingly so at coarse strata (fill censoring 0.45%→15.57%;
   touch→fill censoring 0.21%→10.19%; unconditional quantiles identifiable only up to
   the committed per-stratum ceilings).
5. Direction-adjusted prospective medians: formation 18 neg / 1 zero / 2 pos of 21;
   FIRST_TOUCH 21/21 positive; FILL 21/21 positive; FILL > FIRST_TOUCH in 19/21 cells —
   a cross-cohort descriptive pattern of weak central tendency, not a tested effect.
6. The pooled post-anchor positive tendency is bullish-carried (bullish 42/42 positive;
   bearish 15/42, mostly zero/negative); bearish medians are not sign-consistent.
7. Gap-through fills (1,010; 682/213/81/20/10/4/0) are a legitimate producer-semantic
   case, never imputed as touch, excluded from FIRST_TOUCH cohorts, and a negligible
   mixture (≤ ~0.008 bps median shift) in FILL aggregates.
8. Orphan events (178 touch / 358 fill) are left-truncated pre-development detector
   state, correctly excluded from all in-window cohorts.
9. Observed fill fractions and censoring fractions are accounting complements and
   window-bounded descriptive quantities, never lifetime probabilities or hazards.
10. No robust touched-fill vs gap-through post-fill difference at the two
    paired-eligible strata (15s null; 30s hypothesis-grade).

## 11. Failed interpretations

- **P-04 bearish FILL sign tally "10 pos / 2 zero / 9 neg" — FAILED** mechanical
  verification (`E1B_INTERPRETER_FACT_CHECK.json` → false); canonical tally is
  **10 pos / 1 zero / 10 neg** (disputed cell 30s FILL h1 bearish = −2.1e-12 bps);
  corrected in the P-04 addendum; substantive conclusion unchanged.
- Historical (kept as record): P-01 "19/21 negative" → corrected to 18/1/2; the
  "filling bar necessarily overlaps the zone" premise → false under the linked producer
  implementation; P-02 A4 "4h coverage excess" → methodologically unsupported as stated;
  "1h has strongest clock dependence" → rejected (marginal-quantile comparison).
- Any reading of the stage sign reversal as stage effect / approach-phase spending /
  recentering / continuation / reversion / neutralization — not supported by E1; E2-gated.
- Any drift attribution of the bullish-carried pattern (FVG-specific vs window drift) —
  not separable in E1; E2-gated.

## 12. Unresolved questions

Stage conditioning vs selection (the sign reversal); drift residue vs FVG-specific
asymmetry; same-bar phenotype vs geometric point mass; gap_atr/fvg_quality modification
(including the floor-pinned gap_atr population); gap-through behavior beyond the two
eligible strata; the tail beyond the committed ceilings at 5m–4h (unidentifiable, not
observed short); the bullish-share gradient (51.0%→60.5%) attribution; 4h small-N cells.

## 13. Required E2 questions (none executed)

- **E2-Q1 (A/B)**: within-zone matched formation-vs-first-touch contrast at identical
  clock offsets with a time-matched no-FVG control, per stratum.
- **E2-Q2 (D)**: direction-matched, time-matched drift-aware null for post-touch and
  post-fill medians, per direction, per stratum.
- **E2-Q3 (C)**: post-fill vs matched pre-fill control contrast (pre-fill baseline
  anchor design).
- **E2-Q4 (E)**: touched-fill vs gap-through contrast with composition matching and a
  pre-registered minimum-N rule (in-window eligible: 15s, 30s; 1m marginal).
- **E2-Q5 (F/G)**: censor-aware gap_atr/fvg_quality modification of touch→fill duration;
  same-bar vs multi-bar post-fill contrast, with gap_atr floor-selection sensitivity.

All require a frozen E2 contract (cohorts, denominators, nulls, censoring treatment,
decision rules) before execution.

## 14. Required E3 questions (gated)

Only after E2 adjudication, and only for patterns that survive E2 nulls: out-of-window
replication of stage-contrast and direction-asymmetry results at matched N (e.g., 1h
FILL fvg_quality split separations, 4h cells), under the frozen confirmation discipline.
No E1 descriptive pattern may enter E3 as a prior "finding". Confirmation remains
LOCKED; no E3 work is authorized by this challenge.

## 15. Must-not-claim list (highlights)

- No edge / alpha / profitability / strategy / signal / predictive-advantage claims from
  anything in E1; nothing was tested; confirmation LOCKED.
- The sign reversal is **not** a stage effect, approach-phase spending, recentering,
  continuation, or reversion — descriptive cross-cohort pattern only, until E2.
- No significance, confidence, or p-value language; eligibility labels are size-support
  labels, not significance.
- Fill fractions / touch incidence / censoring fractions are window-censored accounting
  complements, never lifetime probabilities, hazards, or asymptotic rates.
- No unconditional p99/max at 5m–4h for any duration variable; no unconditional
  formation→fill p95 at 1h/4h; no unconditional formation→touch p95 at 4h (ceiling
  p94.0); nothing unconditional at 4h beyond p84.4 / p89.8 / p94.0 (fill / touch→fill /
  touch).
- Duration maxima (~118.5 days) are window-boundary administrative artifacts.
- 4h h5 (+12.5 bps formation; +15.7 bps FILL), 1h h3/h5, 4h bearish-positive (n=63),
  and the 30s gap-through contrast (n=213) are small-sample medians, not effects.
- Raw medians are drift-contaminated; direction-adjusted medians are drift-residual
  under direction imbalance; neither is directional structure without nulls.
- Bearish −0.0 / −2.1e-12 cells are exact-zero medians (sign-bit/epsilon artifacts).
- No cross-scale identity, pooling, or interchangeability of strata; horizons are
  native-bar-relative (no exposure-matched cross-stratum magnitude comparison).
- gap_atr/fvg_quality development median splits are not thresholds or confirmed effects.
- Orphans are not missing formations; A1 gap-through cases are not defects.
- Nothing transfers to E2, E3, confirmation, or live use.

## 16. Findings and freeze conditions

- **F-1 (administrative-provenance, HASH_REGISTRY_MISMATCH)**: in
  `docs/research/ap-002-e1a-scope-repair/ARTIFACT_SHA256.json`, the entries for
  `AP-002_E1A_SYNTHESIS.json` (registry `178f6db8…61a6`, 8,585 bytes) and
  `AP-002_E1A_SYNTHESIS.md` (registry `54828113…0e0e`, 4,841 bytes) do not match the
  files on disk (actual `daf1b94a…0804` and `04c412f8…c555`). The other 16/18 entries
  verify, as do all 11 entries of the E1B registry. File mtimes indicate the synthesis
  files were written one minute after the registry was generated. **Impact: none on
  scanner evidence or scientific content** (E1_RESULTS_corrected.json verifies; the
  synthesis content is independently corroborated by
  `AP-002_E1A_SCOPE_REPAIR_AUDIT.json` and by the E1B package's exact reconciliation to
  381,498 formations / 178+358 orphans). Not a scanner defect; no code modified.
  **Required action**: regenerate the registry before knowledge-base admission.
- **F-2 (info, CENSOR_CEILING_ANNOTATION)**: the unconditional formation→touch p95 at 4h
  is not identifiable (ceiling = touch incidence p94.01); attach the annotation in
  section 5 to the knowledge-base entry.
- **Freeze conditions**: F-1 registry regeneration; attach the must-not-claim list and
  corrected censoring-ceiling language; attach the sign-reversal classification
  (`SURVIVES_AS_DESCRIPTIVE_COHORT_PATTERN`) and the E2 gate to any stage-comparison
  wording.

---

**Confirmation status: LOCKED. E2/E3: NOT STARTED.** The E2 STAGE-MATCHED LIFECYCLE
PANEL is a recommendation only; nothing was executed. No scanner code was modified; no
existing file was modified; the challenger wrote only the two artifacts in this
directory.
