# Feature 002: Applet Expansion

**Feature ID:** F002
**Feature Name:** Applet Expansion
**Priority:** P1 - HIGH
**Target Version:** v0.7.0
**Estimated Duration:** 8-12 weeks
**Status:** IN_PROGRESS

## Overview
This feature tracks the staged implementation of the PRD's 126-appet inventory across coreutils, text processing, file utilities, networking, process tooling, system tooling, editors, and miscellaneous commands. The repository currently has a meaningful coreutils base plus the `grep` family, but most categories remain unimplemented.

The intent is not to finish all applets at once. Instead, this plan groups them by dependency and milestone so each batch lands with real runtime wiring, metadata, and validation rather than placeholder modules.

## Goals
- Expand implemented applets in milestone-sized batches.
- Preserve consistent behavior across dispatch, feature flags, and compliance metadata.
- Prioritize POSIX-heavy applets before Linux-specific tooling.
- Keep every new applet production-ready with tests and measurable scope.

## Success Criteria
- [ ] All tasks completed (T007-T014)
- [ ] Text, file, process, and system categories progress in milestone order
- [ ] New applets are exposed through dispatch and metadata on arrival
- [ ] No placeholder or stub-only applets are introduced
- [ ] Tests passing

## Tasks

### T007: Complete the planned v0.2 text processing batch

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 6 days

#### Description
Finish the text processing tools that the PRD expects for the v0.2 milestone: `sed`, `sort`, `cut`, `tr`, and `uniq`, while keeping the existing `grep`, `egrep`, and `fgrep` implementations stable.

#### Technical Details
This task should reuse the current `applets/src/textutils/` structure and shared core helpers instead of introducing a parallel abstraction layer.

#### Files to Touch
- `applets/src/textutils/mod.rs` (update)
- `applets/src/textutils/sed.rs` (new)
- `applets/src/textutils/sort.rs` (new)
- `applets/src/textutils/cut.rs` (new)
- `applets/src/textutils/tr.rs` (new)
- `applets/src/textutils/uniq.rs` (new)
- `src/main.rs` (update)

#### Dependencies
- T001
- T004

#### Success Criteria
- [ ] All six text applets compile and dispatch correctly
- [ ] Each applet has baseline POSIX-focused behavior
- [ ] Compliance metadata is added for each shipped applet
- [ ] Workspace build and targeted tests pass

### T008: Implement the v0.3 file utility batch

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 8 days

#### Description
Implement the file utility milestone centered on `find`, `xargs`, `tar`, `gzip`, and `awk` support described by the PRD roadmap.

#### Technical Details
Because `awk` is listed under text processing but scheduled alongside file utilities in the milestone roadmap, this task should coordinate cross-category dependencies without duplicating parser infrastructure.

#### Files to Touch
- `applets/src/fileutils/mod.rs` (new)
- `applets/src/fileutils/find.rs` (new)
- `applets/src/fileutils/xargs.rs` (new)
- `applets/src/fileutils/tar.rs` (new)
- `applets/src/fileutils/gzip.rs` (new)
- `applets/src/textutils/awk.rs` (new)
- `applets/src/lib.rs` (update)
- `src/main.rs` (update)

#### Dependencies
- T007

#### Success Criteria
- [ ] File utility module exists and is wired into the workspace
- [ ] Milestone applets are callable through `proteus <applet>`
- [ ] Compression/archive work respects dependency policy
- [ ] Build passes with the new features enabled

### T009: Add process and network milestone applets

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 8 days

#### Description
Implement the milestone-defining process and network tools needed for the beta roadmap: `wget`, `ping`, `nc`, `ps`, `kill`, and `top`.

#### Technical Details
This batch should be planned together because security and platform integration will matter more heavily than in earlier pure-text applets.

#### Files to Touch
- `applets/src/netutils/mod.rs` (new)
- `applets/src/netutils/wget.rs` (new)
- `applets/src/netutils/ping.rs` (new)
- `applets/src/netutils/nc.rs` (new)
- `applets/src/procutils/mod.rs` (new)
- `applets/src/procutils/ps.rs` (new)
- `applets/src/procutils/kill.rs` (new)
- `applets/src/procutils/top.rs` (new)
- `src/main.rs` (update)

#### Dependencies
- T003
- T008

#### Success Criteria
- [ ] Process and network modules are added without placeholder code
- [ ] Security-sensitive applets declare runtime requirements clearly
- [ ] Dispatch and metadata include the new commands
- [ ] Applets build on supported host platforms

### T010: Add system utility and init milestone applets

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 7 days

#### Description
Implement the v0.5 milestone centered on `mount`, `umount`, `dmesg`, `init`, and `mdev`.

#### Technical Details
This work depends heavily on platform and capability abstractions and should not start before foundational runtime wiring is in place.

#### Files to Touch
- `applets/src/sysutils/mod.rs` (new)
- `applets/src/sysutils/mount.rs` (new)
- `applets/src/sysutils/umount.rs` (new)
- `applets/src/sysutils/dmesg.rs` (new)
- `applets/src/misc/` (update)
- `applets/src/lib.rs` (update)
- `src/main.rs` (update)

#### Dependencies
- T003
- T009

#### Success Criteria
- [ ] System applets use platform abstractions where appropriate
- [ ] Capability-sensitive behavior is explicit and testable
- [ ] Init/service scope is bounded to the PRD's minimal expectations
- [ ] Build passes on the primary development platform

### T011: Deliver editor and diff milestone applets

**Status:** NOT_STARTED
**Priority:** P3
**Estimated Effort:** 9 days

#### Description
Implement the late-stage editor and diff tools: `vi`, `diff`, `patch`, `ed`, and `cmp`.

#### Technical Details
This milestone should be delayed until shell and text primitives are mature enough to support reusable parsing and line-handling logic.

#### Files to Touch
- `applets/src/editors/mod.rs` (new)
- `applets/src/editors/vi.rs` (new)
- `applets/src/editors/diff.rs` (new)
- `applets/src/editors/patch.rs` (new)
- `applets/src/editors/ed.rs` (new)
- `applets/src/editors/cmp.rs` (new)
- `src/main.rs` (update)

#### Dependencies
- T007
- T013

#### Success Criteria
- [ ] Editors module is added coherently
- [ ] Minimal `vi` scope is explicitly documented
- [ ] Diff/patch behavior is test-covered for representative cases
- [ ] Build passes with editor features enabled

### T012: Fill remaining coreutils and misc gaps

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 6 days

#### Description
Backfill the unimplemented but already-declared coreutils and miscellaneous commands needed for broad daily usability.

#### Technical Details
This includes commands such as `printf`, `tee`, `env`, `uname`, `id`, `whoami`, `groups`, `date`, `od`, `seq`, and similar gaps already declared in `Cargo.toml`.

#### Files to Touch
- `applets/src/coreutils/mod.rs` (update)
- `applets/src/coreutils/*.rs` (new)
- `applets/src/misc/mod.rs` (new)
- `applets/src/misc/*.rs` (new)
- `src/main.rs` (update)

#### Dependencies
- T004
- T007

#### Success Criteria
- [ ] Coreutils inventory becomes materially closer to the PRD list
- [ ] Miscellaneous module exists and is wired in
- [ ] Added applets include help and compliance metadata
- [ ] No declared command is silently missing from dispatch after implementation

### T013: Define applet-by-applet compliance and help coverage

**Status:** COMPLETED
**Priority:** P2
**Estimated Effort:** 3 days

#### Description
As applets expand, formalize the per-applet compliance level, help output expectations, and documentation readiness.

#### Technical Details
This planning and integration task keeps implementation batches from shipping without metadata discipline.

#### Files to Touch
- `src/main.rs` (update)
- `core/src/compliance.rs` (update)
- `tasks/002-applet-expansion.md` (update)
- `tasks/004-quality-security-release.md` (update)

#### Dependencies
- T002
- T007

#### Success Criteria
- [ ] Every shipped applet has an explicit compliance level
- [ ] Help/reporting output remains consistent across categories
- [ ] Metadata changes are tracked alongside implementation
- [ ] Status documents reflect real coverage

### T014: Track the long-tail applet backlog after milestone coverage

**Status:** NOT_STARTED
**Priority:** P3
**Estimated Effort:** 2 days

#### Description
After milestone-critical applets land, track the remaining long-tail commands and batch them into realistic follow-up work.

#### Technical Details
This avoids prematurely flattening 100+ applets into a single undifferentiated backlog.

#### Files to Touch
- `tasks/tasks-status.md` (update)
- `tasks/task-execution-plan.md` (update)
- `tasks/002-applet-expansion.md` (update)

#### Dependencies
- T012
- T013

#### Success Criteria
- [ ] Remaining applets are categorized by dependency and value
- [ ] Milestone-critical inventory is clearly separated from long-tail work
- [ ] Backlog stays maintainable as implementation grows
- [ ] No completed work is re-planned accidentally

## Performance Targets
- Newly added text applets should stay on a path toward BusyBox-class throughput for common workloads
- Feature growth should not break the sub-1 MB standard-profile target trajectory
- Applet startup overhead should remain negligible versus current dispatch behavior

## Risk Assessment
- The declared applet surface is much larger than the implemented surface; without batching, progress can become too diffuse.
- Network and system tools carry platform-specific and privilege-sensitive complexity.
- Editor work can balloon unless the minimal scope is enforced early.

## Notes
- Current implementation coverage includes a substantial coreutils subset plus `grep`, `egrep`, and `fgrep`.
- `head`, `tail`, and `wc` are implemented in coreutils files today, even though the PRD categorizes them under text processing; keep planning aware of that structural mismatch.
