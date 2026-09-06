# Skill: Detector Research Planning

**Purpose:** Create a detector-specific Program R research plan that is scientifically complete, operationally bounded, and ready for sign-off before execution.

**Applies to:** Detector and market-behavior research only.  
**Does not apply to:** Strategy optimization, execution design, portfolio construction, or retroactive modification of another frozen experiment.

---

## 1. Core rule

Do not copy a prior detector plan verbatim.

Start from the detector's actual role:

- lifecycle/object detector;
- state/regime detector;
- event detector;
- quality detector;
- normalization/context detector.

Then define the research design around that role.

The output must be one readable project plan with a finite definition of done.

---

## 2. Required planning sequence

### Step 1 — Define the scientific object

State:

- detector name;
- instrument;
- producer identity;
- detector role;
- native timeframe or event scale;
- lifecycle or state model;
- valid research anchors;
- what constitutes one scientific unit.

Do not define early cohorts using future completion.

### Step 2 — Freeze the causal clock

Specify:

- lawful availability timestamp;
- tie-break rule;
- provenance timestamp;
- what data are knowable at anchor time;
- native-scale preservation;
- censoring treatment.

The plan must explicitly distinguish:

- event/occurrence time;
- availability time;
- future outcome time.

### Step 3 — Define boundary handling before research

Every detector plan must include an explicit left-boundary policy before the first behavioral scan.

Choose and document one or more of:

- warm-state reconstruction from pre-window data;
- washout period;
- left-truncated cohort flag;
- exclusion from analyses requiring unavailable prior history;
- lawful inclusion in analyses that only require current known state.

Do not defer this as an informal implementation detail.

Right-boundary handling must also be explicit.

### Step 4 — Define the measurement clocks

Use only clocks justified by the detector.

Possible clocks:

- lifecycle/event clock;
- fixed forward market horizons;
- native-bar horizons;
- antecedent windows;
- time-to-transition;
- time-to-resolution.

Every horizon needs a reason.

Do not add arbitrary horizons after seeing outcomes.

### Step 5 — Define context families

Classify context by information role.

Use the exposure classes:

- **E1 PHENOTYPE** — detector itself;
- **E2 SAME_DOMAIN** — same-domain context;
- **E3 CROSS_DOMAIN** — external detector or structural context.

These are permission classes, not mandatory filters.

Preserve native timeframe and lineage.

Do not count dependent detectors as independent confluence without evidence.

### Step 6 — Define the research questions

Questions must be detector-shaped.

Examples by role:

**Lifecycle/object**
- formation;
- persistence;
- transitions;
- termination;
- censoring;
- prospective behavior.

**State/regime**
- occupancy;
- transition matrix;
- persistence;
- duration;
- consequences of state change.

**Event**
- incidence;
- clustering;
- magnitude;
- recurrence;
- prospective consequence.

**Quality**
- failure incidence;
- persistence;
- contamination;
- consequence of degraded quality.

**Normalization/context**
- distribution;
- stability;
- regime dependence;
- lawful use as conditioning information.

### Step 7 — Define evidence levels

Use a staged evidence model.

- **L1 Phenotype:** reproducible descriptive behavior.
- **L2 Conditional information:** behavior changes under lawful context.
- **L3 Incremental information:** survives appropriate controls and confounder challenge.
- **L4 Confirmed information:** frozen L3 relationship reproduces on untouched confirmation data.

Strategy evidence begins later.

Never call L1–L4 an executable trading edge.

### Step 8 — Define candidate promotion before candidate results are reviewed

Discovery should be broad but inexpensive.

Challenge should be selective and expensive.

A candidate promotion rule should consider:

- support;
- chronological breadth;
- effect magnitude;
- direction stability;
- concentration by day/session;
- information-path plausibility;
- dependency/lineage;
- censoring;
- decision relevance.

A p-value alone is never sufficient.

### Step 9 — Separate discovery, challenge, and confirmation

The default Program R structure is:

1. causal descriptive discovery;
2. candidate formation;
3. method challenge and incremental controls;
4. frozen untouched confirmation;
5. characterization.

Do not run expensive resampling across an entire exploratory grid unless that design has been explicitly approved for that experiment.

Do not silently retrofit an existing frozen experiment to this structure.

### Step 10 — Define the work breakdown

Each work package must contain:

- purpose;
- inputs;
- work;
- required artifact;
- exit condition;
- stop condition.

A work package is complete because its evidence requirement is satisfied, not because time elapsed.

### Step 11 — Define resource and concurrency limits

Default:

- one deterministic toolsmith/executor;
- one analyst/interpreter;
- one skeptic only when independent challenge is useful.

Keep active concurrency near two.

Do not solve uncertainty by launching many duplicate agents.

### Step 12 — Define the schedule as a forecast

Use three-point estimates or another explicit forecasting method.

The plan must say:

- forecast is not a guarantee;
- measured execution replaces assumptions;
- reforecast threshold;
- escalation point when pessimistic estimate is exceeded.

Do not use a forecast as a scientific deadline.

### Step 13 — Define the risk register

At minimum cover:

- leakage;
- future conditioning;
- left truncation;
- right censoring;
- duplicate occupancy;
- temporal dependence;
- regime/session confounding;
- lineage duplication;
- low support;
- search explosion;
- performance;
- AI over-interpretation;
- confirmation contamination;
- infrastructure creep.

### Step 14 — Define final deliverables

At minimum:

- final research report;
- machine-readable knowledge ledger;
- decision-relevance summary;
- decision log.

Every major claim must have a terminal status:

- `CONFIRMED`;
- `REJECTED`;
- `NULL`;
- `INCONCLUSIVE`.

---

## 3. Mandatory anti-drift rules

A detector plan must include these rules.

### Rule A — Infrastructure is not research progress

Infrastructure work counts only when it directly enables or repairs a defined scientific work package.

### Rule B — Repair the smallest invalid component

When a defect appears:

1. state the scientific consequence;
2. reproduce it;
3. repair the smallest responsible component;
4. rerun only affected evidence;
5. return to the plan.

### Rule C — Another experiment's frozen design does not change implicitly

Approval of one detector plan does not authorize changing another frozen experiment.

Any change to an existing frozen design requires its own explicit decision record.

### Rule D — Confirmation remains untouched

Development findings cannot borrow from confirmation data.

### Rule E — AI does not become the evidence engine

AI may:

- generate questions;
- interpret frozen outputs;
- critique methods;
- synthesize claims;
- review code.

Deterministic evidence production remains in the approved research pipeline.

---

## 4. Required plan structure

Every detector plan should contain these sections:

1. objective and definition of done;
2. scope;
3. detector role and scientific unit;
4. frozen inputs;
5. causal clock;
6. left- and right-boundary policy;
7. measurement design;
8. context design;
9. research questions;
10. evidence levels;
11. incremental-information controls;
12. candidate-promotion rule;
13. work breakdown;
14. resource plan;
15. schedule and forecast control;
16. risk register;
17. change control;
18. final deliverables;
19. sign-off checklist.

---

## 5. Sign-off standard

A plan is `APPROVED` only when a research director can answer all of these without guessing:

- What exactly is being studied?
- What is one scientific observation?
- What information is lawful at each anchor?
- How are left and right boundaries handled?
- What is measured?
- Why are those horizons or clocks used?
- What contexts are allowed?
- What distinguishes discovery from challenge?
- What causes a candidate to advance?
- What remains untouched for confirmation?
- What artifacts prove each stage is complete?
- What causes execution to stop?
- What is explicitly out of scope?

If any answer is ambiguous, the plan is `REWORK`.

---

## 6. Output style

Write for a research director and implementation team.

Use:

- compact prose;
- tables where they improve traceability;
- explicit states and thresholds;
- exact work-package exits;
- concrete artifact names.

Avoid:

- internal monologue;
- motivational language;
- vague scientific language;
- excessive governance prose;
- generic framework discussion;
- hidden assumptions;
- copy-pasting another detector's plan without adaptation.
