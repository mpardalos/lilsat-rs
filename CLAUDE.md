# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## IMPORTANT: Development Constraints

**DO NOT implement SAT solver logic directly.** This is a learning project where the user is implementing the solver themselves.

You may help with:
- Rust language questions and syntax
- Testing infrastructure and test harness code
- Benchmarking infrastructure
- Build system and tooling
- General Rust best practices

You must NOT:
- Write or modify SAT solver algorithm code
- Implement CDCL logic, unit propagation, conflict analysis, etc.
- Fix bugs in the solver logic
- Optimize or refactor the solver implementation

If asked about solver implementation, provide explanations and guidance only - do not write the code.

## Project Overview

This is a CDCL (Conflict-Driven Clause Learning) SAT solver implemented in Rust for learning purposes. It was translated from a Haskell implementation and solves boolean satisfiability problems in CNF (Conjunctive Normal Form) format.

## Build and Run

```bash
# Build the project
cargo build

# Run the solver on a CNF file
cargo run -- <path-to-cnf-file>

# Build optimized release version
cargo build --release
cargo run --release -- <path-to-cnf-file>
```

The solver accepts DIMACS CNF format files and outputs either SAT with a satisfying valuation or UNSAT.

## Architecture

### Core Algorithm (CDCL)

The solver implements the modern CDCL algorithm with these key phases:

1. **Decision**: Choose an unassigned literal (src/main.rs:270-279)
2. **Unit Propagation**: Derive implications from unit clauses (src/main.rs:281-307)
3. **Conflict Analysis**: When conflicts occur, analyze using 1-UIP to learn a conflict clause (src/main.rs:329-344)
4. **Backtracking**: Non-chronological backjumping to an appropriate decision level (src/main.rs:346-356)
5. **Clause Learning**: Add learned clauses to the formula (src/main.rs:266-268)

### Key Data Structures

- **Literal**: Signed integer representing a variable or its negation
- **Clause**: Vector of literals (disjunction)
- **Formula**: Vector of clauses (conjunction)
- **Valuation**: Tracks variable assignments with decision levels and reasons
- **VarData**: Contains boolean value + reason (Decision or Implied with antecedent clause)
- **Reason**: Records why a variable was assigned (decision vs unit propagation from a clause)

### Critical Implementation Details

**Decision Levels**: The solver maintains a `level` counter that increments with each decision. When conflicts occur, the solver backtracks to a computed earlier level (not necessarily the previous one - this is non-chronological backtracking).

**1-UIP (First Unique Implication Point)**: The conflict analysis (src/main.rs:309-327) iteratively resolves the conflict clause with antecedent clauses until reaching the 1-UIP - a single literal from the current decision level. This creates effective learned clauses.

**Clause Resolution**: `resolve_mut` (src/main.rs:70-80) implements resolution by merging clauses and removing complementary literals. This is the core operation in conflict analysis.

**Valuation Tracking**: Each variable stores not just its value but also the reason for assignment and decision level. This metadata enables conflict analysis and intelligent backtracking.
