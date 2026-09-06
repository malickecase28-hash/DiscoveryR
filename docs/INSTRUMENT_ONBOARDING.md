# Instrument onboarding

Onboarding a second instrument reuses compatible authority and runners. The process must not restart global authority research merely because the symbol changed.

## Reusable flow

1. Declare the instrument, source manifest, native scales, availability and occurrence rules, detector/state versions, parameters, and immutable scope.
2. Validate source schema, timestamps, completeness, and read-only identity.
3. Run the Authority Compatibility Gate against the trusted producer commit and payload contract. It returns `AUTHORITY_COMPATIBLE` or `AUTHORITY_DELTA_REQUIRED` with only changed surfaces.
4. If compatible, reuse adapters, semantic templates, statistics, question ontology, strategy infrastructure, and holdout policy. If a delta exists, investigate that surface only and record the new authority version.
5. Freeze the data scope and holdout leaves before any relevant outcome is exposed. Materialize the development view with expected instrument, source, payload, boundary, and scope identities bound by the caller.
6. Run the existing typed detector adapters/scanners. Use the generic Program R population layer only for semantically compatible bounded operations; it does not replace detector-specific lifecycle/object scanners.
7. Generate outcome-blind question templates from frozen phenotype identity, ontology/lineage and permissions. The generator carries separate anchor and context scales and does not use an observed anchor timestamp or prior winners.
8. Verify, challenge and report machine artifacts by reproducibility identity and actual artifact bytes.
9. Program S may consume only confirmed Program R inputs. Program P may consume only confirmed strategies. Real confirmation remains separately custodied.

## Canonical CLI surfaces

```text
trinity_research validate-instrument <instrument-scope.json>
trinity_research authority-compatibility <authority-comparison.json>
trinity_research materialize-scope <materialize.json>
trinity_research generate-questions <question-template-request.json>
trinity_research run-experiment <population-run.json>
trinity_research verify-result <verify.json>
trinity_research challenge-result <challenge.json>
trinity_research generate-report <report-manifest.json>
```

`confirm-frozen-claim` exists only as a fail-closed worker-side preflight. Authority-bearing holdout consumption requires the external custodian path and is deliberately not an onboarding shortcut.

## When authority is reusable

Compatibility must compare at least producer commit/source identity, payload contract, detector/state versions, parameter identities, source schema, availability contract and required native scales. A compatible instrument reuses existing authority. A delta opens only the changed authority surface.

Instrument-specific behavior, thresholds, lags, sessions, regimes, normalization choices, detector phenotype results and strategy assumptions remain scientific choices. The onboarding contract standardizes provenance and causality without pretending those choices are universal.
