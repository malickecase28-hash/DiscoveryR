# Canonical knowledge intake

Agent markdown and scratch output are not knowledge records.

Only an explicitly promoted, typed KnowledgeRecord JSON document may enter the
canonical immutable record directory:

    cargo run -p research-contracts --bin ingest_knowledge -- record.json

The default store is knowledge/records/<record_id>.json. A duplicate record_id
fails atomically via create_new, and an invalid rejected-finding invariant fails
without changing the store. Provider
and model identity belong to the orchestrator-private mapping, not these
scientific records.
