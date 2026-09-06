# AP-002 E1 Independent Method Challenge

> **CORRECTIONS APPLIED AT FREEZE (2026-09-05, director order):** the DEVELOPMENT start boundary was corrected from the development view's effective 2025-07-31T16:00:00Z to the frozen scope boundary **2025-07-31T16:15:00Z**. A2 re-verified: all 533 orphan events carry detection_ts < 16:15:00Z; classification LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE unchanged. A newly discovered view-builder boundary conformance finding (28 of 381,527 formed zones predate 16:15:00Z) is recorded in `AP-002_E1_ANOMALY_AUDIT.json#development_boundary_conformance_finding`.


- **Challenger:** AP-002-E1-INDEPENDENT-METHOD-CHALLENGER (independent skeptic; did not build the scanner; did not write P-01 or P-02)
- **Generated (UTC):** see `AP-002_E1_METHOD_CHALLENGE.json` (`generated_utc`)
- **Program / Stage:** AP-002 FVG — E1 PHENOTYPE METHOD CHALLENGE
- **Accepted scanner evidence:** logical_result_hash `735bd7ab06ae2caf72e37447d207dec62c17932ee1684c4c393d8ace07684c52`; integrity gate PASS; confirmation LOCKED; E2/E3 NOT STARTED
- **Companion artifact:** `AP-002_E1_ANOMALY_AUDIT.json` (A1 row-level evidence, A2 per-stratum N, clock-audit paired delay distributions, tail/censor appendix)
- **Discipline:** scanner code untouched; no scanner re-run; all frozen artifacts read-only; scratch work confined to `.runs/ap-002-e1-native-strata/challenge-audit-scratch/`; every number below recomputed from primary rows unless cited otherwise; per-stratum only, no pooling, no cross-scale identity.

---

## 1. What was independently verified

1. **Zone-row recomputation.** All 381,527 rows of run-1 `E1_ZONES.jsonl` were parsed and every headline aggregate in `E1_RESULTS.json.tables` was recomputed: per-stratum formations (227,925 / 99,874 / 43,706 / 7,140 / 2,192 / 523 / 167), direction counts, first-touch and fill counts, right-censored counts, the four-way path decomposition (closes exactly at all 7 strata), lifecycle medians, and prospective-response medians. All match.
2. **Determinism spot check.** sha256 of run-1 and run-2 `E1_ZONES.jsonl` both equal `0f0b9555c65d0052bea56c1d668a9032007138233113c244079bba061ecaa6cd`, matching `RUN_MANIFEST.json` / `REPRODUCIBILITY.json` / `SCANNER_INTEGRITY_GATE.json`.
3. **Event-stream reconciliation.** `payload_fvg` from the exact development parts the scanner ingested (`RUN_MANIFEST.provenance.source_parts`) was parsed for all seven strata. Touch events = recorded + orphan_touch and fill events = recorded + orphan_fill **exactly** at every stratum (e.g. 15s: 226,777 = 226,702 + 75 touches; 227,066 = 226,908 + 158 fills); formations = 381,527. Nothing was lost or duplicated in serialization. Development parts were spot-verified byte-identical in `payload_fvg` to the lake parts in-window (15s and 4h part-00000, including null patterns).

**No scanner reconstruction defect was found.** The fail-closed identity validation in `research-tape/src/fvg_e1.rs` (touch/fill payload identity checks, duplicate-terminal-fill rejection, touch-before-fill ordering, same-bar touch+fill processing) behaved correctly on every audited case.

---

## 2. Per-claim classifications (claims 1–15)

| # | Claim | Verdict |
|---|-------|---------|
| 1 | Median formation→touch = 2 native bars in all seven strata | **SURVIVED** |
| 2 | Median formation→fill ≈ 5–8 native bars | **SURVIVED** |
| 3 | Rapid body + long-lived right tail | **SURVIVED** (understated, not distorted — see §5) |
| 4 | Observed fill fractions are window-cumulative, not lifetime probabilities | **SURVIVED** |
| 5 | Fill rate and censor rate are accounting complements, not two findings | **SURVIVED** |
| 6 | Bullish fraction descriptively rises with native scale | **SURVIVED** (descriptive only) |
| 7 | Raw prospective medians positive in 21/21 cells | **SURVIVED** |
| 8 | Direction-adjusted medians: 18 negative / 1 zero / 2 positive | **SURVIVED** |
| 9 | Not evidence of a confirmed reversal edge | **SURVIVED** |
| 10 | Fill-without-recorded-first-touch is a genuine anomaly | **SURVIVED** — and fully resolved by this audit (§3) |
| 11 | Orphan events are probably left-truncated pre-development state | **SURVIVED** — confirmed, no longer "probably" (§4) |
| 12 | N-bars × timeframe is not a valid coverage calculation | **SURVIVED** (rejection confirmed quantitatively, §6) |
| 13 | Known-vs-market quantile differences descriptive only; paired distributions required | **SURVIVED** (§7) |
| 14 | FIRST_TOUCH E1B justified if no-touch-fill anomaly resolved | **SURVIVED** (condition met; contract precondition remains, §8) |
| 15 | FILL E1B justified but lower priority | **SURVIVED** |

Evidence paragraphs with exact sources for each claim are in `AP-002_E1_METHOD_CHALLENGE.json` (`per_claim_classifications`).

Highlights of independent verification:

- **Claims 1–2:** touch p50 = 2.0 native bars and p25 = 1.0 bar at all seven strata; fill p50 = 7/7/7/7/8/8/5 bars (15s→4h). Recomputed from zone rows, not restated.
- **Claims 7–8:** recomputed from all 381,527 `prospective_outcomes` entries: raw median > 0 in 21/21 cells; direction-adjusted 18 negative, 1 zero (15s h1 — median is exactly 0.0; the frozen `-0.0` is a sign-bit artifact), 2 positive (1h h3 +0.852 bps n=523; 4h h5 +12.508 bps n=167).
- **Claim 5:** `fill_n + right_censored_n = lawful_formation_n` at all seven strata; the identity is definitional (`right_censored = fill_ts.is_none()` in `fvg_e1.rs finish()`).

---

## 3. A1 — fill without recorded first touch: EXPECTED_SEMANTIC_CASE

**Counts reconciled exactly (reported = observed):** 15s 682, 30s 213, 1m 81, 5m 20, 15m 10, 1h 4, 4h 0 — total 1,010.

**Mechanism (from producer source, `F:/TrinityR/crates/trinity-analytics/src/bar/mod.rs`, `FvgDetector::process_bar`, v4.2):**
the producer tests far-edge fill **independently of range overlap**. First touch is emitted only when `bar.h >= zone.lower && bar.l <= zone.upper` (and the zone is not yet touched); fill is emitted when bullish `bar.l <= zone.lower` / bearish `bar.h >= zone.upper`; the fill payload records `first_touch_observed: zone.touched`; the zone is then removed from active state. A bar that trades **entirely beyond the far edge** (bullish: high < zone.lower; bearish: low > zone.upper) reaches the fill edge without its range ever overlapping the zone → fill with no first-touch, `first_touch_observed=false`.

The director reconciliation's premise — "a filling bar necessarily overlaps the zone" — is **false** for beyond-edge bars. The frozen authority semantics themselves ("fill = low reaches the lower edge / high reaches the upper edge"; "touch = range overlaps") admit this case.

**Evidence (all 1,010 rows):**

| Check | Result |
|---|---|
| Fill payload `first_touch_observed` | false in 1,010/1,010; true in 0 |
| Touch events anywhere in the ingested stream for those zone ids | 0 (searched every `payload_fvg` row of every part, all strata) |
| Same-bar touch payload for those zones | 0 |
| Fill bar entirely beyond the far edge (bar OHLC vs zone bounds) | 1,010/1,010 |
| Session gap immediately before the fill bar | 15s 260/682, 30s 143/213, 1m 76/81, 5m 20/20, 15m 9/10, 1h 4/4 (remainder = one-bar full traverses) |
| Serialization loss / duplication | none — event totals reconcile exactly per stratum |
| Same-bar touch+fill (the legitimate touch-before-fill path) | present and handled correctly (79,871 at 15s … 57 at 4h); not the A1 source |
| Producer state restoration | `ActiveFvg` incl. `touched` flag and `next_zone_id` serialize/restore in the state envelope; part boundaries cannot reset touch state |

**Representative exact traces** (full detail in `AP-002_E1_ANOMALY_AUDIT.json`):

- `15s:26769` — bearish zone [3399.21, 3400.03]; 61-minute session gap (20:59:00 → 22:00:00); fill bar 22:00:15 low 3403.555, entirely **above** the upper edge by 3.525. Weekend/evening gap-through.
- `15s:20811` — bearish zone [3291.025, 3291.085]; the very next 15s bar's low (3291.145) was already above the upper edge by 0.06 — one-bar full traverse, no session gap.
- `1h:325` — bearish zone upper 4345.24; filled Monday 01:00 after a 50-hour weekend gap; fill bar low 4355.185.
- `5m:2932` — bullish zone lower 4082.345; filled Monday 00:05 after a weekend gap; fill bar high 4067.87, entirely **below** the lower edge by 14.475.

**Classification: EXPECTED_SEMANTIC_CASE.** The lake legitimately omits first-touch while preserving fill. The research reconstruction handled these rows faithfully (recorded the fill, recorded no touch, fabricated nothing). **Not a scanner defect; not a serialization effect; not left truncation.**

**Consequence (per the challenge rules):** formation E1 survives; **FIRST_TOUCH E1B remains BLOCKED until the measurement contract is defined** (§8).

---

## 4. A2 — orphan lifecycle events: LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE

**Counts reconciled exactly:** orphan touches 177 (75/46/32/8/7/6/3), orphan fills 356 (158/94/60/18/13/7/6) for 15s→4h.

**Decisive test, performed on the orphan payloads themselves:**

| tf | orphan touch / fill | detection_ts ≤ dev start | max orphan zone_id | min in-window zone_id |
|---|---|---|---|---|
| 15s | 75 / 158 | 233 / 233 | 20,554 | 20,555 |
| 30s | 46 / 94 | 140 / 140 | 8,769 | 8,770 |
| 1m | 32 / 60 | 92 / 92 (90 strictly before; 2 at the boundary exactly) | 3,756 | 3,757 |
| 5m | 8 / 18 | 26 / 26 | 660 | 661 |
| 15m | 7 / 13 | 20 / 20 | 201 | 202 |
| 1h | 6 / 7 | 13 / 13 | 40 | 41 |
| 4h | 3 / 6 | 9 / 9 | 16 | 17 |
| **total** | **177 / 356** | **533 / 533** | — | — |

- Development start boundary = **2025-07-31T16:15:00Z** (frozen `XAUUSD_DATA_SCOPE_V1` `development.start_utc_inclusive`; `development_end_exclusive` 2026-05-05T12:39:00Z). [Corrected at freeze: this audit originally used 2025-07-31T16:00:00Z, which is the development view's effective materialized start boundary (`bar_close > 16:00:00Z`; first bar closes 16:00:15Z–20:00:00Z depending on timeframe), not the frozen scope boundary. All audited classifications are unchanged.]
- All 533 orphan formations carry `detection_ts` ≤ the boundary (531 strictly before; 2 exactly at it — 1m zone 3756, formed on the bar closing exactly at the boundary instant, which the scope excludes; its touch+fill occur in-window at 16:01:00Z).
- The producer entered the window with **warm detector state**: in-window zone ids start at 20,555 (15s) / 8,770 (30s) / 3,757 (1m) / 661 (5m) / 202 (15m) / 41 (1h) / 17 (4h); the earliest orphan formation detection is 2025-07-23T00:25:45Z — before the lake's first bar. Orphan events occur in-window up to 2025-08-29T14:31:00Z.
- `max orphan zone_id = min in-window zone_id − 1` at every stratum: no id collision, no misattribution possible. **Unexplained subset: 0.**

Orphans are already excluded from every formed-* cohort count by the reconstruction; no E1 statistic is contaminated.

---

## 5. Tail / censor audit

- **"Rapid body + long tail" SURVIVED and is understated.** Body: touch p50 = 2 bars, fill p50 = 5–8 bars everywhere. Tail: observed-fill p99 = 14,176 / 11,481 / 10,697 / 4,665 / 2,832 / 1,363 / 462 bars (15s→4h) vs p95 = 717 / 695 / 669 / 701 / 561 / 402 / 170; max observed fill = 682,740 bars (~118 days) at 15s.
- **p95 distortion is material at 1h and 4h.** Right-censoring (7.84% / 15.57%) removes exactly the slowest zones, so the observed-fill p95 conditions on the fastest 92.2% / 84.4% of the population; the unconditional p95 is **not identifiable** from this window (max identifiable unconditional quantile = 1 − censor_rate = 92.2% at 1h, 84.4% at 4h).
- **Even at 15s–15m the frozen p95s understate the unconditional tail.** Unconditional p95 (all formations as denominator, censored = unfilled) = 857 / 870 / 996 / 1,471 / 2,835 bars at 15s–15m vs observed-fill p95 717 / 695 / 669 / 701 / 561 — up to 5.1x at 15m because the tail is extremely fat.

**E1B recommendation (no new framework):** add per stratum (1) p99 and max of formation→fill and formation→touch market durations; (2) unconditional-denominator quantiles (censored = unfilled — under this design's single-end administrative censoring these equal the Kaplan-Meier estimate for all t below the window end); (3) an explicit "max identifiable unconditional quantile = 1 − censor_rate" line. A full survival-model framework is **not** warranted.

---

## 6. Cross-scale discipline and the 4h "coverage excess"

- All statistics in this audit are per native stratum; no cross-scale object identity was inferred; no pooled denominator was used.
- **Claim 12 rejection confirmed with exact bucket accounting:** 4h bars × 4 h = 194.83 bar-days vs 4,406 occupied 1h buckets = 183.58 days. The 270-hour "excess" equals **exactly** the 270 hourly sub-buckets that are unoccupied at 1h granularity but sit inside 197 partially-occupied 4h buckets (972 of 1,169 4h bars have all four sub-buckets occupied). `bars × width` counts empty sub-buckets as covered whenever any tick lands in the coarser bucket. It is not an observed-coverage estimator. There is no coverage defect: all seven strata span the identical wall-clock window.

---

## 7. Clock audit (paired, per event; descriptive only, no significance testing)

Delay = `available_time_ns/1e6 − event_time_ms` per zone row (units: ms). Full table in `AP-002_E1_ANOMALY_AUDIT.json`.

| tf | touch N | touch p50 | touch p95 | touch max | fill N | fill p50 | fill p95 | fill max |
|---|---|---|---|---|---|---|---|---|
| 15s | 226,702 | 119 | 1,632 | 183,660,064 | 226,908 | 115 | 1,414 | 183,675,029 |
| 30s | 99,313 | 119 | 1,502 | 188,100,071 | 99,232 | 114 | 1,323 | 183,660,097 |
| 1m | 43,433 | 115 | 1,182 | 183,660,279 | 43,313 | 110 | 1,075 | 183,660,064 |
| 5m | 7,039 | 108 | 888 | 183,600,276 | 6,981 | 103 | 836 | 12,608,176 |
| 15m | 2,136 | 100 | 748 | 183,600,117 | 2,104 | 99 | 605 | 12,608,176 |
| 1h | 497 | 92 | 352 | 183,600,267 | 482 | 85 | 395 | 183,600,267 |
| 4h | 157 | 108 | 1,518 | 172,800,276 | 141 | 91 | 341 | 187,200,071 |

(p05/p25/p75 in the JSON; p05 ≈ 13–20 ms, p75 ≈ 125–350 ms everywhere.)

**Reading:** median paired receipt latency is ~0.09–0.12 s at every stratum for both events; the maxima (~2.1 days) are session-gap receipts. **No stratum — including 1h — shows a distinctively large paired delay.** The frozen tables' known-vs-market quantile divergences (e.g. 1h fill p75 known 104,400,134 vs market 118,800,000 ms) compare separately computed quantiles whose difference mixes formation receipt latency with event receipt latency; the "1h strongest clock dependence" reading is **not supported at event level**. Claim 13's caution is validated. Any clock-dependence claim still requires a pre-registered anchor-clock contract in E1B.

---

## 8. Decisions

| Decision | Value | Basis |
|---|---|---|
| **E1A_FORMATION_PHENOTYPE** | **PASS** | Every headline statistic reproduces exactly from the zone rows; determinism re-verified; A1 and A2 resolved with zero scanner defect; reconstruction behaved fail-closed and faithful throughout. Freeze may proceed with two labeling annotations: `formed_fill_without_prior_touch_n` = producer-semantic gap-through-fill case (not an unexplained anomaly), and orphan counts = pre-development left-truncated state. The reconciliation premise "a filling bar necessarily overlaps the zone" is false and must not enter frozen text. |
| **FIRST_TOUCH_E1B** | **BLOCKED** | The lake legitimately omits first-touch while preserving fill; per the challenge rule, blocked until the measurement contract is defined. The anomaly itself is fully resolved — the remaining item is administrative. The contract must state: (1) touch-cohort denominator (zones with a recorded first touch, per stratum); (2) treatment of gap-through-fill zones that never emit a touch — excluded from touch-anchored cohorts, reported separately, identifiable via fill-payload `first_touch_observed=false`, never silently imputed; (3) the anchor clock (market vs known). Once frozen, FIRST_TOUCH E1B is unblocked with no further audit required. |
| **FILL_E1B** | **READY** | Fill event stream verified complete and correctly attributed; A1 gap-through fills legitimately belong to fill cohorts (self-documenting via `first_touch_observed=false`); A2 orphan fills classified and already excluded. Second priority after FIRST_TOUCH, one E1B stage-anchor extension. Register survivor-conditioning at 1h/4h (7.84%/15.57% censored), anchor clock, and same-bar touch+fill handling. |
| **QUESTION_GENERATOR_ELIGIBILITY** | **PARTIAL** | Eligible as registered hypotheses requiring formal nulls: the direction-adjusted negative-median pattern (19/21 non-positive, magnitude growing with horizon/scale), the bullish-share gradient (0.5103→0.6048), tail beyond p95, censored-population heterogeneity, paired clock-delay structure. NOT eligible: any edge/alpha framing; 4h h5 (+12.508 bps, n=167) or 1h h3 (+0.852 bps, n=523) as findings; lifetime-probability readings of fill fractions; cross-scale pooling or identity. |
| **CONFIRMATION** | **LOCKED** (unchanged) | No confirmation or E2/E3 activity performed or licensed by this audit. |

**Prospective response assessment:** the direction-adjusted median pattern is scientifically interesting enough for later Question Generator entry as `DESCRIPTIVE_DIRECTIONAL_ASYMMETRY` (window drift + mild counter-directional median tendency after formation, both direction cohorts, nearly all scales/horizons). It is a hypothesis family, not a finding; the sign counts (18/1/2) were verified exactly. No E2 run, no formal confirmation, no alpha/edge language.

**Scanner defect found:** **No.** Nothing blocks freezing E1A beyond the two labeling annotations and the one frozen-text premise correction listed above.
