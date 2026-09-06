# AP-002 E1B PHENOTYPE REVIEW — P-03

- **Interpreter:** P-03 (independent phenotype interpreter; scanner not built by me; E1A interpretation not written by me; no confirmation role; no strategy construction; no peer E1B interpretation inspected)
- **Generated (UTC):** 2026-09-06T13:05:27Z
- **Program / Stage / Instrument / Exposure:** AP-002 FVG / E1B lifecycle-stage phenotype / XAUUSD / E1_PHENOTYPE
- **Active logical result hash:** `2a7a142cadb36449d14d4128361d11ce99a3a4d7d8a004dd539bc4dc32fb823c` (verified against E1B_RESULTS.json, RUN_MANIFEST.json output_identity, and REPRODUCIBILITY.json run_1 = run_2)
- **Native strata:** 15s, 30s, 1m, 5m, 15m, 1h, 4h — each treated independently; no pooling; cross-scale object identity PROHIBITED and respected
- **Development window:** 2025-07-31T16:15:00Z (inclusive) to 2026-05-05T12:39:00Z (exclusive)
- **Confirmation:** LOCKED. **E2/E3:** NOT STARTED.
- **Overall verdict: PASS_TO_SYNTHESIS**

## Evidence integrity context

All quantitative statements below cite E1B_RESULTS.json (`tables.strata.rows[tf]...`; horizons indexed 0/1/2 = 1/3/5 bars). Integrity chain: preflight 51,969,691 rows, 0 regressions (PREFLIGHT.json); integrity gate PASS 5/5 with 758,380 anchors = 379,248 FIRST_TOUCH + 379,132 FILL, 1,010 gap-through (E1B_SCANNER_INTEGRITY_GATE.json); fill-touch audit 0 disagreements across 379,132 fill anchors (FILL_TOUCH_CONSISTENCY_AUDIT.json); two independent runs with identical logical hash and anchor hash (REPRODUCIBILITY.json). Stratum sums reconcile with the frozen E1A synthesis exactly: formations 381,498; orphans 178 touch / 358 fill.

**Reading conventions.** Native-bar counts are the interpreter's arithmetic (market_ms / bar duration) on committed values, marked *derived*. Positive `direction_adjusted_return_bps` is read as movement in the zone's registered direction — an INTERPRETATION derived from the joint pattern (formation-anchored medians predominantly negative while zones fill from the far side within a median of 2 bars; touch/fill-anchored medians uniformly positive), not re-derived from scanner code. Bearish medians serialized `-0.0` are sign-bit artifacts of exact zeros (same class as E1A's recorded artifact) and are read as zero. Every conclusion is classed DIRECT OBSERVATION / INTERPRETATION / HYPOTHESIS FOR LATER TESTING.

---

## 1. Executive phenotype summary

The FVG lifecycle body is extremely fast: median formation→touch is 2 native bars and median touch→fill is 1 native bar at **all seven strata**, with 34.2–40.4% of touched fills resolving on the same bar. A scale-growing minority of touched zones never fills inside the window (touch→fill right-censor 0.21% at 15s → 10.19% at 4h), so unconditional tail quantiles are identifiable only up to p99.8/p99.7/p99.5/p98.9/p98.0/p96.2/p89.8 (15s→4h). Post-anchor direction-adjusted medians are positive in **21/21 cells at FIRST_TOUCH and 21/21 at FILL** — a descriptive sign flip versus the formation anchor (E1A: 18 negative / 1 zero / 2 positive of 21) — but magnitudes are tiny relative to dispersion at fine strata (15s FT h1 median +0.014 bps vs p05–p95 span 6.53 bps, n=226,686) and small-N at coarse strata (4h FILL n=141). The positive lean is carried by the **bullish** direction (bearish medians ≈ 0 or negative at most cells). Gap-through fills (1,010 total; 0.19–0.83% of fills per stratum; 0 at 4h) are too few — and not reported per-population in the committed aggregates — to support any post-fill comparison. Development median splits on gap_atr/fvg_quality show no scale-consistent gradient. All of this is phenotype description; nothing is tested, nothing is confirmed.

## 2. FIRST_TOUCH per-stratum observations

Anchor = first actual emitted `fvg_first_touch` on lawful in-window formations; anchor price = boundary-crossing tick mid; gap-through fills excluded, never imputed (contract `first_touch_anchor`).

| TF | Formations | Touch anchors | Incidence | Same-bar share of touched fills | t2f censor | FT dir-adj p50 bps h1/h3/h5 | End-of-window missing h1/h3/h5 |
|----|-----------|---------------|-----------|--------------------------------|------------|------------------------------|-------------------------------|
| 15s | 227,909 | 226,686 | 0.9946 | 35.3% (79,865/226,210) | 0.210% | +0.014 / +0.030 / +0.036 | 0/0/0 |
| 30s | 99,866 | 99,305 | 0.9944 | 35.4% (35,055/99,011) | 0.296% | +0.014 / +0.048 / +0.044 | 0/0/0 |
| 1m | 43,701 | 43,428 | 0.9938 | 35.4% (15,286/43,227) | 0.463% | +0.065 / +0.076 / +0.074 | 0/0/1 |
| 5m | 7,140 | 7,039 | 0.9859 | 34.4% (2,397/6,961) | 1.108% | +0.145 / +0.235 / +0.169 | 1/1/1 |
| 15m | 2,192 | 2,136 | 0.9745 | 34.2% (716/2,094) | 1.966% | +0.117 / +0.993 / +1.082 | 0/3/3 |
| 1h | 523 | 497 | 0.9503 | 36.6% (175/478) | 3.823% | +1.408 / +2.710 / +5.168 | 0/0/0 |
| 4h | 167 | 157 | 0.9401 | 40.4% (57/141) | 10.191% | +10.676 / +9.375 / +13.267 | 0/0/0 |

**Touch→fill duration (derived from `touch_to_fill_market_ms`, touched fills):** p25 = 0 and p50 = 1 native bar at every stratum; p75 = 4–9 bars (15s 8, 30s 7, 1m 7, 5m 8, 15m 9, 1h 8, 4h 4); p95 = 204/188/181/209/226/152/66 bars (15s→4h); observed-event p99 = 3,885/4,394/3,582/2,388/1,648/699/175 bars. In wall-clock terms p95 runs from ~51 minutes (15s) to ~11 days (4h).

**Formation→touch duration (derived):** p50 = 2 bars at every stratum (reconciles with E1A); p95 = 190/184/185/158/186/193/89 bars; observed-event p99 = 3,495/3,619/3,630/2,173/1,238/742/312 bars; never-touched fraction (= 1 − incidence) = 0.54%/0.56%/0.63%/1.41%/2.55%/4.97%/5.99%.

**Key per-stratum readings:**

- **15s (n=226,686):** h1 direction-adjusted p50 +0.014 bps against IQR [−0.818, +0.870], p05–p95 [−3.22, +3.32] — weak central tendency. Bearish split p50 = −0.0 at all horizons (n=111,069); pooled positive is entirely bullish-carried (n=115,617: +0.027/+0.059/+0.073).
- **30s (n=99,305):** medians positive h1/h3, slightly lower at h5 (+0.048→+0.044); bearish −0.0/−0.0/−0.030 (n=48,006).
- **1m (n=43,428):** bearish split turns negative with horizon (+0.031→−0.012→−0.064, n=20,669) while bullish grows (+0.098→+0.194, n=22,759). Both development splits run low>high at all horizons here — a pattern that does not replicate elsewhere.
- **5m (n=7,038–7,039):** sign-opposite direction splits: bullish +0.30/+0.68/+0.98 (n=3,806) vs bearish −0.038/−0.207/−0.756 (n=3,232). One anchor lacks all three horizons.
- **15m (n=2,136; h3/h5 n=2,133):** medians jump ~10× from h1 to h3 (+0.117→+0.993); direction splits sign-opposite at all horizons (+0.63/+2.41/+2.93 vs −0.36/−0.93/−1.52).
- **1h (n=497):** medians grow monotonically (+1.41/+2.71/+5.17) but h1 IQR is [−15.93, +19.47] bps; bearish flips to −3.64/−4.37 at h3/h5 (n=209).
- **4h (n=157):** largest medians (+10.7/+9.4/+13.3 bps) with h1 IQR [−25.27, +45.13], p05 −92.05, p95 +237.60 — still weak relative to dispersion; the only stratum with clearly positive bearish medians (+4.5/+1.4/+7.7, n=63); gap_atr split runs low>high at all horizons (unique).

**Answers to the FIRST_TOUCH questions (compact):**
1. *Speed to fill after genuine touch:* median 1 native bar everywhere; p75 4–9 bars; body resolves within 0–1 bars for >50% of touched zones.
2. *Unfilled/right-censored fraction after touch:* 0.21% (15s) → 0.30% (30s) → 0.46% (1m) → 1.11% (5m) → 1.97% (15m) → 3.82% (1h) → 10.19% (4h) — DIRECT OBSERVATION (`touch_to_fill_right_censor_fraction`).
3. *Distribution shape:* extreme body mass at 0–1 bars, p75 4–9 bars, then a multi-month tail (observed-event maxima ~118.5 days at fine strata, 76.5 days at 4h).
4. *Same-bar touch+fill:* yes, a substantial mode at every stratum (34.2–40.4%).
5. *Very rapid resolution:* yes. *Persistent touched-but-unfilled zones:* yes, but a small and scale-growing fraction (max durations 459–682,739 native bars). *Scale dependence:* incidence and censoring degrade monotonically with scale; body shape (p50=1 bar) is scale-invariant in bar units.
6. *Prospective 1/3/5-bar behavior:* direction-adjusted medians positive 21/21, growing with horizon in most strata; missing N negligible (0–3 per cell). Raw medians positive 21/21 at FT.
7. *Direction:* bullish positive and growing everywhere; bearish ≈0/negative almost everywhere (asymmetric composition). *Splits:* no stable gap_atr or fvg_quality gradient at FIRST_TOUCH.

## 3. FILL per-stratum observations

Anchor = lawful `fvg_filled` events; the committed prospective aggregates pool first_touch_observed true/false (see §5).

| TF | Fill anchors | Gap-through | F2F p50 (bars, derived) | Fill censor | FILL dir-adj p50 bps h1/h3/h5 |
|----|--------------|-------------|--------------------------|-------------|-------------------------------|
| 15s | 226,892 | 682 | 7 | 0.446% | +0.021 / +0.066 / +0.085 |
| 30s | 99,224 | 213 | 7 | 0.643% | +0.030 / +0.109 / +0.162 |
| 1m | 43,308 | 81 | 7 | 0.899% | +0.104 / +0.190 / +0.266 |
| 5m | 6,981 | 20 | 7 | 2.227% | +0.176 / +0.403 / +0.190 |
| 15m | 2,104 | 10 | 8 | 4.015% | +0.118 / +1.111 / +1.607 |
| 1h | 482 | 4 | 8 | 7.839% | +2.020 / +1.771 / +8.092 |
| 4h | 141 | 0 | 5 | 15.569% | +8.309 / +18.163 / +15.723 |

**Key per-stratum readings:**

- **15s:** post-fill medians exceed post-touch at every horizon; both splits run high>low at h3/h5. Formation→fill p50 = 7 bars, matching E1A.
- **30s:** monotone growth with horizon; both splits high≥low at all horizons (fvg_quality h1 an exact tie). h5 missing 1 outcome.
- **1m:** both directions positive at all horizons after fill (bullish +0.18/+0.32/+0.43, n=22,673; bearish +0.04/+0.06/+0.07, n=20,635) — unlike FIRST_TOUCH. Cleanest split ordering in the dataset (both splits high>low, all horizons), still development-partition-only.
- **5m:** direction asymmetry persists and strengthens (bullish +0.37/+0.97/+1.21 vs bearish −0.02/−0.10/−0.97); the pooled h5 dip (0.403→0.190) is a bearish-side composition effect.
- **15m:** h1 median (+0.118, n=2,103) far below h3/h5 (+1.111/+1.607) — response concentrated at h3–h5; sign-opposite direction splits; missing 1/2/2.
- **1h:** fvg_quality split high>low at all horizons with the low half near zero or negative (−0.07/−2.18/+0.24 vs +6.48/+4.20/+10.92, ~241/cell) — strongest split separation anywhere, entirely hypothesis-grade; pooled h3 dip (2.020→1.771) then h5 jump (8.092); censoring 7.84% caps identifiability at p92.2.
- **4h:** raw h1 median **negative** (−0.145 bps) vs direction-adjusted **+8.309** (n=141) — raw/direction-adjusted sign divergence from sign-pooling; 15.57% censoring caps identifiability at p84.4; zero gap-through fills (cohort is 100% touched-fill).

## 4. Formation vs touch vs fill — descriptive stage comparison

- **Sign-class flip (DIRECT OBSERVATION):** FORMATION direction-adjusted medians 18 negative / 1 zero / 2 positive of 21 cells (E1A synthesis) → FIRST_TOUCH 21/21 positive → FILL 21/21 positive.
- **INTERPRETATION:** stage conditioning materially changes the descriptive picture. At formation the central tendency is movement against the zone's registered direction (drift into the zone — which is why zones touch within a median of 2 bars and fill within 1–8); after touch and after fill the central tendency is movement back in the registered direction. The formation-stage apparent tendency is **reversed**, not merely weakened, at later anchors — subject to the sign-convention note and to selection/clock caveats (touched/filled populations are selected subsets; anchor clocks differ).
- **Sharpening with stage (INTERPRETATION):** FILL medians exceed FIRST_TOUCH medians in 19/21 cells (exceptions 1h h3, 4h h1); e.g. 1m h5 +0.074 (FT) → +0.266 (FILL); 15m h5 +1.082 → +1.607; 4h h5 +13.267 → +15.723. Dispersion grows too — stage conditioning re-centers the distribution, it does not visibly concentrate it (FILL h1 IQRs wider than FT h1 IQRs at fine strata). This is descriptive sharpening with stage progression, **not** incremental predictive value.
- **Cross-stage reconciliation (DIRECT OBSERVATION):** formation→fill p50 = 7/7/7/7/8/8/5 bars and formation→touch p50 = 2 bars reproduce the E1A figures exactly.
- **Is stage conditioning more informative than formation alone? Descriptively yes** — the sign flip and the bullish-carried direction asymmetry are only visible once lifecycle stage is conditioned on. This is a statement about descriptive resolution. Do NOT call it incremental predictive value; that belongs to later testing.

## 5. Gap-through analysis

- **Counts (DIRECT OBSERVATION):** 1,010 total — 682 (15s), 213 (30s), 81 (1m), 20 (5m), 10 (15m), 4 (1h), 0 (4h); 0.19–0.83% of per-stratum fill anchors; never imputed as touch; excluded from the FIRST_TOUCH cohort (E1A anomaly A1, resolved as expected producer semantics).
- **Adequacy for descriptive comparison (INTERPRETATION): NOT SUPPORTED at any stratum.** The committed FILL tables do not split outcomes by `first_touch_observed`, and counts at 5m/15m/1h (20/10/4) are an order of magnitude below any defensible descriptive sample; 4h is empty. Absolute counts concentrate at 15s (682 of 1,010). Aggregate FILL statistics carry a 0.19–0.83% gap-through mixture whose influence on quantiles is expected to be negligible but is unverified.
- **HYPOTHESIS FOR LATER TESTING:** does gap-through fill have different post-fill behavior from ordinary touch→fill? Requires a per-population outcome split in a future measurement contract. Not testable from committed aggregates; not tested here.

## 6. Tail / censoring assessment

**Committed ceilings (DIRECT OBSERVATION, `*_right_censor_fraction` / `*_max_identifiable_quantile`):**

| TF | Fill censor | Fill max identifiable q | Touch→fill censor | Touch→fill max identifiable q |
|----|-------------|--------------------------|--------------------|-------------------------------|
| 15s | 0.446% | 0.9955 | 0.210% | 0.9979 |
| 30s | 0.643% | 0.9936 | 0.296% | 0.9970 |
| 1m | 0.899% | 0.9910 | 0.463% | 0.9954 |
| 5m | 2.227% | 0.9777 | 1.108% | 0.9889 |
| 15m | 4.015% | 0.9599 | 1.966% | 0.9803 |
| 1h | 7.839% | 0.9216 | 3.823% | 0.9618 |
| 4h | 15.569% | 0.8443 | 10.191% | 0.8981 |

- **Observed-event vs unconditional (DIRECT OBSERVATION):** touch→fill unconditional p99 is identifiable **only at 15s/30s/1m**. At 5m/15m/1h/4h the committed p99 values (2,388/1,648/699/175 native bars) are observed-event (filled-zone) quantiles; 1.1%/2.0%/3.8%/10.2% of touched zones never filled in-window. Same for formation→fill (identifiable p99 only at 15s/30s/1m). Derived: never-touched fraction caps formation→touch identifiability at the touch-incidence values (p99 identifiable only at 15s/30s/1m).
- **Observed-event maxima (touch→fill):** 682,739 native bars at 15s (~118.5 days), 341,358 (30s), 170,679 (1m), 34,135 (5m), 11,257 (15m), 2,831 (1h), 459 (4h) (~76.5 days). Max/p99 ratios are extreme (15s ≈ 176×).
- **Challenge to "rapid body + long tail" (INTERPRETATION):** the label **survives** for the touch→fill lifecycle at fine/mid strata and for the body everywhere: p25=0, p50=1 bar, 34–40% same-bar, p75 4–9 bars, p95 66–226 bars, observed-event p99 175–4,394 bars at fine strata. Required refinements: (1) at 5m–4h the "long tail" is administratively truncated — unconditional quantiles beyond p98.9/p98.0/p96.2/p89.8 are unidentifiable, so no unconditional p99 exists beyond 1m and none may be quoted as observed; (2) "long" must be quantified per stratum in native bars (wall-clock p95 scales from ~51 minutes to ~11 days); (3) the label describes duration-to-fill only, not post-anchor response.
- **Do not pretend:** right-censored lifetimes are not completed fills; coarse-scale fill fractions are window-censored lower bounds (E1A must-not-claim reaffirmed).

## 7. Prospective response assessment

- **Sign census (DIRECT OBSERVATION):** direction-adjusted p50 positive 21/21 cells at FIRST_TOUCH and 21/21 at FILL; raw p50 positive 21/21 at FT and 20/21 at FILL (4h FILL h1 raw −0.145 bps, n=141). Sign consistency across horizons holds in all seven strata at both anchors; magnitude is non-monotone in horizon in several cells (30s FT h5, 5m FT h5, 5m FILL h5, 1h FILL h3, 4h FT h3).
- **Magnitude vs dispersion (DIRECT OBSERVATION):** fine-strata medians are 0.2–3% of the p05–p95 span (15s FT h1: 0.014 vs 6.53 bps; 1m FT h1: 0.065 vs 13.76 bps) — weak central tendency. Coarse-strata medians are proportionally larger but rest on n=141–497 with IQRs tens of bps wide (4h FILL h3 +18.16 vs h1 IQR [−36.13, +41.03]).
- **Direction asymmetry (INTERPRETATION):** bullish medians positive and growing with horizon at every stratum; bearish ≈0/negative at most cells (15s FT −0.0/−0.0/−0.0, n=111,069; 5m FILL −0.02/−0.10/−0.97, n=3,214; 1h FILL h3/h5 −3.06/−4.89, n=203). Only 4h shows clearly positive bearish medians at FIRST_TOUCH (+4.5/+1.4/+7.7, n=63). Sign consistency across directions is NOT established; the positive lean is a bullish-side structure.
- **Post-fill characterization (INTERPRETATION):** positive direction-adjusted medians growing with horizon are most consistent with movement in the zone's registered direction after structural fill — *continuation of the gap-direction leg*, equivalently *reversion of the filling move* (same committed quantity, two reference frames). Neutralization fits only in the trivial sense that all fine-strata medians are near zero relative to dispersion; it does not fit coarse strata. "No clear central tendency" would overstate the null case given 21/21 sign consistency at both anchors; "strong directional behavior" would overstate 0.014-bps medians against 6.5-bps spans. Defensible description: **sign-consistent, magnitude-weak, bullish-carried in-zone-direction lean after touch and after fill, strengthening with horizon and native scale.** These labels are descriptive leanings, not trading claims.
- **Raw vs direction-adjusted:** raw medians carry the development-window upward drift and direction mixing; the direction-adjusted field is the appropriate within-zone description. The bullish-positive / bearish-zero asymmetry is structure beyond uniform drift (uniform drift would make bearish raw positive and hence bearish direction-adjusted negative, not zero).

## 8. Development-derived split assessment

gap_atr and fvg_quality median splits are **DEVELOPMENT-derived descriptive partitions** — not pre-registered thresholds, not decision thresholds, not confirmed cutoffs, not strategy rules.

- FIRST_TOUCH gap_atr: low>high at 1m and 4h (all horizons); high>low at 1h (all horizons); mixed at 15s/30s/5m/15m. No cross-stratum consistency.
- FIRST_TOUCH fvg_quality: low>high at 1m and 15m; sign flips across horizons within strata (4h: h1 low>high, h3 high>low, h5 low>high).
- FILL gap_atr: high>low at 15s h3/h5, 30s, 1m, 5m (all horizons); mixed at 15m/1h; low>high at 4h h1/h3.
- FILL fvg_quality: high>low at 15s/30s/1m/5m/1h (all horizons); mixed at 15m and 4h (4h h1: low +12.74 vs high −4.76, n=71/70).
- **Assessment (INTERPRETATION):** no partition exhibits a stable, scale-consistent gradient; locally stable orderings sit at mid-size strata and the largest separations occur in the smallest cells (1h ~241–249, 4h ~70–79 per cell) — precisely where median-split differences are least trustworthy. The splits generate hypotheses ("does fvg_quality modify post-fill response?"), not findings.

## 9. Anomalies or suspicious patterns

| ID | Severity | Class | Finding |
|----|----------|-------|---------|
| AN-1 | cosmetic | DIRECT OBSERVATION | Bearish direction-adjusted p50 serialized −0.0 in multiple cells (15s FT h1/h3/h5; 30s FT h1/h3; 15s FILL h1; 30s FILL h1 = −2.1e-12). Known sign-bit artifact of exact-zero medians; read as zero; recommend canonicalizing −0.0 → 0.0 in future artifacts. |
| AN-2 | interpretive-hazard | DIRECT OBSERVATION | Raw vs direction-adjusted sign divergence at 4h FILL h1 (−0.145 vs +8.309 bps, n=141) and magnitude divergence at 4h FT h5 (+1.106 vs +13.267, n=157). Expected under sign-pooling with unbalanced directions; coarse-strata raw medians will mislead future readers. |
| AN-3 | low | DIRECT OBSERVATION | Non-monotone maxima across horizons at fixed N (15s FT raw-delta max 77.89 at h3 vs 65.37 at h5, both n=226,686, missing 0): genuine property of differing window endpoints, not a defect; do not assume monotonicity. |
| AN-4 | low | DIRECT OBSERVATION | Bullish anchors outnumber bearish at every stratum (15s FT 115,617 vs 111,069; 4h FT 94 vs 63), consistent with the E1A bullish-share gradient; carries into every pooled median. |
| AN-5 | scope-limitation | DIRECT OBSERVATION | Contract permits `first_touch_observed` conditioning at FILL, but committed prospective aggregates omit the true/false outcome split; the gap-through post-fill question is unanswerable from this evidence base. Fill-touch audit shows perfect true/false payload agreement, so this is a reporting-scope gap, not a scanner defect. |
| AN-6 | info | DIRECT OBSERVATION | Repeated duration maxima near 1.024e10 ms (~118.5 days) at 15s/30s/1m across all three duration variables: long-lived zones formed early resolving late; not window-clipped (window ≈ 277.9 days); consistent across strata. |

## 10. Question Generator candidates

Registered only — not tested here:

1. Does post-first-touch response differ from an appropriately matched formation-stage control (same zones, matched clock), separating stage conditioning from population selection?
2. Does gap-through fill have different post-fill behavior from ordinary touch→fill? (requires per-population outcome split; currently unmeasured)
3. Does FVG quality modify touch→fill duration after controlling for native scale?
4. Is the bullish-positive / bearish-zero direction-adjusted asymmetry at FIRST_TOUCH and FILL robust to a formal drift-aware null?
5. Does gap_atr modify touch→fill duration?
6. Do same-bar touch+fill zones (34–40% of touched fills) differ in post-fill response from multi-bar touch→fill zones (two candidate lifecycle modes)?
7. Is the 4h bearish-positive exception at FIRST_TOUCH (+4.5/+1.4/+7.7 bps, n=63) stable or small-sample noise?
8. Does touch→fill duration vary with session/time-of-day (links to E1A gap-through vs session-gap-density candidate)?
9. Are the 1h FILL fvg_quality split separations reproducible at matched N on out-of-window data?

## 11. E2 candidates that must remain untested

- All nine Question Generator candidates: no test, no threshold fitting, no optimization, no out-of-sample probing in E1B.
- Any conversion of the direction-adjusted positive lean into an entry/exit rule or filter.
- Any use of development median-split values (gap_atr ≈ 0.44–0.56; fvg_quality ≈ 0.74–0.80, varying by stratum and anchor) as trading cutoffs.
- Any cross-scale stacking or identity construction (PROHIBITED).
- Any E2 design work beyond registering candidates — E2 is NOT STARTED; confirmation is LOCKED.

## 12. Must-not-claim list

- No alpha / edge / profitability / strategy / signal / predictive-advantage claims: nothing was tested; confirmation LOCKED; E2/E3 NOT STARTED.
- No significance or confirmation claims of any kind.
- No cross-scale identity; no pooling of strata; strata are not interchangeable.
- Direction-adjusted sign patterns are not tradable reversal/continuation effects; no null test was run.
- Continuation/reversion/neutralization labels are descriptive leanings under a derived sign convention, not established behavior.
- The stage comparison is not incremental predictive value.
- Observed-event p99/max at 5m/15m/1h/4h are not unconditional population quantiles; no unconditional p99 beyond 1m.
- Touch incidence, fill fractions, censor fractions are window-censored accounting quantities, not lifetime probabilities or hazards.
- Development median splits are not thresholds, cutoffs, or confirmed gap/quality effects.
- 1h/4h split separations and 4h bearish-positive medians are small-sample medians, not established effects.
- Gap-through post-fill behavior was NOT characterized; no claim about gap-through zones' post-fill response.
- Bearish −0.0 cells are sign-bit artifacts of exact zeros, not negative tendencies.
- Horizons are native-bar-relative; no exposure-matched cross-stratum magnitude comparison.
- Nothing here transfers to E2, E3, confirmation, or live use.

## 13. Recommended next scientific action

**Verdict: PASS_TO_SYNTHESIS.** Proceed to the E1B synthesis stage (peer-interpretation reconciliation), carrying: (1) the stage sign-flip as the headline descriptive result; (2) the bullish-carried direction asymmetry; (3) the censoring-ceiling table as the binding constraint on all tail language; (4) the gap-through reporting gap and the nine Question Generator candidates as registered items. If the director elects a measurement amendment before E2, the highest-value additions are (a) per-population FILL outcomes stratified by first_touch_observed and (b) a matched formation-stage control for the touch/fill anchors — both are contract amendments, not interpreter work, and neither should be started informally. Prohibited next steps: no E2 design, no threshold work, no survival-analysis subsystem, no scanner re-runs, no use of this phenotype as a strategy component.
