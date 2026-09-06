# AP-001 WP1 Preflight Report

- Contract: `AP-001-WP1-CONTRACT` (frozen 2026-09-06)
- Amendment: **D-005** applied — WEAK/ONLINE anchors are now first milestone sightings made while the snapshot's event_id is bound to the armed episode (pre-origin sightings treated as carryover); STRONG/DECAY_RISK/DYING unchanged.
- Partition: development leaf = `received_ts_ns < 1773619200000000000` (2026-03-16T00:00:00Z); confirmation leaf untouched
- Preflight: `F:\TrinityR-research\Research Program\AP-001\R1_CONTRACT\AP-001_WP1_PREFLIGHT.json` (content sha256 `dbea294d38f67751f6e797d5c9a11501731589719f623df756fcfebde5d83248`)

## Gates

| Gate | Result | Key numbers |
|---|---|---|
| G1 input identity | PASS | manifest.json + payload_manifest.json SHA256 and byte size match contract |
| G2 causal ordering | PASS | 43007489 rows; regressions=0; received<event=0 |
| G3 anchor sanity | PASS | 24253 completed episodes cross-checked under D-005; mismatch total=0; entries_without_track=0; closed_without_emission=0 |
| G4 benchmark | PASS | 118673 rows/s over 43007489 rows in 362.4s; peak RSS 0.11 GiB |

## G3 traceability: run 1 (pre-amendment) vs run 2 (D-005)

| Metric | Run 1 (frozen first-sighting rule) | Run 2 (D-005 armed-episode rule) |
|---|---|---|
| WEAK mismatches | 10123 (10100 value + 23 spurious) | 0 |
| ONLINE mismatches | 4140 (4114 value + 26 spurious) | 0 |
| STRONG / DECAY_RISK / DYING mismatches | 0 / 0 / 0 | 0 / 0 / 0 |
| Run-1 at-arm diagnostic | 24220/24220 WEAK, 24196/24196 ONLINE matched, 0 mismatch | superseded: the at-arm value is now the gate anchor |
| Carryover (first armed sighting predates origin_ts) | not measured | WEAK 395 (replaced in-window: 0); ONLINE 75 (replaced in-window: 0) |


## Anchors (armed-episode detections / ground-truth present / matched)

| Anchor | detected | gt present | matched | value mismatch | missing | spurious |
|---|---|---|---|---|---|---|
| WEAK | 24220 | 24220 | 24220 | 0 | 0 | 0 |
| ONLINE | 24196 | 24196 | 24196 | 0 | 0 | 0 |
| STRONG | 24253 | 24253 | 24253 | 0 | 0 | 0 |
| DECAY_RISK | 23427 | 23427 | 23427 | 0 | 0 | 0 |
| DYING | 22600 | 22600 | 22600 | 0 | 0 | 0 |

Episodes: completed in leaf 24253; left-truncated 1; right-censored (forming 0, armed 0); forming ended without burst 471037.
Re-entry/occupancy signal: BURST re-entries from DECAY_RISK 99692; from DYING 5462; end-of-episode supersessions (state-episode end merged into the next forming tick) appear in the transition table below; emitted entry mutations 0; duplicate sightings 0.

## Per-part inventory

| Part | Footer rows | Scanned | Event ts range (ns) | Policy |
|---|---|---|---|---|
| part-00000.parquet | 12616326 | 12616326 | 1752019268175000000 .. 1760486399365000000 | full_development_scan |
| part-00001.parquet | 11503322 | 11503322 | 1760486400051000000 .. 1765576736334000000 | full_development_scan |
| part-00002.parquet | 11228177 | 11228177 | 1765756800408000000 .. 1770335999979000000 | full_development_scan |
| part-00003.parquet | 10724905 | 7659664 | 1770336000147000000 .. 1774483198583000000 | stopped_at_cutoff |
| part-00004.parquet | 11222475 | 0 | 1774483200104000000 .. 1779148798924000000 | confirmation_leaf_footer_only |
| part-00005.parquet | 11150454 | 0 | 1779148800197000000 .. 1783987199973000000 | confirmation_leaf_footer_only |

Cutoff boundary: first confirmation row at part 3 row 7659664 (row group 58), received_ts_ns 1773619200072000000.

## Benchmark and ETA

- Development leaf: 43007489 rows in 362.4s = 118673 rows/s (WP2 projection: drift_burst + ts + sequence + bid/ask, batch 16384)
- WP2-scale full development-leaf scan ETA: 362.4s wall (measured directly)
- Full-tape extrapolation (68445659 rows): 9.6 min
- Total WP1 wall including G1 and report: 362.5s

## Anomalies and observations

- D-005 amended WEAK/ONLINE rule reproduces emitted weak_ts/online_ts with zero mismatches. Pre-origin (carryover) first armed sightings: 395 WEAK / 75 ONLINE, of which 0 / 0 were replaced by a later in-window crossing; the rest kept the standing pre-origin value, which the producer itself emits.
- 236714 rows have null payload_drift_burst
- 1 episode(s) in progress at leaf start were left-truncated (anchors present at leaf start excluded from cross-check)

### State transitions (consecutive state-bearing rows, episode boundaries included)

| From | To | Count |
|---|---|---|
| IDLE | IDLE | 25002613 |
| IDLE | DEVELOPING | 485249 |
| IDLE | BURST | 571 |
| DEVELOPING | IDLE | 471037 |
| DEVELOPING | DEVELOPING | 15056639 |
| DEVELOPING | BURST | 23222 |
| BURST | IDLE | 735 |
| BURST | DEVELOPING | 279 |
| BURST | BURST | 449279 |
| BURST | DECAY_RISK | 125586 |
| BURST | DYING | 2347 |
| DECAY_RISK | IDLE | 1536 |
| DECAY_RISK | DEVELOPING | 303 |
| DECAY_RISK | BURST | 99692 |
| DECAY_RISK | DECAY_RISK | 653841 |
| DECAY_RISK | DYING | 24055 |
| DYING | IDLE | 12513 |
| DYING | DEVELOPING | 8427 |
| DYING | BURST | 5462 |
| DYING | DYING | 347388 |

Generated 2026-09-06T22:58:51Z (wall clock only; not part of the preflight hash).
