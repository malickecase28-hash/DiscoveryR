# Authority review — BG-001 / A-01

The V2 authority bundle directly establishes only the canonical replay-aggregated bar schema and its declared Parquet/Hive storage surface. It explicitly leaves lineage pending source approval.

The eight required detector topics—raw3, range, fvg, bos_choch, local_structure, order_blocks, micro_liquidity, and micro_liquidity_context—remain unresolved. No producer, upstream input, object ID, occurrence time, known-time lineage, or detector relationship is asserted. The graph edge set is intentionally empty.

The manifest-declared protocol surface was not available in the private workspace, so the general fake-confluence rule could not be cited directly and remains unresolved. E3 lineage research remains blocked.

Artifact: `authority_candidate.json`.
