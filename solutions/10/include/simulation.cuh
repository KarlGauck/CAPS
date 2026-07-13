#pragma once
#include "kernel.cuh"

// Written by Claude — CUB for single-launch prefix scan
#include <cub/cub.cuh>

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

    Simulation(SimulationData<N>* data, bool use_cub_scan = true) : host_data(data), _use_cub_scan(use_cub_scan) {
        CUDA_CHECK(cudaMalloc(&_device_data, sizeof(SimulationData<N>)));
        CUDA_CHECK(cudaMemcpy(_device_data, host_data, sizeof(SimulationData<N>), cudaMemcpyHostToDevice));
        launch_build_prefix_counts<N>(_device_data);
        if (_use_cub_scan) init_cub();  // Claude
    }

    ~Simulation() {
        cudaFree(_device_data);
        if (_use_cub_scan) cudaFree(_cub_temp);  // Claude
    }

    float step() {
        float sl3 = this->host_data->smoothing_length * this->host_data->smoothing_length * this->host_data->smoothing_length;
        float sl4 = sl3 * this->host_data->smoothing_length;

        if (_use_cub_scan) fill_prefix_sum_cub();  // Claude
        else                fill_prefix_sum();

        launch_populate_buckets<N>(_device_data);
        launch_solve_density<N>(_device_data, sl3);
        launch_reduce_max_sound_speed<N>(_device_data);

        float max_sound_speed;
        CUDA_CHECK(cudaMemcpyFromSymbol(&max_sound_speed, d_max_sound_speed, sizeof(float)));
        float dt = 0.3*this->host_data->smoothing_length/max_sound_speed;

        launch_sph_solve<N>(_device_data, dt, sl4);
        launch_integrate_sph<N>(_device_data, dt);
        return dt;
    }

    SimulationResult<N> step_and_fetch() {
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
            this->host_data->pos,
            this->host_data->density,
            dt
        };
    }

    void fetch_all() {
        CUDA_CHECK(cudaMemcpy(this->host_data, this->_device_data, sizeof(SimulationData<N>), cudaMemcpyDeviceToHost));
    }

private:
    SimulationData<N>* _device_data;

    // Written by Claude — CUB scan state
    bool   _use_cub_scan        = true;
    int*   _d_bucket_sizes      = nullptr;
    int*   _d_bucket_prefix_sum = nullptr;
    void*  _cub_temp            = nullptr;
    size_t _cub_temp_bytes      = 0;

    // Written by Claude — resolves __device__ global pointers, sizes and allocates CUB scratch.
    // Called once from the constructor when _use_cub_scan is true.
    void init_cub() {
        CUDA_CHECK(cudaGetSymbolAddress((void**)&_d_bucket_sizes,      bucket_sizes));
        CUDA_CHECK(cudaGetSymbolAddress((void**)&_d_bucket_prefix_sum, bucket_prefix_sum));
        CUDA_CHECK(cub::DeviceScan::ExclusiveSum(
            nullptr, _cub_temp_bytes, _d_bucket_sizes, _d_bucket_prefix_sum, BUCKET_COUNT));
        CUDA_CHECK(cudaMalloc(&_cub_temp, _cub_temp_bytes));
    }

    // Written by Claude — replaces fill_prefix_sum() with a single CUB launch.
    void fill_prefix_sum_cub() {
        CUDA_CHECK(cub::DeviceScan::ExclusiveSum(
            _cub_temp, _cub_temp_bytes, _d_bucket_sizes, _d_bucket_prefix_sum, BUCKET_COUNT));
        CUDA_CHECK(cudaDeviceSynchronize());
    }
};
