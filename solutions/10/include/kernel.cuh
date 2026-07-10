#pragma once

#include <cuda_runtime.h>
#include <cstdio>


template <int N>
struct ParticleData {
    float[N] posX;
    float[N] posY;
    float[N] posZ;
    float[N] velX;
    float[N] velY;
    float[N] velZ;
    float[N] int_e;
    float[N] mass;
}


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

template <int N> 
void launch_sph_solve(ParticleData<N> *const particles) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    solve_sph<N><<<grid, block>>>(particles);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
__global__ solve_sph(ParticleData<N> *const particles) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    particles.poxX[i] += 1.0;
}


