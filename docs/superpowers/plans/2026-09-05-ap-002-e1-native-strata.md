# AP-002 FVG E1 Native-Strata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and run a deterministic Rust-only AP-002 E1 FVG phenotype scanner over the complete XAUUSD development view without cross-scale identity or receipt-time leakage.

**Architecture:** Extend `research-tape` with one typed FVG E1 library module and two narrow binaries: the scanner and an independent method-challenge checker. The scanner preflights all development ticks, streams each native bar stratum with state preserved across parts, consumes the frozen availability sidecar, writes bounded machine-readable artifacts, and hashes only logical results; the challenge binary rechecks the resulting zone table and provenance independently.

**Tech Stack:** Rust 2021, existing Arrow/Parquet 53 reader, serde/serde_json, sha2, standard-library buffered I/O and ordered maps.

**Spec:** User AP-002 `FROZEN_NATIVE_STRATA / READY_FOR_E1_NATIVE_STRATA` request; frozen authority `knowledge/wave1_authority_closure/AP-002_AUTHORITY_V1.json`.

## Global Constraints

- Scope is XAUUSD_DATA_SCOPE_V1 DEVELOPMENT only; E2/E3, confirmation, strategy rules, and other detector outcomes remain excluded.
- Lawful identity is `(timeframe, zone_id)`; cross-scale object identity is prohibited.
- Formation availability comes from the frozen sidecar `received_ts_ns` plus equal-receipt `source_sequence`; `bar_close_ts` is never used as availability.
- The complete development tick view is scanned in source-sequence order before behavior; any receipt regression writes preflight evidence and aborts results.
- Rust only for the active pipeline; Parquet is projected and streamed; state survives part boundaries; no raw-lake copies.
- Formation fields are independent of later touch/fill outcomes; unfilled terminal zones are right-censored.
- Every emitted table has `n` and provenance; logical result hashing excludes runtime timestamps and filesystem paths.

---

### Task 1: Establish typed FVG contracts and red tests

**Files:**
- Modify: `crates/research-tape/src/lib.rs`
- Create: `crates/research-tape/src/fvg_e1.rs`
- Test: `crates/research-tape/src/fvg_e1.rs` module tests

**Interfaces:**
- Consumes: projected Arrow batches and frozen sidecar records.
- Produces: typed `FvgPayload`, `Formation`, `LifecycleEvent`, `ZoneLifecycle`, `PreflightSummary`, and deterministic hash helpers.

- [ ] Write failing tests for single-object and array payload decoding, bullish/bearish native identity separation, malformed payload rejection, first-touch/fill exclusion from formation fields, receipt-time availability lookup, part-boundary state continuity, and right-censor finalization.
- [ ] Run `cargo test -p research-tape fvg_e1 -- --nocapture`; confirm the tests fail because the new types/functions do not exist.
- [ ] Implement only the typed serde structs, explicit event ordering, sidecar cursor, preflight accumulator, lifecycle state, and logical serialization required by those tests.
- [ ] Run the focused tests and then `cargo test -p research-tape`; require zero failures.

### Task 2: Add scanner and challenge binaries

**Files:**
- Create: `crates/research-tape/src/bin/ap002_fvg_e1.rs`
- Create: `crates/research-tape/src/bin/ap002_fvg_e1_method_challenge.rs`
- Modify: `crates/research-tape/src/lib.rs`
- Modify: `crates/research-tape/Cargo.toml` only if explicit binary entries are required by Cargo

**Interfaces:**
- Scanner CLI: `--view-root`, `--sidecar`, `--sidecar-manifest`, `--output-root`, `--scanner-commit`.
- Challenge CLI: `--results`, `--zones`, `--preflight`, `--output`.
- Scanner outputs: `E1_EXPERIMENT.json`, `E1_RESULTS.json`, `E1_ZONES.jsonl`, `E1_FINDINGS.md`, `RUN_MANIFEST.json`, `PREFLIGHT.json`, `METHOD_CHALLENGE_INPUT.json`.

- [ ] Write failing integration tests for deterministic hashing, native-stratum separation, sidecar lookup, and receipt-regression fail-closed behavior using temporary Parquet/JSONL fixtures.
- [ ] Run those tests and confirm expected failures.
- [ ] Implement the scanner as seven sequential native-stratum passes with a fresh sidecar cursor per timeframe, a tick preflight before any behavioral scan, strict schema/row-count checks, and fixed response horizons of the next 1, 3, and 5 strictly-future native closes after availability.
- [ ] Implement the independent challenge binary to reparse zone rows and verify identity uniqueness, formation/outcome separation, causal response timestamps, right-censoring, sidecar/hash provenance, and preflight pass status.
- [ ] Run focused integration tests and `cargo test --workspace`.

### Task 3: Execute E1 and verify reproducibility

**Files:**
- Create: `.runs/ap-002-e1-native-strata/run-1/` generated evidence
- Create: `.runs/ap-002-e1-native-strata/run-2/` generated evidence
- Modify: none after scanner implementation except generated run evidence

- [ ] Build the release scanner and record the committed scanner SHA.
- [ ] Run the complete development-view preflight and scanner in a fresh process into run 1; require `regressions_count == 0`.
- [ ] Run the scanner again in a fresh process into run 2; require the same logical result hash.
- [ ] Run the independent challenge package against run 1 and require a pass without using other detectors or confirmation data.
- [ ] Validate JSON artifacts against existing contract conventions, inspect `git diff --check`, and perform the deletion-focused simplification and lean review.
- [ ] Commit the narrow change on `research/ap-002-e1-native-strata`, push the branch, and report commit SHA, preflight artifact/result, both logical hashes, test results, and absolute artifact paths.
