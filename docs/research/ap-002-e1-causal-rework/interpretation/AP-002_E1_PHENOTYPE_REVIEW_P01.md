# AP-002 E1 PHENOTYPE REVIEW — Interpreter P-01

- **Interpreter:** P-01 (independent phenotype researcher; not scanner author, not strategy researcher, not confirming)
- **Generated (UTC):** 2026-09-06T02:49:25Z
- **Experiment:** AP-002-FVG-E1-NATIVE-STRATA — Exposure E1 PHENOTYPE, instrument XAUUSD, DEVELOPMENT scope only (XAUUSD_DATA_SCOPE_V1), confirmation LOCKED
- **Overall assessment:** **PASS_TO_SYNTHESIS**
- **Evidence base (complete, nothing else consumed):** E1_EXPERIMENT.json, E1_RESULTS.json, PREFLIGHT.json, RUN_MANIFEST.json, SCANNER_INTEGRITY_GATE.json, REPRODUCIBILITY.json (all in `docs/research/ap-002-e1-causal-rework/`), and frozen `knowledge/wave1_authority_closure/AP-002_AUTHORITY_V1.json`
- **Compliance:** No peer interpretation, no scanner source, no sidecar/zone rows, no re-run, no cross-scale identity, no E2/E3/confirmation data, no scale pooling. Every quantitative statement cites its source table or is marked as arithmetic derivation from one table.

---

## 1. Executive phenotype summary

The frozen E1 DEVELOPMENT evidence characterizes **381,527 lawful FVG formations across 7 native strata** of XAUUSD (E1_RESULTS.json.tables.strata.rows), scanned from 54,415,940 tick rows with 0 regressions (PREFLIGHT.json). The evidence base is deterministic (two runs, identical logical_result_hash `735bd7ab…c4c52`; REPRODUCIBILITY.json.comparison) and integrity-gated (9/9 checks PASS; SCANNER_INTEGRITY_GATE.json), with **zero unavailable counters at every stratum** (E1_RESULTS.json.tables.strata.rows[tf=*].unavailable_*).

Core phenotype findings:

1. **The lifecycle is governed by native bar counts, not wall-clock.** Median formation→first-touch is exactly **2 native bars at every stratum** (p25 = 1 bar everywhere); median formation→fill is **5–8 native bars**. In wall-clock these span 30 s → 8 h (touch) and ~105 s → 20 h (fill) purely because the bar period grows (E1_RESULTS.json.tables.strata.rows[tf=*].formation_to_touch_market_ms, formation_to_fill_market_ms; converted to bars).
2. **Observed fill fractions fall with scale — 99.55% (15s) → 84.43% (4h) — in near lockstep with right-censoring rising 0.45% → 15.57%.** At coarse scales the fill fraction is a window-truncated lower bound, **not** a lifetime probability (X5, X12 below).
3. **A four-way path decomposition (touched-then-filled / touched-never-filled / filled-without-prior-touch / never-touched-never-filled) closes exactly at every stratum**, and the censored population splits roughly half/half between "touched but never filled" and "never touched" (X7, X8).
4. **Direction populations drift from near-balanced (51.0% bullish at 15s) to bullish-majority (60.5% at 4h)** monotonically with scale (X6) — descriptive; awaits a formal test.
5. **Prospective native closes (1/3/5 bars): raw-return medians are positive at all 21 stratum-horizon cells** (development-window drift), while **direction-adjusted medians are negative in 19 of 21 cells** and grow in magnitude with horizon — a descriptive counter-directional median tendency that requires a formal null test (P1–P3).
6. **Anomalies are small, quantified, registered, and unresolved:** 356 orphan fills / 177 orphan touches, fill-without-prior-recorded-touch up to 0.83% of stratum formations (0 at 4h), known-time vs market-time divergences up to ~1 bar in both directions, and a 4h incidence uptick.

Nothing in the frozen evidence blocks phenotype synthesis.

---

## 2. Per-stratum observations

Master table (all values from E1_RESULTS.json.tables.strata.rows[tf=*]; bar-unit conversions and path decompositions are derived arithmetic from the same rows):

| TF | Bars | Formations | Incidence | Bullish share | Touch frac. | Fill frac. | Censored | Touch bars p25/50/75/95 | Fill bars p25/50/75/95 | gap_atr p50 | quality p50 | dir-adj median h1/h3/h5 (bps) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 15s | 1,054,977 | 227,925 | 21.60% | 51.03% | 99.46% | 99.55% | 0.45% | 1/2/8/190 | 2/7/30/717 | 0.564 | 0.797 | −0.000 / −0.050 / −0.076 |
| 30s | 527,896 | 99,874 | 18.92% | 51.71% | 99.44% | 99.36% | 0.64% | 1/2/8/184 | 2/7/30/695 | 0.520 | 0.783 | −0.027 / −0.107 / −0.142 |
| 1m | 263,976 | 43,706 | 16.56% | 52.44% | 99.38% | 99.10% | 0.90% | 1/2/8/184 | 2/7/31/669 | 0.492 | 0.774 | −0.075 / −0.206 / −0.214 |
| 5m | 52,841 | 7,140 | 13.51% | 54.29% | 98.59% | 97.77% | 2.23% | 1/2/8/158 | 2/7/33/701 | 0.457 | 0.750 | −0.225 / −0.553 / −0.516 |
| 15m | 17,615 | 2,192 | 12.44% | 57.98% | 97.45% | 95.99% | 4.01% | 1/2/8/186 | 2/8/34/562 | 0.459 | 0.750 | −0.455 / −0.672 / −1.114 |
| 1h | 4,406 | 523 | 11.87% | 58.32% | 95.03% | 92.16% | 7.84% | 1/2/9/193 | 3/8/33/403 | 0.454 | 0.745 | −0.108 / +0.852 / −1.589 |
| 4h | 1,169 | 167 | 14.29% | 60.48% | 94.01% | 84.43% | 15.57% | 1/2/12/89 | 2/5/31/170 | 0.527 | 0.756 | −1.134 / −2.094 / +12.508 |

### 2.1 15s
- **DIRECT_OBSERVATION:** Observed fill fraction 99.55% (226,908/227,925) **exceeds** first-touch fraction 99.46% (226,702/227,925) because 682 fills had no prior recorded touch (fields: fill_rate, first_touch_rate, formed_fill_without_prior_touch_n).
- **DIRECT_OBSERVATION:** The ~1.05M-bar window dwarfs the fill-time tail (p95 ≈ 717 bars), so 0.45% censoring barely touches the observable tail; the fine-scale fill fraction is close to window-asymptotic within this scope.
- **DIRECT_OBSERVATION:** h3/h5 responses have n = 227,924 vs denominator 227,925 (horizons[1].n, horizons[2].n) — one formation lacked 3 later lawful closes (end-of-stream).

### 2.2 30s
- **DIRECT_OBSERVATION:** Lifecycle quantiles in native-bar units are essentially identical to 15s (touch 1/2/8; fill 2/7/30). All horizons full n = 99,874.

### 2.3 1m
- **DIRECT_OBSERVATION:** Fill known-time p95 is 120,009 ms (~2 bars) shorter than market-time p95 (40,019,991 vs 40,140,000; formation_to_fill_known_ms.p95 vs formation_to_fill_market_ms.p95) — intra-bar knowability at the tail. All horizons full n = 43,706.

### 2.4 5m
- **DIRECT_OBSERVATION:** Fill known-time p75 is 299,772 ms (~1 bar) shorter than market-time p75 (9,600,228 vs 9,900,000). h3/h5 n = 7,139 vs 7,140.

### 2.5 15m
- **DIRECT_OBSERVATION:** Fill p95 shortens to ~562 bars (from ~669–717 finer) while censoring rises to 4.01% (88/2,192) — the observable tail is beginning to truncate.
- **DIRECT_OBSERVATION:** h3 n = 2,191, h5 n = 2,189 vs 2,192.

### 2.6 1h
- **DIRECT_OBSERVATION:** Strongest clock dependence: fill known-time p75 is 14,399,866 ms (~4 bars) shorter than market-time p75; fill p25 known ~1 bar shorter; touch p75 known ~1 bar shorter (formation_to_fill_known_ms / formation_to_touch_known_ms vs market_ms).
- **DIRECT_OBSERVATION:** One of only two positive direction-adjusted medians: h3 +0.852 bps (n = 523).
- **INTERPRETATION:** n = 523 formations, 41 censored — wide sampling uncertainty; no significance claim made.

### 2.7 4h
- **DIRECT_OBSERVATION:** Smallest cohort (167 formations on 1,169 bars); 26 zones (15.57%) censored.
- **DIRECT_OBSERVATION:** Touch p75 = 12 bars, touch p95 ≈ 89 bars; fill p95 ≈ 170 bars ≈ 14.5% of the entire history — the observable upper tail is truncated by the short window.
- **DIRECT_OBSERVATION:** Touch known-time p75 is 14,399,950 ms (**~1 bar longer** than market-time p75: 187,199,950 vs 172,800,000) — known time can exceed market time here.
- **DIRECT_OBSERVATION:** Zero fill-without-prior-touch; the only positive direction-adjusted h5 median (+12.508 bps, n = 167). Small-n; descriptive only.

---

## 3. Cross-stratum observations

- **X1 (DIRECT):** Incidence declines monotonically 21.60% → 11.87% (15s→1h), then ticks up to 14.29% at 4h (formation_incidence.rate).
- **X2 (DIRECT):** Median touch = exactly 2 native bars at every stratum; p25 = 1 bar everywhere (formation_to_touch_market_ms.p25/p50 ÷ bar period; derived).
- **X3 (DIRECT):** Median fill = 7 bars (15s–5m), 8 bars (15m/1h), 5 bars (4h); p75 = 30–34 bars everywhere (formation_to_fill_market_ms; derived). **Lifecycle timing in the distribution body is approximately invariant in native-bar time across a 960× span of bar periods.**
- **X4 (DIRECT):** Touch p95 in bars = 158–193 (15s–1h) but ~89 at 4h; fill p95 in bars declines ~717 → ~562 → ~403 → ~170 (15s→15m→1h→4h) — consistent with censoring truncation, **not** demonstrably genuine tail shortening.
- **X5 (DIRECT):** Fill fraction falls monotonically 99.55% → 84.43% while censoring rises monotonically 0.45% → 15.57%; the gradients are near-complementary.
- **X6 (DIRECT):** Bullish share rises monotonically 51.03% → 60.48% with scale (bullish_n/(bullish_n+bearish_n); derived).
- **X7 (DIRECT):** Four-way path decomposition (touched-then-filled = formed_touch_fill_n; touched-never-filled = first_touch_n − formed_touch_fill_n; filled-without-prior-touch = formed_fill_without_prior_touch_n; never-touched-never-filled = lawful_formation_n − first_touch_n − formed_fill_without_prior_touch_n) **sums exactly to lawful_formation_n and reproduces right_censored_n at all 7 strata** (derived).

| TF | Touched→filled | Touched, never filled | Filled w/o prior touch | Never touched, never filled | Censored (check) |
|---|---|---|---|---|---|
| 15s | 226,226 | 476 (0.209%) | 682 (0.299%) | 541 (0.237%) | 1,017 ✓ |
| 30s | 99,019 | 294 (0.294%) | 213 (0.213%) | 348 (0.348%) | 642 ✓ |
| 1m | 43,232 | 201 (0.460%) | 81 (0.185%) | 192 (0.439%) | 393 ✓ |
| 5m | 6,961 | 78 (1.092%) | 20 (0.280%) | 81 (1.134%) | 159 ✓ |
| 15m | 2,094 | 42 (1.916%) | 10 (0.456%) | 46 (2.099%) | 88 ✓ |
| 1h | 478 | 19 (3.633%) | 4 (0.765%) | 22 (4.207%) | 41 ✓ |
| 4h | 141 | 16 (9.581%) | 0 (0.000%) | 10 (5.988%) | 26 ✓ |

- **X8 (DIRECT):** Both censored sub-populations grow with scale (touched-never-filled 0.209% → 9.581%; never-touched-never-filled 0.237% → 5.988% of formations). At every stratum except 4h (61.5%), roughly 46–51% of censored zones were touched but never filled.
- **X9 (DIRECT):** gap_atr p05 is pinned at 0.2205–0.2294 against the min_gap_atr = 0.2 detector floor (AP-002_AUTHORITY_V1.json.parameters) at **every** stratum; median declines 0.564 → 0.454 (15s→1h) then 0.527 at 4h; p95 is U-shaped (1.62, 1.44, 1.34, 1.35, 1.48, 1.62, 1.60).
- **X10 (DIRECT):** fvg_quality is nearly scale-stable: median 0.745–0.797, p05 0.479–0.501, p95 0.943–0.989.
- **X11 (DIRECT):** Orphan lifecycle events total 177 touches / 356 fills; per-formation burden rises with scale (orphan fills: 0.069% of formations at 15s → 3.593% at 4h).
- **X12 (INTERPRETATION):** X2/X3/X4/X5 together indicate the apparent scale dependence of the fill fraction is largely an **observation-window effect in bar-time**; the evidence does not establish that coarse zones are intrinsically less likely to fill.
- **X13 (HYPOTHESIS_FOR_LATER_TESTING):** The bullish-share gradient may reflect development-window drift interacting with bar aggregation, or session-structure effects at coarse bars. Testable per stratum with a formal direction-balance null; no claim made here.

---

## 4. Prospective-response observations

Definition: native closes 1, 3, 5 lawful bars after formation; raw and direction-adjusted bps vs anchor_mid (E1_EXPERIMENT.json.prospective_response). Denominator = lawful_formation_n per stratum; 21 cells total.

- **P1 (DIRECT):** Raw-return medians are positive at **all 21 cells**, growing with horizon and bar period: 15s h1 +0.012 → h5 +0.059 bps; 4h h1 +2.531 → h5 +25.980 bps (horizons[*].raw_return_bps.p50).
- **P2 (DIRECT):** Direction-adjusted medians are negative in **19 of 21 cells**; exceptions 1h h3 (+0.852, n=523) and 4h h5 (+12.508, n=167). h1 medians: −0.000 (15s), −0.027 (30s), −0.075 (1m), −0.225 (5m), −0.455 (15m), −0.108 (1h), −1.134 (4h) (horizons[*].direction_adjusted_return_bps.p50).
- **P3 (INTERPRETATION):** The split reads as a **drift confound plus a small counter-directional median tendency after formation**: conditional on formation, the median native forward close moves opposite to the forming impulse direction, in both direction cohorts, at nearly all scales and horizons. Median statement only; tails are wide; no null test applied.
- **P4 (DIRECT):** Direction-adjusted tails are mostly near-symmetric, with left-heavier tails at some coarse cells: 4h h1 p05 −124.45 vs p95 +84.10; 1h h1 p05 −55.33 vs p95 +49.53; while 1h h5 (−104.65 / +125.69) and 4h h5 (−278.56 / +282.74) are right-heavy or symmetric.
- **P5 (DIRECT):** Dispersion scales with bar period as expected for returns: raw h1 (p95−p05) ≈ 6.4 bps (15s), 31 bps (5m), 113.6 bps (1h), 209.7 bps (4h). No anomalous scaling regime.
- **P6 (DIRECT):** Sample sizes full at 30s/1m/1h/4h all horizons; shortfalls only 15s (h3/h5, −1), 5m (h3/h5, −1), 15m (h3 −1, h5 −3) vs full denominators.
- **P7 (HYPOTHESIS_FOR_LATER_TESTING):** Register for formal testing: (a) per-stratum zero-median tests of direction-adjusted returns at each horizon; (b) whether the counter-directional median per bar intensifies with scale (h1: ~0 bps at 15s vs −1.134 bps at 4h); (c) whether the 4h h5 positive median survives at n=167; (d) left-vs-right tail weight at 1h/4h h1. None established here.

---

## 5. Anomalies

- **A1 — Fill without prior recorded touch (DIRECT):** 682 (15s), 213 (30s), 81 (1m), 20 (5m), 10 (15m), 4 (1h), 0 (4h) — up to 0.83% of stratum formations; included in fill_n/fill_rate (formed_fill_without_prior_touch_n). *Interpretation:* possibly bars that gap entirely through the zone (no overlap → no touch) while the extreme reaches the fill edge (e.g., across session breaks), or same-bar touch/fill ordering semantics (authority semantics: AP-002_AUTHORITY_V1.json.authoritative_semantics). Not verifiable from the frozen tables. **Requires explanation before any fill-anchored cohort is promoted.**
- **A2 — Orphan lifecycle events (DIRECT):** 177 orphan touches, 356 orphan fills with no formation payload counterpart, while formation payloads cover all 381,527 identities (orphan_touch_n, orphan_fill_n). *Interpretation:* plausible detector-state restoration or availability-gating edges; not resolvable here. Orphans are already excluded from formed_* counts, so headline fractions are uncontaminated. **Requires scanner-author explanation before E1B cohort freeze; largest proportional burden at 4h (3.593%).**
- **A3 — Known-time vs market-time divergence (DIRECT):** Known-time durations are **shorter** than market-time at upper quantiles at mid strata — up to ~1 bar (5m/15m fill p75), ~2 bars (1m fill p95), ~1 bar (1h fill p25, 1h touch p75), ~4 bars (1h fill p75: 104,400,134 vs 118,800,000 ms) — and **longer** by ~1 bar at 4h touch p75 (187,199,950 vs 172,800,000 ms). Fine strata agree to sub-second ~0.3 s. *Interpretation:* known ≤ market is consistent with intra-bar knowability of the touching/filling extreme; known > market at 4h is consistent with delayed bar-completion receipt across quiet/closed sessions. Availability-clock mechanics, not causality violations (integrity gate: prospective_response_causality PASS). **E1B anchors must declare their clock.**
- **A4 — 4h incidence uptick (DIRECT):** 14.29% after a monotone decline to 11.87% at 1h. With 1,169 bars / 167 formations the estimate is imprecise; noise or a session-gap aggregation effect. Open descriptive question.
- **A5 — Fill fraction exceeds touch fraction at 15s (DIRECT):** 99.554% vs 99.463%, an arithmetic consequence of A1 (682 no-touch fills vs 476 touched-never-filled); flagged so it is not misread as an inconsistency.
- **A6 — End-of-stream response shortfalls (DIRECT):** n falls 1–3 short of denominator at 15s/5m/15m long horizons, consistent with the authority's no-synthetic-tail-bar rule (AP-002_AUTHORITY_V1.json.lawful_availability.end_of_stream). Expected; quantified.

---

## 6. Censoring cautions

1. Censored zones are active, unfilled zones at the final lawful native bar (SCANNER_INTEGRITY_GATE.json, right_censoring check). Censoring: 0.45% (15s) → 15.57% (4h) of formations (right_censored_rate.rate).
2. **Observed fill fractions are window-truncated lower bounds on eventual fill behavior — never hazards or lifetime probabilities.** At 4h, fill p95 (~170 bars) is ~14.5% of the entire 1,169-bar history, so the coarse-scale fraction is materially truncated.
3. The censored population is **heterogeneous**: touched-never-filled vs never-touched-never-filled (e.g., 16 vs 10 of 26 at 4h). Any E1B design must keep them distinct; pooling assumes an unestablished common phenotype.
4. Censoring truncates time-to-touch and time-to-fill upper quantiles at coarse scales (X4); cross-stratum tail comparisons in bar-time are observation-limited, not necessarily behavioral.
5. Prospective responses are complete or near-complete (A6), so response statistics are far less censoring-exposed than duration statistics; but 1/3/5-bar fixed horizons say nothing about longer horizons.
6. Development-only scope: everything here is in-sample description of XAUUSD DEVELOPMENT; no out-of-sample or confirmation inference is licensed.

---

## 7. Possible E1B stage-anchor requirements

### FIRST_TOUCH anchor — **justified: YES**
Near-universal where observed (touch fractions 94.0–99.5%) and median latency 2 native bars at every stratum → large, fast-realizing cohorts at all scales, with touched-never-filled (476 at 15s … 16 at 4h) as a built-in contrast class.

Requirements: cohort denominator = first_touch_n per stratum; pre-register anchor clock (known vs market; A3); pre-register treatment of filled-without-prior-touch zones (A1 — they cannot enter a touch-anchored cohort); keep touched-never-filled vs touched-then-filled distinct; per-stratum only, (timeframe, zone_id) identity only.

### FILL anchor — **justified: YES**
Near-complete cohorts at fine/mid strata (97.8–99.6%) and fill is the cleanest zone-consumption event; **but** at 1h/4h the cohort is survivor-conditioned (92.2%/84.4% of formations, 7.8%/15.6% censoring), which must be registered as a selection caveat.

Requirements: cohort denominator = fill_n per stratum with the censored complement reported; register survivor-conditioning at 1h/4h before freezing; pre-register anchor clock (A3) and same-bar touch+fill / fill-without-prior-touch handling (A1); response grid strictly after the fill event's lawful availability; per-stratum only.

### Shared preconditions
- Orphan events (A2) need a scanner-author explanation and a fixed exclusion/reconciliation policy before any E1B cohort freeze.
- Explicit right-censoring accounting with the two censored sub-populations tracked separately (X8).
- gap_atr p05 floor-pinning at min_gap_atr = 0.2 (X9) means near-threshold formations may be a selected population; any gap_atr conditioning must account for threshold selection.

---

## 8. Questions for the later Question Generator

1. **Q1:** What mechanism produces the 177 orphan touch / 356 orphan fill events, and do they concentrate at detector-state restoration boundaries or availability edges? (Scanner-author question; blocks E1B cohort-freeze policy, not synthesis.)
2. **Q2:** Are fill-without-prior-touch rows gap-through bars across session breaks/weekends, and does their per-stratum rate track session-gap density?
3. **Q3:** Is the monotone bullish-share gradient with scale (51.0% → 60.5%) explained by development-window drift, session-aggregation asymmetry, or detector mechanics? Requires a formal per-stratum direction-balance test.
4. **Q4:** Is the counter-directional median tendency (negative direction-adjusted medians in 19/21 cells) a reversion-to-impulse artifact of the three-bar formation geometry (middle body ≥ 0.20 ATR) rather than a zone property?
5. **Q5:** Does the 4h incidence uptick (14.29% after a monotone decline to 11.87%) survive an interval estimate at n = 1,169 bars / 167 formations?
6. **Q6:** Which clock (known-time vs market-time) should define E1B anchors and horizon grids, given divergences up to ~1 bar in both directions (A3)?
7. **Q7:** Do touched-never-filled and never-touched-never-filled censored zones differ in gap_atr, fvg_quality, or formation context (censoring heterogeneity, X8)?
8. **Q8:** Does gap_atr p05 pinning at ~0.22 against the min_gap_atr = 0.2 floor indicate a threshold-selected formation population, and should near-floor sensitivity be studied before any gap_atr conditioning is frozen?

---

## 9. Claims that must NOT be made

1. That any observed fill fraction is a hazard, lifetime probability, or the probability an FVG eventually fills — they are window-truncated descriptive fractions with material censoring at coarse scales.
2. That raw or direction-adjusted return patterns constitute an edge, alpha, profitability, predictive advantage, or any strategy-grade result — no null test has been run and confirmation is locked.
3. That strata are statistically interchangeable or may be pooled to raise N — every statement here is per-stratum.
4. That cross-scale zone identity exists — identity is (timeframe, zone_id) only; cross-scale identity is prohibited.
5. That the 4h bullish majority (60.48%) or the bullish-share gradient is a statistically established direction bias — descriptive gradient awaiting a formal test.
6. That unfilled 1h/4h zones would or would not eventually fill, or that coarse-scale fill fractions are asymptotic.
7. That fill-without-prior-touch rows or orphan events are scanner defects — unexplained, not demonstrated to be errors.
8. Any causal claim that formation causes subsequent native returns beyond the descriptive in-development-sample association recorded here.
9. Any transfer of these development-only quantities to E2/E3 contexts, confirmation, or live behavior.
10. That lifecycle-duration distributions are fully scale-invariant including tails — only body quantiles are approximately bar-time invariant, and coarse-scale tails are censoring-truncated.

---

## 10. Overall assessment

**PASS_TO_SYNTHESIS.**

Rationale: the frozen evidence is complete (zero unavailable counters; all 381,527 zone identities reconciled to formation payloads), deterministic across two independent runs (identical logical hashes), integrity-gated on all nine checks including prospective-response causality and right-censoring handling, and internally consistent to the zone at every stratum (the four-way lifecycle decomposition closes exactly). Every phenotype question posed for E1 — formation incidence, direction balance, first-touch/fill frequency, right censoring, path structure, duration distributions in both clocks, gap_atr/fvg_quality distributions, and lawful prospective behavior at 1/3/5 bars — is answerable from these tables. Registered anomalies (orphans, fill-without-prior-touch, clock divergences, 4h incidence uptick) are small, quantified, and do not contaminate cohort-level fractions; they are carried forward as questions, not blockers. No rework of the scanner or the experiment is indicated by this evidence.
