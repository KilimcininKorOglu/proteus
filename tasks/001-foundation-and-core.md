# Feature 001: Foundation and Core

**Feature ID:** F001
**Feature Name:** Foundation and Core
**Priority:** P1 - CRITICAL
**Target Version:** v0.2.0
**Estimated Duration:** 3-4 weeks
**Status:** IN_PROGRESS

## Overview
This feature covers the foundational crates and runtime surfaces that every later Proteus milestone depends on. It includes the shared core library, dispatch/runtime entry points, build profile definitions, and the metadata systems that describe applet capabilities.

The current repository already contains substantial progress in this area. Core modules for argument parsing, I/O, UTF-8 handling, globbing, regex, sandboxing, permissions, compliance metadata, and platform abstraction exist. The remaining work is to finish integration, expand feature coverage, and convert foundational modules from isolated building blocks into actively consumed runtime infrastructure.

## Goals
- Stabilize the shared `proteus-core` API for subsequent applet work.
- Close the gap between declared Cargo features and real runtime integration.
- Ensure dispatch, compliance reporting, and profile composition stay synchronized as new applets land.
- Prepare the codebase for later security, testing, and release automation work.

## Success Criteria
- [ ] All tasks completed (T001-T006)
- [x] Core runtime and metadata layers are actively used by implemented applets
- [ ] Cargo feature graph matches shipped applet inventory without dead declarations
- [ ] Build profiles are validated by successful workspace builds
- [ ] Tests passing

## Tasks

### T001: Implement core library primitives

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 4 days

#### Description
Implement the foundational `proteus-core` modules required by applets and shell code.

#### Technical Details
Completed modules include argument parsing, buffered I/O, UTF-8 helpers, locale helpers, path handling, globbing, regex support, permission parsing, sandbox abstractions, and platform backends.

#### Files to Touch
- `core/src/lib.rs` (update)
- `core/src/args.rs` (update)
- `core/src/io.rs` (update)
- `core/src/utf8.rs` (update)
- `core/src/locale.rs` (update)
- `core/src/path.rs` (update)
- `core/src/glob.rs` (update)
- `core/src/regex.rs` (update)
- `core/src/permissions.rs` (update)
- `core/src/sandbox.rs` (update)
- `core/src/platform/mod.rs` (update)
- `core/src/platform/linux.rs` (update)
- `core/src/platform/freebsd.rs` (update)
- `core/src/platform/macos.rs` (update)

#### Dependencies
- None

#### Success Criteria
- [x] Core modules compile in the workspace
- [x] Regex, sandbox, permissions, and platform abstractions exist
- [x] Modules are exported through `proteus-core`
- [x] Unit tests passing where implemented

### T002: Add compliance metadata and reporting surface

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 1.5 days

#### Description
Add a structured metadata model that reports category, compliance level, and help data for implemented applets.

#### Technical Details
The repository already includes compliance metadata definitions and runtime reporting via `proteus --list-full` and `proteus --posix-info`.

#### Files to Touch
- `core/src/compliance.rs` (update)
- `src/main.rs` (update)

#### Dependencies
- T001

#### Success Criteria
- [x] Compliance metadata model exists
- [x] `--list-full` prints metadata-backed output
- [x] `--posix-info` works for implemented applets
- [x] Metadata stays sortable and queryable at runtime

### T003: Wire sandbox and platform abstractions into runtime execution

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 3 days

#### Description
Integrate the existing sandbox and platform layers into actual applet execution paths instead of leaving them as unused infrastructure.

#### Technical Details
This work should define where sandbox mode selection lives, when profiles are applied, how `--no-sandbox` and future strict/permissive modes are parsed, and how platform services are injected into applets that need OS-specific behavior.

#### Files to Touch
- `src/main.rs` (update)
- `core/src/sandbox.rs` (update)
- `core/src/platform/mod.rs` (update)
- `applets/src/lib.rs` (update)
- `applets/src/coreutils/*.rs` (update)

#### Dependencies
- T001
- T002

#### Success Criteria
- [ ] Runtime exposes sandbox mode selection
- [ ] Implemented applets can opt into sandbox profiles safely
- [ ] Platform abstraction is consumed by at least one real runtime path
- [ ] Workspace build passes with integration enabled

### T004: Reconcile feature declarations with implemented applets

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 2 days

#### Description
Audit the feature matrix so Cargo features, dispatch entries, metadata, and source files remain consistent as the project expands.

#### Technical Details
`Cargo.toml` already declares the long-term applet inventory, while the current codebase implements only a subset. This task tracks the ongoing synchronization work needed to prevent stale features, broken dispatch entries, or undocumented applet states.

#### Files to Touch
- `Cargo.toml` (update)
- `src/main.rs` (update)
- `applets/src/lib.rs` (update)
- `applets/src/coreutils/mod.rs` (update)
- `applets/src/textutils/mod.rs` (update)

#### Dependencies
- T002

#### Success Criteria
- [ ] Feature declarations remain buildable
- [ ] Dispatch table matches compiled applets
- [ ] Metadata list matches actual availability
- [ ] No newly added applet ships without feature wiring

### T005: Add build tooling crate and size/compliance commands

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 4 days

#### Description
Create the build tooling described by the PRD for feature matrix validation, size reporting, and compliance checks.

#### Technical Details
The repository does not yet contain the planned `proteus-build` or `build-tools/` utilities. This task introduces the minimal structure needed to generate repeatable build metadata.

#### Files to Touch
- `Cargo.toml` (update)
- `build-tools/size-report/` (new)
- `build-tools/compliance-check/` (new)
- `tasks/004-quality-security-release.md` (update)

#### Dependencies
- T004

#### Success Criteria
- [ ] Build tooling crate or utilities exist in the workspace
- [ ] Size report command can inspect a built binary
- [ ] Compliance check command validates implemented metadata
- [ ] Tooling is documented for contributors

### T006: Finalize foundation milestone boundaries

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 1 day

#### Description
Document which foundational scope belongs to v0.2 versus later milestones so execution stays realistic.

#### Technical Details
This task converts the current implementation snapshot into a stable milestone baseline for later planning, especially where PRD ambition exceeds the current repository state.

#### Files to Touch
- `tasks/tasks-status.md` (update)
- `tasks/task-execution-plan.md` (update)
- `tasks/001-foundation-and-core.md` (update)

#### Dependencies
- T003
- T004

#### Success Criteria
- [ ] Foundation scope is explicitly bounded
- [ ] Remaining foundation risks are listed
- [ ] Downstream features reference the correct baseline
- [ ] Status documents stay aligned with actual code

## Performance Targets
- Workspace builds succeed without regressing the current binary baseline
- Dispatch and metadata queries remain instantaneous for implemented applets
- Foundation work keeps the minimal profile on a path toward sub-500 KB goals

## Risk Assessment
- The PRD is broader than the current repository, so planning can drift into speculative work if milestone boundaries are not enforced.
- Feature declarations may outpace implementation and create false signals unless regularly reconciled.
- Sandbox integration carries cross-platform behavior risks and should be staged carefully.

## Notes
- This feature was reconstructed from the current repository because `tasks/` was absent in the working tree.
- Current repo license is MIT-only, while the PRD still states dual licensing. Treat that as a planning warning, not an implementation instruction.
