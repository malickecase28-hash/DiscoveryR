# Skill: Engineering Execution Discipline

This skill governs how a single task actually gets written and run. The Detector Research Planning skill decides what gets studied. The Research Project Execution skill governs work-package schedule and governance. Neither one stops a work package from costing four hours of blind trial and error when it should have cost fifteen minutes. This skill closes that gap.

## 1. Core rule

No code is written before the task is observed, planned, and given a measured-cost estimate with named checkpoints. This is not a formality gate before "real work" starts. It is the mechanism that prevents rerunning the same slow, wrong attempt three times before anyone asks why.

If a task cannot state its own predicted cost and its own checkpoints before it runs, it is not ready to run.

## 2. The required sequence

Every task, before a line of new code is written:

1. **Read the plan you were given.** If it is broken or incomplete, flag or fix it. Guessing and moving on is not a substitute.
2. **Read the current code.** Before writing anything new, check whether an existing tool in this repo already does this job. Rebuilding a slower version of something that already exists is not a rounding error, it is the single most expensive mistake this program has already made once. Name the tool you checked and why it does or doesn't fit, even if the answer is "nothing existing covers this."
3. **Understand the data.** Row counts, partition layout, column set, actual scale, before deciding whether the interaction pattern needs to change. Do not guess at scale when the manifest or a one-line count query will tell you exactly.
4. **State the actual goal in plain terms.** What result gets produced, and to what precision. If you can't say this in one sentence, you don't understand the task yet.
5. **Know the hardware.** The real ceiling on this machine: how much can genuinely run in parallel, what the realistic floor is for this class of workload.
6. **Scope the whole problem**, not just the piece directly in front of you.
7. **Do the rough math and write it down before running anything.** Section 5 defines exactly what this estimate must contain. A hand-wavy "should take a couple hours" is not this step.
8. **Run it, then check your own math.** A small deviation (see Section 5's frozen multiplier) is noise; move on. A large deviation is not noise. It means the task was not understood, and now you find out why before you run it again.

## 3. Parallelism is the default, not a naive one

Default to parallel wherever the workload allows it. But "default to parallel" does not mean parallelizing a step that has a genuine sequential dependency and calling it done. It means correctly identifying which part of the task is actually sequential and which part isn't, and not leaving the parallel part serial out of habit.

**Concrete example, already in this repository.** `crates/research-tape/src/ap001_drift_burst.rs` is 2,590 lines, has zero uses of `rayon`, `par_iter`, `thread::spawn`, or any other parallel primitive anywhere in its dependency tree, and reads its six Parquet parts with a single sequential `for (part_index, path) in parts.iter().enumerate()` loop. It still benchmarked at 118,673 rows/s. That's fine in absolute terms, but it is not evidence the design is correct, it is evidence Arrow's columnar decode is fast even when the surrounding orchestration is naive.

The Drift-Burst episode reconstruction genuinely requires ticks to be consumed in strict causal order. That part cannot be naively parallelized without corrupting anchor reconstruction, and no skill should tell you to `par_iter` over ticks just because a rule says "parallel by default." But the six parts are independent files on disk, and decoding part N+1 into Arrow batches does not need to wait for the causal state machine to finish consuming part N. That is a prefetch/pipeline problem, not a "make everything parallel" problem. The fix is a read-ahead thread or async prefetch queue in front of the sequential consumer, not a blanket parallel rewrite.

**Rule:** every task states, in one line, which part of the work is sequential-by-necessity and which part is parallel-by-neglect. A task that can't answer this has not actually been thought through, regardless of how fast it benchmarks.

## 4. Instrumentation is mandatory, not a nice-to-have

Any task that scans, processes, or iterates over more than a trivial row count emits named, timestamped checkpoints per logical phase; not one print statement at the end that says "done."

If a section takes 20 seconds and nobody put a checkpoint before and after it, nobody knows if that 20 seconds is normal or the entire problem. Without checkpoints, a slow run only tells you "it was slow." With them, it tells you which segment was slow, and that's the difference between diagnosing a problem and rerunning a slot machine.

## 5. The estimate replaces the hour guess

A three-point hour estimate (`0.75h / 1.25h / 2h`) with no stated basis is not a forecast. It is a number someone made up that later becomes an excuse to burn the pessimistic value without ever asking whether it was true. AP-001's own WP1 did exactly this: forecast was `0.75h most likely / 1.5h pessimistic`; actual was `~3.5h active`, roughly double the pessimistic estimate, and the schedule format gave nobody a reason to stop and ask why until a human said stop. Every AP-series schedule table in this program (AP-001 through AP-006) still uses this exact format today, even after the V3/V2 hardening pass fixed three other specification gaps. It is the one gap that pass didn't touch.

Replace it with this format for every task and every work package:

```text
TASK:

KNOWN_THROUGHPUT: <measured rows/s or "NOT_YET_MEASURED — first checkpoint measures it">
INPUT_SIZE: <row count / byte count / event count, from the manifest or a count query, not a guess>
PREDICTED_WALL_TIME: <input size / throughput, or "unknown until first checkpoint" if throughput is unmeasured>
CHECKPOINTS: <named phase boundaries the run will report against>
DEVIATION_RULE: actual > 3x predicted at any checkpoint -> stop, do not continue to the next checkpoint, escalate per Section 6
```

Once a task has run once, its measured throughput becomes the known throughput for every later task of the same shape. AP-001's WP1 already measured `118,673 rows/s` for a drift_burst tick scan; WP2 on the same detector and surface should predict from that number, not re-guess in hours. A schedule table that has real throughput sitting in an already-committed report and still estimates in vague hour ranges is not using the evidence it already has.

The 3x multiplier is a starting default, not a law of physics. If a task's own risk is genuinely open-ended (an unresolved semantic question, not a compute-cost question), that risk gets named explicitly as its own line, not folded into a bigger hour range that hides it. Compute-cost uncertainty and semantic-discovery uncertainty are different kinds of unknowns and do not belong in the same number.

## 6. Composition with existing skills

This skill does not reimplement work other installed skills already do well. It calls them at the right moment.

- **`research`** — use before writing code whenever the task depends on an external API, library behavior, or documented format you have not verified firsthand. Do not guess at how a library works when a quick lookup answers it.
- **`ponytail`** (default intensity: full) — use when deciding how much code a step actually needs. A step that one line can do is not allowed to become twenty because it was faster to write that way. This applies inside every step of Section 2, not as a separate pass afterward.
- **`diagnosing-bugs`** — use the moment a checkpoint shows a deviation past the frozen multiplier in Section 5, or when a result doesn't match the predicted output. This is the mechanism Section 2 Step 8 points to. Do not rerun the same attempt hoping for a different number; open the diagnosis loop.
- **`show-me-your-work`** — use to log the estimate, the checkpoint readings, and the actual outcome as one row per task. AP-001's `decisions.tsv` already does this at the project-decision level; this pushes the same discipline down to the individual task level, so "we predicted X and got Y" has a permanent, reviewable trail instead of living only in a chat transcript.
- **`figure-it-out`** — use when a single task is big enough that the eight-step sequence in Section 2 needs its own sub-plan (a multi-day migration, not a single work package). Don't run this skill's lightweight loop on something that actually needs the heavier one.
- **`retro`** — use at the close of a work package or session to capture what the estimate got wrong and why, and feed that back into the next task's known-throughput baseline. This is the structural fix for "estimates become excuses": the retro output changes what the next estimate is allowed to assume, instead of the same guess getting made again next time.

## 7. Mandatory anti-drift rules

**Rule A — An estimate without a stated throughput basis is not a schedule entry.** It's a guess wearing a number. Reject any task or work-package schedule line that doesn't cite either measured throughput or an explicit `NOT_YET_MEASURED` flag with a plan to measure it at the first checkpoint.

**Rule B — A rerun without a stated hypothesis for why the first run was wrong is not iteration, it's a slot machine.** Every rerun states, in one line, what specifically is expected to be different this time and why, before it runs.

**Rule C — Using a slower, newly-written tool when a faster existing one already does the job requires a stated reason, not silence.** "I didn't check" is not a reason.

**Rule D — Being AI doesn't waive any of the above.** Writing code quickly is not the same as understanding the task. An agent that skips Section 2 because it can type fast is repeating the exact failure this skill exists to stop.

## 8. Ready-to-run checklist

A task is ready to run only when it can state:

- [ ] which existing tool in the repo was checked before writing something new, and why it does or doesn't fit
- [ ] actual data scale, from a real count, not an assumption
- [ ] the goal in one plain sentence
- [ ] which part of the work is sequential-by-necessity and which is parallel-by-neglect
- [ ] known throughput or an explicit not-yet-measured flag
- [ ] predicted wall time computed from that throughput, not guessed in an hour range
- [ ] named checkpoints the run will report against
- [ ] the deviation rule that triggers `diagnosing-bugs` instead of a blind rerun
- [ ] where the estimate and outcome will be logged (`show-me-your-work`)

If any box can't be checked, the task isn't planned. It's a hope.
