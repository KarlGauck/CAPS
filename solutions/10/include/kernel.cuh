#pragma once

#include <cuda_runtime.h>
#include <cstdio>
#include <numbers>

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


struct Vec3 {
    float x;
    float y;
    float z;

    __host__ __device__
    void add(const Vec3& other) {
        x += other.x;
        y += other.y;
        z += other.z;
    }

    __host__ __device__
    void sub(const Vec3& other) {
        x -= other.x;
        y -= other.y;
        z -= other.z;
    }

    __host__ __device__
    void scale(const float scalar) {
        x *= scalar;
        y *= scalar;
        z *= scalar;
    }
};

__host__ __device__
float dot(const Vec3& a, const Vec3& b) {
    return a.x*b.x + a.y*b.y + a.z*b.z;
}

__host__ __device__
float sqMag(const Vec3& a) {
    return dot(a, a);
}

__host__ __device__
float mag(const Vec3& a) {
    return sqrt(dot(a, a));
}

__host__ __device__
Vec3 sum(const Vec3& a, const Vec3& b) {
    return { a.x + b.x, a.y + b.y, a.z + b.z };
}

__host__ __device__
Vec3 diff(const Vec3& a, const Vec3& b) {
    return { a.x - b.x, a.y - b.y, a.z - b.z };
}

__host__ __device__ 
Vec3 prod(const Vec3& vec, float scalar) {
    return {
        vec.x * scalar,
        vec.y * scalar,
        vec.z * scalar
    };
}

__host__ __device__ 
Vec3 prod(float scalar, const Vec3& vec) {
    return prod(vec, scalar);
}

// === === === Spatial Hashgrid === === ===

template <int N>
struct SimulationData {
    Vec3 pos[N];
    Vec3 vel[N];
    float int_e[N];
    float mass[N];
    float density[N];
    float pressure[N];
    float sound_speed[N];
    int grid_cell[N];
    int buckets[N]; // this array is not indexed by particle indices but by bucket positions, instead it holds particle indices.
    Vec3 acceleration[N];
    float deps_dt[N];
    float smoothing_length;
};

__host__ __device__
int mod(int a, int n) {
    if (a > 0) 
        return a % n;
    
    int offset_factor = ((abs(a) - 1) / n) + 1;
    return (a + offset_factor * n) % n;
    // always returns positive numbers
}

constexpr int bucket_cube_diameter = 26;
constexpr int BUCKET_COUNT =
    bucket_cube_diameter * bucket_cube_diameter * bucket_cube_diameter;

__device__  int bucket_sizes[
    BUCKET_COUNT
];

__device__  int bucket_counts[
    BUCKET_COUNT
];

__device__  int bucket_prefix_sum[
    BUCKET_COUNT
];

__host__ __device__
int hash_index(const Vec3& pos, float smoothing_length) {
    return mod((int)floor(pos.x / smoothing_length), bucket_cube_diameter) * bucket_cube_diameter * bucket_cube_diameter
            + mod((int)floor(pos.y / smoothing_length), bucket_cube_diameter) * bucket_cube_diameter
            + mod((int)floor(pos.z / smoothing_length), bucket_cube_diameter);
}

// Builds the hash grid for the first time
template <int N>
__global__ void build_prefix_counts(SimulationData<N>* sim_data) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N)
        return;
    int bucket_index = hash_index(sim_data->pos[i], sim_data->smoothing_length);
    sim_data->grid_cell[i] = bucket_index;
    atomicAdd(&bucket_sizes[bucket_index], 1);
}


template <int N>
void launch_build_prefix_counts(SimulationData<N> *const sim_data) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    build_prefix_counts<N><<<grid, block>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}


// Populateas the buckets from the prefix sum
template <int N>
__global__ void populate_buckets(SimulationData<N>* sim_data) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N)
        return;
    int bucket = sim_data->grid_cell[i];
    int bucket_startindex = bucket_prefix_sum[bucket];
    int within_bucket_index = atomicAdd(&bucket_counts[bucket], 1);
    int final_index = bucket_startindex + within_bucket_index;

    // TODO: Do i need to catch for an index out of bounds or does this work?
    if (final_index == N-1 || final_index == bucket_prefix_sum[bucket+1]-1) 
        bucket_counts[bucket] = 0;
    sim_data->buckets[final_index] = i;
}


template <int N>
void launch_populate_buckets(SimulationData<N> *const sim_data) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    populate_buckets<N><<<grid, block>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}



__device__ int bucket_scan_temp[BUCKET_COUNT];

// Shift bucket_sizes right by one into temp to seed an exclusive scan.
__global__ void scan_init() {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_scan_temp[i] = (i > 0) ? bucket_sizes[i - 1] : 0;
}

// Hillis-Steele step: temp → prefix_sum
__global__ void scan_step_t2p(int stride) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_prefix_sum[i] =
        bucket_scan_temp[i] + (i >= stride ? bucket_scan_temp[i - stride] : 0);
}

// Hillis-Steele step: prefix_sum → temp
__global__ void scan_step_p2t(int stride) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_scan_temp[i] =
        bucket_prefix_sum[i] + (i >= stride ? bucket_prefix_sum[i - stride] : 0);
}

// Copy temp to prefix_sum when the final step landed in temp.
__global__ void copy_temp_to_prefix() {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < BUCKET_COUNT)
        bucket_prefix_sum[i] = bucket_scan_temp[i];
}

// === === === Max sound-speed reduction ===================================
// Used for CFL timestep: dt = C * smoothing_length / max_sound_speed

__device__ float d_max_sound_speed;

// Each block reduces its chunk via shared memory, then atomicMax into the global.
// atomicMax on int is valid here because sound_speed >= 0, so IEEE 754 bit
// patterns compare identically to their uint/int values (sign bit is always 0).
template <int N>
__global__ void reduce_max_sound_speed(SimulationData<N>* sim_data) {
    extern __shared__ float sdata[];
    int tid = threadIdx.x;
    int i   = blockIdx.x * blockDim.x + tid;

    sdata[tid] = (i < N) ? sim_data->sound_speed[i] : 0.0f;
    __syncthreads();

    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s)
            sdata[tid] = fmaxf(sdata[tid], sdata[tid + s]);
        __syncthreads();
    }

    if (tid == 0)
        atomicMax((int*)&d_max_sound_speed, __float_as_int(sdata[0]));
}

template <int N>
void launch_reduce_max_sound_speed(SimulationData<N>* const sim_data) {
    float zero = 0.0f;
    CUDA_CHECK(cudaMemcpyToSymbol(d_max_sound_speed, &zero, sizeof(float)));

    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    reduce_max_sound_speed<N><<<grid, block, block * sizeof(float)>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

// The code for building the prefix sums was written by claude since I always
// get problems with indices when trying to implement this myself :)


// === === === Simulation === === ===


// This function was also implemented by claude to handle dynamic iterating without returning a 
// fixed size array of indices for a dynamic number of neighbours
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
            if (sqMag(diff(sim_data->pos[i], sim_data->pos[p])) >= pow(sim_data->smoothing_length, 2))
                continue;
            callback(p);
        }
    }
}


constexpr float PI = 3.1415926535;
constexpr float sigma = 8.0/PI;
__host__ __device__ float kernel_function(
    const Vec3& p1,
    const Vec3& p2,
    float smoothing_length
) {
    float rh = mag(diff(p1, p2)) / smoothing_length;
    float val;
    if (rh < 0.5) {
        val = 6*pow(rh, 3) - 6*pow(rh, 2) + 1;
    } else if (rh <= 1) {
        val = 2*pow(1-rh, 3);
    } else {
        val = 0;
    };
    return sigma * val / pow(smoothing_length, 3);
}

__host__ __device__ float kernel_derivative(
    const Vec3& p1,
    const Vec3& p2,
    float smoothing_length
) {
    float rh = mag(diff(p1, p2)) / smoothing_length;
    float val;

    if (rh < 0.5) {
        val = 3*pow(rh, 2) - 2*rh;
    } else if (rh <= 1) {
        val = -pow(1 - rh, 2);
    } else {
        val = 0;
    }

    return 6*sigma*val / pow(smoothing_length, 4);
}

__host__ __device__ Vec3 kernel_gradient(
    const Vec3& center,
    const Vec3& sample,
    float smoothing_length
) {
    Vec3 r = diff(center, sample);
    float r_mag = mag(r);
    if (r_mag < 1e-8f) return {0.0f, 0.0f, 0.0f};
    return prod(r, kernel_derivative(center, sample, smoothing_length) / r_mag);
}

// We use lamda = 2 here to obtain our equations

template <int N>
__global__ void solve_density(SimulationData<N>* sim_data) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N)
        return;

    float density = 0;
    for_each_neigbour(sim_data, i, [&](int neighbour) {
        density += sim_data->mass[neighbour] * kernel_function(sim_data->pos[i], sim_data->pos[neighbour], sim_data->smoothing_length);
    });

    sim_data->density[i] = density;

    float gamma = 5.0/3.0;
    float pressure = (gamma - 1) * density * sim_data->int_e[i];
    sim_data->pressure[i] = pressure;
    sim_data->sound_speed[i] = sqrt(gamma*pressure/density);
}

constexpr float alpha = 1;
constexpr float beta = 2;


template <int N>
__global__ void solve_sph(SimulationData<N>* sim_data, float dt) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N)
        return;

    Vec3 accelaration = {};
    float deps_dt = 0;

    for_each_neigbour(sim_data, i, [&](int neighbour) {

        Vec3 kernel_grad = kernel_gradient(sim_data->pos[i], sim_data->pos[neighbour], sim_data->smoothing_length);
        float mdp_term = sim_data->mass[neighbour] * (
                        sim_data->pressure[i] / pow(sim_data->density[i], 2) +
                        sim_data->pressure[neighbour] / pow(sim_data->density[neighbour], 2));
        // Mass density and pressure term that is repeated in both equations

        // artificial velocity for shock wave propagation
        Vec3 del_vel = diff(sim_data->vel[i], sim_data->vel[neighbour]);
        Vec3 del_pos = diff(sim_data->pos[i], sim_data->pos[neighbour]);

        // momentum equations
        accelaration.add(prod(-1 * mdp_term, kernel_grad));

        // energy equations
        deps_dt += 0.5 * mdp_term * dot(del_vel, kernel_grad);

        float direction_term = dot(
            del_vel,
            del_pos            
        );

        if (direction_term < 0) {
            float eps = 1e-8;
            float c = (sim_data->sound_speed[i] + sim_data->sound_speed[neighbour]) / 2;
            float rho = (sim_data->density[i] + sim_data->density[neighbour]) / 2;
            float mu = sim_data->smoothing_length * direction_term /
                (sqMag(diff(sim_data->pos[i], sim_data->pos[neighbour])) + eps * pow(sim_data->smoothing_length, 2));

            float PI_ij = (-alpha * c * mu + beta * pow(mu, 2)) / rho;

            // momentum equation
            accelaration.add(prod(-PI_ij * sim_data->mass[neighbour], kernel_grad));

            // energy equation
            deps_dt += 0.5 * sim_data->mass[neighbour] * PI_ij * dot(del_vel, kernel_grad);
        }

    });

    sim_data->deps_dt[i] = deps_dt;
    sim_data->acceleration[i] = accelaration;
}

template <int N>
__global__ void integrate_sph(SimulationData<N>* sim_data, float dt) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= N)
        return;

    sim_data->vel[i].add(prod(sim_data->acceleration[i], dt));
    sim_data->pos[i].add(prod(sim_data->vel[i], dt));
    sim_data->int_e[i] += sim_data->deps_dt[i] * dt;

    int cell = sim_data->grid_cell[i];
    int new_cell = hash_index(sim_data->pos[i], sim_data->smoothing_length);
    if (cell != new_cell) {
        sim_data->grid_cell[i] = new_cell;
        atomicAdd(&bucket_sizes[new_cell], 1);
        atomicAdd(&bucket_sizes[cell], -1);
    }
}


template <int N> 
void launch_solve_density(SimulationData<N> *const sim_data) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    solve_density<N><<<grid, block>>>(sim_data);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
void launch_integrate_sph(SimulationData<N> *const sim_data, float dt) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    integrate_sph<N><<<grid, block>>>(sim_data, dt);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}

template <int N>
void launch_sph_solve(SimulationData<N> *const sim_data, float dt) {
    constexpr int block = 256;
    int grid = (N + block - 1) / block;
    solve_sph<N><<<grid, block>>>(sim_data, dt);
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());
}