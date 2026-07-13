#pragma once

#include <cuda_runtime.h>
#include <cstdio>
#include <numbers>

#define CUDA_CHECK(call)                                                        \
    do {                                                                        \
        cudaError_t err = (call);                                               \
        if (err != cudaSuccess) {                                               \
            fprintf(stderr, "CUDA error at %s:%d — %s\n",                      \
                    __FILE__, __LINE__, cudaGetErrorString(err));               \
            std::abort();                                                       \
        }                                                                       \
    } while (0)


struct Vec3 {
    float x, y, z;

    __host__ __device__ void add(const Vec3& o) { x+=o.x; y+=o.y; z+=o.z; }
    __host__ __device__ void sub(const Vec3& o) { x-=o.x; y-=o.y; z-=o.z; }
    __host__ __device__ void scale(float s)     { x*=s;   y*=s;   z*=s;   }
};

__host__ __device__ float dot(const Vec3& a, const Vec3& b);
__host__ __device__ float sqMag(const Vec3& a);
__host__ __device__ float mag(const Vec3& a);
__host__ __device__ Vec3  sum(const Vec3& a, const Vec3& b);
__host__ __device__ Vec3  diff(const Vec3& a, const Vec3& b);
__host__ __device__ Vec3  prod(const Vec3& v, float s);
__host__ __device__ Vec3  prod(float s, const Vec3& v);

constexpr int BLOCK_SIZE = 256;

// === Spatial Hashgrid ===

template <int N>
struct SimulationData {
    Vec3  pos[N];
    Vec3  vel[N];
    float int_e[N];
    float mass[N];
    float density[N];
    float pressure[N];
    float sound_speed[N];
    int   grid_cell[N];
    int   buckets[N];
    Vec3  acceleration[N];
    float deps_dt[N];
    float smoothing_length;
};


template <int N>
__host__ __device__ Vec3 pos_diff(SimulationData<N>* sim_data, int i, int j) {
    Vec3& a = sim_data->pos[i];
    Vec3& b = sim_data->pos[j];

    float dx = a.x - b.x;
    float dy = a.y - b.y;
    float dz = a.z - b.z;

    if (abs(dx) > 0.5) dx -= copysign(1.0f, dx);
    if (abs(dy) > 0.5) dy -= copysign(1.0f, dy);
    if (abs(dz) > 0.5) dz -= copysign(1.0f, dz);

    return { dx, dy, dz };
}

__host__ __device__ int mod(int a, int n);
__host__ __device__ int hash_index(const Vec3& pos, float smoothing_length);

constexpr int bucket_cube_diameter = 26;
constexpr int BUCKET_COUNT = bucket_cube_diameter * bucket_cube_diameter * bucket_cube_diameter;

extern __device__ int   bucket_sizes[BUCKET_COUNT];
extern __device__ int   bucket_counts[BUCKET_COUNT];
extern __device__ int   bucket_prefix_sum[BUCKET_COUNT];
extern __device__ int   bucket_scan_temp[BUCKET_COUNT];
extern __device__ float d_max_sound_speed;

template <int N>
__global__ void build_prefix_counts(SimulationData<N>* sim_data) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N) return;
    int bucket_index = hash_index(sim_data->pos[i], sim_data->smoothing_length);
    sim_data->grid_cell[i] = bucket_index;
    atomicAdd(&bucket_sizes[bucket_index], 1);
}

template <int N>
void launch_build_prefix_counts(SimulationData<N>* const sim_data) {
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    build_prefix_counts<N><<<grid, BLOCK_SIZE>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
__global__ void populate_buckets(SimulationData<N>* sim_data) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N) return;
    int bucket = sim_data->grid_cell[i];
    int bucket_startindex = bucket_prefix_sum[bucket];
    int within_bucket_index = atomicAdd(&bucket_counts[bucket], 1);
    int final_index = bucket_startindex + within_bucket_index;
    if (final_index == N-1 || final_index == bucket_prefix_sum[bucket+1]-1)
        bucket_counts[bucket] = 0;
    sim_data->buckets[final_index] = i;
}

template <int N>
void launch_populate_buckets(SimulationData<N>* const sim_data) {
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    populate_buckets<N><<<grid, BLOCK_SIZE>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

__global__ void scan_init();
__global__ void scan_step_t2p(int stride);
__global__ void scan_step_p2t(int stride);
__global__ void copy_temp_to_prefix();
void fill_prefix_sum();

// === Max sound-speed reduction ===

template <int N>
__global__ void reduce_max_sound_speed(SimulationData<N>* sim_data) {
    extern __shared__ float sdata[];
    int tid = threadIdx.x;
    int i   = blockIdx.x * blockDim.x + tid;
    sdata[tid] = (i < N) ? sim_data->sound_speed[i] : 0.0f;
    __syncthreads();
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) sdata[tid] = fmaxf(sdata[tid], sdata[tid + s]);
        __syncthreads();
    }
    if (tid == 0)
        atomicMax((int*)&d_max_sound_speed, __float_as_int(sdata[0]));
}

template <int N>
void launch_reduce_max_sound_speed(SimulationData<N>* const sim_data) {
    float zero = 0.0f;
    CUDA_CHECK(cudaMemcpyToSymbol(d_max_sound_speed, &zero, sizeof(float)));
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    reduce_max_sound_speed<N><<<grid, BLOCK_SIZE, BLOCK_SIZE * sizeof(float)>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

// === Simulation ===

template <int N, typename Callback>
__device__ void for_each_neigbour(SimulationData<N>* sim_data, int i, Callback callback) {
    Vec3& pos = sim_data->pos[i];
    int bx = mod((int)floor(pos.x / sim_data->smoothing_length), bucket_cube_diameter);
    int by = mod((int)floor(pos.y / sim_data->smoothing_length), bucket_cube_diameter);
    int bz = mod((int)floor(pos.z / sim_data->smoothing_length), bucket_cube_diameter);

    for (int dx = -1; dx <= 1; dx++)
    for (int dy = -1; dy <= 1; dy++)
    for (int dz = -1; dz <= 1; dz++) {
        int bucket = mod(bx+dx, bucket_cube_diameter) * bucket_cube_diameter * bucket_cube_diameter
                   + mod(by+dy, bucket_cube_diameter) * bucket_cube_diameter
                   + mod(bz+dz, bucket_cube_diameter);
        int start = bucket_prefix_sum[bucket];
        int end   = start + bucket_sizes[bucket];
        for (int j = start; j < end; j++) {
            int p = sim_data->buckets[j];
            Vec3 r = pos_diff(sim_data, i, p);
            if (sqMag(r) >= sim_data->smoothing_length * sim_data->smoothing_length)
                continue;
            callback(p, r);
        }
    }
}

constexpr float PI    = 3.1415926535f;
constexpr float sigma = 8.0f / PI;

__host__ __device__ float kernel_function(const Vec3& r, float smoothing_length, float sl3);
__host__ __device__ Vec3  kernel_gradient(const Vec3& r, float smoothing_length, float sl4);

constexpr float alpha = 1;
constexpr float beta  = 2;

template <int N>
__global__ void solve_density(SimulationData<N>* sim_data, float sl3) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N) return;
    float density = 0;
    for_each_neigbour(sim_data, i, [&](int neighbour, Vec3& r) {
        density += sim_data->mass[neighbour] * kernel_function(r, sim_data->smoothing_length, sl3);
    });
    sim_data->density[i] = density;
    float gamma = 5.0f / 3.0f;
    float pressure = (gamma - 1) * density * sim_data->int_e[i];
    sim_data->pressure[i] = pressure;
    sim_data->sound_speed[i] = sqrt(gamma * pressure / density);
}

template <int N>
__global__ void solve_sph(SimulationData<N>* sim_data, float dt, float sl4) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N) return;

    Vec3  accelaration = {};
    float deps_dt = 0;

    float pressure_per_density2 = sim_data->pressure[i] / (sim_data->density[i]*sim_data->density[i]);

    for_each_neigbour(sim_data, i, [&](int neighbour, Vec3& del_pos) {
        Vec3  kernel_grad = kernel_gradient(del_pos, sim_data->smoothing_length, sl4);
        float mdp_term = sim_data->mass[neighbour] * (
                            pressure_per_density2 +
                            sim_data->pressure[neighbour] / (sim_data->density[neighbour]*sim_data->density[neighbour]));

        Vec3 del_vel = diff(sim_data->vel[i], sim_data->vel[neighbour]);

        accelaration.add(prod(-1 * mdp_term, kernel_grad));
        deps_dt += 0.5f * mdp_term * dot(del_vel, kernel_grad);

        float direction_term = dot(del_vel, del_pos);
        if (direction_term < 0) {
            float eps = 1e-8f;
            float c   = (sim_data->sound_speed[i] + sim_data->sound_speed[neighbour]) / 2;
            float rho = (sim_data->density[i]      + sim_data->density[neighbour])      / 2;
            float mu  = sim_data->smoothing_length * direction_term /
                        (sqMag(del_pos) + eps * sim_data->smoothing_length * sim_data->smoothing_length);
            float PI_ij = (-alpha * c * mu + beta * mu * mu) / rho;

            accelaration.add(prod(-PI_ij * sim_data->mass[neighbour], kernel_grad));
            deps_dt += 0.5f * sim_data->mass[neighbour] * PI_ij * dot(del_vel, kernel_grad);
        }
    });

    sim_data->deps_dt[i]     = deps_dt;
    sim_data->acceleration[i] = accelaration;
}

template <int N>
__global__ void integrate_sph(SimulationData<N>* sim_data, float dt) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N) return;

    sim_data->vel[i].add(prod(sim_data->acceleration[i], dt));
    sim_data->pos[i].add(prod(sim_data->vel[i], dt));
    sim_data->int_e[i] += sim_data->deps_dt[i] * dt;

    sim_data->pos[i].x -= floor(sim_data->pos[i].x + 0.5f);
    sim_data->pos[i].y -= floor(sim_data->pos[i].y + 0.5f);
    sim_data->pos[i].z -= floor(sim_data->pos[i].z + 0.5f);

    int cell     = sim_data->grid_cell[i];
    int new_cell = hash_index(sim_data->pos[i], sim_data->smoothing_length);
    if (cell != new_cell) {
        sim_data->grid_cell[i] = new_cell;
        atomicAdd(&bucket_sizes[new_cell],  1);
        atomicAdd(&bucket_sizes[cell],     -1);
    }
}

template <int N>
void launch_solve_density(SimulationData<N>* const sim_data, float sl3) {
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    solve_density<N><<<grid, BLOCK_SIZE>>>(sim_data, sl3);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
void launch_integrate_sph(SimulationData<N>* const sim_data, float dt) {
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    integrate_sph<N><<<grid, BLOCK_SIZE>>>(sim_data, dt);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
void launch_sph_solve(SimulationData<N>* const sim_data, float dt, float sl4) {
    int grid = (N + BLOCK_SIZE - 1) / BLOCK_SIZE;
    solve_sph<N><<<grid, BLOCK_SIZE>>>(sim_data, dt, sl4);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

