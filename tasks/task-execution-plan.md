# Task Execution Plan

**Generated:** 2026-03-31
**Last Updated:** 2026-03-31
**PRD Version:** proteus-prd.md (working tree)

## Progress Overview

| Feature                        | Status       | Tasks | Completed | Progress |
|--------------------------------|--------------|-------|-----------|----------|
| F001 - Foundation and Core     | IN_PROGRESS  | 6     | 4         | 67%      |
| F002 - Applet Expansion        | IN_PROGRESS  | 8     | 4         | 63%      |
| F003 - Shell POSIX Roadmap     | IN_PROGRESS  | 6     | 2         | 50%      |
| F004 - Quality and Release     | NOT_STARTED  | 7     | 2         | 29%      |

## Execution Phases

### Phase 1: Foundation Closure
**Goal:** Turn existing core modules into active runtime infrastructure and keep feature wiring accurate.
**Status:** IN_PROGRESS
**Tasks:** T003-T006

| Task | Name                                              | Status       | Priority |
|------|---------------------------------------------------|--------------|----------|
| T003 | Wire sandbox and platform abstractions            | COMPLETED    | P1       |
| T004 | Reconcile feature declarations                    | COMPLETED    | P1       |
| T005 | Add build tooling crate                           | NOT_STARTED  | P2       |
| T006 | Finalize foundation milestone boundaries          | NOT_STARTED  | P2       |

### Phase 2: Text and File Milestones
**Goal:** Complete the PRD's near-term applet roadmap without placeholder implementations.
**Status:** IN_PROGRESS
**Tasks:** T007-T008, T012-T013

| Task | Name                                              | Status       | Priority |
|------|---------------------------------------------------|--------------|----------|
| T007 | Complete v0.2 text processing batch               | COMPLETED    | P1       |
| T008 | Implement v0.3 file utility batch                 | COMPLETED    | P1       |
| T012 | Fill remaining coreutils and misc gaps            | COMPLETED    | P2       |
| T013 | Define applet compliance and help coverage        | COMPLETED    | P2       |

### Phase 3: Shell Maturity
**Goal:** Grow Nereus from a `sh -c` subset into a realistic POSIX shell roadmap.
**Status:** IN_PROGRESS
**Tasks:** T016-T020

| Task | Name                                              | Status       | Priority |
|------|---------------------------------------------------|--------------|----------|
| T016 | Expand POSIX grammar and execution coverage       | COMPLETED    | P1       |
| T017 | Implement interactive line editing and REPL       | NOT_STARTED  | P1       |
| T018 | Add history, completion, and startup files        | NOT_STARTED  | P2       |
| T019 | Add job control and terminal signal handling      | NOT_STARTED  | P2       |
| T020 | Define shell conformance boundaries               | NOT_STARTED  | P2       |

### Phase 4: Platform, Security, and Beta Applets
**Goal:** Deliver network, process, and system milestones with explicit runtime constraints.
**Status:** NOT_STARTED
**Tasks:** T009-T010, T023-T025

| Task | Name                                              | Status       | Priority |
|------|---------------------------------------------------|--------------|----------|
| T009 | Add process and network milestone applets         | NOT_STARTED  | P2       |
| T010 | Add system utility and init milestone applets     | NOT_STARTED  | P2       |
| T023 | Formalize security modes and capability policy    | COMPLETED    | P1       |
| T025 | Add cross-platform validation scaffolding         | NOT_STARTED  | P2       |

### Phase 5: Stabilization and Release Discipline
**Goal:** Add CI, tests, automation, and release structure needed for later RC and stable milestones.
**Status:** NOT_STARTED
**Tasks:** T021-T022, T024, T026-T027

| Task | Name                                              | Status       | Priority |
|------|---------------------------------------------------|--------------|----------|
| T021 | Create repository test structure                  | COMPLETED    | P1       |
| T022 | Add CI workflows                                  | NOT_STARTED  | P1       |
| T024 | Add size and compliance automation                | NOT_STARTED  | P2       |
| T026 | Add release artifact and documentation pipeline   | NOT_STARTED  | P3       |
| T027 | Define contributor acceptance discipline          | NOT_STARTED  | P2       |

## Critical Path
1. T003 → T023 → T009/T010
2. T004 → T007 → T008 → T012 → T014
3. T016 → T017 → T018/T019 → T020
4. T005 → T022 → T024 → T026/T027

## Parallel Execution Opportunities
- T004 and T016 can progress independently once the current baseline is accepted.
- T007 can proceed in parallel with T003 as long as runtime integration decisions remain stable.
- T021 can begin after the next applet or shell increment lands; it does not need to wait for all milestones.
- T013 can run alongside implementation batches to prevent metadata drift.

## Completed Tasks Log
| Task | Feature | Completed   | Duration |
|------|---------|-------------|----------|
| T001 | F001    | 2026-03-31  | Existing |
| T002 | F001    | 2026-03-31  | Existing |
| T003 | F001    | 2026-03-31  | Current  |
| T004 | F001    | 2026-03-31  | Current  |
| T007 | F002    | 2026-03-31  | Current  |
| T008 | F002    | 2026-03-31  | Current  |
| T012 | F002    | 2026-03-31  | Current  |
| T013 | F002    | 2026-03-31  | Current  |
| T015 | F003    | 2026-03-31  | Existing |
| T016 | F003    | 2026-03-31  | Current  |
| T021 | F004    | 2026-03-31  | Current  |
| T023 | F004    | 2026-03-31  | Current  |
