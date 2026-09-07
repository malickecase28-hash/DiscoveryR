# AP-003 and AP-004 field-reconciliation notes (pre-V3)

**Status:** Proposed corrections, smaller in scope than the AP-005/AP-006 amendments
**Trigger:** `manifest.json` / `payload_manifest.json` (verified SHA256 matching frozen identities) confirm most of the AP-003 and AP-004 design, but each has one real gap the schema exposes

## AP-004 Order Blocks: the MITIGATED terminal state does not exist

The V1/V3 plan's lifecycle vocabulary was `FORMED -> FIRST_TOUCH -> {MITIGATED | INVALIDATED | CENSORED}`, borrowing "mitigated" from the general zone-object shape FVG uses. The real `order_blocks` feature-type inventory is:

```text
bullish_order_block:       36,898
bearish_order_block:       36,241
order_block_first_touch:   71,929
order_block_invalidated:   70,991
order_block_cancelled:      1,304
```

There is no `order_block_mitigated` or `order_block_filled` type anywhere in the frozen payload. "Mitigated" is real for `fvg` (`fvg_filled`, 517,623 occurrences) and for `micro_liquidity` (`micro_pool_mitigated`), but it is not a real terminal state for `order_blocks`. Borrowing it by analogy was exactly the kind of copy-instead-of-adapt mistake the Detector Research Planning skill's "adapt, don't copy" rule exists to prevent, and it happened anyway, one layer removed, by borrowing the concept from FVG rather than the document.

**Correction:** the terminal-state vocabulary becomes `FORMED -> {FIRST_TOUCH} -> {INVALIDATED | CANCELLED | CENSORED}`, with `FIRST_TOUCH` bracketed because WP1 must check whether invalidation or cancellation can occur without a recorded touch first (the count gap, roughly 145,000 formations against ~72,000 touches, suggests many blocks resolve without ever being touched, and this needs confirming rather than assuming). `CANCELLED` (1,304 occurrences) is a real, distinct terminal type from `INVALIDATED` and needs its own row in every terminal-state table Section 7 and Section 12 (WP2) produce; it cannot be folded into "invalidated" or dropped as too rare without first checking it against the support floor.

## AP-003 BOS/CHOCH: two refinements, both in the direction of more rigor already present

**1. Breaks can be retroactively invalidated.** The feature-type inventory includes `structure_break_invalidated` (3,931 occurrences against 117,613 total break-type records, roughly 3.3%). The V1/V3 plan's Section 3 states the event has no lifecycle: "not a persistent object... the correct unit is the event itself." That is mostly right but not entirely: a small fraction of recorded breaks are later revoked. This does not turn `bos_choch` into a full `LIFECYCLE_OBJECT`; it means the event carries one lightweight terminal flag (`ACTIVE` vs `INVALIDATED`) that Section 7's per-event attribute list must record and Section 12 (WP2) must report a rate for, separate from the direction/subtype split that already exists.

**2. The swing-lineage dependency is no longer a suspected risk, it's a proven one.** The full field list for `bos_choch` includes `source_swing_id`, `protected_high_swing_id`, `protected_low_swing_id`, and `protected_reference_swing_id`. These are explicit foreign-key-style references from a break record back to specific `swings` records, not a general resemblance. Section 10's Rule 11 check currently treats `swings`/`local_structure` overlap as a control to test for (Control A/B/C, matched comparisons). Given the schema proves direct derivation, the named-lineage status for `bos_choch` vs `swings` should be pre-set to `DEPENDENT_SAME_INFORMATION_FAMILY` for any candidate built from `source_swing_id`-linked attributes specifically, using the same status vocabulary AP-001's V3 hardening already froze (`INDEPENDENT_ENOUGH_FOR_CONTROL` / `PARTIALLY_OVERLAPPING` / `DEPENDENT_SAME_INFORMATION_FAMILY` / `QUALITY_OR_CONDITION_CONTROL_ONLY` / `UNRESOLVED_LINEAGE`), rather than left as something WP4 discovers empirically. Other `bos_choch` attributes not derived through those specific fields (for example tick-context conditioning) remain open for empirical adjudication as originally designed.

## What does not need to change

Everything else in both plans, including the primary-scale selection rule, the native-bar horizon design, the cross-resolution bridge sections, and the WBS shape, holds against the real schema. The field-enumeration requirement in each plan's Section 4 did exactly what it was supposed to do: it did not need to catch anything here, because these two plans never assumed field-level detail beyond what WP1 was already going to verify. The two items above are refinements the schema makes possible now that it's in hand, not corrections of an assumption that turned out false.

## Decision

`APPROVED` / `REWORK` / `REJECTED` for each note independently, before the respective WP1 dispatches.
