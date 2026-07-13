#include "kernel.cuh"

// Vec3 free functions
__host__ __device__ float dot(const Vec3& a, const Vec3& b) { return a.x*b.x + a.y*b.y + a.z*b.z; }
__host__ __device__ float sqMag(const Vec3& a)              { return dot(a, a); }
__host__ __device__ float mag(const Vec3& a)                { return sqrt(dot(a, a)); }
__host__ __device__ Vec3  sum(const Vec3& a, const Vec3& b) { return {a.x+b.x, a.y+b.y, a.z+b.z}; }
__host__ __device__ Vec3  diff(const Vec3& a, const Vec3& b){ return {a.x-b.x, a.y-b.y, a.z-b.z}; }
__host__ __device__ Vec3  prod(const Vec3& v, float s)      { return {v.x*s, v.y*s, v.z*s}; }
__host__ __device__ Vec3  prod(float s, const Vec3& v)      { return prod(v, s); }

__host__ __device__ int mod(int a, int n) {
    if (a > 0) return a % n;
    int offset_factor = ((abs(a) - 1) / n) + 1;
    return (a + offset_factor * n) % n;
}

__host__ __device__ int hash_index(const Vec3& pos, float smoothing_length) {
    return mod((int)floor(pos.x / smoothing_length), bucket_cube_diameter) * bucket_cube_diameter * bucket_cube_diameter
         + mod((int)floor(pos.y / smoothing_length), bucket_cube_diameter) * bucket_cube_diameter
         + mod((int)floor(pos.z / smoothing_length), bucket_cube_diameter);
}

// Device variable definitions
__device__ int   bucket_sizes[BUCKET_COUNT];
__device__ int   bucket_counts[BUCKET_COUNT];
__device__ int   bucket_prefix_sum[BUCKET_COUNT];
__device__ int   bucket_scan_temp[BUCKET_COUNT];
__device__ float d_max_sound_speed;

// Scan kernels
__global__ void scan_init() {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_scan_temp[i] = (i > 0) ? bucket_sizes[i - 1] : 0;
}

__global__ void scan_step_t2p(int stride) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_prefix_sum[i] = bucket_scan_temp[i] + (i >= stride ? bucket_scan_temp[i - stride] : 0);
}

__global__ void scan_step_p2t(int stride) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= BUCKET_COUNT) return;
    bucket_scan_temp[i] = bucket_prefix_sum[i] + (i >= stride ? bucket_prefix_sum[i - stride] : 0);
}

__global__ void copy_temp_to_prefix() {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < BUCKET_COUNT) bucket_prefix_sum[i] = bucket_scan_temp[i];
}

// Kernel functions — r = pos_i - pos_j (periodic minimum-image difference)
__host__ __device__ float kernel_function(const Vec3& r, float smoothing_length, float sl3) {
    float rh = mag(r) / smoothing_length;
    float val;
    if      (rh < 0.5f) val = 6*rh*rh*rh - 6*rh*rh + 1;
    else if (rh <= 1.0f) {
        float rh1 = 1-rh;
        val = 2*rh1*rh1*rh1;
    } 
    else                 val = 0;
    return sigma * val / sl3;
}

__host__ __device__ Vec3 kernel_gradient(const Vec3& r, float smoothing_length, float sl4) {
    float r_mag = mag(r);
    if (r_mag < 1e-8f) return {0.0f, 0.0f, 0.0f};
    float rh = r_mag / smoothing_length;
    float val;
    if      (rh < 0.5f)  val = 3*rh*rh - 2*rh;
    else if (rh <= 1.0f) {
        float rh1 = 1-rh;
        val = -rh1*rh1;
    }
    else                 val = 0;
    float deriv = 6*sigma*val / sl4;
    return prod(r, deriv / r_mag);
}

void fill_prefix_sum() {
    constexpr int block = 1024;
    constexpr int grid  = (BUCKET_COUNT + block - 1) / block;

    scan_init<<<grid, block>>>();
    CUDA_CHECK(cudaGetLastError());
    CUDA_CHECK(cudaDeviceSynchronize());

    bool src_is_temp = true;
    for (int stride = 1; stride < BUCKET_COUNT; stride <<= 1) {
        if (src_is_temp) scan_step_t2p<<<grid, block>>>(stride);
        else             scan_step_p2t<<<grid, block>>>(stride);
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
