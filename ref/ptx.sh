#!/bin/sh
# What would clang do? Compile a .cu file to PTX with Homebrew LLVM.
# usage: ref/ptx.sh ref/vadd.cu [-O0|-O2]
set -eu

CLANG=/opt/homebrew/opt/llvm/bin/clang
SM=sm_70
PTX=ptx70

"$CLANG" -x cuda --cuda-device-only --cuda-gpu-arch="$SM" \
  -Xclang -target-feature -Xclang "+$PTX" \
  -nocudainc -nocudalib "${2:--O2}" -S "$1" -o -
