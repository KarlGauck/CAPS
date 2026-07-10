#pragma once

#include <cuda_runtime.h>
#include <cstdio>

// Checks a CUDA call and aborts on error
#define CUDA_CHECK(call)                                                        \
    do {                                                                        \
        cudaError_t err = (call);                                               \
        if (err != cudaSuccess) {                                               \
            fprintf(stderr, "CUDA error at %s:%d — %s\n",                      \
                    __FILE__, __LINE__, cudaGetErrorString(err));               \
            std::abort();                                                       \
        }                                                                       \
    } while (0)

template <typename T>
__global__ void add_kernel(const T* a, const T* b, T* out, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) {
        out[i] = a[i] + b[i];
    }
}

template <typename T>
void launch_add(const T* d_a, const T* d_b, T* d_out, int n) {
    constexpr int block = 256;
    int grid = (n + block - 1) / block;
    add_kernel<T><<<grid, block>>>(d_a, d_b, d_out, n);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}
