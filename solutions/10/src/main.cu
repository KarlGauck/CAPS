#include <cstdio>
#include <vector>
#include <numeric>

#include "kernel.cuh"

int main() {
    constexpr int N = 1024;

    ParticleData<N>> host_particles {};

    float *device_particles;

    CUDA_CHECK(cudaMalloc(&device_particles, sizeof(ParticleData<N>)));
    CUDA_CHECK(cudaMemcpy(device_particles, &host_particles, sizeof(ParticleData<N>), cudaMemcpyHostToDevice));

    launch_sph_solve<N>(particles);

    CUDA_CHECK(cudaMemcpy(&host_particles, device_particles, sizeof(ParticleData<N>), cudaMemcpyDeviceToHost));

    // Verify
    bool ok = true;
    for (int i = 0; i < N; ++i) {
        if (particles.posX[i] != 1.0) {
            fprintf(stderr, "Mismatch at %d: got %f, expected %f\n", i, h_out[i], expected);
            ok = false;
        }
    }
    printf("%s\n", ok ? "PASS" : "FAIL");

    cudaFree(device_particles);
    return ok ? 0 : 1;
}
