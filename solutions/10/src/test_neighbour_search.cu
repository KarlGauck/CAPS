#include <cstdio>
#include <cassert>
#include <algorithm>

#include "kernel.cuh"

// ── helpers ──────────────────────────────────────────────────────────────────
// This test file was written by claude. It evaluates, weather the spatial hash grid is working

// Duplicates fill_prefix_sum from main.cu so this TU compiles standalone.
static void run_prefix_sum() {
    constexpr int block = 1024;
    constexpr int grid  = (BUCKET_COUNT + block - 1) / block;
    scan_init<<<grid, block>>>();
    CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
    bool from_temp = true;
    for (int stride = 1; stride < BUCKET_COUNT; stride <<= 1) {
        if (from_temp) scan_step_t2p<<<grid, block>>>(stride);
        else           scan_step_p2t<<<grid, block>>>(stride);
        CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
        from_temp = !from_temp;
    }
    if (from_temp) {
        copy_temp_to_prefix<<<grid, block>>>();
        CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
    }
}

// Single-thread kernel: collects every neighbour of `target` into out_list.
template <int N>
__global__ void collect_neighbors_kernel(SimulationData<N>* particles, int target,
                                         int* out_list, int* out_count) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    *out_count = 0;
    for_each_neigbour<N>(particles, target, [&](int nb) {
        out_list[(*out_count)++] = nb;
    });
}

// ── test ─────────────────────────────────────────────────────────────────────

// 27 particles, one per cell in the 3x3x3 block of cells (1,1,1)..(3,3,3).
// Layout: particle index = (cx-1) + (cy-1)*3 + (cz-1)*9
//   → particle 13 sits at cell (2,2,2), the centre.
// With smoothing_length = 1, all 27 cells fall inside the 3x3x3 neighbourhood
// of (2,2,2), so particle 13 must see every other particle as a neighbour.
constexpr int N = 27;

int main() {
    // ── 1. build host particles ───────────────────────────────────────────
    SimulationData<N> host {};
    host.smoothing_length = 1.0f;
    float h = host.smoothing_length;
    for (int cz = 1; cz <= 3; cz++)
    for (int cy = 1; cy <= 3; cy++)
    for (int cx = 1; cx <= 3; cx++) {
        int idx = (cx-1) + (cy-1)*3 + (cz-1)*9;
        host.pos[idx] = { (cx + 0.5f)*h, (cy + 0.5f)*h, (cz + 0.5f)*h };
    }

    SimulationData<N>* dev;
    CUDA_CHECK(cudaMalloc(&dev, sizeof(SimulationData<N>)));
    CUDA_CHECK(cudaMemcpy(dev, &host, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));

    // ── 2. pipeline ───────────────────────────────────────────────────────
    launch_build_prefix_counts<N>(dev);
    run_prefix_sum();
    launch_populate_buckets<N>(dev);

    // ── 3. download ───────────────────────────────────────────────────────
    CUDA_CHECK(cudaMemcpy(&host, dev, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));

    int* h_sizes  = new int[BUCKET_COUNT]();
    int* h_prefix = new int[BUCKET_COUNT]();
    CUDA_CHECK(cudaMemcpyFromSymbol(h_sizes,  bucket_sizes,      BUCKET_COUNT * sizeof(int)));
    CUDA_CHECK(cudaMemcpyFromSymbol(h_prefix, bucket_prefix_sum, BUCKET_COUNT * sizeof(int)));

    // ── check 1: bucket_sizes ────────────────────────────────────────────
    int total = 0, non_empty = 0;
    for (int b = 0; b < BUCKET_COUNT; b++) {
        if (h_sizes[b] > 0) {
            if (h_sizes[b] != 1) {
                printf("FAIL bucket_sizes[%d] = %d (expected 1)\n", b, h_sizes[b]);
                return 1;
            }
            non_empty++;
        }
        total += h_sizes[b];
    }
    if (total != N || non_empty != N) {
        printf("FAIL bucket_sizes: total=%d non_empty=%d (expected %d)\n", total, non_empty, N);
        return 1;
    }
    printf("PASS bucket_sizes:      %d particles in %d non-empty buckets\n", total, non_empty);

    // ── check 2: bucket_prefix_sum is exclusive scan of bucket_sizes ─────
    int running = 0;
    for (int b = 0; b < BUCKET_COUNT; b++) {
        if (h_prefix[b] != running) {
            printf("FAIL bucket_prefix_sum[%d] = %d (expected %d)\n", b, h_prefix[b], running);
            return 1;
        }
        running += h_sizes[b];
    }
    printf("PASS bucket_prefix_sum: exclusive scan correct\n");

    // ── check 3: buckets[] holds the right particle indices ───────────────
    for (int b = 0; b < BUCKET_COUNT; b++) {
        for (int j = h_prefix[b]; j < h_prefix[b] + h_sizes[b]; j++) {
            int p = host.buckets[j];
            if (p < 0 || p >= N || host.grid_cell[p] != b) {
                printf("FAIL buckets[%d] = %d but grid_cell[%d] = %d (bucket %d)\n",
                       j, p, p, host.grid_cell[p], b);
                return 1;
            }
        }
    }
    printf("PASS buckets[]:         all indices in correct bucket slots\n");

    // ── check 4: neighbour search for particle 13 (cell (2,2,2)) ─────────
    int *dev_list, *dev_count;
    CUDA_CHECK(cudaMalloc(&dev_list,  N * sizeof(int)));
    CUDA_CHECK(cudaMalloc(&dev_count, sizeof(int)));
    CUDA_CHECK(cudaMemset(dev_count, 0, sizeof(int)));

    collect_neighbors_kernel<N><<<1, 1>>>(dev, 13, dev_list, dev_count);
    CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());

    int h_count = 0;
    int h_list[N] = {};
    CUDA_CHECK(cudaMemcpy(&h_count, dev_count, sizeof(int),           cudaMemcpyDeviceToHost));
    CUDA_CHECK(cudaMemcpy(h_list,   dev_list,  h_count * sizeof(int), cudaMemcpyDeviceToHost));

    if (h_count != N) {
        printf("FAIL neighbour count = %d (expected %d)\n", h_count, N);
        return 1;
    }
    std::sort(h_list, h_list + h_count);
    for (int i = 0; i < N; i++) {
        if (h_list[i] != i) {
            printf("FAIL missing neighbour %d\n", i);
            return 1;
        }
    }
    printf("PASS neighbour search:  particle 13 found all %d neighbours\n", h_count);

    delete[] h_sizes;
    delete[] h_prefix;
    cudaFree(dev); cudaFree(dev_list); cudaFree(dev_count);
    printf("\nAll tests passed.\n");
    return 0;
}
