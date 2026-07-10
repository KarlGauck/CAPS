#include <cstdio>
#include <vector>
#include <numeric>

#include "kernel.cuh"

int main() {
    constexpr int N = 1024;

    // Host data
    std::vector<float> h_a(N), h_b(N), h_out(N);
    std::iota(h_a.begin(), h_a.end(), 0.0f);   // 0, 1, 2, ...
    std::iota(h_b.begin(), h_b.end(), 100.0f); // 100, 101, 102, ...

    // Device allocations
    float *d_a, *d_b, *d_out;
    CUDA_CHECK(cudaMalloc(&d_a,   N * sizeof(float)));
    CUDA_CHECK(cudaMalloc(&d_b,   N * sizeof(float)));
    CUDA_CHECK(cudaMalloc(&d_out, N * sizeof(float)));

    CUDA_CHECK(cudaMemcpy(d_a, h_a.data(), N * sizeof(float), cudaMemcpyHostToDevice));
    CUDA_CHECK(cudaMemcpy(d_b, h_b.data(), N * sizeof(float), cudaMemcpyHostToDevice));

    launch_add<float>(d_a, d_b, d_out, N);

    CUDA_CHECK(cudaMemcpy(h_out.data(), d_out, N * sizeof(float), cudaMemcpyDeviceToHost));

    // Verify
    bool ok = true;
    for (int i = 0; i < N; ++i) {
        float expected = h_a[i] + h_b[i];
        if (h_out[i] != expected) {
            fprintf(stderr, "Mismatch at %d: got %f, expected %f\n", i, h_out[i], expected);
            ok = false;
        }
    }
    printf("%s\n", ok ? "PASS" : "FAIL");

    cudaFree(d_a);
    cudaFree(d_b);
    cudaFree(d_out);
    return ok ? 0 : 1;
}
