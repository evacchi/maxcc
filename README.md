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

See `docs/toc.md` for the plan and status. Guides: `docs/chNN.md`.

Shelved: an interpreter for the PTX we emit, to test locally and to model
warps and divergence by hand.
