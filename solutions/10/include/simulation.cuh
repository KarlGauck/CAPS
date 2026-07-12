#pragma once
#include "kernel.cuh"

template <int N>
struct SimulationResult {
    Vec3* positions;
    float* density;
    float delta_time;
};

template <int N>
class Simulation {
public:
    SimulationData<N>* host_data;

    Simulation(SimulationData<N>* data) : host_data(data) {
        CUDA_CHECK(cudaMalloc(&_device_data, sizeof(SimulationData<N>)));
        CUDA_CHECK(cudaMemcpy(_device_data, host_data, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));
        launch_build_prefix_counts<N>(_device_data);
    }

    ~Simulation() {
        cudaFree(_device_data);
    }

    float step() {
        fill_prefix_sum();
        launch_populate_buckets<N>(_device_data);
        launch_solve_density<N>(_device_data);
        launch_reduce_max_sound_speed<N>(_device_data);

        float max_sound_speed;
        CUDA_CHECK(cudaMemcpyFromSymbol(&max_sound_speed, d_max_sound_speed, sizeof(float)));
        float dt = 0.3f * host_data->smoothing_length / max_sound_speed;

        launch_sph_solve<N>(_device_data, dt);
        launch_integrate_sph<N>(_device_data, dt);
        return dt;
    }

    SimulationResult<N> step_and_fetch() {
        float dt = step();
        CUDA_CHECK(cudaMemcpy(&host_data->pos,     &_device_data->pos,     N * sizeof(Vec3),  cudaMemcpyDeviceToHost));
        CUDA_CHECK(cudaMemcpy(&host_data->density, &_device_data->density, N * sizeof(float), cudaMemcpyDeviceToHost));
        return { &host_data->pos, &host_data->density, dt };
    }

    void fetch_all() {
        CUDA_CHECK(cudaMemcpy(host_data, _device_data, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));
    }

private:
    SimulationData<N>* _device_data;
};
