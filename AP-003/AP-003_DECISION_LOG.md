# AP-003 Decision Log

Append-only. One entry per decision that affects scope, method, evidence validity, or schedule.

| # | UTC (approx) | Decision | Reason | Evidence | Scope/schedule impact |
| --- | --- | --- | --- | --- | --- |
| D-001 | 2026-09-06 | AP-003 plan authority set to V3 plus the BOS/CHOCH sections of the field-reconciliation notes, adopted prospectively. V1 preserved as historical. | Cross-detector field enumeration against the real `payload_manifest.json` exposed retroactive break invalidation and a proven `swings` lineage dependency before any code ran. | `AP-003_AP-004_FIELD_RECONCILIATION_NOTES.md` | Sections 3, 7, 10 of V3 amended prospectively; no evidence rerun, none exists yet |
| D-002 | 2026-09-06 | Frozen-input identity recorded for AP-003, same corpus as AP-001: manifest `1176d651…53af`, payload_manifest `78d5fc20…3eed`. | WP1 requires exact input identities before it can execute. | uploaded manifest/payload_manifest SHA256, matching AP-001's frozen identity | none |
| D-003 | 2026-09-06 | `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 schedule format adopted prospectively for new work; base-plan PERT hour tables remain historical only. | Unsupported hour-bucket PERT estimates provide no measured cost basis or checkpoint escalation signal. | `docs/ENGINEERING_EXECUTION_DISCIPLINE_SKILL.md` §5 | schedule/execution discipline only; no evidence impact |
