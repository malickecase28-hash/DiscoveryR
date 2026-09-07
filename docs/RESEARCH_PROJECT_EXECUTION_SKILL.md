# Skill: Research Project Execution

**Purpose:** Keep a multi-agent research team executing an approved research plan with consistent project-management discipline, visible progress, bounded scope, and minimal governance overhead.

**Applies to:** Program R execution after plan approval.  
**Primary objective:** Produce valid scientific artifacts on schedule without infrastructure drift or uncontrolled methodological expansion.

---

## 1. Operating principle

The approved plan is the execution contract.

Team members do not reinterpret the project from scratch.

Every active task must map to:

- one work package;
- one required deliverable;
- one exit condition.

If a task cannot be mapped to all three, it is not active project work until approved.

---

## 2. Team roles

### Research director

Owns:

- scope;
- plan approval;
- phase gates;
- candidate promotion;
- confirmation release;
- change decisions;
- final scientific claims.

### Deterministic toolsmith / executor

Owns:

- scanner implementation;
- causal joins;
- reproducibility;
- evidence artifact production;
- integrity gates;
- deterministic hashes.

Does not decide scientific meaning.

### Research analyst

Owns:

- interpretation of frozen evidence;
- claim drafting;
- comparison of cohorts;
- identification of candidate relationships;
- explicit `NULL` and `INCONCLUSIVE` outcomes.

Does not modify evidence-generating methods after seeing results.

### Independent skeptic

Owns:

- leakage challenge;
- confounder challenge;
- alternative explanations;
- support and censoring challenge;
- claim downgrading when evidence is insufficient.

Used at challenge gates, not as a mandatory reviewer of every routine artifact.

---

## 3. Concurrency rule

Default maximum active workers: **2**.

A third worker is allowed only for clearly independent work.

Examples:

- acceptable: one scanner running while an analyst interprets a previously frozen artifact;
- acceptable: two sealed independent interpreters reading the same frozen result;
- not acceptable: multiple agents changing the same scanner or mutable evidence;
- not acceptable: many parallel agents exploring the same raw data without distinct scientific assignments.

Concurrency is a resource control, not a productivity target.

---

## 4. Work-package execution loop

Every work package follows the same loop.

### 1. Start

Record:

- work-package ID;
- exact objective;
- input identities;
- expected deliverable;
- exit condition;
- active owner;
- forecast.

### 2. Execute

Work only on tasks required for the deliverable.

### 3. Verify

Before interpretation, verify:

- causal validity;
- row or object counts;
- required exclusions;
- censoring;
- deterministic rerun where required;
- artifact identity;
- integrity gate.

### 4. Interpret

Interpret only frozen evidence.

Do not alter the scanner to improve a disappointing result.

### 5. Gate

Set the work package to:

- `COMPLETE`;
- `REWORK_REQUIRED`;
- `BLOCKED`;
- `REJECTED`.

### 6. Advance

Only `COMPLETE` advances to the next approved work package.

---

## 5. Progress reporting

Use one compact status record.

Required fields:

```text
PROJECT:
WORK_PACKAGE:
STATUS:
OWNER:
STARTED:
FORECAST:
DELIVERABLE:
EVIDENCE_ID:
EXIT_CONDITION:
CURRENT_RESULT:
BLOCKER:
NEXT_ACTION:
SCOPE_CHANGE:
CONFIRMATION_STATUS:
```

Rules:

- `CURRENT_RESULT` describes evidence, not activity.
- "Worked on scanner" is not progress.
- "Lifecycle artifact frozen, 5/5 anchor cohorts valid" is progress.
- Infrastructure work is listed only when it repaired or enabled the current work package.
- Confirmation status is always visible.

---

## 6. Forecast control

A forecast is not a promise.

At work-package start, use the approved estimate.

Replace assumptions with measured runtime as soon as a representative benchmark exists.

Reforecast only when:

- expected total effort changes by more than the plan's threshold;
- a validity defect creates new required work;
- scope changes.

If a work package reaches its pessimistic estimate without satisfying the exit condition:

1. stop;
2. classify the reason;
3. issue a change decision;
4. do not continue by inertia.

---

## 7. Defect protocol

A defect is not permission to redesign the project.

Use this exact sequence:

1. **Scientific consequence** — what evidence becomes invalid?
2. **Reproduction** — prove the defect exists.
3. **Blast radius** — identify affected artifacts only.
4. **Narrow repair** — change the smallest responsible component.
5. **Verification** — rerun integrity checks.
6. **Evidence rerun** — rerun only affected evidence.
7. **Closure** — record what remained unchanged.

If the scientific consequence is zero, do not reopen completed evidence.

---

## 8. Scope-change protocol

A proposed change must state:

```text
CHANGE_ID:
TRIGGER:
CURRENT_APPROVED_DESIGN:
PROPOSED_CHANGE:
SCIENTIFIC_REASON:
AFFECTED_WORK_PACKAGES:
AFFECTED_FROZEN_ARTIFACTS:
SCHEDULE_IMPACT:
CONFIRMATION_IMPACT:
DECISION:
```

Allowed decisions:

- `APPROVED`;
- `REJECTED`;
- `DEFERRED`.

No methodological change is implied by another project's approval.

### Cross-project rule

A new detector plan may establish a better default for future work.

It does not automatically amend an already frozen experiment.

Example:

Changing AP-002 E2 from exhaustive pre-interpretation resampling to a discovery-first, challenge-second design requires its own approved change record before execution resumes.

A confirmed quality-instrumentation finding follows the same rule. If a project such as `feed_health` recommends excluding or flagging degraded intervals, that finding triggers a separate change decision for each affected frozen detector project. It does not automatically invalidate, amend, or rerun those projects. The decision for each affected project must state the evidence blast radius and whether a sensitivity rerun, exclusion rerun, flag-only treatment, or no change is warranted.

---

## 9. Candidate-promotion gate

Discovery output is not automatically a candidate.

Promotion requires the predeclared project criteria.

The director receives a compact promotion packet:

```text
CANDIDATE_ID:
SOURCE_WORK_PACKAGE:
ANCHOR:
CONTEXT:
OBSERVED_RELATIONSHIP:
SUPPORT:
CHRONOLOGICAL_STABILITY:
EFFECT_MAGNITUDE:
CENSORING_NOTE:
DEPENDENCY_NOTE:
PLAUSIBLE_INFORMATION_PATH:
DECISION_RELEVANCE:
PROMOTION_DECISION:
```

Allowed decisions:

- `PROMOTE_TO_CHALLENGE`;
- `KEEP_DESCRIPTIVE`;
- `NULL`;
- `REJECT`;
- `INCONCLUSIVE`.

---

## 10. Confirmation gate

Confirmation stays locked until all of the following are frozen:

- candidate identity;
- population;
- anchor;
- context;
- direction;
- horizon;
- statistic;
- challenge result;
- acceptance rule;
- code identity.

Once confirmation is opened:

- no candidate redesign;
- no replacement threshold;
- no new horizon;
- no reinterpretation of failure as a new test.

A failed confirmation remains failed.

---

## 11. Agent assignment standard

Every agent task should contain only:

1. role;
2. scientific objective;
3. exact allowed inputs;
4. exact forbidden inputs;
5. required output;
6. stop condition.

Example:

```text
ROLE:
Independent phenotype interpreter

OBJECTIVE:
Interpret the frozen E1 artifact for lifecycle behavior.

ALLOWED INPUTS:
Committed E1 summaries and claim tables.

FORBIDDEN:
Raw confirmation data, strategy design, scanner modification, new thresholds.

OUTPUT:
Claim ledger with SUPPORTED / NULL / INCONCLUSIVE status.

STOP:
Return after the ledger is complete. Do not open the next research stage.
```

Do not ask one agent to be toolsmith, analyst, skeptic, and director at the same time.

---

## 12. Evidence language

Use exact evidence states.

Preferred:

- `SUPPORTED`
- `NULL`
- `INCONCLUSIVE`
- `REJECTED`
- `SURVIVED_CHALLENGE`
- `CONFIRMED`
- `FAILED_CONFIRMATION`

Avoid:

- "looks good";
- "probably meaningful";
- "strong edge";
- "promising trade";
- "seems predictive";

Program R evidence must remain descriptive or informational until later strategy research.

---

## 13. Meeting and review discipline

A review exists to make a decision.

Every review must end with one of:

- pass;
- rework;
- reject;
- defer;
- promote;
- hold.

Do not hold architecture discussions without a specific defect or decision.

Do not create new governance artifacts when the decision fits the existing:

- plan;
- status;
- decision log;
- risk register.

---

## 14. Escalation triggers

Escalate to the research director when:

- a causal rule is ambiguous;
- a frozen contract would need to change;
- confirmation access is requested;
- a defect affects frozen evidence;
- runtime exceeds the pessimistic estimate;
- support collapses below the predeclared minimum;
- two independent interpretations materially conflict;
- a new research question would expand scope;
- a tool or data limitation prevents the exit condition.

Do not escalate routine implementation choices that preserve the approved contract.

---

## 15. End-of-work-package report

Use this format:

```text
WORK_PACKAGE:
DECISION: PASS / REWORK / REJECT / BLOCKED

DELIVERABLE:
EVIDENCE_ID:
DETERMINISM:
INTEGRITY_GATE:
SCIENTIFIC_RESULT:
NULL_OR_INCONCLUSIVE_RESULTS:
KNOWN_LIMITATIONS:
SCOPE_CHANGES:
FORECAST_VS_ACTUAL:
CONFIRMATION_STATUS:
NEXT_APPROVED_WORK_PACKAGE:
```

The report should fit on one screen unless a defect requires more detail.

---

## 16. Team behavior rules

- Produce evidence before narrative.
- Freeze evidence before interpretation.
- Preserve null and inconclusive results.
- Keep native scale and causal time visible.
- Do not manufacture candidate thresholds from descriptive splits.
- Do not treat dependent detectors as independent evidence.
- Do not let infrastructure work consume the project.
- Do not interpret a forecast as a deadline.
- Do not silently change another frozen experiment.
- Do not unlock confirmation early.
- Stop when the work-package exit condition is satisfied.

---

## 17. Success criterion

This skill is working when the team can answer, at any time:

- What work package is active?
- What exact artifact is being produced?
- What proves the package is complete?
- What is blocked?
- What changed?
- What has not changed?
- Is confirmation still locked?
- What is the next approved scientific action?

If those answers require reconstructing history from chat, execution control has failed.
