# Feature 003: Shell POSIX Roadmap

**Feature ID:** F003
**Feature Name:** Shell POSIX Roadmap
**Priority:** P1 - HIGH
**Target Version:** v0.6.0
**Estimated Duration:** 5-7 weeks
**Status:** IN_PROGRESS

## Overview
This feature tracks the evolution of the Nereus shell from the current non-interactive subset into the PRD's intended POSIX shell implementation. The repository already contains lexer, parser, interpreter, and builtin modules, and `sh -c` works for basic command execution, pipelines, and simple environment expansion.

The remaining gap is significant. Interactive mode, startup files, job control, persistent history, completion, error-context improvements, and broader POSIX grammar support are still absent. This roadmap keeps shell work incremental and tied to observable runtime behavior rather than abstract parser ambitions.

## Goals
- Stabilize the existing non-interactive shell subset.
- Expand POSIX syntax coverage in milestone-sized slices.
- Add interactive shell capabilities only after parser and interpreter behavior are solid.
- Keep shell usability and correctness visible through focused tests.

## Success Criteria
- [ ] All tasks completed (T015-T020)
- [ ] `sh -c` behavior covers a broader POSIX subset reliably
- [ ] Interactive mode exists with bounded first-release scope
- [ ] Startup files, history, and job control are staged realistically
- [ ] Tests passing

## Tasks

### T015: Consolidate the current non-interactive shell baseline

**Status:** COMPLETED
**Priority:** P1
**Estimated Effort:** 3 days

#### Description
Establish the first usable shell baseline with tokenization, parsing, interpretation, pipelines, simple variable expansion, and builtin execution.

#### Technical Details
The repository already contains lexer, parser, interpreter, builtin modules, and `run_shell()` support for `sh -c` command execution.

#### Files to Touch
- `shell/src/lib.rs` (update)
- `shell/src/lexer.rs` (update)
- `shell/src/parser.rs` (update)
- `shell/src/interpreter.rs` (update)
- `shell/src/builtins.rs` (update)

#### Dependencies
- T001

#### Success Criteria
- [x] `sh -c` executes commands
- [x] Simple pipelines work
- [x] Basic variable expansion exists
- [x] Builtins are available for core shell flow

### T016: Expand POSIX grammar and execution coverage

**Status:** IN_PROGRESS
**Priority:** P1
**Estimated Effort:** 6 days

#### Description
Broaden parser and interpreter support to cover more of the POSIX shell command language before interactive features are attempted.

#### Technical Details
This includes tighter handling for quoting, redirections, compound command forms, control operators, exit statuses, and shell positional parameter behavior.

#### Files to Touch
- `shell/src/lexer.rs` (update)
- `shell/src/parser.rs` (update)
- `shell/src/interpreter.rs` (update)
- `shell/src/builtins.rs` (update)
- `shell/src/lib.rs` (update)

#### Dependencies
- T015

#### Success Criteria
- [ ] Parser accepts a materially broader POSIX subset
- [ ] Interpreter behavior is validated with focused shell tests
- [ ] Redirection and control-flow semantics are documented through tests
- [ ] Regressions in existing `sh -c` support are prevented

### T017: Implement interactive line editing and REPL entry path

**Status:** NOT_STARTED
**Priority:** P1
**Estimated Effort:** 5 days

#### Description
Add the first interactive shell mode with a bounded REPL, line editing, and terminal-aware input loop.

#### Technical Details
The PRD expects editline-compatible interaction with emacs and vi modes. For initial delivery, the work should focus on getting a stable interactive loop and basic editing primitives in place before richer UX features.

#### Files to Touch
- `shell/src/lib.rs` (update)
- `shell/src/line_editing.rs` (new)
- `shell/src/interpreter.rs` (update)
- `shell/src/builtins.rs` (update)

#### Dependencies
- T016

#### Success Criteria
- [ ] Running `proteus sh` enters an interactive loop
- [ ] Basic line editing works on a terminal
- [ ] Interactive mode exits cleanly on EOF and shell exit
- [ ] Existing script mode remains intact

### T018: Add history, completion, and startup files

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 4 days

#### Description
Layer in interactive quality-of-life features once the base REPL is stable.

#### Technical Details
This task covers history persistence, reverse search groundwork, simple completion, and startup file loading such as `/etc/proteus/profile` and `~/.proteusrc`.

#### Files to Touch
- `shell/src/completion.rs` (new)
- `shell/src/line_editing.rs` (update)
- `shell/src/lib.rs` (update)
- `shell/src/interpreter.rs` (update)

#### Dependencies
- T017

#### Success Criteria
- [ ] Shell can load startup files in a deterministic order
- [ ] History persists across sessions with bounded scope
- [ ] Basic command/path completion works
- [ ] Interactive startup remains responsive

### T019: Add job control and terminal signal handling

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 5 days

#### Description
Implement foreground/background process management and terminal job control primitives.

#### Technical Details
This should include `fg`, `bg`, `jobs`, process group management, and signal handling aligned with supported host behavior.

#### Files to Touch
- `shell/src/interpreter.rs` (update)
- `shell/src/builtins.rs` (update)
- `shell/src/lib.rs` (update)
- `core/src/platform/mod.rs` (update)
- `core/src/platform/linux.rs` (update)

#### Dependencies
- T017
- T003

#### Success Criteria
- [ ] Basic job table exists
- [ ] `fg`, `bg`, and `jobs` operate on tracked processes
- [ ] Terminal signals are handled without corrupting shell state
- [ ] Behavior is documented for unsupported platforms

### T020: Define shell conformance and extended feature boundaries

**Status:** NOT_STARTED
**Priority:** P2
**Estimated Effort:** 2 days

#### Description
Separate required POSIX shell behavior from optional `shell-extended` features so implementation remains disciplined.

#### Technical Details
This task documents what belongs to base shell delivery and what is deferred to opt-in extensions such as `[[ ]]`, arrays, brace expansion, and extra variables.

#### Files to Touch
- `tasks/003-shell-posix-roadmap.md` (update)
- `tasks/tasks-status.md` (update)
- `tasks/task-execution-plan.md` (update)
- `Cargo.toml` (update)

#### Dependencies
- T016

#### Success Criteria
- [ ] POSIX-required scope is clearly separated from optional shell extensions
- [ ] Milestone planning references the same boundary everywhere
- [ ] Feature flags remain aligned with actual shell behavior
- [ ] Future shell work is easier to prioritize

## Performance Targets
- Non-interactive `sh -c` startup remains on a path toward low-millisecond startup
- Interactive shell idle memory stays aligned with the PRD's low-RSS target trajectory
- Parser and interpreter growth should avoid pathological behavior on malformed input

## Risk Assessment
- Shell work can sprawl rapidly if parser, REPL, and job control evolve simultaneously.
- Interactive terminal support is platform-sensitive and should be staged after grammar stability.
- Bash-compatibility pressure can dilute the POSIX-first scope unless boundaries are kept explicit.

## Notes
- Current shell state: non-interactive subset only; interactive mode explicitly returns `not yet implemented` in `shell/src/lib.rs`.
