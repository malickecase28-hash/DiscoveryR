# TC-001 / A-02 authority review — revision-002

Scope: six XAUUSD tick-conditioning families, using exactly the previously mounted authority sources.

## Result

Each `payload_family` column is AUTHORITATIVE as a physical nullable `LargeUtf8` output column in the Tick source. This proves physical presence only; it does not define scientific meaning, record variants, producer purpose, supported role, event/state behavior, identity, timing, reset, persistence, transition, direction, baseline, history, or future mapping. Those semantic contracts remain UNRESOLVED for every family.

`event_ts_ns`, `received_ts_ns`, `source_sequence`, `bid`, and `ask` are `PHYSICALLY_PRESENT_ON_TICK_ROW` only. Their lawful use as detector state is unresolved. Occurrence/origin is distinct from availability/emission-known time. The general causal rule is AUTHORITATIVE: only information actually available by anchor time may be used; exact future-field mappings remain UNRESOLVED. Names such as `from`, `to`, or `transition` are not blanket future classifications.

The registry's `derives_from=[]` records no declared dependency. It is not proof of no dependency or independence. Semantic conditioning contracts are unresolved; behavioral research is blocked pending detector-specific contracts and emission timing.

The JSON artifact contains one structured-evidence claim for each physical surface and each mandatory semantic topic, plus the lifecycle-stage future-information matrix.
