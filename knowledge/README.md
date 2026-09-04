# Canonical knowledge intake

Agent markdown and scratch output are not knowledge records.

Only an explicitly promoted, typed KnowledgeRecord JSON document may enter the
canonical append-only store:

    cargo run -p research-contracts --bin ingest_knowledge -- record.json

The default store is knowledge/records.jsonl. A duplicate record_id or an
invalid rejected-finding invariant fails without changing the store. Provider
and model identity belong to the orchestrator-private mapping, not these
scientific records.
