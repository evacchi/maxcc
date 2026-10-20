#!/bin/sh
# End-to-end: compile each examples/*.k, run it on the GPU box, diff the
# runner's output against the example's `// expect:` lines.
#
# Example header:
#   // run: <kernel> <nparams>
#   // expect: param 0 = 42
#
# usage: GPU=user@host tests/gpu.sh
set -eu

: "${GPU:?set GPU=user@host}"
REMOTE=maxxcc-test
OUT=target/gpu

# Build compiler locally, runner remotely.
mkdir -p "$OUT"
cargo build -q
ssh "$GPU" "mkdir -p $REMOTE"
scp -q runner/run.c "$GPU:$REMOTE/"
ssh "$GPU" "cd $REMOTE && cc run.c -o run -I/usr/local/cuda/include -lcuda"

fail=0
for k in examples/*.k; do
    name=$(basename "$k" .k)
    sed -n 's|^// expect: ||p' "$k" > "$OUT/$name.want"

    # Compile errors count as failures, not aborts.
    if ! target/debug/maxxcc "$k" > "$OUT/$name.ptx" 2> "$OUT/$name.got"; then
        echo "FAIL $name (compile)"; cat "$OUT/$name.got"; fail=1; continue
    fi

    scp -q "$OUT/$name.ptx" "$GPU:$REMOTE/"
    ssh "$GPU" "cd $REMOTE && ./run $name.ptx $(sed -n 's|^// run: ||p' "$k")" > "$OUT/$name.got" 2>&1 || true

    if diff -u "$OUT/$name.want" "$OUT/$name.got"; then
        echo "ok   $name"
    else
        echo "FAIL $name"; fail=1
    fi
done
exit $fail
