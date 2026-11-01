# Table of contents

Plan, not scaffolding. Only the next chapter gets built, and its design is
agreed before any code. Chapters past it are tentative; revise when reached.

Each chapter: language delta, IR/pass delta, GPU concept, runner/test delta.
Every new pass gets a unit test on the IR and a `// expect:` GPU example
(`tests/gpu.sh`). Mutation-check new assertions: reintroduce the bug and
watch the test fail.

## Status

| ch | title                          | status |
|----|--------------------------------|--------|
| 1  | One thread, one store          | done   |
| 2  | Thread identity, indexing      | done   |
| 3  | Expressions and loads          | in progress |
| 4  | `if`                           |        |
| 5  | `let` and assignment           |        |
| 6  | `while`                        |        |
| 7  | Shared memory and barriers     |        |
| 8  | `f32`                          |        |
| 9  | Memory performance             |        |

## 1. One thread, one store (done)

- Lang: `kernel k(p: *u32) { *p = 42; }`
- Passes: `select` (L0 -> L-ptx). `emit`.
- GPU: kernel launch, `.param` (constant bank), state spaces, `ld`/`st`/`cvta`,
  virtual registers, ptxas does real allocation.
- Runner: `runner/run.c`, 1 block x 1 thread, u32 buffer per param.
- Findings (measured, sm_70): `cvta.to.global` + `st.global` and `st.global`
  alone both give `STG`; generic `st` gives `ST`. ptxas trusts the suffix.
  `cvta` can matter for other spaces (shared, local): check in ch 7.

## 2. Thread identity, indexing (done, see ch02.md)

- Lang: `thread_idx.x`, `out[i] = i;`
  e.g. `kernel iota(out: *u32) { out[thread_idx.x] = thread_idx.x; }`
- Passes: `address` (L0 -> L1, explicit byte addressing, constant indexes
  fold into offsets), then `select` (L1 -> L-ptx).
- GPU: grid/block/thread, `%tid.x`, 32-bit index to 64-bit byte offset
  (`mul.wide`/`cvt`), what `.reg` widths mean.
- Runner: `./run x.ptx kernel nparams threads [blocks]`; one u32 per thread
  per param.
- Decisions: fold constant indexes; naive `mov` per `thread_idx.x` use;
  widening instruction left to `select`.
- Findings (sm_70, details in ch02.md): ptxas merges the two `S2R`, fuses
  `ld.param`+`cvta`+`mul.wide`+`add` into one `IMAD.WIDE.U32`; 1025 threads
  fail at launch; 2 blocks of 32 both write slots 0..31 (`%tid.x` is per
  block). Same-address race: stable per launch shape, not predictable.

## 3. Expressions and loads (in progress, see ch03.md)

- 3a. Lang: `+ *`, parens, `block_idx.x`, `block_dim.x`. Goal: global iota,
  2 blocks of 32.
- 3b. Lang: reading `a[i]`. Goal: `c[i] = a[i] + b[i]` on u32, unguarded.
- Passes: `flatten` (L0 -> L0-flat, three-address) before `address`;
  select `ld.global`, `add`, `mul.lo`.
- Decision: `flatten` is its own pass, so `address` sees flat values only.
- GPU: global index = block*dim + tid, multiple blocks, coalescing (first look).
- Open: `verify_ptx` pass once there are enough rules (defined-before-use,
  register widths). See ch 1 test rules.

## 4. `if`

- Lang: `if i < n { ... }`. Full `vadd` with bounds guard.
- Passes: lower `if` to labels and branches; comparisons to predicates
  (`setp`, `@%p bra`).
- GPU: predicates, branches, warps, divergence. Run with n not a multiple
  of the block size.
- Runner: pass scalar args (not only pointers).

## 5. `let` and assignment

- Lang: `let x = e;`, `x = e;`
- Passes: name resolution / scoping; variable -> virtual register.
- GPU: registers vs memory; why locals are free on a GPU.

## 6. `while`

- Lang: `while cond { ... }`
- Passes: lower loops to labels and branches; basic blocks.
- GPU: loops under divergence, reconvergence.

## 7. Shared memory and barriers

- Lang: `shared` arrays, `sync()` (barrier).
- GPU: `.shared`, `bar.sync`, `cvta.to.shared`, per-block scratchpad, races.
- Good test: block-level reduction.
- Open: carry the address space per pointer in the IR instead of hard-coding
  global (see ch 1 finding).

## 8. `f32`

- Lang: `f32` type, literals, casts.
- Passes: typecheck (first need for types beyond u32); float immediates.
- GPU: float regs (`%f`), `add.f32`, `fma`. Float `saxpy`.

## 9. Memory performance

- Tiled matmul. Coalescing, bank conflicts, occupancy
  (`ptxas -v`, `.maxnreg`), spills.
- Compare against a naive version and measure.

## Backlog (unscheduled)

- `verify_ptx` pass between `select` and `emit`.
- Local PTX interpreter modelling warps/divergence (shelved side project).
- Clang reference diff (`ref/ptx.sh`); outputs need not match.
- Timing in the runner (CUDA events).

## Workflow agreements

- Chapter by chapter, smallest runnable step (*Writing a C Compiler* style).
- Nanopass: many small passes; new chapters add passes.
- Claude scaffolds front end, PTX IR, emit, runner; user writes the passes.
- Visibility stays private unless approved.
