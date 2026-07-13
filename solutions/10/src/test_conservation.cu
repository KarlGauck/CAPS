#include <cstdio>
#include <cmath>
#include "kernel.cuh"

static void fill_prefix_sum() {
    constexpr int block = 1024;
    constexpr int grid  = (BUCKET_COUNT + block - 1) / block;
    scan_init<<<grid, block>>>();
    CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
    bool src_is_temp = true;
    for (int stride = 1; stride < BUCKET_COUNT; stride <<= 1) {
        if (src_is_temp) scan_step_t2p<<<grid, block>>>(stride);
        else             scan_step_p2t<<<grid, block>>>(stride);
        CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
        src_is_temp = !src_is_temp;
    }
    if (src_is_temp) {
        copy_temp_to_prefix<<<grid, block>>>();
        CUDA_CHECK(cudaGetLastError()); CUDA_CHECK(cudaDeviceSynchronize());
    }
}

struct Diagnostics {
    float px, py, pz;   // total momentum components
    float energy;        // total KE + IE
};

template <int N>
static Diagnostics compute_diagnostics(const SimulationData<N>& p) {
    Diagnostics d = {};
    for (int i = 0; i < N; i++) {
        float m = p.mass[i];
        d.px     += m * p.vel[i].x;
        d.py     += m * p.vel[i].y;
        d.pz     += m * p.vel[i].z;
        d.energy += 0.5f * m * (p.vel[i].x*p.vel[i].x +
                                p.vel[i].y*p.vel[i].y +
                                p.vel[i].z*p.vel[i].z)
                  + m * p.int_e[i];
    }
    return d;
}

static bool has_nan(const Diagnostics& d) {
    return std::isnan(d.px) || std::isnan(d.py) || std::isnan(d.pz)
        || std::isnan(d.energy) || std::isinf(d.energy);
}

// ─────────────────────────────────────────────────────────────────────────────

constexpr int N = 1024;

int main() {
    constexpr float total_time = 5.0f;

    SimulationData<N> host {};
    host.smoothing_length = 1.0f;
    for (int i = 0; i < N; i++) {
        host.pos[i]  = { (i % 10 * 0.5f + 0.5f) * host.smoothing_length,
                         (i / 10 % 10 * 0.5f + 0.5f) * host.smoothing_length,
                         (i / 100 * 0.5f + 0.5f) * host.smoothing_length };
        host.mass[i]  = 1.0f;
        host.int_e[i] = 1.0f;
    }

    Diagnostics d0 = compute_diagnostics(host);
    printf("%-6s  %-10s  %-10s  %-10s  %-12s  %-10s\n",
           "step", "time", "px", "py", "pz", "energy");
    printf("%-6s  %-10.5f  %-10.5f  %-10.5f  %-10.5f  %-12.5f  (initial)\n",
           "0", 0.0f, d0.px, d0.py, d0.pz, d0.energy);

    SimulationData<N>* dev;
    CUDA_CHECK(cudaMalloc(&dev, sizeof(SimulationData<N>)));
    CUDA_CHECK(cudaMemcpy(dev, &host, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));

    launch_build_prefix_counts<N>(dev);

    int   step = 0;
    float time = 0.0f;
    bool  ok   = true;

    while (time < total_time) {
        fill_prefix_sum();
        launch_populate_buckets<N>(dev);
        launch_solve_density<N>(dev);


        launch_reduce_max_sound_speed<N>(dev);
        float max_cs;
        CUDA_CHECK(cudaMemcpyFromSymbol(&max_cs, d_max_sound_speed, sizeof(float)));
        float dt = 0.3f * host.smoothing_length / max_cs;
        time += dt;
        step++;

        launch_sph_solve<N>(dev, dt);
        launch_integrate_sph<N>(dev, dt);

        CUDA_CHECK(cudaMemcpy(&host, dev, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));
        Diagnostics d = compute_diagnostics(host);

        printf("%-6d  %-10.5f  %-10.5f  %-10.5f  %-10.5f  %-12.5f\n",
               step, time, d.px, d.py, d.pz, d.energy);

        if (has_nan(d)) {
            printf("FAIL  step %d: NaN or Inf detected\n", step);
            ok = false;
            break;
        }

        // Momentum should stay near zero (started at rest, SPH conserves momentum).
        // Tolerance: 0.1% of initial energy worth of impulse.
        float pmag = sqrtf(d.px*d.px + d.py*d.py + d.pz*d.pz);
        if (pmag > 1e-3f * d0.energy) {
            printf("FAIL  step %d: momentum drift too large (|p|=%.4e)\n", step, pmag);
            ok = false;
            break;
        }

        // Energy should not blow up. Forward Euler allows O(dt^2) increase per
        // step; we allow up to 2x the initial energy before calling it a failure.
        if (d.energy > 2.0f * d0.energy) {
            printf("FAIL  step %d: energy blowup (%.5f, initial was %.5f)\n",
                   step, d.energy, d0.energy);
            ok = false;
            break;
        }
    }

    if (ok) {
        printf("\nPASS  %d steps, t=%.4f\n", step, time);
        printf("      momentum drift |p| = %.2e  (initial energy = %.4f)\n",
               sqrtf(d0.px*d0.px + d0.py*d0.py + d0.pz*d0.pz), d0.energy);
        CUDA_CHECK(cudaMemcpy(&host, dev, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));
        float final_energy = compute_diagnostics(host).energy;
        float pct = 100.f * (final_energy - d0.energy) / d0.energy;
        printf("      energy: %.4f → %.4f  (%+.2f%% vs initial; "
               "forward-Euler O(dt^2) drift expected)\n",
               d0.energy, final_energy, pct);
    }

    cudaFree(dev);
    return ok ? 0 : 1;
}
