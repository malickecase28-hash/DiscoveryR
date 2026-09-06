# PIPELINE RULES — HARD RULES FOR RESEARCH PIPELINE TOOLING

## RULE 1 — FULL RUST PIPELINE (adopted 2026-09-06, director decision)

Every stage of the research pipeline that produces, transforms, verifies, or
hashes evidence **must be implemented in Rust, inside this repository's crates**.
This is a hard rule, effective immediately and permanently.

In scope (Rust only, no exceptions without a new explicit director ruling):

- development-view materialization and verification
- availability/anchor sidecar construction
- detector scanners and lifecycle reconstruction
- integrity gates and any verification passes
- audits whose output is cited by a freeze record or synthesis
- hash computation for any committed artifact registry
- any future E1B/E2/E3/confirmation tooling

Out of scope (permitted, non-pipeline):

- ad-hoc interactive inspection of artifacts already produced by the Rust tools
  (schema peeks, min/max checks, quick counts) — provided nothing they print is
  cited by a freeze record without recomputation by a Rust tool;
- repository operations (git, shell utilities such as `sha256sum`).

Rationale: the scope-repair audit benchmark (2026-09-06) showed the Rust
implementation of the audit layer at 5.7 s against 113.1 s for a
functionally identical Python script with byte-equivalent outputs, and the Rust
implementation reuses the scanner's canonical parsers — removing a whole class
of reimplementation drift. Python tooling additionally produced one incorrect
intermediate result during the repair (a payload-serialization mismatch) that
the canonical Rust parser handles by construction.

Enforcement: any freeze record citing a non-Rust-produced artifact is invalid.
New pipeline stages are merged only with their Rust implementation.
