#include <cstdio>
#include <iostream>

#include "simulation.cuh"
#include "kernel.cuh"
#include "particle_loader.h"


template <int N>
Simulation<N>::Simulation(SimulationData<N>* host_data): host_data(host_data) {
    // Initialize memory
    CUDA_CHECK(cudaMalloc(this->_device_data, sizeof(SimulationData<N>)));
    CUDA_CHECK(cudaMemcpy(this->_device_data, this->host_data, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));

    // Initialize spatial hashsgrid sizes
    launch_build_prefix_counts<N>(this->_device_data);
}


template <int N>
Simulation<N>::~Simulation() {
    cudaFree(this->_device_data);
}


template <int N>
float Simulation<N>::step() {
    // Update spatial hashgrid
    fill_prefix_sum();
    launch_populate_buckets<N>(this->_device_data);

    // Calculate kernel quantities
    launch_solve_density<N>(this->_device_data);


    // Fetch max soundspeed
    launch_reduce_max_sound_speed<N>(this->_device_data);

    float max_sound_speed;
    CUDA_CHECK(cudaMemcpyFromSymbol(&max_sound_speed, d_max_sound_speed, sizeof(float)));
    float dt = 0.3*this->host_data->smoothing_length/max_sound_speed;

    launch_sph_solve<N>(this->_device_data, dt);
    launch_integrate_sph<N>(this->_device_data, dt);

    return dt;
}


template <int N>
SimulationResult<N> Simulation<N>::step_and_fetch() {
    float dt = step();

    CUDA_CHECK(
        cudaMemcpy(
            &(this->host_data->pos),
            &(this->_device_data->pos),
            N * sizeof(Vec3),
            cudaMemcpyDeviceToHost
        )
    );

    CUDA_CHECK(
        cudaMemcpy(
            &(this->host_data->density),
            &(this->_device_data->density),
            N * sizeof(float),
            cudaMemcpyDeviceToHost
        )
    );

    return {
        &(this->host_data->pos),
        &(this->host_data->density),
        dt
    };
}

template <int N>
void Simulation<N>::fetch_all() {
    CUDA_CHECK(cudaMemcpy(this->host_data, this->_device_data, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));
}
