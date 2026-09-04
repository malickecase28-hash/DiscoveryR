# Stage 1 Architecture Proposal — TrinityR Research Program

**Status**: Design-only proposal for human review. No implementation performed.
**Author**: Stage 1 Architecture Agent
**Date**: 2026-09-03
**Repository**: `F:\TrinityR-research\Research Program`

---

## 1. Executive Summary

This proposal designs the smallest coherent operating architecture for the new TrinityR research program. The program studies the full lifecycles of ~23 detector/tool families under **one causal information clock**, organized around **anchor phenomena** rather than isolated timeframe universes.

The architecture answers: what the scientific units are, what states they pass through, what contracts govern them, what roles researchers play, what evidence classes exist, and what access boundaries protect causal integrity.

**Core thesis**: The legacy system's *safeguards* (causal availability, occur-vs-known, provenance, protected validation, negative-evidence preservation) are transplantable. Its *ontology* (timeframe = scientific universe, 8 isolated streams, G0-G5 gate machine) is not.

---

## 2. Scientific Architecture

### 2.1 The Four Pillars

| Pillar | Principle | Failure it prevents |
|--------|-----------|---------------------|
| One Causal Clock | `available_time <= anchor_time` | Lookahead, back-projection, future-conditioned discovery |
| Anchor-Based Lifecycle | Research organized around an anchor phenomenon | Scope creep, meaningless isolation, generic "context" |
| Controlled Exposure | E1 → E2 → E3 staged information release | Premature exposure contaminating discovery |
| Separation of Discovery & Confirmation | Discovery ≠ confirmation in status, role, and blindness | Self-declared validation, confirmation bias |

### 2.2 The Primary Scientific Unit: The Anchor Experiment

**Primary unit = Anchor Experiment (AE)**

An Anchor Experiment is the investigation of a specific anchor phenomenon at a specific anchor time, under a specific exposure stage, by a specific researcher.

It answers: "What does this anchor phenomenon look like at this point in its lifecycle, given what was knowable at that time?"

An **Anchor Program** is a named collection of related anchor experiments studying a phenomenon family (e.g., all `drift_burst.weak` experiments across instruments or anchor variants). The name "Anchor Program" is acceptable but I prefer **Anchor Program** because it implies a structured research program rather than a single isolated study. An alternative, **Anchor Campaign**, would emphasize the temporal boundedness of the investigation. I retain **Anchor Program** as the clearer term.

### 2.3 The Causal Clock

At research time `t`:
- Only information with `available_time <= anchor_time` may be used.
- Occurrence/origin timestamps remain available for provenance and lifecycle reconstruction.
- They do NOT grant earlier causal knowledge.
- No future-state conditioning. No lookahead. No retrospective back-projection of terminal information.

The causal clock is a **single universal ordering** shared by all instruments, timeframes, and detectors. A 15s object and a 4h object coexist on the same clock. Timeframe is a native attribute/stratum, not a separate scientific universe.

---

## 3. Core Entities and Their Relationships

```
┌─────────────────┐     defines     ┌─────────────────┐
│   Anchor Program │────────────────▶│  Anchor Phenomenon│
│  (program scope) │                 │  (e.g. drift_burst)│
└────────┬────────┘                 └────────┬────────┘
         │                                   │
         │  composed of                      │ has instances at
         │                                   │ specific anchor times
         ▼                                   ▼
┌─────────────────┐     assigns      ┌─────────────────┐
│  Experiment      │────────────────▶│  Anchor Time      │
│  (AE)            │                  │ (the clock point) │
└────────┬────────┘                 └─────────────────┘
         │
         │  conducted by
         ▼
┌─────────────────┐     produces      ┌─────────────────┐
│  Researcher      │────────────────▶│  Finding          │
│  (P-01 etc.)     │                  │  (DISCOVERY/etc.) │
└─────────────────┘                  └────────┬────────┘
                                              │
                                              │  classified as
                                              ▼
                                     ┌─────────────────┐
                                     │  Evidence Record  │
                                     │  (with provenance)│
                                     └─────────────────┘
```

### 3.1 Entity Descriptions

| Entity | Description | Key property |
|--------|-------------|--------------|
| **Anchor Phenomenon** | The scientific object being studied (e.g., `drift_burst.weak`, `fvg.formed`, `range.breakout`) | Defined by its lifecycle vocabulary |
| **Anchor Program** | A bounded research program around a phenomenon family | Has instrument scope, exposure plan, and success criteria |
| **Anchor Experiment** | One investigation of a specific anchor at a specific anchor time | Has an exposure stage and an evidence-maturity state |
| **Anchor Time** | The specific clock point defining the experiment's causal boundary | `available_time <= anchor_time` |
| **Researcher** | One blinded agent/model instance | Identified by neutral ID (P-01), blind to peers |
| **Lab** | A scientific function housing researchers | Defines what information a researcher sees |
| **Finding** | Any result from an experiment | Has evidence status and provenance |
| **Evidence Record** | Immutable record tying a finding to its generating code and data | Git SHA + script hash + data reference |
| **Knowledge Record** | Cumulative scientific memory entry | Links questions, findings, rejections, contradictions |

---

## 4. Proposed Research Lifecycle

Detector/lifecycle research moves through **two orthogonal dimensions**:

### 4.1 Exposure Stages (Information Access)

| Stage | Name | What the researcher sees |
|-------|------|--------------------------|
| **E1** | PHENOTYPE | The anchor's native lifecycle/phenotype only. No conditioning context. |
| **E2** | SAME_DOMAIN | Appropriate same-domain conditioning information exposed. |
| **E3** | CROSS_DOMAIN | Full eligible causally available cross-domain/multi-resolution context. |

E1 is the starting point. E2 and E3 are reached only when the research question requires it. Exposure is **staged upward, not bypassed**.

### 4.2 Evidence Maturity States

| State | Meaning | What it permits |
|-------|---------|-----------------|
| **DISCOVERY** | Exploratory finding from systematic or serendipitous discovery | Not predeclared; cannot be claimed as confirmation |
| **CANDIDATE** | Selected for challenge/confirmation testing | May proceed to Challenge Lab |
| **CHALLENGED** | Under adversarial/independent review | Cannot be reported as confirmed |
| **CONFIRMED** | Survived independent challenge | May become accepted shared knowledge |
| **REJECTED** | Failed confirmation; exact implementation refuted | Rejected implementation, not entire detector family |
| **NULL** | No effect detected | Preserved as negative evidence |

**Why these six?** The instruction requires that exploratory and confirmed work not collapse into one label, that negative evidence survives, and that discovery ≠ confirmation. Six states cover all cases without excessive granularity. I evaluated dropping CHALLENGED into CANDIDATE but retained it because adversarial review is a distinct phase with distinct blindness requirements.

### 4.3 The Two-Path Flow

```
                    ┌─────────────────────────────┐
                    │      Discovery Path          │
                    │  (systematic + serendipitous)│
                    └──────────┬──────────────────┘
                               │
                    DISCOVERY state
                               │
                    ┌──────────▼──────────────────┐
                    │  Candidate Selection         │
                    │  (not all discoveries become  │
                    │   candidates; only those      │
                    │   worth challenging)          │
                    └──────────┬──────────────────┘
                               │
                    CANDIDATE state
                               │
                    ┌──────────▼──────────────────┐
                    │  Challenge Lab               │
                    │  (adversarial review)        │
                    └──────────┬──────────────────┘
                               │
                    ┌──────────┼──────────┐
                    ▼          ▼          ▼
               CONFIRMED    REJECTED    NULL
```

A discovery does NOT automatically become a candidate. A candidate does NOT automatically become confirmed. Each transition requires explicit evidence and independent review.

---

## 5. Exposure-State Model (Detail)

The exposure model controls **what information a researcher is allowed to see**, independent of what they found.

### E1 — PHENOTYPE
- Study the anchor's native lifecycle/phenotype.
- No conditioning context from other detectors, timeframes, or domains.
- This is where all anchor research begins.
- Prevents premature cross-contamination.

### E2 — SAME_DOMAIN
- Expose appropriate same-domain conditioning information.
- Example: studying `fvg.formed` at E2 might expose other FVG lifecycle states or same-resolution structural objects.
- The "same domain" boundary is defined by the anchor's causal neighborhood, not by timeframe.

### E3 — CROSS_DOMAIN
- Expose full eligible causally available cross-domain/multi-resolution context.
- All information with `available_time <= anchor_time` across any timeframe is eligible.
- Different timeframes are NOT automatically one homogeneous statistical population. They share a clock but retain their stratum identity.

**Key design decision**: Exposure stage is a property of the *experiment contract*, not of the researcher's preference. A researcher cannot unilaterally decide to study at E3.

---

## 6. Evidence-Maturity Model (Detail)

Evidence status answers: "How strongly is this result established?"

### The State Machine

```
DISCOVERY ──select──▶ CANDIDATE ──assign──▶ CHALLENGED ──survive──▶ CONFIRMED
    │                                          │                    │
    │                     ┌────────────────────┘                    │
    │                     ▼                                         ▼
    └── reject ────── REJECTED                              (accepted knowledge)
    │
    └── no effect ─── NULL
```

### Rules

1. **DISCOVERY** results must be labeled as discovery. They cannot be presented as predeclared or confirmed.
2. **CANDIDATE** selection is deliberate and recorded. Not every discovery advances.
3. **CHALLENGED** results are under adversarial review. They cannot be reported as established.
4. **CONFIRMED** results survived independent challenge. They may enter shared knowledge.
5. **REJECTED** results reject the exact implementation/hypothesis, never the entire detector family.
6. **NULL** results are preserved and discoverable. Absence of evidence is not evidence absence.
7. **UNRESOLVED** is not a permanent state; it means more work is needed. It should not be used to avoid making a call.

### Why not UNRESOLVED as a permanent state?

UNRESOLVED is useful as a transitional label (e.g., "this finding is unresolved pending more data"), but a permanent UNRESOLVED state creates a graveyard where nothing is ever classified. If after exhaustive challenge a result remains unclear, it should be classified as NULL or REJECTED with the reason documented.

---

## 7. Experiment Contract Proposal

### 7.1 Why a Machine-Readable Contract?

The contract is the **single source of truth** that allows:
- Multiple researchers to work the same experiment independently
- A Rust runner to execute deterministically
- A skeptic to reconstruct exactly what was tested
- Causal rules to be enforced programmatically

### 7.2 Minimum Required Fields

| Field | Type | Purpose |
|-------|------|---------|
| `experiment_id` | string | Unique identity (e.g., `AE-001`) |
| `anchor_phenomenon` | string | The anchor being studied (e.g., `drift_burst.weak`) |
| `instrument` | string | Target instrument (e.g., `XAUUSD`) |
| `anchor_time` | timestamp | The causal boundary point |
| `exposure_stage` | enum `[E1, E2, E3]` | What information is available |
| `native_timeframe` | string | The native resolution (e.g., `15m`) |
| `population` | string/object | Definition of the study population |
| `development_data_scope` | object | Data range for development |
| `confirmation_data_scope` | object | Data range for confirmation (may be locked) |
| `discovery_status` | enum `[predeclared, discovered]` | Was this found systematically or discovered unexpectedly? |
| `researcher_ids` | array | Who worked this experiment |
| `code_identity` | string | Git SHA + script hash of generating code |
| `normalization_basis` | string | How native measurements are normalized |
| `created_utc` | timestamp | When the contract was written |

### 7.3 What Is NOT in the Contract

- Strategy parameters (strategy research is later)
- Entry/stop/target definitions (later)
- Model provider identities (hidden)
- Researcher names or model identities (blinded)
- Unrelated legacy findings

### 7.4 Contract as Enforcement

The contract must be powerful enough to:
- Prevent silent goalpost changes (changing `anchor_phenomenon` or `anchor_time` after results)
- Allow three blinded researchers to work the same experiment identically
- Drive deterministic Rust execution
- Preserve the causal rule `available_time <= anchor_time`

---

## 8. Detector Registry Proposal

### 8.1 Why a Registry?

Some detectors derive from others (e.g., raw detector → structural detector → derived object → contextualized object). Two detector outputs occurring together may NOT represent independent confirmation. The registry tracks these dependencies.

### 8.2 Minimum Contract Shape

Each detector entry contains:

| Field | Purpose |
|-------|---------|
| `detector_id` | Unique identifier (e.g., `drift_burst_weak`) |
| `name` | Human-readable name |
| `description` | What the detector identifies |
| `lifecycle_vocabulary` | States/transitions this detector knows (e.g., `formed`, `burst`, `resolved`) |
| `roles` | Taxonomy roles (directional evidence, state/regime, structural object, temporal context, data quality, lifecycle state, normalization/reference) |
| `derives_from` | Array of parent detector IDs (dependency graph) |
| `native_timeframes` | Array of timeframes this detector operates on |
| `availability_semantics` | When is this detector's output knowable relative to its anchor? |
| `domain` | What causal domain it belongs to |

### 8.3 Roles Taxonomy (Draft)

- `directional_evidence` — indicates direction of a move
- `state_regime` — identifies market state or regime
- `structural_object` — identifies a structural market object
- `temporal_context` — provides temporal framing
- `data_quality` — indicates data reliability
- `lifecycle_state` — marks a stage in a lifecycle
- `normalization_reference` — used as a normalization basis

A detector may belong to multiple roles.

### 8.4 What Is NOT in the Registry (Yet)

- Full 23-detector population
- Detailed parameterization
- Strategy mapping
- Performance metrics

The registry is a **contract shape**, not a populated database.

---

## 9. Lab / Researcher Model

### 9.1 Definitions

**Lab** = A scientific function. It defines *what information* a researcher is allowed to see and *what role* they play in the scientific process.

**Researcher** = One blinded agent/model instance working inside a Lab. Identified by a neutral ID (P-01, P-02, P-03). Blind to peer researchers' outputs and identities.

### 9.2 Which Labs Truly Require Separation?

The proposed permanent labs are:

| Lab | Function | Blindness Required | Independence Required |
|-----|----------|-------------------|----------------------|
| **Discovery Lab** | Systematic discovery, hypothesis generation, phenotype research (E1) | Blind to Challenge Lab inputs | Yes — must not be influenced by confirmation bias |
| **Challenge Lab** | Adversarial review, independent confirmation testing | Blind to Discovery Lab's reasoning process | Yes — must independently evaluate |

**Why only two permanent labs?**

The instruction asks to evaluate which roles are genuinely separate, which can merge, and which should be task modes. Here is my analysis:

- **Question/Hypothesis Generation** → Merges into Discovery Lab (discovery naturally generates questions)
- **Context Research (E2/E3)** → A task mode within Discovery Lab, gated by the experiment contract
- **Skeptical/Adversarial Review** → Is the Challenge Lab's core function
- **Confirmation** → Is the Challenge Lab's output function
- **Synthesis** → A task mode, not a permanent lab; occurs after Challenge Lab produces confirmed results

The architecture should not become an org chart for its own sake. Two labs provide the minimum separation needed to prevent discovery-from-being-mislabeled-as-confirmation.

### 9.3 Researcher Assignment

A researcher is assigned to one Lab per experiment. The experiment contract specifies:
- Which Lab the researcher belongs to
- What exposure stage they receive
- What data scope they can access
- What neutral ID they use

### 9.4 The "Discovery ≠ Confirmation" Guarantee

The Challenge Lab is never told that a finding came from the Discovery Lab's specific reasoning. It receives the experiment contract, the evidence records, and the finding — and independently evaluates. This prevents a discovered result from being mislabeled as predeclared confirmation.

---

## 10. Blindness and Synthesis Model

### 10.1 Blindness Requirements

During blind phases, researchers must not receive:
- Peer-agent private outputs
- Provider/model identity information
- Withheld confirmation/validation data
- Unrelated legacy findings
- The secret/provider identity registry

### 10.2 How Model/Provider Identity Is Hidden

- Researchers are identified by neutral IDs (P-01, P-02, P-03)
- Provider/model identity is operational metadata visible only to the orchestrator/research director
- The experiment contract does not reference a model or provider
- Artifacts are signed by neutral researcher ID, not by model identity

**Important caveat**: Multiple model providers are NOT automatically statistically independent merely because the vendor differs. Independence is achieved through *procedural* mechanisms: separate initial context, separate outputs, delayed synthesis, adversarial review, different role framing.

### 10.3 Synthesis After Blindness

After blinded researchers complete their work:
1. Their outputs are collected under neutral IDs
2. A Synthesis step compares findings
3. Disagreements are explicitly documented
4. The Challenge Lab adjudicates contradictions
5. The Synthesis result is a knowledge record, not a chat summary

Synthesis must happen **after** blind work is complete, not during it. Premature synthesis contaminates blindness.

---

## 11. Agent Autonomy Model

### 11.1 The Principle: Autonomy Within a Contract

Future agents may:
- Generate research questions
- Propose experiments
- Write Rust scanners
- Execute approved/bounded experiments
- Inspect deterministic outputs
- Generate follow-up questions
- Challenge results
- Create reports

They must NOT be able to wander indefinitely down unrelated research paths.

### 11.2 The Mechanism: Bounded Autonomy

Autonomy is bounded by the **experiment contract**, not by a chain of human approvals.

**Within a contract**, an agent has full autonomy:
- Choose methods
- Write code
- Explore data
- Generate follow-up questions
- Create artifacts

**Across contracts**, an agent must request a new experiment contract. The contract itself is the boundary.

**What this prevents**: Infinite drift into unrelated paths (the agent can only work within an assigned experiment's scope).
**What this preserves**: Meaningful researcher autonomy (no human approval needed for every trivial step within the contract).

### 11.3 What Requires Human Approval

- Creating a new Anchor Program (defines the research scope)
- Escalating an experiment from E1 to E2 or E3 (changes information exposure)
- Moving a finding from DISCOVERY to CANDIDATE (changes evidence status)
- Declaring CONFIRMED (requires Challenge Lab)

Everything else is within the contract.

---

## 12. Access/Sandbox Contract

### 12.1 The Principle

Design the **ACCESS CONTRACT** — what must be enforced — separate from HOW it will later be enforced. Do not choose sandbox mechanisms yet.

### 12.2 Access Matrix Per Researcher

| Access Class | Type | Examples |
|-------------|------|----------|
| **WRITE** | Own workspace | The researcher's assigned experiment directory |
| **READ** | Contract | The frozen assignment/experiment contract |
| **READ** | Shared contracts | Explicitly authorized scientific contracts (detector registry, normalization rules) |
| **READ** | Data views | Explicitly authorized read-only corpus/data views |
| **DENY** | Peer outputs | Other researchers' private outputs during blind phases |
| **DENY** | Confirmation data | Withheld confirmation/validation data |
| **DENY** | Legacy findings | Unrelated legacy findings unless deliberately provided |
| **DENY** | Identity registry | Secret/provider identity registry |
| **DENY** | Arbitrary FS | Arbitrary filesystem access where avoidable |

### 12.3 Isolation Model

Each researcher receives:
- A dedicated writable workspace (isolated from peers)
- Read-only access to the experiment contract and authorized shared contracts
- Read-only access to authorized data views (e.g., specific Parquet partitions)
- No access to other researchers' workspaces or outputs

The bootstrap did NOT prove the current runtime enforces these boundaries. The architecture specifies what must be enforced; the mechanism (Docker, ACLs, filesystem permissions, runtime sandbox) is a later implementation decision.

### 12.4 What Must Be Enforced (Summary)

1. Per-agent isolated writable workspace
2. Read-only shared scientific inputs
3. Peer output blindness during blind phases
4. No access to withheld confirmation data
5. No arbitrary filesystem access
6. Provider identity hidden from researchers

---

## 13. Knowledge-Record Model

### 13.1 Why a Knowledge Layer?

The program needs cumulative scientific memory. It cannot rely on chat memory. Chat is a reasoning surface, not canonical memory. Git is canonical audit memory, but it needs a structured knowledge layer on top.

### 13.2 Minimum Canonical Record Types

| Record Type | Purpose | Stable ID |
|-------------|---------|-----------|
| **QUESTION** | An open research question | `Q-XXX` |
| **FINDING** | Any result (discovery, null, rejection) | `F-XXX` |
| **CHALLENGE** | An adversarial challenge to a finding | `C-XXX` |
| **KNOWLEDGE** | Accepted, confirmed knowledge | `K-XXX` |

### 13.3 Record Structure

Each record has:
- `record_id` (stable identifier)
- `record_type` (QUESTION/FINDING/CHALLENGE/KNOWLEDGE)
- `content` (the actual finding/question/challenge)
- `provenance` (links back to experiment_id, code_identity, evidence)
- `status` (current evidence-maturity state)
- `created_utc`
- `supersedes` (optional, for versioning)
- `superseded_by` (optional)

### 13.4 How Negative Evidence Survives

- NULL findings are stored as FINDING records with `status: NULL`
- REJECTED findings are stored with `status: REJECTED`
- CONTRADICTIONS are stored as FINDING records linking to the contradicted findings
- Failed replications are stored with full provenance
- Unresolved questions remain as QUESTION records with no superseding KNOWLEDGE

The knowledge layer must be **queryable by negation** — one must be able to find "all rejected hypotheses about X" or "all null results for detector Y."

### 13.5 Provenance Links

Every knowledge record traces back through:
- `experiment_id` → which experiment produced this
- `code_identity` → what code generated this
- `evidence_ids` → what data supports this
- `researcher_id` → who produced this (neutral ID)

### 13.6 Superseded Results

When a result is superseded:
- The original record is NOT deleted
- `superseded_by` is set to the new record ID
- The reason for supersession is documented
- The original remains discoverable

---

## 14. Instrument Reuse Model

### 14.1 The Principle

XAUUSD is one instrument. The program expects to repeat for ~15 additional instruments. Therefore:

- **Instrument-specific** behavior belongs in configuration/contracts/results
- **Core research engine** and methodology must be reusable
- No custom methodology or copied code tree per instrument

### 14.2 Separation

| Reusable (Core) | Instrument-Specific (Config) |
|-----------------|------------------------------|
| Experiment contract schema | Instrument name |
| Detector registry schema | Data access paths |
| Anchor phenomenon definitions | Anchor time boundaries |
| Exposure stage definitions | Development/validation split |
| Evidence-maturity model | Normalization parameters |
| Knowledge record model | Specific detector parameters |
| Lab/researcher model | Instrument-specific authority |
| Causal clock rules | Access scope boundaries |

### 14.3 Configuration, Not Code

When a new instrument is added:
1. Create an instrument configuration (JSON/YAML)
2. Register the instrument in the detector registry
3. Define the anchor time boundaries
4. Set access scope

No new methodology, no new code tree, no new lifecycle.

---

## 15. Legacy Migration Decisions

### 15.1 Transplant (Keep and Adapt)

| Legacy Concept | Why Transplant | Adaptation Needed |
|----------------|----------------|-------------------|
| **Occur vs known** | Core causal safeguard | Generalize from "stream" to "anchor experiment" |
| **No future-conditioned discovery** | Core causal safeguard | Already aligns with one causal clock |
| **Evidence provenance** | Reproducibility | Add code identity and normalization basis |
| **Immutable Git identity** | Reproducibility | Keep Git SHA as identity; add experiment contract ID |
| **Protected validation** | Prevent contamination | Generalize to confirmation data scope |
| **Preservation of mistakes** | Scientific integrity | Knowledge records, not just journals |
| **Preservation of negative evidence** | Scientific integrity | NULL/REJECTED record types |
| **Isolated workspaces** | Parallel worker safety | Per-researcher workspace, not per-timeframe |

### 15.2 Redesign (Keep Spirit, Change Form)

| Legacy Concept | Problem | Redesign |
|----------------|---------|----------|
| **G0-G5 state machine** | Coupled to timeframe-isolation; 6 gates for what is really 2 phases (discovery, confirmation) | Replace with exposure stages (E1-E3) + evidence maturity states |
| **Stream labs (`stream/<timeframe>`)** | Each timeframe as isolated universe | Anchor experiments organized by phenomenon, not timeframe |
| **Branch-per-stream** | `stream/15s`, `stream/1h` etc. | Branches per experiment or per anchor program |
| **Web Reviewer per stream** | Reviewer bound to timeframe | Reviewer bound to experiment contract |
| **Strategy categories (MICRO-SCALP etc.)** | Strategy-first thinking | Strategy research is later; not part of detector science |
| **Progress Index (per stream)** | Stream-centric tracking | Experiment-centric tracking |

### 15.3 Retire (Do Not Carry Over)

| Legacy Concept | Why Retire |
|----------------|------------|
| **Timeframe = scientific universe** | Contradicts one causal clock + anchor-based lifecycle |
| **Every timeframe requiring isolated research completion** | Contradicts anchor-based lifecycle; timeframes share a clock |
| **Cross-timeframe work delayed until "graduation"** | Artificially serializes research |
| **Old strategy categories (MICRO-SCALP etc.)** | Strategy research is later; not detector science |
| **Old G0-G5 semantics as mandatory architecture** | Replaced by exposure stages + evidence maturity |
| **Stream isolation during G1** | Replaced by E1 phenotype isolation per anchor |
| **Branch naming conventions tied to timeframe** | Branches should follow experiment/anchor program |
| **`All Read.md` as launch instruction** | Historical; authority for new program is unresolved |
| **Legacy reports/journals as seed context** | Preserve as historical evidence; do not preload into new agent context |
| **Known Failure Regressions as mandatory per-rule attestations** | Retain as adversarial reference; not mandatory attestation per gate |

---

## 16. Minimal Stage 1 Implementation Plan

### What Must Be Implemented BEFORE the First Real Anchor Study

1. **Experiment Contract schema** — JSON/YAML definition of the minimum fields (Section 7)
2. **Detector Registry schema** — Minimum contract shape for detector entries (Section 8)
3. **Knowledge Record schema** — Minimum record types and structure (Section 13)
4. **Anchor Phenomenon registry (seed entries)** — The ~23 detector families with lifecycle vocabulary
5. **Causal clock enforcement check** — A validator that confirms `available_time <= anchor_time` in any experiment
6. **Researcher workspace model** — Directory structure for isolated workspaces
7. **Experiment contract template** — A minimal, fillable template for creating an experiment

### What Can Safely Wait Until AFTER the First Real Study

1. **Rust scanner implementation** — Can be written once the first experiment contract is validated
2. **Full detector registry population** — Populate as experiments proceed
3. **Automated knowledge graph** — Can be built once enough records exist
4. **Provider identity hiding mechanism** — Can be implemented when researchers are actually deployed
5. **Filesystem sandbox enforcement** — Can be implemented when the runtime is proven
6. **Synthesis tooling** — Can be built after blinded results accumulate
7. **Question generator** — Can be built after the first anchor program produces discoveries
8. **Full normalization formulas** — Can be defined per-phenomenon as needed

### The Absolute Smallest Stage 1 Implementation

The smallest Stage 1 implementation that gives a scientifically safe foundation is:

1. **One JSON schema** for the experiment contract
2. **One JSON schema** for the detector registry entry
3. **One JSON schema** for knowledge records
4. **One seed file** listing the ~23 anchor phenomena with lifecycle vocabulary
5. **One validator** checking `available_time <= anchor_time`
6. **One directory template** for researcher workspaces
7. **This document** as the architectural contract

That's 7 artifacts, not 14,000 lines of scanner code.

---

## 17. Explicit "NOT YET" List

- NOT implementing Rust scanners
- NOT creating Rust code
- NOT populating the full detector registry
- NOT creating agent workspaces
- NOT choosing sandbox mechanism (Docker/ACL/WSL)
- NOT implementing provider identity hiding
- NOT building the question generator
- NOT designing strategy research infrastructure
- NOT creating the synthesis tool
- NOT migrating any legacy code or data
- NOT creating schemas for future stages
- NOT building the full knowledge graph
- NOT implementing automated validation tooling beyond the clock check
- NOT defining all normalization formulas
- NOT populating knowledge records
- NOT deploying any AI researchers

---

## 18. Risks / Failure Modes

| Risk | Severity | Mitigation |
|------|----------|------------|
| Architecture becomes another bureaucracy | High | Every process step must answer "What failure does this prevent?" If no meaningful answer, remove it |
| Accidentally recreating isolated timeframe research | High | Anchor-based lifecycle replaces timeframe isolation; exposure stages replace stream gates |
| Too many statuses | Medium | Six evidence states; exposure stages are separate from evidence states |
| Confusing exposure with evidence maturity | High | Explicitly separate concepts in contract and documentation |
| Forcing hypothesis-first research | Medium | Discovery is legitimate; `discovery_status: discovered` is valid |
| Making autonomous agents too dangerous | Medium | Bounded by experiment contract; human approval needed for scope changes |
| Making agents too restricted | Medium | Within a contract, full autonomy; human approval only for scope changes |
| Huge universal schema before observing real studies | High | Minimal schemas; populate as experiments proceed |
| XAUUSD architecture becomes instrument-specific | Medium | Configuration vs core separation; instrument-specific in config files |
| Strategy research enters detector science | High | Explicitly excluded from Stage 1 scope |
| Negative evidence lost | Medium | Knowledge records preserve NULL/REJECTED; queryable by negation |
| Raw measurements discarded | Medium | Normalization principle preserves RAW + NORMALIZED + BASIS |
| Rust runner cannot interpret contract | Medium | Contract is machine-readable JSON; validated before use |
| Three blinded researchers cannot work from same contract | Medium | Contract is the single source of truth; neutral IDs |
| Skeptic cannot reconstruct what was tested | Medium | Evidence records tie findings to code and data |
| Cannot produce XAUUSD report within days | Medium | Minimal implementation plan; report can use E1 phenotype study |

---

## 19. Open Questions Requiring Research-Director Decision

1. **Primary unit name**: Is "Anchor Experiment" the right term, or should it be "Anchor Study," "Phenotype Investigation," or something else?
2. **Evidence states**: Should UNRESOLVED be a permanent state or only transitional?
3. **Lab count**: Are two permanent labs (Discovery + Challenge) sufficient, or do we need a separate Synthesis lab?
4. **Exposure stage naming**: Are E1/E2/E3 clear, or should they have descriptive names?
5. **Detector count**: When should the ~23 detector families be fully registered?
6. **Normalization scope**: Should normalization rules be defined per-experiment or per-phenomenon-family?
7. **Knowledge storage**: Should knowledge records live in Git files, a database, or both?
8. **Researcher count**: How many blind researchers per experiment is optimal?
9. **Rust timeline**: When should the first Rust scanner be written?
10. **Data access mechanism**: How will agents access the frozen Parquet corpus? (Mount, copy, DuckDB view?)

---

## 20. Proposed Directory Tree (DESIGN ONLY — Do Not Create)

```
Research Program/
├── README.md
├── STAGE1_ARCHITECTURE_PROPOSAL.md          ← This document
├── bootstrap/
│   ├── COMPONENT_INVENTORY.json
│   ├── ENVIRONMENT_MAP.md
│   ├── MIGRATION_CANDIDATES.md
│   ├── OPEN_QUESTIONS.md
│   └── STAGE1_ARCHITECTURE_PROPOSAL.md
├── contracts/                                 ← Machine-readable contracts
│   ├── experiment_contract_v1.json            ← Experiment Contract schema
│   ├── detector_registry_v1.json              ← Detector Registry schema
│   └── knowledge_record_v1.json               ← Knowledge Record schema
├── registry/                                  ← Seed registries
│   └── anchor_phenomena.json                  ← ~23 anchor phenomena with lifecycle vocabulary
├── programs/                                  ← Anchor Programs (one per phenomenon family)
│   └── drift_burst_weak/
│       ├── program_contract.json              ← Anchor Program definition
│       ├── experiment_001/                    ← Individual experiments
│       │   ├── experiment_contract.json
│       │   ├── researcher_P-01/               ← Isolated researcher workspace
│       │   │   ├── workspace/                 ← Writable only
│       │   │   └── artifacts/
│       │   └── researcher_P-02/
│       │       └── workspace/
│       │       └── artifacts/
│       └── knowledge_records/                 ← Knowledge layer
│           ├── findings.jsonl
│           ├── challenges.jsonl
│           └── questions.jsonl
├── core/                                      ← Reusable research engine (Rust, later)
│   ├── causal_clock.rs                        ← available_time <= anchor_time validator
│   ├── contract_validator.rs                  ← Experiment contract validation
│   └── normalization.rs                       ← RAW + NORMALIZED + BASIS preservation
├── instruments/                               ← Instrument-specific configuration
│   └── XAUUSD/
│       ├── instrument_config.json
│       ├── access_scope.json
│       └── anchor_times.json
├── knowledge/                                 ← Cumulative scientific memory
│   ├── findings.jsonl
│   ├── questions.jsonl
│   └── rejections.jsonl
└── docs/
    └── ARCHITECTURE_DECISIONS.md              ← Rationale for key decisions
```

**Key design notes**:
- `contracts/` contains machine-readable schemas
- `programs/` contains anchor programs organized by phenomenon, not by timeframe
- `core/` is the reusable engine (Rust implementation deferred)
- `instruments/` contains configuration, not methodology
- `knowledge/` is the cumulative scientific memory layer
- Each researcher workspace is isolated and writable only by that researcher

---

## 21. Adversarial Self-Review

Before finalizing, I attacked this design against the criteria in Section 18:

1. **Did I accidentally recreate isolated timeframe research?** No. The architecture is anchor-based, not timeframe-based. Timeframes are native attributes, not scientific universes. E1-E3 exposure stages replace stream gates.

2. **Did I create too many statuses?** Six evidence states is the minimum that prevents collapsing exploratory and confirmed work. Exposure stages are separate.

3. **Did I confuse exposure with evidence maturity?** No. Sections 5 and 6 explicitly separate them. Exposure = what information was available. Evidence status = how strongly established.

4. **Did I force hypothesis-first research and suppress discovery?** No. DISCOVERY is a valid evidence state. `discovery_status: discovered` is an explicit contract field.

5. **Did I make autonomous agents too dangerous?** No. Autonomy is bounded by the experiment contract. Scope changes require human approval.

6. **Did I make agents too restricted?** No. Within a contract, agents have full autonomy — write code, explore, generate questions.

7. **Did I create a huge universal schema before observing real studies?** No. Minimal schemas for experiment, detector, and knowledge records. Population deferred.

8. **Did I make XAUUSD architecture instrument-specific?** No. Instrument-specific configuration is separated from the reusable core engine.

9. **Did I accidentally make strategy research part of detector science?** No. Strategy research is explicitly in the "NOT YET" list.

10. **Did I preserve negative results?** Yes. NULL and REJECTED are first-class evidence states. Knowledge records preserve them with full provenance.

11. **Did I preserve raw/native measurements alongside normalized values?** Yes. The normalization principle (RAW + NORMALIZED + BASIS) is a contract requirement.

12. **Can a future Rust runner interpret the experiment contract?** Yes. The contract is machine-readable JSON with explicit fields.

13. **Can three blinded researchers independently work from the same contract?** Yes. The contract is the single source of truth; researchers are identified by neutral IDs.

14. **Can a skeptic reconstruct exactly what was tested?** Yes. Evidence records tie findings to experiment_id, code_identity, and evidence_ids.

15. **Could this realistically help produce the XAUUSD report within days?** Yes. The minimal implementation plan (7 artifacts) can be completed quickly. An E1 phenotype study of a single anchor can proceed immediately.

---

## 22. Final Summary of Key Decisions

| Question | Answer |
|----------|--------|
| Primary unit of research | **Anchor Experiment** |
| Primary program unit | **Anchor Program** |
| Exposure states | **E1 (PHENOTYPE), E2 (SAME_DOMAIN), E3 (CROSS_DOMAIN)** |
| Evidence states | **DISCOVERY, CANDIDATE, CHALLENGED, CONFIRMED, REJECTED, NULL** (6 states) |
| Permanent lab types | **2** (Discovery Lab, Challenge Lab) |
| Mandatory core record/contracts | **3** (Experiment Contract, Detector Registry, Knowledge Record) |
| Biggest simplifications vs legacy | See Section 23 |
| Biggest safeguards retained | See Section 23 |
| Implementation before first study | **7 artifacts** (Section 16) |

---

## 23. Three Biggest Simplifications vs Legacy

1. **Anchor-based lifecycle replaces timeframe isolation**: Instead of 8 parallel stream labs each studying a timeframe as its own universe, research is organized around anchor phenomena. Timeframes are native attributes, not scientific universes. This eliminates the serial "graduation" bottleneck where cross-timeframe work waits for all streams to complete.

2. **Two-phase research replaces six-gate machine**: Instead of G0-G5 gates, the architecture uses exposure stages (E1-E3) for information access and evidence-maturity states (DISCOVERY → CANDIDATE → CHALLENGED → CONFIRMED/REJECTED/NULL) for evidence. This eliminates the bureaucratic overhead of a 6-gate machine while preserving the critical separation between discovery and confirmation.

3. **Contract-based autonomy replaces gate-by-gate approval**: Instead of requiring human approval at every checkpoint, agents have full autonomy within an experiment contract. Human approval is required only for scope changes (new program, new exposure stage, status transitions). This eliminates the approval chain bottleneck while keeping agents bounded.

---

## 24. Three Biggest Safeguards Retained

1. **Causal clock (`available_time <= anchor_time`)**: The fundamental safeguard against lookahead and back-projection is preserved and generalized from stream-level to experiment-level. Every experiment contract enforces this rule.

2. **Separation of discovery from confirmation**: The architecture guarantees that a discovered result cannot be mislabeled as predeclared confirmation. Discovery and Challenge Labs are separate, blinded functions. This preserves the scientific integrity that was a hallmark of the legacy system.

3. **Preservation of negative evidence**: NULL findings, rejected candidates, and contradictions are stored as first-class knowledge records with full provenance. The knowledge layer is queryable by negation. This prevents the "only positive results survive" failure mode that plagued earlier research.

---

## 25. Three Most Important Unresolved Decisions

1. **Exposure stage granularity**: Are E1/E2/E3 sufficient, or do we need intermediate stages? The current design assumes three stages is the minimum viable model, but a specific anchor phenomenon might reveal a need for finer-grained exposure control.

2. **Knowledge storage mechanism**: Should knowledge records live in Git files (simple, versioned) or a database (queryable, scalable)? Git files align with the existing infrastructure but may not scale to hundreds of experiments. A hybrid approach (Git for provenance, database for querying) is possible but adds complexity.

3. **Researcher count per experiment**: How many blind researchers should work each experiment? The instruction mentions P-01, P-02, P-03 as examples, suggesting at least 3. But the optimal number depends on the phenomenon and the resources available. Too few reduces independence; too many increases cost and coordination overhead.

---

## 26. Confirmation: No Implementation or Migration Performed

This document is a **design-only proposal**. No implementation code has been written. No migration of legacy code or data has been performed. No schemas have been created. No agent workspaces have been set up. No Rust code has been generated.

The only file created is `bootstrap/STAGE1_ARCHITECTURE_PROPOSAL.md` itself.

---

*End of Stage 1 Architecture Proposal. Pending human review before any implementation begins.*
