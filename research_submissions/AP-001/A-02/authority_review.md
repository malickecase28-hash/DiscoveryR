# AP-001 / A-02 authority review

Subject: `drift_burst`

## Evidence gate

- Research base: `4cf641369f64255624349696c9937eb56cb7dbcc`.
- Manifest: `F:\TrinityR-manifests\wave1-v3-4cf6413\AP-001\A-02\research_input_manifest.json`.
- Manifest authority source: `xauusd.wave1.ap001`.
- Authority bundle content identity: `b485920ff2961a2e995e7ee6b0a74bfcb18bfb826495c6679b32e4e657b29ef5`.
- Authority bundle directory transport hash: `959f07628fc89dffe5b2183e786129578d0b39a9907271f0de719d67ad4c8168`.
- Mount attestation: `45f0360a7645ddc8991d74e9cb3045330a803543b45376d2942c4bbdbfdf9cd2`.
- Repository evidence was read at the approved research-base commit. Only the manifest-declared repository surfaces and authority bundle were used.

## Reconstruction

The authority bundle provides a declared native detector serialization contract. It lists paths for completed records and state snapshots, including event identifiers, origin/phase/end fields, availability-labelled fields, scores, directions, and 5s/15s/30s/60s multiscale fields. The repository registry independently declares `drift_burst` as a tick-surface detector with tick native scale and `UNRESOLVED` semantic status.

Only serialization structure is authoritative. The evidence does not establish lifecycle, episode identity, occurrence or availability timing, transitions, future-field knowledge, direction meaning, score formulas, multiscale construction, censoring, or segmentation. The candidate records each point explicitly as `UNRESOLVED`; no behavioral or outcome claim is made, and no activation or promotion conclusion is drawn.

See `authority_candidate.json` for claim-level evidence and unresolved questions.
