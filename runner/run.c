// Load a PTX file, launch `kernel<<<1, 1>>>` with one zeroed u32 buffer
// per parameter, print each buffer afterwards.
//
// build (on the NVIDIA box):  cc run.c -o run -I/usr/local/cuda/include -lcuda
// usage:                      ./run file.ptx kernel [nparams]

#include <cuda.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_PARAMS 8
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
        fprintf(stderr, "usage: %s file.ptx kernel [nparams]\n", argv[0]);
        return 2;
    }
    int nparams = argc > 3 ? atoi(argv[3]) : 1;
    if (nparams < 0 || nparams > MAX_PARAMS) {
        fprintf(stderr, "nparams must be 0..%d\n", MAX_PARAMS);
        return 2;
    }

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

    // One zeroed u32 in global memory per parameter.
    CUdeviceptr bufs[MAX_PARAMS];
    void *args[MAX_PARAMS];
    for (int i = 0; i < nparams; i++) {
        CHECK(cuMemAlloc(&bufs[i], sizeof(unsigned)));
        CHECK(cuMemsetD32(bufs[i], 0, 1));
        args[i] = &bufs[i];
    }

    // grid 1x1x1, block 1x1x1: a single thread.
    CHECK(cuLaunchKernel(fn, 1, 1, 1, 1, 1, 1, 0, NULL, args, NULL));
    CHECK(cuCtxSynchronize());

    for (int i = 0; i < nparams; i++) {
        unsigned v;
        CHECK(cuMemcpyDtoH(&v, bufs[i], sizeof v));
        printf("param %d = %u\n", i, v);
    }
    return 0;
}
