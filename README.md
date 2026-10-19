# maxxcc

Toy nanopass compiler from a tiny kernel language to NVIDIA PTX. It grows
chapter by chapter, like *Writing a C Compiler* (Sandler).

```
source --lex--> tokens --parse--> L0 --[passes]--> L-ptx --emit--> .ptx --(driver JIT)--> SASS
```

Each pass is small and maps one language to the next. New chapters add
passes; they don't grow existing ones.

## Layout

| path        | what                                   | who      |
|-------------|----------------------------------------|----------|
| `src/lexer.rs`, `src/parser.rs`, `src/ast.rs` | front end | scaffold |
| `src/ptx.rs`, `src/emit.rs` | PTX IR and printer      | scaffold |
| `src/select.rs` (+ future passes) | the interesting part | you |
| `runner/run.c` | loads PTX on the GPU via the driver API | scaffold |
| `ref/`      | clang's PTX for comparison (not golden) | scaffold |
| `docs/chNN.md` | per-chapter guide                    |          |

## Chapters

1. One thread stores a constant. Params, state spaces, `ld`/`st`/`cvta`.
2. `thread_idx.x`: many threads, special registers, address arithmetic.
3. Expressions, `block_idx`/`block_dim`: grids, 32/64-bit widening.
4. `if`: predicates, branches, divergence.
5. `let` and assignments: registers vs memory.
6. `while`: loops, convergence.
7. `shared` memory and barriers.
8. f32, then vector add, reductions, tiled matmul: coalescing and occupancy.

Shelved: an interpreter for the PTX we emit, to test locally and to model
warps and divergence by hand.
