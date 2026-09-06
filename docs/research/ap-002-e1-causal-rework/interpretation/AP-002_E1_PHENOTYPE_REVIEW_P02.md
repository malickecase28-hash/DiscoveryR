# AP-002 E1 PHENOTYPE REVIEW — P-02

- **Interpreter:** P-02 (independent phenotype researcher; not scanner author, not strategy researcher, not confirmation)
- **Generated (UTC):** 2026-09-06T03:02:29Z
- **Experiment:** AP-002-FVG-E1-NATIVE-STRATA | Program AP-002 FVG | Exposure E1_PHENOTYPE | Instrument XAUUSD
- **Scope:** XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only | Confirmation LOCKED | Native strata: 15s, 30s, 1m, 5m, 15m, 1h, 4h
- **Lawful identity:** (timeframe, zone_id); cross-scale identity prohibited and never assumed
- **Overall assessment:** **PASS_TO_SYNTHESIS** (with flagged preconditions — see Anomalies and E1B requirements)

**Evidence base (sole sources; nothing else was opened):** E1_EXPERIMENT.json, E1_RESULTS.json, PREFLIGHT.json, RUN_MANIFEST.json, SCANNER_INTEGRITY_GATE.json, REPRODUCIBILITY.json, and frozen knowledge/wave1_authority_closure/AP-002_AUTHORITY_V1.json. The peer P-01 file present in this directory was not opened. Scanner code untouched.

**Evidence integrity (direct observation):** Integrity gate PASS on all 9 checks (SCANNER_INTEGRITY_GATE.json), including 381,527 unique (timeframe, zone_id) identities, formation/outcome separation, prospective-response causality, and explicit right-censoring. Two independent runs produced identical logical_result_hash, preflight hash, and zone physical hash (REPRODUCIBILITY.json.comparison). Unavailable counters all zero; receipt regressions 0; E2/E3 absent. Interpreter arithmetic cross-checks: all seven per-stratum lifecycle path decompositions close exactly, and stratum formations sum to exactly 381,527 = zones.n (identity check only; no pooled analysis).

---

## 1. Executive phenotype summary

The DEVELOPMENT evidence characterizes the native FVG phenotype as a **rapidly-terminating, right-tailed lifecycle**. Median first touch is exactly **2 native bars** and median fill is **5–8 native bars in every stratum**; a minority long-lived population exists (p95 fill time ~562–717 bars at 15s–15m) whose extreme tail is not described by the p95-capped frozen tables.

Fill fractions fall monotonically with scale, **0.9955 (15s) → 0.8443 (4h)**, while right-censoring rises **0.0045 → 0.1557**. The near-1.0 fine-stratum fill fractions are largely a **long-observation-window effect** (bar-time window is ~1,471× the 15s p95 fill time vs ~6.9× at 4h) and **must not be read as unconditional fill probabilities**.

Formation incidence declines with scale (0.2160/bar at 15s → 0.1187 at 1h) with a small-N 4h rebound (0.1429 on 167 formations / 1,169 bars). Direction is near-balanced at fine scales (bullish share 0.5103 at 15s) but skews monotonically bullish with scale to **0.6048 at 4h**.

Prospective direction-adjusted medians are **negative in 18 of 21 stratum×horizon cells** while raw pooled medians are positive in all 21 cells — a sign pattern diagnostic of **upward drift in the development window** coexisting with a mild descriptive drift **adverse to formation direction**; magnitude grows with horizon and bar scale.

Flagged anomalies needing mechanistic explanation before promotion: **fill-without-prior-recorded-touch** (682/227,925 at 15s declining to 0/167 at 4h), **orphan lifecycle events rising with scale** (orphan fills 0.070% of fills at 15s vs 4.255% at 4h), **±1-bar known-vs-market quantile jumps** at coarse strata (small-N quantile granularity suspected), and a **~11 bar-day coverage excess at 4h**. No scanner rework is indicated from the interpretation seat.

---

## 2. Per-stratum observations

All numbers from E1_RESULTS.json.tables.strata.rows[timeframe=…]. Path decomposition cells are derived and close exactly per stratum (touch→fill + touched-never-filled = first_touch_n; touch→fill + fill-without-prior-touch = fill_n; censored = touched-never-filled + never-touched-never-filled).

| | 15s | 30s | 1m | 5m | 15m | 1h | 4h |
|---|---|---|---|---|---|---|---|
| bars (denominator) | 1,054,977 | 527,896 | 263,976 | 52,841 | 17,615 | 4,406 | 1,169 |
| formations | 227,925 | 99,874 | 43,706 | 7,140 | 2,192 | 523 | 167 |
| incidence/bar | 0.2160 | 0.1892 | 0.1656 | 0.1351 | 0.1244 | 0.1187 | 0.1429 |
| bars/formation (derived) | 4.63 | 5.29 | 6.04 | 7.40 | 8.04 | 8.42 | 7.00 |
| bullish share | 0.5103 | 0.5171 | 0.5244 | 0.5429 | 0.5798 | 0.5832 | 0.6048 |
| first_touch rate | 0.99463 | 0.99438 | 0.99375 | 0.98585 | 0.97445 | 0.95029 | 0.94012 |
| fill rate | 0.99554 | 0.99357 | 0.99101 | 0.97773 | 0.95985 | 0.92161 | 0.84431 |
| right-censored n (rate) | 1,017 (0.0045) | 642 (0.0064) | 393 (0.0090) | 159 (0.0223) | 88 (0.0401) | 41 (0.0784) | 26 (0.1557) |
| touch→fill | 226,226 | 99,019 | 43,232 | 6,961 | 2,094 | 478 | 141 |
| touched, never filled | 476 | 294 | 201 | 78 | 42 | 19 | 16 |
| fill w/o prior recorded touch | 682 | 213 | 81 | 20 | 10 | 4 | 0 |
| never touched, never filled | 541 | 348 | 192 | 81 | 46 | 22 | 10 |
| touch p50 / p95 (bars, derived) | 2 / 190 | 2 / 184 | 2 / 184 | 2 / 158 | 2 / 186 | 2 / 193 | 2 / 89 |
| fill p50 / p95 (bars, derived) | 7 / 717 | 7 / 695 | 7 / 669 | 7 / 701 | 8 / 562 | 8 / 403 | 5 / 170 |
| gap_atr p05 / p50 / p95 | 0.229 / 0.564 / 1.622 | 0.226 / 0.520 / 1.445 | 0.223 / 0.492 / 1.343 | 0.221 / 0.457 / 1.347 | 0.221 / 0.459 / 1.483 | 0.221 / 0.454 / 1.616 | 0.228 / 0.527 / 1.603 |
| fvg_quality p05 / p50 / p95 | 0.480 / 0.797 / 0.989 | 0.479 / 0.783 / 0.980 | 0.484 / 0.774 / 0.972 | 0.479 / 0.750 / 0.960 | 0.494 / 0.750 / 0.955 | 0.501 / 0.745 / 0.943 | 0.488 / 0.756 / 0.954 |
| orphan touch / fill (n) | 75 / 158 | 46 / 94 | 32 / 60 | 8 / 18 | 7 / 13 | 6 / 7 | 3 / 6 |
| orphan fill as % of fills (derived) | 0.070% | 0.095% | 0.139% | 0.258% | 0.618% | 1.452% | 4.255% |

**Stratum notes (direct observation unless labeled):**

- **15s** — Only stratum where fill_n (226,908) exceeds first_touch_n (226,702); arithmetic consequence of 682 fill-without-prior-touch vs 476 touched-never-filled. Fastest wall-clock lifecycle (p50 touch 30 s, p50 fill 105 s). h3/h5 response n = 227,924 vs denominator 227,925 (one formation lacks a lawful close 3 and 5 bars later).
- **30s** — Among censored zones, never-touched (348) exceeds touched-never-filled (294). Known-vs-market quantiles within ~0.5 s except touch p95 (+28,591 ms ≈ one-bar quantile jump).
- **1m** — p95 fill 669 bars (~11.1 h) shows the heavy fill-time right tail. Known-vs-market deltas at p95 are exactly +1 bar (touch) and +2 bars (fill): quantile granularity.
- **5m** — h3 vs h5 dir-adj medians are non-monotone (−0.553 vs −0.516 bps). h3/h5 n = 7,139 vs 7,140.
- **15m** — h5 response n = 2,189/2,192 (three formations lack a lawful close 5 bars later). h5 dir-adj adverse tail (p05 −63.41) heavier than favorable tail (p95 +60.75).
- **1h** — Only 523 formations in the entire development stream; small-N caution applies to every statistic here. h3 is one of only two dir-adj-positive cells (+0.852 bps) while h1/h5 are negative — sign is not monotone in horizon. h1 adverse tail (−55.33) heavier than favorable (+49.53).
- **4h** — Smallest population (167 formations / 1,169 bars); censoring material (26 censored, 15.57%). h5 dir-adj median +12.508 bps is the only strongly positive cell; on N=167 with p05/p95 spanning ±~280 bps this is a small-sample median, not an established sign reversal. Fill p95 compression (170 bars vs ~562–717 at finer strata) is consistent with censoring removing slow zones from the filled-only conditional. Derived bar-time coverage = 194.8 days vs 183.2–183.6 days at all six finer strata (unexplained; anomaly A4).

---

## 3. Cross-stratum observations (descriptive; strata never pooled, never assumed interchangeable)

1. **Formation incidence** declines with scale (0.2160 → 0.1187 per bar, 15s→1h) with a small-N 4h rebound (0.1429) that must not be treated as an established scale effect.
2. **Direction balance:** bullish share rises monotonically with bar size (0.5103 → 0.6048). Populations are near-balanced at 15s–1m and visibly bullish-skewed at 15m–4h. **INTERPRETATION:** confounded between development-window upward drift and genuine scale-dependent formation asymmetry; the frozen tables cannot separate these.
3. **Lifecycle timing bar-invariance:** touch p25 = 1 bar and p50 = 2 bars at *every* stratum; fill p50 = 7 bars (15s–5m), 8 bars (15m–1h), 5 bars (4h). Fill p95 ≈ 562–717 bars at 15s–15m but 403 (1h) / 170 (4h); the coarse compression is **partially a censoring selection effect** (fill-time quantiles condition on filled zones). **HYPOTHESIS_FOR_LATER_TESTING:** timing is approximately invariant in bar units; wall-clock duration necessarily scales with bar duration.
4. **Fill rate vs censoring:** fill rate declines monotonically as censoring rises; the two are complementary to 1 within each stratum by construction (censor = never filled by final lawful bar).
5. **Observation-window asymmetry:** bar-time window relative to own p95 fill time: ~1,471× (15s), ~759×, ~395×, ~75×, ~31×, ~11×, ~6.9× (4h). Near-1.0 fine-stratum fill rates are almost mechanical given window length.
6. **Censored substructure:** never-touched-never-filled exceeds touched-never-filled in every stratum except 4h (10 vs 16) — most censored zones were never touched at all.
7. **gap_atr:** median declines 0.564 → 0.454 (15s→1h), rebounds 0.527 (4h, N=167). p05 = 0.2205–0.2294 everywhere, hugging the frozen **min_gap_atr = 0.2** filter (AP-002_AUTHORITY_V1.json.parameters) — the population is **left-truncated by construction**.
8. **fvg_quality:** median declines monotonically 0.797 → 0.745 (15s→1h), small-N rebound 0.756 (4h). Mild scale dependence.
9. **Orphan gradient:** orphan event rates rise with bar size in both event types (see table); absolute counts are small, the gradient is the load-bearing observation.
10. **Coverage discrepancy:** derived bar-time coverage is 183.2–183.6 days for 15s–1h but 194.8 days at 4h (~11 bar-day excess; anomaly A4).

---

## 4. Prospective-response observations (formation-anchored; lawful closes +1/+3/+5)

**Denominators:** h1 n = denominator everywhere; shortfalls confined to h3/h5 (15s 227,924/227,925; 5m 7,139/7,140; 15m 2,191/2,192 and 2,189/2,192; 1h and 4h full). Cause: end-of-stream formations have no later lawful close.

| dir-adj median (bps) | h1 | h3 | h5 | raw p50 h1 | raw p50 h5 |
|---|---|---|---|---|---|
| 15s | −0.000 | −0.050 | −0.076 | +0.012 | +0.059 |
| 30s | −0.027 | −0.107 | −0.142 | +0.022 | +0.091 |
| 1m | −0.075 | −0.206 | −0.214 | +0.059 | +0.135 |
| 5m | −0.225 | −0.553 | −0.516 | +0.182 | +1.030 |
| 15m | −0.455 | −0.672 | −1.114 | +0.470 | +2.456 |
| 1h | −0.108 | +0.852 | −1.589 | +2.085 | +7.301 |
| 4h | −1.134 | −2.094 | +12.508 | +2.531 | +25.980 |

- **Sign split (the key descriptive regularity):** raw pooled medians positive in **21/21** cells; dir-adj medians negative in **18/21** (exceptions: 1h h3 +0.852; 4h h5 +12.508; 15s h1 −0.000). **INTERPRETATION:** this is the signature of a development window with net upward drift (which raw pooling inherits) plus a mild average post-formation movement against formation direction once cohort direction is removed. Descriptive of this window only.
- **Magnitude:** dir-adj median magnitude grows with horizon within strata and broadly with bar scale at h5 (15s −0.076 → 1h −1.589 bps), 4h h5 the outlier.
- **Tails:** dir-adj p05/p95 approximately symmetric at 15s–5m (15s h1 [−3.23, +3.18]); adverse tails modestly heavier at several 15m–4h cells (15m h5 [−63.41, +60.75]; 1h h1 [−55.33, +49.53]; 4h h1 [−124.45, +84.10]); 4h h5 spans [−278.56, +282.74].
- **Dispersion scaling:** 15s dir-adj IQR width 1.675 → 3.064 → 3.975 bps (h1/h3/h5), slightly faster than √h.
- **Cross-scale caveat:** horizons are native bars — 5 bars is 75 s at 15s but 20 h at 4h. Cross-stratum forward-return comparison therefore compares different wall-clock exposures.
- **No significance claims:** no formal null test was performed or is possible from frozen quantile tables. Everything above is descriptive and registered as hypotheses for later testing.

---

## 5. Anomalies

| ID | Observation (counts / denominators) | Suspected class | Required before promotion |
|---|---|---|---|
| **A1 fill w/o prior recorded touch** | 682/227,925 (15s), 213, 81, 20, 10, 4, **0/167 (4h)** — exactly zero at 4h | Definitional or pipeline artifact: under authority semantics a filling bar's range must overlap the zone, so either first_touch is suppressed when the same bar fills (one-bar full-traverse phenotype) or a touch event was dropped/ordered out | Mechanistic explanation from scanner semantics; directly conditions validity of any FIRST_TOUCH-anchored cohort |
| **A2 orphan lifecycle events** | orphan fills 158→6, rate 0.070%→4.255% of fills; orphan touches 75→3, 0.033%→1.911% | Detector-state carryover across serialization boundaries or pre-window formations (authority: zones serialized/restored with state envelope) | Reconcile orphans to source formations; decide reportability of 1h/4h lifecycle stats |
| **A3 known-vs-market quantile jumps** | sub-second at fine strata; at coarse strata exact bar multiples, mixed sign (4h touch p75 +1 bar; 1h touch p75 −1 bar; 1h fill p75 −4 bars) | Small-N quantile granularity on n=157–497 populations + formation receipt latency at fine strata | Confirm on full distributions that known-time anchoring introduces no systematic shift beyond receipt latency |
| **A4 4h coverage excess** | 4h derived coverage 194.8 bar-days vs 183.2–183.6 at all finer strata | 4h bucket alignment partially counting non-trading time | Explain 4h bar construction over weekends/holidays before cross-scale comparison |
| **A5 response right-edge truncation** | h3/h5 n short by 1–3 rows at 15s, 5m, 15m | Expected end-of-stream truncation | Document the same truncation rule in any E1B |

---

## 6. Censoring cautions

1. **Definition:** right-censored = never filled by the final lawful native bar (227,925−226,908 = 1,017 at 15s; 167−141 = 26 at 4h). Administrative end-of-stream censoring, not a zone-level outcome; unfilled zones remain active per integrity gate.
2. **Fill-time quantiles condition on filled zones only** — censoring removes precisely the slowest zones, so coarse-stratum fill-time p50/p95 (8/403 bars at 1h; 5/170 at 4h) are downward-biased for the unconditional distribution. **Do not read coarse-scale fill-time compression as faster filling.**
3. **fill_rate is a within-window fraction, not a hazard or lifetime probability.** Window ≈ 1,471× p95 fill time at 15s vs 6.9× at 4h; cross-stratum fill-rate comparison conflates window length with filling behavior.
4. **The tail beyond p95 is uncharacterized** (tables report p05–p95 only): the weeks-long population, including the 26 censored 4h zones, has no quantile representation.
5. **gap_atr population is left-truncated** at min_gap_atr = 0.2 (p05 ≈ 0.22 everywhere confirms the filter binds); all gap_atr statements describe the truncated conditional population.
6. **Touch-time quantiles condition on touched zones** (99.46% of formations at 15s but 94.01% at 4h); the never-touched minority (541 → 10) is absent.
7. **Response panel right-truncated** at end of stream (1–3 rows).
8. No session/weekend metadata exists in the frozen tables; bar-count durations embed weekend gaps differently across strata.

---

## 7. Possible E1B stage-anchor requirements

**FIRST_TOUCH anchor — JUSTIFIED: YES.**
Touch is the first lifecycle gate with large N at fine/mid strata (226,702 at 15s down to 157 at 4h), and the touched-never-filled minority (476 at 15s → 16 at 4h) plus touch-conditional fill timing are entirely uncharacterized by formation-anchored E1 data.
*Preconditions:* resolve A1 first (if first_touch excludes same-bar fill bars, FIRST_TOUCH cohorts systematically exclude one-bar full traverses — must be stated in design); anchor on touch availability (known time); report fill vs no-fill as competing censored outcomes; no pooling.

**FILL anchor — JUSTIFIED: YES.**
Post-fill behavior is wholly unobserved in E1 (no post-fill response fields exist), while fills are the terminal event for 84.4%–99.6% of stratum populations with ample N at fine/mid scales (226,908 at 15s → 2,104 at 15m).
*Preconditions:* reconcile orphan fills (A2) first; expect limited power at 1h (482) and 4h (141) — strata analyzed independently; anchor on fill availability (known time) and handle the fill bar's own close explicitly to avoid same-bar contamination.

**Additional E1B requirements:** preserve (timeframe, zone_id) identity; include censored populations as outcome classes, not missing data; record market-time and known-time for every new anchored event; extend duration reporting beyond p95 (p99/max or full-distribution summaries).

---

## 8. Questions for the later Question Generator

1. What mechanism produces fill-without-prior-recorded-touch (682/227,925 at 15s, monotonically down to 0/167 at 4h), and does first_touch exclude bars that also fill (one-bar full traverse)?
2. What produces orphan touch/fill events, and why does the orphan rate rise with bar size (0.070% → 4.255% of fills)? State restoration vs pre-window formations?
3. Is the monotone rise in bullish formation share with bar size (0.5103 → 0.6048) explained by window drift alone, or is there scale-dependent formation asymmetry? Requires a drift-controlled null.
4. Is the direction-adjusted negative median (18/21 cells) robust under a formal drift-aware null, and does its magnitude scale with horizon, bar scale, or gap_atr/fvg_quality conditionals?
5. Why does 4h show ~194.8 derived bar-days of coverage vs ~183.2–183.6 at all finer strata?
6. Are the ±1-to-4-bar known-vs-market quantile discrepancies at 15m–4h pure small-N granularity or systematic receipt-latency structure?
7. What does the lifecycle tail beyond p95 look like, and how many zones persist for weeks (the 26 censored 4h zones among them)?
8. Is the 4h formation-incidence rebound (0.1429 vs 0.1187 at 1h) real or small-N variation (167 formations / 1,169 bars)?
9. Do the 4h h5 (+12.508 bps) and 1h h3 (+0.852 bps) positive dir-adj medians reflect genuine sign structure or small-sample medians? Formal null required before registration as a finding.
10. Does one-bar touch dominance (p25 = 1 bar, p50 = 2 bars everywhere) survive a null accounting for the mechanical overlap probability of adjacent-bar ranges, or is it partly geometric near-necessity?

---

## 9. Claims that must NOT be made

- That fill_rate at any stratum is the probability a zone eventually fills, a hazard rate, or a lifetime probability (censoring and window length forbid this).
- Any edge / alpha / profitability / strategy / predictive-advantage claim: E1 is a locked phenotype exposure; no such property has been established or tested.
- That the direction-adjusted negative medians constitute a tradable reversal effect (no formal null test performed).
- That raw pooled positive forward-return medians indicate post-formation continuation in the formation direction (sign split shows window-drift contamination).
- That the 4h h5 dir-adj median (+12.508 bps, N=167) is a genuine scale-dependent sign reversal.
- That strata are statistically interchangeable, or that populations may be pooled across timeframes to raise N (cross-scale identity prohibited).
- That lifecycle timing is faster/slower at coarse scales in wall-clock terms from bar-unit quantiles alone (coarse fill-time compression is partly censoring selection).
- That formation populations at 15m–4h are direction-balanced (bullish share 0.58–0.60 descriptively).
- Any statement attributing identity or correspondence to the same market gap observed at multiple timeframes.
- Any inference from E2, E3, confirmation data, or scanner internals beyond the frozen authority semantics.
- That the 4h incidence rebound (0.1429) is an established scale effect.

---

## 10. Overall assessment

**PASS_TO_SYNTHESIS.** The DEVELOPMENT evidence is internally consistent (all seven path decompositions close; stratum formations sum exactly to the 381,527 zone identities), the integrity gate is PASS on all nine checks, two independent runs produced identical logical hashes, unavailable counters are zero, and censoring is explicitly tracked. The phenotype is characterized at every native stratum with exact denominators. Flagged anomalies (A1–A4) are small-count or small-N items requiring mechanistic explanation before specific coarse-stratum statistics or any touch-anchored cohort are promoted, but none undermines the descriptive characterization itself — REWORK_REQUIRED is not warranted, and the evidence is decisive enough for synthesis, so INCONCLUSIVE is not warranted.

**Conditions carried into synthesis:** A1 must be resolved before FIRST_TOUCH-anchored design; A2 before coarse-stratum lifecycle promotion; formal null tests are required before any direction-adjusted asymmetry is treated as a finding.

---

*Evidence discipline: every quantitative statement cites E1_RESULTS.json.tables.strata.rows[timeframe=…] or is labeled derived with its arithmetic. No pooled-scale statistic is used analytically. Raw fill fractions are never described as hazards or lifetime probabilities. DIRECT_OBSERVATION / INTERPRETATION / HYPOTHESIS_FOR_LATER_TESTING are labeled where load-bearing. No significance language is used. Scanner code was not modified; nothing outside the two P-02 output files was written.*
