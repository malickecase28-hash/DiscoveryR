# Stage 2 technical note

Stage 2 provides causal temporal primitives and bounded Parquet projection for
the frozen XAUUSD lake. `AsOfLatestCursor` retains the latest observation with
`available_time <= anchor_time`; `CausalWindowCursor` retains the bounded
lookback window under the same law. Both reject unordered input and never use
occurrence time as availability time.

The release pilot scans the six sequential 15m parts using only
`bar_close_ts`. It emits canonical completed-bar anchors for infrastructure
verification only. The detector registry has no authoritative lifecycle or
availability semantics, so no detector lifecycle adapter was claimed.

Large pilot artifacts remain outside Git under `TRINITYR_RESEARCH_ARTIFACTS`.
