# Instrument onboarding

Onboarding a second instrument reuses compatible authority and runners.

1. Declare the instrument, source manifest, native scales, availability and
   occurrence rules, detector/state versions, parameters, and immutable scope.
2. Validate source schema, timestamps, completeness, and read-only identity.
3. Run the Authority Compatibility Gate against the trusted producer commit and
   payload contract. It returns `AUTHORITY_COMPATIBLE` or
   `AUTHORITY_DELTA_REQUIRED` with only changed surfaces.
4. If compatible, reuse adapters, semantic templates, statistics, question
   ontology, strategy infrastructure, and holdout policy. If a delta exists,
   investigate that surface only and record the new authority version.
5. Freeze the data scope and holdout leaves before any relevant outcome is
   exposed. Materialize the development view with expected instrument, source,
   payload, boundary, and scope identities bound by the caller.
6. Run existing synthetic-validated R, S, and P runners with the instrument
   identity supplied in every artifact. Real scientific execution remains a
   separate authorized step.

Instrument-specific behavior, thresholds, lags, sessions, regimes, and
normalization are research choices. The onboarding contract standardizes
provenance and causality without assuming those choices are universal.
