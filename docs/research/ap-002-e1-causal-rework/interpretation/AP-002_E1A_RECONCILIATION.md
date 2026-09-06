# AP-002 E1A Director Reconciliation

Recorded verbatim from the research director's reconciliation of the two independent
phenotype reviews (P-01, P-02). Frozen before the independent Method Challenge.

---

## Reconciliation complete: **PASS_TO_METHOD_CHALLENGE**

The two reviews are sufficiently concordant to freeze an AP-002 E1A synthesis. I checked the disputed quantitative points directly against the committed `E1_RESULTS.json`.

The key correction is that **P-02 is right about the sign count**:

```text
direction-adjusted median returns across 7 scales × 3 horizons:

NEGATIVE      18 / 21
ZERO           1 / 21
POSITIVE       2 / 21

non-positive  19 / 21
```

The zero is the 15s one-bar median serialized as `-0.0`. The two positive cells are 1h/3-bar at about +0.85 bps and 4h/5-bar at about +12.51 bps.

So P-01's `19/21 negative` becomes:

```text
18/21 negative
19/21 non-positive
```

That is resolved.

# Canonical AP-002 E1A synthesis

### AGREED_SUPPORTED — rapid early lifecycle

The most striking phenotype is real and very clean:

```text
median formation → first touch

15s    30 sec      = 2 bars
30s    60 sec      = 2 bars
1m    120 sec      = 2 bars
5m    600 sec      = 2 bars
15m  1800 sec      = 2 bars
1h   7200 sec      = 2 bars
4h  28800 sec      = 2 bars
```

That is a remarkably consistent **two-native-bar median touch time** across all seven independently treated strata.

Median fill time is also relatively compact in native-bar units:

```text
15s     7 bars
30s     7 bars
1m      7 bars
5m      7 bars
15m     8 bars
1h      8 bars
4h      5 bars
```

But there is a **very long right tail**, so "rapid lifecycle" describes the body, not the entire distribution.

### AGREED_SUPPORTED — substantial long-lived minority

The p95 fill duration is hundreds of native bars at most scales. That means the FVG population is not simply:

```text
forms
→ touches immediately
→ fills immediately
```

It looks more like:

```text
large fast-resolution population
+
smaller long-lived persistent population
```

P-02's warning that the tables stop at p95 is correct. We do not yet characterize the extreme tail well enough to make strong statements about it.

That is a good E1B extension target, not a reason to reject E1A.

---

# Fill-rate interpretation needs one correction

Both interpreters correctly warned against treating observed fill fraction as lifetime fill probability.

But there is an even more precise point.

The scanner defines:

```text
right_censored = fill_ts.is_none()
```

so:

```text
fill_rate + right_censored_rate = 1
```

by construction.

Therefore:

> "fill rate falls as censoring rises"

is not two independent empirical findings.

It is one accounting identity.

What is empirically interesting is that the amount of censoring increases materially as native scale becomes coarser:

```text
15s    0.45%
30s    0.64%
1m     0.90%
5m     2.23%
15m    4.01%
1h     7.84%
4h    15.57%
```

So coarse-scale raw fill percentages are increasingly window-censored and should not be compared as unconditional lifetime probabilities.

---

# Direction composition: real descriptive gradient, not yet a market law

Bullish formation share moves approximately:

```text
15s    51.0%
30s    51.7%
1m     52.4%
5m     54.3%
15m    58.0%
1h     58.3%
4h     60.5%
```

That is a genuine DEVELOPMENT-period descriptive gradient.

But we do **not** promote:

> higher-timeframe FVGs are structurally bullish

from this.

It could reflect the particular XAUUSD directional environment in this development window.

So canonical status:

```text
OBSERVED
not yet GENERALIZED
not yet CONFIRMED
```

---

# Prospective response: interesting, but weak at the median

Raw return medians are positive in all 21 cells.

Direction-adjusted medians are:

```text
18 negative
1 zero
2 positive
```

That combination strongly suggests the raw-positive result contains a broad DEVELOPMENT-period upward component.

The more important E1 observation is that most direction-adjusted medians are slightly negative.

But look at magnitude.

At the fine/mid scales many medians are fractions of one basis point while distributional tails are several to tens of basis points.

So I classify this as:

```text
DESCRIPTIVE_DIRECTIONAL_ASYMMETRY
WORTH_LATER_TESTING

not:
PREDICTIVE_EDGE
not:
REVERSAL_EFFECT
not:
ALPHA
```

The coarse positive exceptions are particularly unsuitable for strong interpretation because sample sizes are only 523 at 1h and 167 at 4h.

This absolutely belongs in the later Question Generator.

---

# A1 — fill without recorded first touch: **genuine anomaly**

This is the most important reconciliation result.

Observed counts are:

```text
15s    682
30s    213
1m      81
5m      20
15m     10
1h       4
4h       0
```

These are real rows in the E1 result.

And I investigated the producer semantics.

The FVG implementation checks zone overlap first. If an untapped zone overlaps the bar, it sets `zone.touched = true` and emits `fvg_first_touch`. **Only afterward** does it test far-edge fill and emit `fvg_filled`; that fill payload records `first_touch_observed: zone.touched`.

The frozen authority says the same thing: first later overlap emits first-touch; far-edge reach emits fill.

A far-edge fill necessarily overlaps the zone.

Therefore, under the linked producer semantics:

```text
known in-window formation
        ↓
later fill
        ↓
should have recorded first touch
```

So `formed_fill_without_prior_touch_n > 0` is **not something I am willing to wave away as ordinary phenotype**.

Possible explanations include:

```text
payload materialization omission
multi-event serialization issue
state/restoration edge case
event loss
research reconstruction mismatch
another producer-path detail
```

We do not guess which.

**P-02 is correct that this must be resolved before FIRST_TOUCH E1B is frozen.**

It does **not** invalidate the FVG formation cohort.

---

# A2 — orphan events: probably left truncation, but verify

Totals reported by the interpreters are correct:

```text
orphan first-touch events = 177
orphan fills              = 356
```

The natural explanation is much less alarming than A1.

Our DEVELOPMENT scanner initializes its own research zone book at the development boundary.

But the precomputed analytical lake was produced with detector state that may contain FVGs formed **before** DEVELOPMENT began.

Therefore:

```text
zone formed before development
        ↓
development begins
        ↓
old zone touches/fills
        ↓
E1 scanner has no local formation
        ↓
orphan lifecycle event
```

That would also explain why the issue becomes relatively more visible at coarse scales where zones can persist longer.

This is a **strong hypothesis**, not yet an established fact.

The decisive test is trivial:

> inspect `detection_ts` for the orphan events.

If their formation/detection timestamps precede DEVELOPMENT start, classify them as:

```text
LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE
```

and the mystery is over.

If not, investigate further.

---

# P-02's "4h coverage excess": I reject this as currently formulated

This should **not** enter the canonical anomaly ledger as an unexplained 11-bar-day data problem.

The calculation:

```text
number_of_bars × nominal_bar_width
```

does not equal actual observed trading coverage when the underlying process contains session gaps and each timeframe has its own occupied buckets.

For example, one tick somewhere inside a four-hour bucket can create an occupied 4h bar without implying that all four constituent 1h buckets were occupied.

The producer explicitly runs the timeframes as separate native detector runtimes.

Therefore:

```text
1169 × 4 hours
```

is not a valid estimator of observed market-data duration.

Canonical classification:

```text
P02 A4
METHODOLOGICALLY_UNSUPPORTED_AS_STATED
```

We can still compare actual first/last timestamps if desired, but there is no current evidence of a coverage defect.

---

# Known-time versus market-time differences

Both reviewers noticed interesting differences.

Keep them, but narrowly.

Current tables report separate quantiles for:

```text
market duration
known/availability duration
```

A difference between their p75 values is **not the same thing** as the p75 of event-level:

```text
known_duration - market_duration
```

So statements such as:

> 1h shows the strongest clock dependence

are premature.

For E1B, we should directly calculate the paired per-zone latency difference:

```text
touch_information_delay =
touch_available_time
-
touch_event_time

fill_information_delay =
fill_available_time
-
fill_event_time
```

and characterize that distribution.

---

# Reconciled E1B decision

Both interpreters said yes to FIRST_TOUCH and FILL.

My canonical decision is slightly more prioritized:

```text
FIRST_TOUCH E1B
YES — REQUIRED

but:
A1 must be resolved first
```

Why required?

Because first touch is:

* near universal in the observed formation population,
* typically reached in only two native bars,
* an actual live lifecycle transition,
* causally available,
* potentially much more informative than formation alone.

For FILL:

```text
FILL E1B
YES — APPROVED
SECOND PRIORITY
```

Fill is terminal rather than emerging, but post-fill behavior is scientifically useful for studying continuation, reversion and structural resolution.

We should implement both in **one E1B stage-anchor extension**, not create two projects.

---

# Final reconciliation table

| Question                              | Canonical result                            |
| ------------------------------------- | ------------------------------------------- |
| P-01 overall                          | **PASS_TO_SYNTHESIS**                       |
| P-02 overall                          | **PASS_TO_SYNTHESIS**                       |
| Median touch = 2 native bars          | **AGREED_SUPPORTED**                        |
| Median fill = roughly 5–8 native bars | **AGREED_SUPPORTED**                        |
| Long-lived tail exists                | **AGREED_SUPPORTED**                        |
| High observed fill fractions          | **OBSERVED, WINDOW-CENSORED**               |
| Fill decline vs censoring rise        | **ACCOUNTING COMPLEMENT, not two findings** |
| Bullish share increases with scale    | **AGREED_OBSERVED, NOT GENERALIZED**        |
| Raw medians positive                  | **21/21 OBSERVED**                          |
| Direction-adjusted medians negative   | **18/21 NEGATIVE, 1 ZERO, 2 POSITIVE**      |
| Directional reversal edge             | **NOT ESTABLISHED**                         |
| Fill without first touch              | **REAL ANOMALY, MUST RESOLVE**              |
| Orphan events                         | **LIKELY LEFT TRUNCATION, MUST VERIFY**     |
| 4h "coverage excess"                  | **REJECT AS STATED**                        |
| p95 tail limitation                   | **VALID CAUTION**                           |
| Known vs market-time differences      | **OBSERVED, NEED PAIRED DELTA**             |
| FIRST_TOUCH E1B                       | **YES, after A1 resolution**                |
| FILL E1B                              | **YES, secondary**                          |
| Strategy/alpha claim                  | **NO**                                      |
| Confirmation                          | **LOCKED**                                  |

# Next step: one Method Challenger only

We stay with the low-concurrency model.

Current:

```text
AP-001 scanner       1 active
AP-002 reviewers     0 active
```

Next:

```text
AP-001 scanner       1
AP-002 challenger    1

TOTAL                2
```

Since P-01 and P-02 happened to use the same reasoning engine, I would preferably make this **one different-model skeptic** if your tooling allows it. This is a good place for the Claude skeptic role we reserved earlier. If that is inconvenient, one sealed independent challenger is still acceptable.

First freeze the two interpreter outputs into Git. Then give the challenger both reviews plus this reconciliation.

Paste:

```text
AP-002 E1 INDEPENDENT METHOD CHALLENGE

Program:
AP-002 FVG

Stage:
E1 PHENOTYPE METHOD CHALLENGE

Current accepted scanner evidence:
logical_result_hash =
735bd7ab06ae2caf72e37447d207dec62c17932ee1684c4c393d8ace07684c52

Scanner integrity gate:
PASS

Confirmation:
LOCKED

E2:
NOT STARTED

E3:
NOT STARTED

You are an independent scientific skeptic.

You did not build the scanner.
You did not write either phenotype review.

==================================================
INPUTS
==================================================

Read frozen:

AP-002_AUTHORITY_V1.json

docs/research/ap-002-e1-causal-rework/E1_EXPERIMENT.json
docs/research/ap-002-e1-causal-rework/E1_RESULTS.json
docs/research/ap-002-e1-causal-rework/PREFLIGHT.json
docs/research/ap-002-e1-causal-rework/RUN_MANIFEST.json
docs/research/ap-002-e1-causal-rework/SCANNER_INTEGRITY_GATE.json
docs/research/ap-002-e1-causal-rework/REPRODUCIBILITY.json

and frozen interpretation outputs:

AP-002_E1_PHENOTYPE_REVIEW_P01.json
AP-002_E1_PHENOTYPE_REVIEW_P01.md
AP-002_E1_PHENOTYPE_REVIEW_P02.json
AP-002_E1_PHENOTYPE_REVIEW_P02.md

==================================================
MASTER RECONCILIATION TO CHALLENGE
==================================================

Treat these as claims to attack, not truths to repeat:

1. median formation-to-touch is 2 native bars across all seven strata

2. median formation-to-fill is approximately 5–8 native bars

3. the phenotype has a rapid body and a long-lived right tail

4. observed fill fractions are development-window cumulative observations,
not lifetime probabilities

5. fill rate and right-censor rate are accounting complements under the
scanner definition and must not be presented as independent findings

6. bullish formation fraction descriptively rises with native scale

7. raw prospective median returns are positive in 21/21 cells

8. direction-adjusted prospective medians are:
   18 negative
   1 zero
   2 positive

9. this is not evidence of a confirmed reversal edge

10. fill-without-recorded-first-touch is a genuine anomaly

11. orphan lifecycle events are probably left-truncated pre-development state

12. the claimed 4h coverage excess based on N-bars × timeframe is not a valid
coverage calculation

13. known-time versus market-time quantile differences are descriptive only;
paired event-level delay distributions are required for a clock-dependence claim

14. FIRST_TOUCH E1B is justified if the no-touch-fill anomaly is resolved

15. FILL E1B is justified but lower priority

==================================================
MANDATORY ANOMALY AUDIT A1
==================================================

Investigate every:

formed_fill_without_prior_touch

or, if a complete audit is unnecessarily expensive, perform deterministic
aggregate classification plus representative exact-row traces.

Observed counts:

15s 682
30s 213
1m 81
5m 20
15m 10
1h 4
4h 0

Frozen producer semantics process first-touch overlap before far-edge fill.

A filling bar necessarily overlaps the zone.

Determine why these rows have fill without scanner-recorded first touch.

Inspect:

fill payload first_touch_observed
preceding first-touch payloads
same-bar first-touch payload
payload one-versus-many serialization
zone identity
part boundaries
producer state restoration
research reconstruction

Classify:

EXPECTED_SEMANTIC_CASE
LAKE_SERIALIZATION_EFFECT
LEFT_TRUNCATION
SCANNER_RECONSTRUCTION_DEFECT
PRODUCER_INCONSISTENCY
UNRESOLVED

Do not guess.

If scanner reconstruction is wrong:
FAIL the current E1 lifecycle result.

If the lake legitimately omits first-touch while preserving fill:
formation E1 may survive but FIRST_TOUCH E1B remains blocked until the
measurement contract is defined.

==================================================
MANDATORY ANOMALY AUDIT A2
==================================================

Audit orphan events:

orphan touches = 177
orphan fills = 356

Test whether their payload detection_ts / origin identity places their
formation before the DEVELOPMENT start.

If yes classify:

LEFT_TRUNCATED_PRE_DEVELOPMENT_STATE

and quantify N by native stratum.

If not:
identify the unexplained subset.

==================================================
CLOCK AUDIT
==================================================

Do not infer information-clock effects from differences between separately
reported quantiles.

Using existing zone-level rows, calculate paired descriptive distributions:

first_touch_available_time - first_touch event time
fill_available_time - fill event time

or the exact equivalent units required by the stored clocks.

Report:

N
p05
p25
p50
p75
p95
max if useful

Do not turn this into significance testing.

==================================================
TAIL / CENSOR AUDIT
==================================================

Challenge the "rapid body + long tail" characterization.

Check whether p95 conclusions are materially distorted by censoring.

Do not require confirmation.

Recommend whether E1B should add:

p99
max
censor-aware survival summaries

Do not implement a massive new framework.

==================================================
CROSS-SCALE DISCIPLINE
==================================================

Scales are separate native strata.

Do not infer cross-scale object identity.

Reject any comparison whose denominator is not semantically comparable.

Do not treat:

number_of_bars × timeframe

as observed market coverage without accounting for sparse buckets and
session gaps.

==================================================
PROSPECTIVE RESPONSE
==================================================

Verify exact count:

18 negative medians
1 zero median
2 positive medians

Assess whether the direction-adjusted median pattern is scientifically
interesting enough for later Question Generator entry.

Do NOT run E2.

Do NOT run formal confirmation.

Do NOT call it alpha or an edge.

==================================================
OUTPUT
==================================================

Produce:

AP-002_E1_METHOD_CHALLENGE.json
AP-002_E1_METHOD_CHALLENGE.md
AP-002_E1_ANOMALY_AUDIT.json

For every synthesized claim classify:

SURVIVED
FAILED
INCONCLUSIVE
REQUIRES_E1B

Return explicit decisions:

E1A_FORMATION_PHENOTYPE =
PASS / REWORK / REJECT

FIRST_TOUCH_E1B =
READY / BLOCKED

FILL_E1B =
READY / BLOCKED

QUESTION_GENERATOR_ELIGIBILITY =
YES / NO / PARTIAL

CONFIRMATION must remain LOCKED.

Do not modify scanner code unless a reproducible scanner defect is found.

STOP.
```

Once this **one** challenger comes back, we can either freeze AP-002 E1A immediately or issue a very targeted lifecycle correction. No additional interpreter pair is needed.
