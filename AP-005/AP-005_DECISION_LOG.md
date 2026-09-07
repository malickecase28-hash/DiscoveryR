# AP-005 Decision Log

Append-only. One entry per decision that affects scope, method, evidence validity, or schedule.

| # | UTC (approx) | Decision | Reason | Evidence | Scope/schedule impact |
| --- | --- | --- | --- | --- | --- |
| D-001 | 2026-09-06 | AP-005 plan authority set to V2 plus `AP-005_FIELD_RECONCILIATION_AMENDMENT.md`, adopted prospectively. V1 preserved as historical. **Detector role changed from `STATE_REGIME` to paired `EVENT`.** | Field enumeration against the real `payload_manifest.json` showed no `state` field exists for `quote_pressure`; the real schema is two independent firing signals, `buy_quote_pressure` and `sell_quote_pressure`. The original design's transition-matrix and discretization framing does not apply to this detector. | `AP-005_FIELD_RECONCILIATION_AMENDMENT.md` | Sections 3, 5, 6, 7, 8 of V2 amended prospectively; no evidence rerun, none exists yet under the old design |
| D-002 | 2026-09-06 | Frozen-input identity recorded for AP-005, same corpus as AP-001: manifest `1176d651…53af`, payload_manifest `78d5fc20…3eed`. | WP1 requires exact input identities before it can execute. | uploaded manifest/payload_manifest SHA256, matching AP-001's frozen identity | none |
| D-003 | 2026-09-06 | `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 schedule format adopted prospectively for new work. AP-001 WP1 throughput may be used only as a provisional same-shape starting reference and must be self-verified at the first checkpoint. | Unsupported hour-bucket PERT estimates provide no measured cost basis; borrowed throughput must be self-verified before it is trusted past the first checkpoint. | `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 | schedule/execution discipline only; no evidence impact |
