# AP-001 / A-01 authority review

Subject: `drift_burst` on XAUUSD.

## Gate

- `research_base_sha` matches the approved `4cf641369f64255624349696c9937eb56cb7dbcc`.
- The authority bundle identity and `authority.json` hash match the manifest declarations; its mount attestation is recorded by the manifest.
- The declared XAUUSD instrument and source-inventory file hashes match the manifest.
- The assignment file at the available declared repository source did not match the manifest's declared assignment hash, so repository authority is treated as limited to directly corroborated structure and protocol rules.

## Authority result

The V3 bundle directly establishes serialization structure: `drift_burst_completed`, `drift_burst_state`, `emitted_events[]`, the declared origin/milestone/peak/decay/dying/end paths, direction-shaped paths, score objects, four multiscale objects, and availability-labeled paths.

It does not establish lifecycle transitions, episode identity behavior, timestamp or availability semantics, state/completed relation, direction or score formulas, censoring, or segmentation. Those topics are explicitly `UNRESOLVED` in the candidate. No behavioral research, outcome inspection, or causal interpretation was performed.

The protocol rule requiring an as-of causal information clock is authoritative, but it does not make any detector field point-in-time safe. Completed, peak, decay, dying, end, and remaining fields must remain behind the future-information firewall until producer timing semantics are supplied.

See `authority_candidate.json` for the machine-readable claim set and unresolved questions.
