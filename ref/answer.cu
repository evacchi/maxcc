// Reference for examples/answer.k. See ref/ptx.sh.
#define __global__ __attribute__((global))

extern "C" __global__ void answer(unsigned *out) {
  *out = 42;
}
