# AP-002 E1B PHENOTYPE REVIEW - Interpreter P-04

- Generated (UTC): 2026-09-06T13:21:18Z
- Interpreter: P-04 - independent scientific phenotype interpreter. Did not build the scanner, did not write the E1A interpretation, not performing confirmation, not constructing a strategy, did not inspect peer E1B interpretations.
- Program: AP-002 FVG | Stage: E1B lifecycle-stage phenotype | Instrument: XAUUSD | Exposure: E1_PHENOTYPE
- E1B logical result hash (verified in evidence): `2a7a142cadb36449d14d4128361d11ce99a3a4d7d8a004dd539bc4dc32fb823c`
- Cross-scale object identity: PROHIBITED (respected) | Confirmation: LOCKED | E2/E3: NOT STARTED
- **OVERALL VERDICT: PASS_TO_SYNTHESIS**
- Evidence base: exactly the nine frozen files listed in Appendix A. No scanner re-run, no row-level anchor tables, no peer interpretations, no pipeline code changes.

Citation notation: `E1B_RESULTS.json:tables.strata[tf].<anchor>.horizons[h].<field>`; 'derived' quantities show their formula. Duration quantiles are market/event time expressed in native bars (bar_ms 15,000..14,400,000); the separate known/availability clock is never merged (contract clock_rule).

## 1. Executive phenotype summary

**Verdict line.** PASS_TO_SYNTHESIS - the E1B evidence is internally consistent, complete for its frozen contract, and supports a descriptive lifecycle phenotype: a same-bar-or-one-bar resolution body, a heavy multi-bar tail that becomes administratively censored at coarse strata, and a descriptive stage-conditioning sign flip of weak pooled direction-adjusted central tendency between formation and contact.

### Direct observations
- **D1.** Identity/reconciliation (DIRECT OBSERVATION): two fresh runs produced the identical logical result hash 2a7a142cadb36449d14d4128361d11ce99a3a4d7d8a004dd539bc4dc32fb823c (REPRODUCIBILITY.json:logical_result_hash.identical=true); integrity gate PASS with 379,248 FIRST_TOUCH and 379,132 FILL anchors and 1,010 gap-through fills (E1B_SCANNER_INTEGRITY_GATE.json:stage_stratification); fill/touch consistency audit 0 disagreements across 379,132 fills (FILL_TOUCH_CONSISTENCY_AUDIT.json:disagreement_n); per-stratum sums reconcile exactly to the E1A corrected cohort: formations 381,498 (E1A accepted_scanner_evidence.zones), orphan events 178 touch + 358 fill.
- **D2.** Touch incidence is near-universal at fine strata and declines monotonically with native scale: 99.463% (15s, 226,686/227,909), 99.438% (30s, 99,305/99,866), 99.375% (1m, 43,428/43,701), 98.585% (5m, 7,039/7,140), 97.445% (15m, 2,136/2,192), 95.029% (1h, 497/523), 94.012% (4h, 157/167) (E1B_RESULTS.json:tables.strata[tf].touch_incidence.rate). The complement (never touched by window end) is an administrative right-censoring fraction, not a lifetime probability.
- **D3.** The touch->fill body is extremely fast at EVERY stratum: market-time p25 = 0 (same-bar touch+fill) and p50 = 1.0 native bar at all 7 strata (E1B_RESULTS.json:tables.strata[tf].touch_to_fill_market_ms.p50). Same-bar touch+fill is a substantial lifecycle mode everywhere: 35.31% (15s), 35.41% (30s), 35.36% (1m), 34.43% (5m), 34.19% (15m), 36.61% (1h), 40.43% (4h) of touched fills (derived: same_bar_touch_fill_n/touched_fill_n).
- **D4.** Touched-but-unfilled at window end (administrative right censoring of touch->fill) rises monotonically with scale: 0.210% (15s, 476/226,686), 0.296% (30s, 294/99,305), 0.463% (1m, 201/43,428), 1.108% (5m, 78/7,039), 1.966% (15m, 42/2,136), 3.823% (1h, 19/497), 10.191% (4h, 16/157) (E1B_RESULTS.json:tables.strata[tf].touch_to_fill_right_censor_fraction). Never-filled fraction (formation->fill censoring) likewise: 0.446% -> 0.643% -> 0.899% -> 2.227% -> 4.015% -> 7.839% -> 15.569%.
- **D5.** Touch->fill tail (conditional on fill): p95 = 204 (15s), 188 (30s), 181 (1m), 209 (5m), 226 (15m), 152 (1h), 66 (4h) native bars; p99 = 3,885 (15s), 4,394 (30s), 3,582 (1m), 2,388 (5m), 1,648 (15m), 699 (1h), 175 bars (4h). Observed maxima are ~118.5 days market time and coincide with the development-window end boundary: administrative censoring, not natural lifecycle endpoints (E1B_RESULTS.json:tables.strata[tf].touch_to_fill_market_ms).
- **D6.** Post-anchor prospective direction-adjusted p50 is positive in 42/42 anchor x stratum x horizon cells (21 FIRST_TOUCH + 21 FILL); raw p50 is positive in 41/42 (sole exception: 4h FILL h1 = -0.145 bps, N=141). At 15s-15m the positive medians are tiny relative to dispersion: |p50|/IQR-width in the h1 cells spans 0.56%-2.64% (e.g., 15s FIRST_TOUCH h1 +0.0136 bps vs IQR width 1.688 bps), i.e., the median is roughly 38x-180x smaller than the interquartile width.
- **D7.** Direction split of direction-adjusted p50: bullish positive in 21/21 FIRST_TOUCH and 21/21 FILL cells; bearish positive in 5/21 FIRST_TOUCH cells (all three 4h cells, 1h h1, 1m h1), exactly zero (serialized -0.0) in 5, negative in 11; at FILL bearish is positive in 10, zero in 2, negative in 9. There is NO sign-consistent bearish pattern across strata or horizons.
- **D8.** gap_atr and fvg_quality median splits show no sign-consistent differences: the sign of (high-median minus low-median) flips between h1 and h5 within the same anchor and stratum in 6 (stratum, anchor, split) combinations (e.g., 15s FIRST_TOUCH gap_atr h1 low>high, h5 high>low; 5m FILL fvg_quality h1 low<high then h5 similar but 15m FIRST_TOUCH fvg_quality h1 low>high reverses the fine-stratum ordering). Split thresholds are cohort-specific (they differ between FIRST_TOUCH and FILL anchors, e.g., 15s gap_atr 0.56437207354277 vs 0.5632869877940201) and development-derived.

### Interpretation
- **I1.** Phenotype (INTERPRETATION): the native FVG lifecycle is a same-bar-or-one-bar resolution body with a heavy multi-bar tail and scale-widening administrative censoring. E1A's 'rapid body + long tail' survives only with three qualifications: (i) the body is faster than formation->fill medians (7-8 bars) imply because after touch the median is 1 bar with a ~34-40% same-bar mode; (ii) the tail is heavier than p95 suggests (p99 in the thousands of native bars at 15s-1m, identifiable only there); (iii) at 5m-4h the long-tail regime is increasingly unidentifiable rather than observed (censor ceilings p97.77/p95.99/p92.16/p84.43 for formation->fill at 5m/15m/1h/4h).
- **I2.** Stage conditioning changes the descriptive sign (INTERPRETATION): formation-anchored direction-adjusted medians were 18/21 negative, 1 zero, 2 positive (frozen AP-002_E1A_SYNTHESIS.json canonical_observations[5]); touch-anchored and fill-anchored medians are 42/42 positive. Descriptively, the negative formation-stage central tendency is spent on the approach into the zone; after contact the pooled central tendency is flat-to-weakly-positive. This is a sign statement only - magnitudes remain 38x-180x smaller than IQR width at 15s-15m. Candidate mechanisms (not separable with this evidence): approach-phase concentration, anchor-price definition change (formation availability vs boundary-crossing tick mid), and cohort conditioning (touched-only).
- **I3.** Post-fill descriptive character (INTERPRETATION): weak zone-direction drift at fine strata (equivalently, reversion of the filling move); no scale-consistent label at mid strata (bearish-zone adjusted medians continue adversely at 5m/15m h3/h5); apparent stronger zone-direction continuation at 1h/4h exactly where N is smallest (482/141 fills) and in-window instrument drift is an unresolved confound. No 'continuation'/'reversion'/'neutralization' label is scale-consistent; none is converted to a trading statement.

### Hypotheses for later testing
- **H1.** HYPOTHESIS FOR LATER TESTING: the formation->touch sign flip reflects approach-phase dynamics rather than post-contact behavior (QG1).
- **H2.** HYPOTHESIS FOR LATER TESTING: post-fill zone-direction drift at fine strata is an FVG-specific effect rather than unconditional instrument drift (QG5, requires drift-matched null).
- **H3.** HYPOTHESIS FOR LATER TESTING: gap-through fills behave differently after fill from ordinary touch->fill (QG2; requires new measurement, not derivable from E1B aggregate tables).

## 2. FIRST_TOUCH per-stratum observations

Stage anchor: actual emitted `fvg_first_touch` event; anchor price = boundary-crossing tick mid; horizons = lawful future native bar closes after the anchor bar; gap-through fills (first_touch_observed=false) are NOT in this cohort. All cells N as shown; end-of-window missing reported per cell.

### 15s - FIRST_TOUCH

- Counts: formations 227,909; First_Touch anchors 226,686; touched-fill 226,210; gap-through 682; same-bar touch+fill 79,865 (= 35.31% of touched fills); touched-but-unfilled at window end 476 (0.210% of touch anchors); orphans excluded 75 touch / 159 fill. `E1B_RESULTS.json:tables.strata[15s].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 190 / p99 3495; touch->fill (conditional on fill, n=226,210) p25 0 (same-bar) / p50 1 / p95 204 / p99 3885, max 118.5 d; formation->fill p50 7 / p95 717 / p99 14178.
- Censoring: touch->fill 0.210% (max identifiable unconditional quantile 0.9979); formation->fill 0.446% (0.9955). `E1B_RESULTS.json:tables.strata[15s].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 226,686 | 0 | +0.0119 | +0.0136 | -0.8180..+0.8698 | -3.2172..+3.3161 | +7.4710 | +67.9973 |
| 3 | 226,686 | 0 | +0.0273 | +0.0303 | -1.5008..+1.5734 | -5.5744..+5.7330 | +12.6012 | +192.6443 |
| 5 | 226,686 | 0 | +0.0363 | +0.0362 | -1.9781..+2.0435 | -7.2315..+7.3730 | +16.1295 | +182.6496 |

- h1 splits (adj p50 bps): bullish +0.0274 (n=115,617) / bearish -0.0000 (n=111,069); gap_atr low +0.0151 vs high +0.0122 (thr 0.5644); fvg_quality low +0.0123 vs high +0.0148 (thr 0.7977). |median| = 0.8% of IQR width.
- h3 splits (adj p50 bps): bullish +0.0594 (n=115,617) / bearish -0.0000 (n=111,069); gap_atr low +0.0389 vs high +0.0274 (thr 0.5644); fvg_quality low +0.0313 vs high +0.0301 (thr 0.7977). |median| = 1.0% of IQR width.
- h5 splits (adj p50 bps): bullish +0.0726 (n=115,617) / bearish -0.0000 (n=111,069); gap_atr low +0.0304 vs high +0.0407 (thr 0.5644); fvg_quality low +0.0300 vs high +0.0411 (thr 0.7977). |median| = 0.9% of IQR width.

### 30s - FIRST_TOUCH

- Counts: formations 99,866; First_Touch anchors 99,305; touched-fill 99,011; gap-through 213; same-bar touch+fill 35,055 (= 35.41% of touched fills); touched-but-unfilled at window end 294 (0.296% of touch anchors); orphans excluded 47 touch / 94 fill. `E1B_RESULTS.json:tables.strata[30s].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 184 / p99 3619; touch->fill (conditional on fill, n=99,011) p25 0 (same-bar) / p50 1 / p95 188 / p99 4394, max 118.5 d; formation->fill p50 7 / p95 695 / p99 11481.
- Censoring: touch->fill 0.296% (max identifiable unconditional quantile 0.9970); formation->fill 0.643% (0.9936). `E1B_RESULTS.json:tables.strata[30s].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 99,305 | 0 | +0.0211 | +0.0141 | -1.2325..+1.2886 | -4.6939..+4.8210 | +10.7230 | +133.2789 |
| 3 | 99,305 | 0 | +0.0578 | +0.0482 | -2.1905..+2.2978 | -8.0963..+8.2445 | +18.0711 | +132.5769 |
| 5 | 99,305 | 0 | +0.0738 | +0.0444 | -2.8496..+2.9728 | -10.3456..+10.5173 | +23.2270 | +204.8825 |

- h1 splits (adj p50 bps): bullish +0.0297 (n=51,299) / bearish -0.0000 (n=48,006); gap_atr low +0.0212 vs high +0.0116 (thr 0.5201); fvg_quality low +0.0110 vs high +0.0207 (thr 0.7832). |median| = 0.6% of IQR width.
- h3 splits (adj p50 bps): bullish +0.1045 (n=51,299) / bearish -0.0000 (n=48,006); gap_atr low +0.0445 vs high +0.0549 (thr 0.5201); fvg_quality low +0.0431 vs high +0.0566 (thr 0.7832). |median| = 1.1% of IQR width.
- h5 splits (adj p50 bps): bullish +0.1175 (n=51,299) / bearish -0.0300 (n=48,006); gap_atr low +0.0254 vs high +0.0598 (thr 0.5201); fvg_quality low +0.0439 vs high +0.0445 (thr 0.7832). |median| = 0.8% of IQR width.

### 1m - FIRST_TOUCH

- Counts: formations 43,701; First_Touch anchors 43,428; touched-fill 43,227; gap-through 81; same-bar touch+fill 15,286 (= 35.36% of touched fills); touched-but-unfilled at window end 201 (0.463% of touch anchors); orphans excluded 33 touch / 62 fill. `E1B_RESULTS.json:tables.strata[1m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 185 / p99 3630; touch->fill (conditional on fill, n=43,227) p25 0 (same-bar) / p50 1 / p95 181 / p99 3582, max 118.5 d; formation->fill p50 7 / p95 670 / p99 10701.
- Censoring: touch->fill 0.463% (max identifiable unconditional quantile 0.9954); formation->fill 0.899% (0.9910). `E1B_RESULTS.json:tables.strata[1m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 43,428 | 0 | +0.0358 | +0.0654 | -1.8126..+1.9456 | -6.8243..+6.9395 | +15.8304 | +84.7348 |
| 3 | 43,428 | 0 | +0.0899 | +0.0759 | -3.0831..+3.3144 | -11.5385..+11.7002 | +26.4608 | +232.1854 |
| 5 | 43,427 | 1 | +0.1350 | +0.0741 | -4.0386..+4.2973 | -14.6150..+14.9216 | +34.1303 | +277.6201 |

- h1 splits (adj p50 bps): bullish +0.0981 (n=22,759) / bearish +0.0313 (n=20,669); gap_atr low +0.0843 vs high +0.0499 (thr 0.4918); fvg_quality low +0.0686 vs high +0.0622 (thr 0.7738). |median| = 1.7% of IQR width.
- h3 splits (adj p50 bps): bullish +0.1699 (n=22,759) / bearish -0.0120 (n=20,669); gap_atr low +0.1313 vs high +0.0243 (thr 0.4918); fvg_quality low +0.0896 vs high +0.0635 (thr 0.7738). |median| = 1.2% of IQR width.
- h5 splits (adj p50 bps): bullish +0.1937 (n=22,759) / bearish -0.0644 (n=20,668); gap_atr low +0.1350 vs high +0.0098 (thr 0.4918); fvg_quality low +0.1065 vs high +0.0457 (thr 0.7738). |median| = 0.9% of IQR width.

### 5m - FIRST_TOUCH

- Counts: formations 7,140; First_Touch anchors 7,039; touched-fill 6,961; gap-through 20; same-bar touch+fill 2,397 (= 34.43% of touched fills); touched-but-unfilled at window end 78 (1.108% of touch anchors); orphans excluded 8 touch / 18 fill. `E1B_RESULTS.json:tables.strata[5m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 158 / p99 2173; touch->fill (conditional on fill, n=6,961) p25 0 (same-bar) / p50 1 / p95 209 / p99 2388, max 118.5 d; formation->fill p50 7 / p95 701 / p99 4654.
- Censoring: touch->fill 1.108% (max identifiable unconditional quantile 0.9889); formation->fill 2.227% (0.9777). `E1B_RESULTS.json:tables.strata[5m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 7,038 | 1 | +0.1912 | +0.1449 | -4.2281..+4.4157 | -15.5576..+15.0157 | +33.1078 | +174.1076 |
| 3 | 7,038 | 1 | +0.4282 | +0.2353 | -7.4663..+7.8531 | -26.5578..+27.0100 | +57.1305 | +347.6085 |
| 5 | 7,038 | 1 | +0.8621 | +0.1689 | -9.3680..+9.6365 | -34.2408..+34.3605 | +69.8438 | +428.1910 |

- h1 splits (adj p50 bps): bullish +0.3009 (n=3,806) / bearish -0.0378 (n=3,232); gap_atr low +0.0960 vs high +0.2122 (thr 0.4565); fvg_quality low +0.0429 vs high +0.3046 (thr 0.7503). |median| = 1.7% of IQR width.
- h3 splits (adj p50 bps): bullish +0.6795 (n=3,806) / bearish -0.2068 (n=3,232); gap_atr low +0.3475 vs high +0.0673 (thr 0.4565); fvg_quality low +0.2193 vs high +0.2353 (thr 0.7503). |median| = 1.5% of IQR width.
- h5 splits (adj p50 bps): bullish +0.9831 (n=3,806) / bearish -0.7556 (n=3,232); gap_atr low +0.2335 vs high +0.1334 (thr 0.4565); fvg_quality low +0.1376 vs high +0.1729 (thr 0.7503). |median| = 0.9% of IQR width.

### 15m - FIRST_TOUCH

- Counts: formations 2,192; First_Touch anchors 2,136; touched-fill 2,094; gap-through 10; same-bar touch+fill 716 (= 34.19% of touched fills); touched-but-unfilled at window end 42 (1.966% of touch anchors); orphans excluded 7 touch / 12 fill. `E1B_RESULTS.json:tables.strata[15m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 186 / p99 1238; touch->fill (conditional on fill, n=2,094) p25 0 (same-bar) / p50 1 / p95 226 / p99 1648, max 117.3 d; formation->fill p50 8 / p95 562 / p99 2835.
- Censoring: touch->fill 1.966% (max identifiable unconditional quantile 0.9803); formation->fill 4.015% (0.9599). `E1B_RESULTS.json:tables.strata[15m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 2,136 | 0 | +0.5533 | +0.1167 | -7.4523..+7.8303 | -25.0155..+29.7511 | +90.2924 | +210.8515 |
| 3 | 2,133 | 3 | +1.5572 | +0.9927 | -12.1799..+14.3196 | -44.8405..+50.9812 | +148.6171 | +331.7680 |
| 5 | 2,133 | 3 | +2.3846 | +1.0821 | -16.2508..+19.0366 | -58.3426..+59.1794 | +149.8517 | +391.1271 |

- h1 splits (adj p50 bps): bullish +0.6325 (n=1,231) / bearish -0.3598 (n=905); gap_atr low -0.2727 vs high +0.6546 (thr 0.4585); fvg_quality low +0.2670 vs high +0.0223 (thr 0.7498). |median| = 0.8% of IQR width.
- h3 splits (adj p50 bps): bullish +2.4100 (n=1,229) / bearish -0.9342 (n=904); gap_atr low +0.7893 vs high +1.0053 (thr 0.4585); fvg_quality low +1.2338 vs high +0.6747 (thr 0.7498). |median| = 3.7% of IQR width.
- h5 splits (adj p50 bps): bullish +2.9308 (n=1,229) / bearish -1.5156 (n=904); gap_atr low +1.1696 vs high +0.8922 (thr 0.4585); fvg_quality low +1.1714 vs high +0.9349 (thr 0.7498). |median| = 3.1% of IQR width.

### 1h - FIRST_TOUCH

- Counts: formations 523; First_Touch anchors 497; touched-fill 478; gap-through 4; same-bar touch+fill 175 (= 36.61% of touched fills); touched-but-unfilled at window end 19 (3.823% of touch anchors); orphans excluded 5 touch / 7 fill. `E1B_RESULTS.json:tables.strata[1h].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 193 / p99 742; touch->fill (conditional on fill, n=478) p25 0 (same-bar) / p50 1 / p95 152 / p99 699, max 118.0 d; formation->fill p50 8 / p95 403 / p99 1249.
- Censoring: touch->fill 3.823% (max identifiable unconditional quantile 0.9618); formation->fill 7.839% (0.9216). `E1B_RESULTS.json:tables.strata[1h].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 497 | 0 | +0.7914 | +1.4079 | -15.9287..+19.4735 | -53.7862..+58.9426 | +156.0216 | +173.4232 |
| 3 | 497 | 0 | +6.0873 | +2.7102 | -27.4709..+33.0764 | -107.3601..+96.9441 | +230.2854 | +399.6309 |
| 5 | 497 | 0 | +9.7623 | +5.1683 | -37.8891..+38.2916 | -115.1630..+122.4278 | +382.4987 | +499.9651 |

- h1 splits (adj p50 bps): bullish +1.6122 (n=288) / bearish +0.9325 (n=209); gap_atr low -1.6039 vs high +6.0915 (thr 0.4481); fvg_quality low -0.9150 vs high +6.6475 (thr 0.7427). |median| = 4.0% of IQR width.
- h3 splits (adj p50 bps): bullish +8.3829 (n=288) / bearish -3.6372 (n=209); gap_atr low +0.9980 vs high +3.6610 (thr 0.4481); fvg_quality low +2.7102 vs high +1.3160 (thr 0.7427). |median| = 4.5% of IQR width.
- h5 splits (adj p50 bps): bullish +14.7226 (n=288) / bearish -4.3689 (n=209); gap_atr low +1.7827 vs high +7.2397 (thr 0.4481); fvg_quality low +4.7266 vs high +6.2725 (thr 0.7427). |median| = 6.8% of IQR width.

### 4h - FIRST_TOUCH

- Counts: formations 167; First_Touch anchors 157; touched-fill 141; gap-through 0; same-bar touch+fill 57 (= 40.43% of touched fills); touched-but-unfilled at window end 16 (10.191% of touch anchors); orphans excluded 3 touch / 6 fill. `E1B_RESULTS.json:tables.strata[4h].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 89 / p99 312; touch->fill (conditional on fill, n=141) p25 0 (same-bar) / p50 1 / p95 66 / p99 175, max 76.5 d; formation->fill p50 5 / p95 170 / p99 462.
- Censoring: touch->fill 10.191% (max identifiable unconditional quantile 0.8981); formation->fill 15.569% (0.8443). `E1B_RESULTS.json:tables.strata[4h].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 157 | 0 | +4.8126 | +10.6759 | -25.2690..+45.1305 | -92.0496..+237.6016 | +389.5971 | +445.4482 |
| 3 | 157 | 0 | +10.9879 | +9.3747 | -42.0713..+69.9327 | -162.4037..+213.9591 | +376.1206 | +556.0587 |
| 5 | 157 | 0 | +1.1058 | +13.2671 | -63.9811..+96.9308 | -305.4587..+285.0989 | +483.4705 | +929.4643 |

- h1 splits (adj p50 bps): bullish +11.2237 (n=94) / bearish +4.5013 (n=63); gap_atr low +14.0850 vs high +6.7062 (thr 0.5293); fvg_quality low +12.7375 vs high +9.1085 (thr 0.7596). |median| = 15.2% of IQR width.
- h3 splits (adj p50 bps): bullish +17.2807 (n=94) / bearish +1.3519 (n=63); gap_atr low +17.2807 vs high +8.1311 (thr 0.5293); fvg_quality low +2.0338 vs high +14.1805 (thr 0.7596). |median| = 8.4% of IQR width.
- h5 splits (adj p50 bps): bullish +15.7234 (n=94) / bearish +7.7319 (n=63); gap_atr low +18.7328 vs high +7.7319 (thr 0.5293); fvg_quality low +18.7328 vs high +0.7361 (thr 0.7596). |median| = 8.2% of IQR width.

## 3. FILL per-stratum observations

Stage anchor: lawful `fvg_filled` event (same causal anchor rule); stratification first_touch_observed true/false is reported as counts only (Section 5).

### 15s - FILL

- Counts: formations 227,909; Fill anchors 226,892; touched-fill 226,210; gap-through 682; same-bar touch+fill 79,865 (= 35.31% of touched fills); touched-but-unfilled at window end 476 (0.210% of touch anchors); orphans excluded 75 touch / 159 fill. `E1B_RESULTS.json:tables.strata[15s].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 190 / p99 3495; touch->fill (conditional on fill, n=226,210) p25 0 (same-bar) / p50 1 / p95 204 / p99 3885, max 118.5 d; formation->fill p50 7 / p95 717 / p99 14178.
- Censoring: touch->fill 0.210% (max identifiable unconditional quantile 0.9979); formation->fill 0.446% (0.9955). `E1B_RESULTS.json:tables.strata[15s].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 226,892 | 0 | +0.0208 | +0.0214 | -0.8393..+0.9102 | -3.4038..+3.5173 | +8.0710 | +67.9973 |
| 3 | 226,892 | 0 | +0.0387 | +0.0662 | -1.5083..+1.6537 | -5.7955..+5.9949 | +13.3718 | +151.8703 |
| 5 | 226,892 | 0 | +0.0547 | +0.0850 | -1.9783..+2.1498 | -7.5319..+7.6062 | +16.9603 | +297.2623 |

- h1 splits (adj p50 bps): bullish +0.0402 (n=115,665) / bearish -0.0000 (n=111,227); gap_atr low +0.0214 vs high +0.0214 (thr 0.5633); fvg_quality low +0.0197 vs high +0.0232 (thr 0.7973). |median| = 1.2% of IQR width.
- h3 splits (adj p50 bps): bullish +0.1046 (n=115,665) / bearish +0.0271 (n=111,227); gap_atr low +0.0561 vs high +0.0754 (thr 0.5633); fvg_quality low +0.0534 vs high +0.0803 (thr 0.7973). |median| = 2.1% of IQR width.
- h5 splits (adj p50 bps): bullish +0.1415 (n=115,665) / bearish +0.0297 (n=111,227); gap_atr low +0.0600 vs high +0.1063 (thr 0.5633); fvg_quality low +0.0666 vs high +0.1037 (thr 0.7973). |median| = 2.1% of IQR width.

### 30s - FILL

- Counts: formations 99,866; Fill anchors 99,224; touched-fill 99,011; gap-through 213; same-bar touch+fill 35,055 (= 35.41% of touched fills); touched-but-unfilled at window end 294 (0.296% of touch anchors); orphans excluded 47 touch / 94 fill. `E1B_RESULTS.json:tables.strata[30s].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 184 / p99 3619; touch->fill (conditional on fill, n=99,011) p25 0 (same-bar) / p50 1 / p95 188 / p99 4394, max 118.5 d; formation->fill p50 7 / p95 695 / p99 11481.
- Censoring: touch->fill 0.296% (max identifiable unconditional quantile 0.9970); formation->fill 0.643% (0.9936). `E1B_RESULTS.json:tables.strata[30s].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 99,224 | 0 | +0.0400 | +0.0299 | -1.2549..+1.3684 | -4.9357..+5.1703 | +11.6731 | +133.2789 |
| 3 | 99,224 | 0 | +0.0773 | +0.1092 | -2.1929..+2.4467 | -8.5719..+8.6657 | +19.1528 | +107.4086 |
| 5 | 99,223 | 1 | +0.1097 | +0.1620 | -2.8190..+3.1262 | -10.8563..+11.0711 | +25.4858 | +192.6610 |

- h1 splits (adj p50 bps): bullish +0.0721 (n=51,235) / bearish -0.0000 (n=47,989); gap_atr low +0.0282 vs high +0.0354 (thr 0.5192); fvg_quality low +0.0299 vs high +0.0299 (thr 0.7828). |median| = 1.1% of IQR width.
- h3 splits (adj p50 bps): bullish +0.1812 (n=51,235) / bearish +0.0375 (n=47,989); gap_atr low +0.0847 vs high +0.1364 (thr 0.5192); fvg_quality low +0.1052 vs high +0.1166 (thr 0.7828). |median| = 2.4% of IQR width.
- h5 splits (adj p50 bps): bullish +0.2678 (n=51,234) / bearish +0.0545 (n=47,989); gap_atr low +0.1184 vs high +0.2033 (thr 0.5192); fvg_quality low +0.1495 vs high +0.1770 (thr 0.7828). |median| = 2.7% of IQR width.

### 1m - FILL

- Counts: formations 43,701; Fill anchors 43,308; touched-fill 43,227; gap-through 81; same-bar touch+fill 15,286 (= 35.36% of touched fills); touched-but-unfilled at window end 201 (0.463% of touch anchors); orphans excluded 33 touch / 62 fill. `E1B_RESULTS.json:tables.strata[1m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 185 / p99 3630; touch->fill (conditional on fill, n=43,227) p25 0 (same-bar) / p50 1 / p95 181 / p99 3582, max 118.5 d; formation->fill p50 7 / p95 670 / p99 10701.
- Censoring: touch->fill 0.463% (max identifiable unconditional quantile 0.9954); formation->fill 0.899% (0.9910). `E1B_RESULTS.json:tables.strata[1m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 43,308 | 0 | +0.0598 | +0.1041 | -1.8261..+2.1094 | -7.2715..+7.5804 | +17.6192 | +84.7348 |
| 3 | 43,308 | 0 | +0.1408 | +0.1896 | -3.1028..+3.5415 | -12.1606..+12.6942 | +28.5756 | +193.1671 |
| 5 | 43,308 | 0 | +0.1897 | +0.2662 | -3.9897..+4.5903 | -15.2696..+15.7241 | +36.8906 | +188.8146 |

- h1 splits (adj p50 bps): bullish +0.1776 (n=22,673) / bearish +0.0399 (n=20,635); gap_atr low +0.0799 vs high +0.1296 (thr 0.4909); fvg_quality low +0.0827 vs high +0.1262 (thr 0.7734). |median| = 2.6% of IQR width.
- h3 splits (adj p50 bps): bullish +0.3177 (n=22,673) / bearish +0.0555 (n=20,635); gap_atr low +0.1817 vs high +0.1959 (thr 0.4909); fvg_quality low +0.1708 vs high +0.2126 (thr 0.7734). |median| = 2.9% of IQR width.
- h5 splits (adj p50 bps): bullish +0.4328 (n=22,673) / bearish +0.0651 (n=20,635); gap_atr low +0.2227 vs high +0.3128 (thr 0.4909); fvg_quality low +0.2517 vs high +0.2808 (thr 0.7734). |median| = 3.1% of IQR width.

### 5m - FILL

- Counts: formations 7,140; Fill anchors 6,981; touched-fill 6,961; gap-through 20; same-bar touch+fill 2,397 (= 34.43% of touched fills); touched-but-unfilled at window end 78 (1.108% of touch anchors); orphans excluded 8 touch / 18 fill. `E1B_RESULTS.json:tables.strata[5m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 158 / p99 2173; touch->fill (conditional on fill, n=6,961) p25 0 (same-bar) / p50 1 / p95 209 / p99 2388, max 118.5 d; formation->fill p50 7 / p95 701 / p99 4654.
- Censoring: touch->fill 1.108% (max identifiable unconditional quantile 0.9889); formation->fill 2.227% (0.9777). `E1B_RESULTS.json:tables.strata[5m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 6,980 | 1 | +0.2232 | +0.1762 | -4.5217..+4.8884 | -16.5078..+16.8874 | +42.3601 | +174.1076 |
| 3 | 6,980 | 1 | +0.6150 | +0.4032 | -7.8180..+8.4972 | -28.8483..+29.3521 | +64.4674 | +347.6085 |
| 5 | 6,980 | 1 | +1.0827 | +0.1905 | -9.8657..+10.4382 | -36.0674..+35.8287 | +84.7177 | +428.1910 |

- h1 splits (adj p50 bps): bullish +0.3673 (n=3,766) / bearish -0.0192 (n=3,214); gap_atr low +0.1072 vs high +0.3046 (thr 0.4557); fvg_quality low +0.0493 vs high +0.3653 (thr 0.7498). |median| = 1.9% of IQR width.
- h3 splits (adj p50 bps): bullish +0.9658 (n=3,766) / bearish -0.0976 (n=3,214); gap_atr low +0.3045 vs high +0.5983 (thr 0.4557); fvg_quality low +0.1990 vs high +0.6240 (thr 0.7498). |median| = 2.5% of IQR width.
- h5 splits (adj p50 bps): bullish +1.2093 (n=3,766) / bearish -0.9708 (n=3,214); gap_atr low +0.0220 vs high +0.4915 (thr 0.4557); fvg_quality low +0.0844 vs high +0.3701 (thr 0.7498). |median| = 0.9% of IQR width.

### 15m - FILL

- Counts: formations 2,192; Fill anchors 2,104; touched-fill 2,094; gap-through 10; same-bar touch+fill 716 (= 34.19% of touched fills); touched-but-unfilled at window end 42 (1.966% of touch anchors); orphans excluded 7 touch / 12 fill. `E1B_RESULTS.json:tables.strata[15m].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 186 / p99 1238; touch->fill (conditional on fill, n=2,094) p25 0 (same-bar) / p50 1 / p95 226 / p99 1648, max 117.3 d; formation->fill p50 8 / p95 562 / p99 2835.
- Censoring: touch->fill 1.966% (max identifiable unconditional quantile 0.9803); formation->fill 4.015% (0.9599). `E1B_RESULTS.json:tables.strata[15m].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 2,103 | 1 | +0.9931 | +0.1178 | -8.0519..+8.4640 | -31.6591..+30.1669 | +88.7056 | +216.0947 |
| 3 | 2,102 | 2 | +1.4220 | +1.1108 | -13.2509..+15.8361 | -48.9365..+52.2998 | +162.8870 | +313.5513 |
| 5 | 2,102 | 2 | +3.5369 | +1.6074 | -16.6576..+20.9234 | -64.1585..+72.2773 | +197.2482 | +391.1271 |

- h1 splits (adj p50 bps): bullish +1.0943 (n=1,212) / bearish -0.8049 (n=891); gap_atr low -0.2306 vs high +0.5308 (thr 0.4572); fvg_quality low -0.1251 vs high +0.4033 (thr 0.7492). |median| = 0.7% of IQR width.
- h3 splits (adj p50 bps): bullish +2.1097 (n=1,211) / bearish -0.6228 (n=891); gap_atr low +1.2551 vs high +0.8440 (thr 0.4572); fvg_quality low +0.7594 vs high +1.3476 (thr 0.7492). |median| = 3.8% of IQR width.
- h5 splits (adj p50 bps): bullish +4.5953 (n=1,211) / bearish -1.7444 (n=891); gap_atr low +0.5741 vs high +2.1320 (thr 0.4572); fvg_quality low +0.8411 vs high +2.0993 (thr 0.7492). |median| = 4.3% of IQR width.

### 1h - FILL

- Counts: formations 523; Fill anchors 482; touched-fill 478; gap-through 4; same-bar touch+fill 175 (= 36.61% of touched fills); touched-but-unfilled at window end 19 (3.823% of touch anchors); orphans excluded 5 touch / 7 fill. `E1B_RESULTS.json:tables.strata[1h].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 193 / p99 742; touch->fill (conditional on fill, n=478) p25 0 (same-bar) / p50 1 / p95 152 / p99 699, max 118.0 d; formation->fill p50 8 / p95 403 / p99 1249.
- Censoring: touch->fill 3.823% (max identifiable unconditional quantile 0.9618); formation->fill 7.839% (0.9216). `E1B_RESULTS.json:tables.strata[1h].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 482 | 0 | +1.3576 | +2.0200 | -18.4269..+23.5744 | -59.5583..+79.2137 | +157.5649 | +220.0138 |
| 3 | 482 | 0 | +5.4304 | +1.7706 | -25.6913..+32.3098 | -100.4441..+108.2418 | +301.8258 | +476.1345 |
| 5 | 481 | 1 | +9.9229 | +8.0925 | -35.0129..+38.4735 | -146.6121..+164.3684 | +409.8213 | +655.0474 |

- h1 splits (adj p50 bps): bullish +3.7745 (n=279) / bearish +0.8197 (n=203); gap_atr low +0.0637 vs high +4.6654 (thr 0.4405); fvg_quality low -0.0720 vs high +6.4764 (thr 0.7379). |median| = 4.8% of IQR width.
- h3 splits (adj p50 bps): bullish +8.0965 (n=279) / bearish -3.0579 (n=203); gap_atr low +1.9514 vs high +1.0023 (thr 0.4405); fvg_quality low -2.1762 vs high +4.1956 (thr 0.7379). |median| = 3.1% of IQR width.
- h5 splits (adj p50 bps): bullish +15.4124 (n=279) / bearish -4.8899 (n=202); gap_atr low +2.1900 vs high +11.4317 (thr 0.4405); fvg_quality low +0.2422 vs high +10.9166 (thr 0.7379). |median| = 11.0% of IQR width.

### 4h - FILL

- Counts: formations 167; Fill anchors 141; touched-fill 141; gap-through 0; same-bar touch+fill 57 (= 40.43% of touched fills); touched-but-unfilled at window end 16 (10.191% of touch anchors); orphans excluded 3 touch / 6 fill. `E1B_RESULTS.json:tables.strata[4h].{lawful_formation_n,touch_anchors_n,fill_anchors_n,touched_fill_n,gap_through_fill_n,same_bar_touch_fill_n,orphan_*_n,n_bars}`
- Durations (native bars, market time): formation->touch p50 2 / p95 89 / p99 312; touch->fill (conditional on fill, n=141) p25 0 (same-bar) / p50 1 / p95 66 / p99 175, max 76.5 d; formation->fill p50 5 / p95 170 / p99 462.
- Censoring: touch->fill 10.191% (max identifiable unconditional quantile 0.8981); formation->fill 15.569% (0.8443). `E1B_RESULTS.json:tables.strata[4h].{fill_right_censor_fraction,fill_max_identifiable_quantile,touch_to_fill_right_censor_fraction,touch_to_fill_max_identifiable_quantile}`

| h | N | miss | raw p50 bps | adj p50 bps | adj IQR (p25..p75) | adj p05..p95 | adj p99 | adj max |
|---|---|------|-------------|-------------|--------------------|--------------|---------|---------|
| 1 | 141 | 0 | -0.1451 | +8.3089 | -36.1272..+41.0301 | -95.6500..+237.6016 | +389.5971 | +418.5104 |
| 3 | 141 | 0 | +6.8939 | +18.1627 | -43.0926..+69.9327 | -320.2068..+213.9591 | +462.6894 | +506.5152 |
| 5 | 141 | 0 | +15.7234 | +15.7234 | -78.2223..+98.0394 | -410.3685..+337.7192 | +483.4705 | +486.9657 |

- h1 splits (adj p50 bps): bullish +8.3250 (n=84) / bearish +5.8094 (n=57); gap_atr low +8.3250 vs high +2.3972 (thr 0.4913); fvg_quality low +12.7375 vs high -4.7588 (thr 0.7452). |median| = 10.8% of IQR width.
- h3 splits (adj p50 bps): bullish +25.5115 (n=84) / bearish +6.1734 (n=57); gap_atr low +22.9403 vs high +8.1311 (thr 0.4913); fvg_quality low +25.5756 vs high +6.1734 (thr 0.7452). |median| = 16.1% of IQR width.
- h5 splits (adj p50 bps): bullish +39.1847 (n=84) / bearish -1.1681 (n=57); gap_atr low +13.8413 vs high +21.0732 (thr 0.4913); fvg_quality low +17.8286 vs high +12.4840 (thr 0.7452). |median| = 8.9% of IQR width.

## 4. Formation (E1A) vs FIRST_TOUCH (E1B) vs FILL (E1B) - descriptive stage comparison

| stage | raw p50 | direction-adjusted p50 | source |
|---|---|---|---|
| FORMATION (E1A, frozen) | positive 21/21 | 18 negative / 1 zero / 2 positive | AP-002_E1A_SYNTHESIS.json:canonical_observations[5] |
| FIRST_TOUCH (E1B) | positive 21/21 | positive 21/21 | E1B_RESULTS.json:tables.strata[tf].first_touch.horizons[h] |
| FILL (E1B) | positive 20/21 (4h h1 = -0.145 bps, N=141) | positive 21/21 | E1B_RESULTS.json:tables.strata[tf].fill.horizons[h] |

- **does_directional_behavior_materially_change_after_contact**: YES, descriptively: pooled direction-adjusted sign flips from predominantly negative at formation (18/21) to uniformly positive at touch/fill (42/42). Magnitudes at 15s-15m remain 38x-180x smaller than IQR width, so the change is a sign change of a weak central tendency, not the emergence of strong directional behavior.
- **does_it_materially_change_after_fill**: No material change from FIRST_TOUCH to FILL: both anchors show 21/21 positive direction-adjusted medians; fill-anchored medians are equal or modestly larger at most strata (e.g., 1m h5 +0.0741 -> +0.2662 bps; 15m h5 +1.0821 -> +1.6074 bps) with overlapping IQRs.
- **is_stage_conditioning_more_informative_than_formation_alone**: DESCRIPTIVELY YES: formation-only measurement attributes to the zone a negative direction-adjusted central tendency that, on this evidence, is concentrated in the formation->touch approach; conditioning on FIRST_TOUCH/FILL removes it. This is a stage-conditioning observation, NOT 'incremental predictive value' (prohibited).
- **formation_tendencies_weakened_reversed_or_sharpened**: REVERSED (sign) at the pooled level; bullish-cohort positive medians persist and sharpen with horizon; bearish-cohort medians move from negative (formation, per E1A sign summary) to ~zero at 15s/30s and remain negative at 5m/15m/1h h3/h5 - i.e., the reversal is NOT uniform across direction cohorts.
- **confounds_registered**: ['anchor-price definition differs (formation availability vs boundary-crossing tick mid inside the zone)', 'cohort conditioning (formation cohort includes never-touched zones; touch/fill cohorts are post-contact)', 'in-window XAUUSD upward drift (raw medians positive in 41/42 cells) interacts with the bullish share gradient (E1A: 51.0% at 15s -> 60.5% at 4h)']
- **Confounds registered**: anchor-price definition differs (formation availability vs boundary-crossing tick mid inside the zone); cohort conditioning (formation cohort includes never-touched zones; touch/fill cohorts are post-contact); in-window XAUUSD upward drift (raw medians positive in 41/42 cells) interacts with the bullish share gradient (E1A: 51.0% at 15s -> 60.5% at 4h).

Discipline: descriptive only; the term 'incremental predictive value' is prohibited and not used; no significance claims.

## 5. Gap-through analysis

Total gap-through fills (first_touch_observed=false): **1,010 of 379,132 fills (0.266%%)**: 15s 682 (0.301%% of stratum fills), 30s 213 (0.215%%), 1m 81 (0.187%%), 5m 20 (0.287%%), 15m 10 (0.475%%), 1h 4 (0.830%%), 4h 0 (0%%). 67.5%% of all gap-through fills sit at 15s. Citation: E1B_RESULTS.json:tables.strata[tf].gap_through_fill_n; E1B_SCANNER_INTEGRITY_GATE.json:gap_through_fill_n; FILL_TOUCH_CONSISTENCY_AUDIT.json (0 disagreements).

- **Adequacy for descriptive comparison**: 15s n=682 nominally adequate; 30s n=213 and 1m n=81 marginal; 5m n=20, 15m n=10, 1h n=4 inadequate; 4h n=0 impossible. Do not overinterpret coarse-stratum gap-through samples.
- **Behavioral comparison status**: the E1B aggregate tables contain counts only for this stratification - no outcome quantiles by first_touch_observed. A touched-fill vs gap-through post-fill comparison is NOT derivable from this evidence base and must be a later, separately frozen measurement (QG2). None is asserted here.
- E1A anomaly A1 (producer semantics; gap-through is not imputed as a touch) stands; no new anomaly.

## 6. Tail / censoring assessment

| stratum | never-touched | touched-unfilled | max id q (touch->fill) | never-filled | max id q (form->fill) | t2f p99 bars | t2f p99 uncond. identifiable? |
|---|---|---|---|---|---|---|---|
| 15s | 0.537% | 0.210% | 0.9979 | 0.446% | 0.9955 | 3885 | yes |
| 30s | 0.562% | 0.296% | 0.9970 | 0.643% | 0.9936 | 4394 | yes |
| 1m | 0.625% | 0.463% | 0.9954 | 0.899% | 0.9910 | 3582 | yes |
| 5m | 1.415% | 1.108% | 0.9889 | 2.227% | 0.9777 | 2388 | NO |
| 15m | 2.555% | 1.966% | 0.9803 | 4.015% | 0.9599 | 1648 | NO |
| 1h | 4.971% | 3.823% | 0.9618 | 7.839% | 0.9216 | 699 | NO |
| 4h | 5.988% | 10.191% | 0.8981 | 15.569% | 0.8443 | 175 | NO |

- Reported duration quantiles are OBSERVED-EVENT quantiles conditional on the event having occurred (touch->fill n = touched_fill_n; formation->fill n = fill_anchors_n).
- Under administrative right censoring at the development-window end, the unconditional touched-population distribution is identifiable only up to the quantile 1 - censor_fraction: touch->fill up to p99.79 (15s) down to p89.81 (4h); formation->fill up to p99.55 (15s) down to p84.43 (4h).
- Consequently: touch->fill p99 is an unconditional-population-identifiable statistic ONLY at 15s/30s/1m (censor 0.210%/0.296%/0.463% < 1%); the reported p99 at 5m/15m/1h/4h (2,388/1,648/699/175 bars) is conditional-on-fill and must not be quoted as an unconditional population p99.
- formation->fill p95 is unconditionally identifiable through 15m (censor 4.015% < 5%) but NOT at 1h/4h (7.839%/15.569%).

**Challenge to 'rapid body + long tail'.** The E1A description 'rapid body + long tail' remains ADEQUATE but INCOMPLETE. Refinements required by E1B: (1) the body includes a substantial same-bar mode (34.19%-40.43% of touched fills) that formation-anchored durations could not show; (2) the tail is heavier than p95 implies (p99 3,885/4,394/3,582 bars at 15s/30s/1m; observed maxima ~118.5 days market time); (3) at 5m-4h the 'long tail' is increasingly an UNIDENTIFIABLE region (censor ceilings), so the phenotype at coarse strata is 'rapid body + censored tail', not 'rapid body + observed long tail'.

**Maxima are administrative.** Maxima ~10.24e9 ms (~118.5 days) recur across 15s-1h and coincide with the window end boundary: administrative censoring artifacts, not natural endpoints.

## 7. Prospective response assessment

- Cell inventory: 42 anchor x stratum x horizon cells; direction-adjusted p50 positive in 42/42; raw p50 positive in 41/42 (non-positive: [('4h', 'FILL', 1, -0.14514297187368858)]).
- Magnitude discipline: h1 |median|/IQR-width spans 0.56%-15.16% across all strata and 0.56%-2.64% within 15s-15m (median roughly 38x-180x smaller than IQR width at 15s-15m): WEAK central tendency, not strong directional behavior. Dispersion grows with horizon and bar size (e.g., 4h FIRST_TOUCH h5 adj p99 483.5 bps, max 929.5 bps).
- Sign consistency: pooled - consistent across horizons and strata (42/42); bullish cohort positive 42/42 (confounded with in-window upward drift); bearish cohort NOT consistent (FIRST_TOUCH 5 pos / 5 zero / 11 neg; FILL 10 pos / 2 zero / 9 neg).
- End-of-window missing N: explicit per cell; max 3 of 2,133 (15m h3/h5); isolated 1s elsewhere; all other cells 0.
- Post-fill characterization: Post-fill behavior is descriptively most consistent with WEAK zone-direction drift at fine strata and NO CLEAR CENTRAL TENDENCY at mid strata for bearish zones; 'neutralization' cannot be distinguished from 'no clear central tendency' without a pre-event baseline anchor, which E1B does not provide. None of these labels is a trading claim.

## 8. Development-derived split assessment (gap_atr, fvg_quality median splits)

**Status: development-derived descriptive partitions only - NOT pre-registered thresholds, NOT decision thresholds, NOT confirmed cutoffs, NOT strategy rules.**

- gap_atr: differences are 0.001-0.05 bps scale and change sign across horizons (15s FT: h1 low>high, h3 low>high, h5 high>low; 1m FT h1 low>high); sign flips within anchor across horizons (5m FT h1 high>low +0.2122 vs +0.0960, h3 low>high +0.3475 vs +0.0673); largest contrasts exactly where N is smallest (1h FT h1: low -1.6039 vs high +6.0915, n=249/248; 1h FILL h1: +0.0637 vs +4.6654, n=241/241). Verdict: NO stable descriptive pattern; register as noise-prone development partition (QG8).
- fvg_quality: near-degenerate differences (15s FT h1: +0.0123 vs +0.0148); 5m FT h1 high>low (+0.3046 vs +0.0429) collapses by h3 (+0.2353 vs +0.2193); 15m FT h1 reverses (low +0.2670 > high +0.0223); 1h FT h1 high +6.6475 vs low -0.9150 (n=248/249); 4h FILL h1 reverses (low +12.7375 vs high -4.7588, n=71/70). Verdict: NO stable descriptive pattern; sign instability across horizons, anchors, and strata.
- Cross-cutting: The sign of (high - low) flips between h1 and h5 in 6 (stratum, anchor, split) combinations; split thresholds differ between FIRST_TOUCH and FILL cohorts (medians recomputed within each anchor cohort), reinforcing that these are descriptive partitions, not fixed cutpoints.

## 9. Anomalies or suspicious patterns

- **AN1 [BENIGN_SERIALIZATION_ARTIFACT]** Direction-adjusted bearish medians serialized as -0.0 (15s/30s FIRST_TOUCH h1-h3; 30s FILL h1 bearish = -2.1e-12 bps). -> Exact-zero medians with float sign-bit/epsilon artifacts (same class as the E1A-noted -0.0). Substantive consequence: at 15s/30s the 1-bar post-touch distribution carries massive exact-zero mass (boundary-tick mid to next close often unchanged), so the median is a blunt summary there; use IQR/tails.
- **AN2 [FLAGGED_SMALL_N_SIGN_SENSITIVITY]** 4h FILL h1 raw p50 = -0.1451 bps vs direction-adjusted p50 = +8.3089 bps (N=141; bullish 84 / bearish 57). -> At this N the pooled median's sign is sensitive to the direction adjustment and cohort mix; neither sign should be quoted alone.
- **AN3 [ADMINISTRATIVE_CENSORING_ARTIFACT]** Near-identical maxima (~10.241e9 ms ~ 118.5 days) for formation->touch, touch->fill, and formation->fill across 15s-1h. -> Extremes pinned at the development-window end boundary; the max is administrative, not a natural lifecycle endpoint.
- **AN4 [EXPECTED_NOISE]** Non-monotone pooled medians across horizons in small cells (1h FILL +2.0200/+1.7706/+8.0925; 4h FILL +8.3089/+18.1627/+15.7234; 5m FILL h3 +0.4032 > h5 +0.1905). -> Within-dispersion wiggle; no horizon-trend claim is made at these N.
- **AN5 [EXPECTED_NOISE]** Largest split contrasts occur where N is smallest (1h/4h split cells, n~240-250 per arm). -> Small-N instability; no pattern registered.
- **AN6 [OBSERVED_ONLY]** Same-bar touch+fill share mildly rises from 34.19% (15m) to 40.43% (4h, 57/141). -> Not interpretable at 4h given N; no scale law claimed.
- **AN7 [CLEAN]** End-of-window missing outcomes present and explicit (max 3 of 2,133 at 15m h3/h5; isolated 1s elsewhere); integrity gate PASS 5/5; fill/touch audit 0 disagreements; preflight 0 regressions over 51,969,691 rows; reproducibility hashes identical. -> No defect.

## 10. Question Generator candidates (registered, NOT tested)

- **QG1.** Does the post-first-touch direction-adjusted response differ from an appropriately matched formation-stage control (matched on stratum, direction, gap_atr, fvg_quality, calendar time) under a drift-aware null? _Motivation_: Formation 18/21 negative vs touch/fill 42/42 positive direction-adjusted medians (I2).. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG2.** Does gap-through fill have different post-fill behavior from ordinary touch->fill, at strata with adequate gap-through N (15s n=682, 30s n=213, 1m n=81) under a pre-registered minimum-N rule? _Motivation_: E1B tables carry counts only; no outcome comparison exists.. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG3.** Does the same-bar touch+fill subpopulation (~34-40% of touched fills) behave differently after fill than multi-bar touch->fill? _Motivation_: The FILL anchor pools two lifecycle modes.. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG4.** Does FVG quality modify touch->fill duration after controlling for native scale, under censor-aware estimation? _Motivation_: Descriptive splits are sign-unstable; duration modification is untested.. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG5.** Is the bullish/bearish post-touch asymmetry at 5m/15m/1h (bearish adjusted p50 negative at h3/h5) instrument drift or FVG-specific (drift-matched time-control null)? _Motivation_: Bearish cohort sign inconsistency (D7).. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG6.** Is the touch->fill duration distribution scale-consistent when expressed in native bars vs market hours, once administrative censoring is modeled? _Motivation_: Bar-scale tails shrink with scale while censoring grows.. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG7.** Do the never-touched and never-filled fractions continue rising with native scale beyond the development window (censoring artifact vs lifetime property)? _Motivation_: Monotone censoring gradient 0.210% -> 10.191% (touched-unfilled).. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.
- **QG8.** Does gap_atr modify post-fill response at coarse strata (1h FILL h1 high +4.6654 vs low +0.0637 bps, n=241/241), or is this small-N noise? _Motivation_: Largest split contrast at smallest N.. _Status_: REGISTERED ONLY - not tested in E1B; requires its own frozen measurement contract before any test.

## 11. E2 candidates that must remain untested

- Any conversion of QG1-QG8 into executed tests without a frozen, director-approved E2 measurement contract (denominators, null model, censoring treatment fixed in advance).
- Any threshold, cutoff, or quantile-surface selection on gap_atr, fvg_quality, touch->fill duration, or post-anchor returns (the E1B median splits are descriptive partitions only).
- Any conditional-expectancy, sign-persistence, or hit-rate estimation on the E1B anchor populations.
- Any cross-stratum combination, ranking, or voting rule (cross-scale object identity PROHIBITED; no pooling).
- Any use of E1B DEVELOPMENT-window quantities for out-of-sample, confirmation, or live purposes (confirmation LOCKED; E2/E3 NOT STARTED).

## 12. Must-not-claim list

- No alpha/edge/profitability/strategy/signal/predictive-advantage claim: E1B is a locked phenotype measurement; nothing was tested.
- No significance, confidence, or confirmation claims; no null model was run anywhere in this review.
- No cross-scale zone identity; no pooling of strata; cross-stratum statements are side-by-side descriptive comparisons only.
- Duration quantiles are conditional-on-fill observed-event quantiles, not unconditional population quantiles.
- Unfilled fractions (never-touched, touched-unfilled, never-filled) are window-censored accounting complements, not lifetime fill probabilities or hazards.
- Do not quote touch->fill p99 at 5m/15m/1h/4h, or formation->fill p95/p99 at 1h/4h, as unconditional population quantiles (censor ceilings: p97.77/p95.99/p92.16/p84.43).
- Observed maxima (~118.5 days) are administrative window-boundary artifacts, not natural lifecycle endpoints.
- gap_atr and fvg_quality median splits are development-derived descriptive partitions - not pre-registered thresholds, decision thresholds, confirmed cutoffs, or strategy rules.
- Positive direction-adjusted medians after touch/fill are not a tradable continuation or reversal effect; instrument drift is an unresolved confound.
- The formation->touch/fill sign flip is a descriptive stage comparison; calling it 'incremental predictive value' is PROHIBITED.
- 1h/4h cells (N=497/157 touches, 482/141 fills) are small-sample descriptions, not established scale phenomena.
- Serialized -0.0 bearish medians are exact-zero medians (sign-bit artifacts), not negative findings.
- Gap-through cohorts support no behavioral comparison at 5m/15m/1h/4h (n=20/10/4/0); none was performed here.
- Nothing in this review transfers to E2, E3, confirmation, or any live context.

## 13. Recommended next scientific action

**PASS_TO_SYNTHESIS: advance to the E1B multi-interpreter synthesis and reconciliation.**
- Reconcile this review's per-stratum tables and sign counts against peer E1B interpretations at synthesis.
- Freeze the Question Generator register (QG1-QG8) with proposed null models, cohort definitions, and censoring treatment; none may be executed under E1.
- Flag to the director: (a) the touched-fill vs gap-through and same-bar vs multi-bar post-fill comparisons require a separate frozen measurement contract (E1B aggregate tables contain counts only); (b) any drift-control design must be fixed before E2 exposure; (c) coarse-stratum tail claims are bound by the censor ceilings and any successor design should state identifiability limits up front.
- Explicitly NOT recommended: No threshold selection, no optimization, no testing, no strategy construction; confirmation remains LOCKED..

## Appendix A. Evidence base

- docs/research/ap-002-e1b/AP-002_E1B_EXPERIMENT_CONTRACT.json
- docs/research/ap-002-e1b/E1B_EXPERIMENT.json
- docs/research/ap-002-e1b/E1B_RESULTS.json
- docs/research/ap-002-e1b/PREFLIGHT.json
- docs/research/ap-002-e1b/RUN_MANIFEST.json
- docs/research/ap-002-e1b/E1B_SCANNER_INTEGRITY_GATE.json
- docs/research/ap-002-e1b/REPRODUCIBILITY.json
- docs/research/ap-002-e1b/FILL_TOUCH_CONSISTENCY_AUDIT.json
- docs/research/ap-002-e1a-scope-repair/AP-002_E1A_SYNTHESIS.json

Reconciliation verified in this review: formations 381,498 (= E1A zones); FIRST_TOUCH anchors 379,248; FILL anchors 379,132; touched-fills 378,122; gap-through 1,010; orphans 178/358 (all matching E1B_SCANNER_INTEGRITY_GATE.json, FILL_TOUCH_CONSISTENCY_AUDIT.json, REPRODUCIBILITY.json, AP-002_E1A_SYNTHESIS.json); censor-fraction formulas verified to 1e-9 for all 7 strata; reproducibility: two fresh runs, identical logical result hash.

