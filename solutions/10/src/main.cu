#include <cstdio>
#include <iostream>
#include "kernel.cuh"
#include "particle_loader.h"

void fill_prefix_sum() {
    constexpr int block = 1024;
    constexpr int grid  = (BUCKET_COUNT + block - 1) / block;

    scan_init<<<grid, block>>>();
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());

    bool src_is_temp = true;
    for (int stride = 1; stride < BUCKET_COUNT; stride <<= 1) {
        if (src_is_temp)
            scan_step_t2p<<<grid, block>>>(stride);
        else
            scan_step_p2t<<<grid, block>>>(stride);
        CUDA_CHECK(cudaGetLastError());
        CUDA_CHECK(cudaDeviceSynchronize());
        src_is_temp = !src_is_temp;
    }

    if (src_is_temp) {
        copy_temp_to_prefix<<<grid, block>>>();
        CUDA_CHECK(cudaGetLastError());
        CUDA_CHECK(cudaDeviceSynchronize());
    }
}

int main() {
    constexpr int N = 262144;
    constexpr float total_time = 3e-2;
    float time = 0;

    auto* host = new SimulationData<N>();
    ParticleFile pf = load_particles("../springel_sedov_smeared.0000");
    copy_to_particle_data(pf, *host);
    host->smoothing_length =  0.03984126984126984;

    SimulationData<N>* dev;
    CUDA_CHECK(cudaMalloc(&dev, sizeof(SimulationData<N>)));
    CUDA_CHECK(cudaMemcpy(dev, host, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));

    launch_build_prefix_counts<N>(dev);

    while (time < total_time) {
        fill_prefix_sum();
        launch_populate_buckets<N>(dev);
        launch_solve_density<N>(dev);

        launch_reduce_max_sound_speed<N>(dev);
        float max_sound_speed;
        CUDA_CHECK(cudaMemcpyFromSymbol(&max_sound_speed, d_max_sound_speed, sizeof(float)));
        float dt = 0.3*host->smoothing_length/max_sound_speed;
        time += dt;

        launch_sph_solve<N>(dev, dt);
        launch_integrate_sph<N>(dev, dt);

        int progress = (time / total_time) * 100;

        std::cout << "[" << progress << "] t = " << time << std::endl;
    }

    CUDA_CHECK(cudaMemcpy(host, dev, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));

    write_distribution_csv(*host, "../resulting_distribution.csv");

    printf("Done. pos[0]=(%.3f %.3f %.3f)  vel[0]=(%.3f %.3f %.3f)  int_e[0]=%.3f\n",
           host->pos[0].x, host->pos[0].y, host->pos[0].z,
           host->vel[0].x, host->vel[0].y, host->vel[0].z,
           host->int_e[0]);

    cudaFree(dev);
    return 0;
}
