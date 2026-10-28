// Load a PTX file, launch `kernel<<<blocks, threads>>>` with one zeroed buffer
// of `blocks * threads` u32 per parameter, print each buffer afterwards.
//
// build (on the NVIDIA box):  cc run.c -o run -I/usr/local/cuda/include -lcuda
// usage:                      ./run file.ptx kernel [nparams [threads [blocks]]]

#include <cuda.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_PARAMS 8
#ifndef MAX_THREADS // override with -DMAX_THREADS=N to probe the limit
#define MAX_THREADS 1024 // per block, all current GPUs
#endif
#define MAX_BLOCKS 64 // ours, keeps output readable
#define LOG_SIZE 8192

#define CHECK(call)                                                       \
    do {                                                                  \
        CUresult r_ = (call);                                             \
        if (r_ != CUDA_SUCCESS) {                                         \
            const char *s_ = "?";                                         \
            cuGetErrorString(r_, &s_);                                    \
            fprintf(stderr, "%s:%d: %s: %s\n", __FILE__, __LINE__, #call, s_); \
            exit(1);                                                      \
        }                                                                 \
    } while (0)

static char *slurp(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        perror(path);
        exit(1);
    }

    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    rewind(f);

    char *buf = malloc(n + 1);
    if (fread(buf, 1, n, f) != (size_t)n) {
        perror(path);
        exit(1);
    }
    buf[n] = 0;
    fclose(f);
    return buf;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: %s file.ptx kernel [nparams [threads [blocks]]]\n", argv[0]);
        return 2;
    }
    int nparams = argc > 3 ? atoi(argv[3]) : 1;
    if (nparams < 0 || nparams > MAX_PARAMS) {
        fprintf(stderr, "nparams must be 0..%d\n", MAX_PARAMS);
        return 2;
    }
    int threads = argc > 4 ? atoi(argv[4]) : 1;
    if (threads < 1 || threads > MAX_THREADS) {
        fprintf(stderr, "threads must be 1..%d\n", MAX_THREADS);
        return 2;
    }
    int blocks = argc > 5 ? atoi(argv[5]) : 1;
    if (blocks < 1 || blocks > MAX_BLOCKS) {
        fprintf(stderr, "blocks must be 1..%d\n", MAX_BLOCKS);
        return 2;
    }
    int slots = blocks * threads;
    size_t bytes = slots * sizeof(unsigned);

    // Device 0, its primary context.
    CUdevice dev;
    CUcontext ctx;
    CHECK(cuInit(0));
    CHECK(cuDeviceGet(&dev, 0));
    CHECK(cuDevicePrimaryCtxRetain(&ctx, dev));
    CHECK(cuCtxSetCurrent(ctx));

    // The driver JIT-compiles PTX to the GPU's native SASS here.
    // On rejection, the log says which line and why.
    char log[LOG_SIZE] = {0};
    CUjit_option opts[] = {CU_JIT_ERROR_LOG_BUFFER, CU_JIT_ERROR_LOG_BUFFER_SIZE_BYTES};
    void *vals[] = {log, (void *)(size_t)LOG_SIZE};
    CUmodule mod;
    if (cuModuleLoadDataEx(&mod, slurp(argv[1]), 2, opts, vals) != CUDA_SUCCESS) {
        fprintf(stderr, "PTX JIT failed:\n%s\n", log);
        return 1;
    }

    CUfunction fn;
    CHECK(cuModuleGetFunction(&fn, mod, argv[2]));

    // One zeroed u32 array in global memory per parameter, one slot per thread
    // across all blocks.
    CUdeviceptr bufs[MAX_PARAMS];
    void *args[MAX_PARAMS];
    for (int i = 0; i < nparams; i++) {
        CHECK(cuMemAlloc(&bufs[i], bytes));
        CHECK(cuMemsetD32(bufs[i], 0, slots));
        args[i] = &bufs[i];
    }

    // grid blocks x 1 x 1, block threads x 1 x 1.
    CHECK(cuLaunchKernel(fn, blocks, 1, 1, threads, 1, 1, 0, NULL, args, NULL));
    CHECK(cuCtxSynchronize());

    unsigned *host = malloc(bytes);
    for (int i = 0; i < nparams; i++) {
        CHECK(cuMemcpyDtoH(host, bufs[i], bytes));
        printf("param %d =", i);
        for (int t = 0; t < slots; t++) {
            printf(" %u", host[t]);
        }
        printf("\n");
    }
    return 0;
}
