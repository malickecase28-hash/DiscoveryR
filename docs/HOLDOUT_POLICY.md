# Holdout policy

Holdouts are immutable scope leaves identified by dataset identity, instrument,
and interval. A policy declares roles such as development,
detector-confirmation, strategy-holdout, portfolio-holdout, and future. Leaves
must be disjoint, and exposure is append-only in an immutable ledger.

The policy is frozen before the relevant information is exposed. A detector
confirmation result cannot automatically become an untouched strategy holdout;
use a nested holdout or a future-data policy. Renaming a leaf or changing a
label cannot repair overlap or contamination. Completed objects and later
outcomes may be used only at lawful later availability times.

Workers may receive only declared scope and context permissions. The Windows
workflow records and validates this policy but is not an operating-system
security boundary. External custody and authentication are required to unlock
real confirmation. Synthetic confirmation labels are test-only and never grant
authority.
