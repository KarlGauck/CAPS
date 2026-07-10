cmake_minimum_required(VERSION 3.18)
project(solution10 CUDA CXX)

set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CXX_STANDARD 17)

find_package(CUDAToolkit REQUIRED)

add_executable(solution10 src/main.cu)

target_include_directories(solution10 PRIVATE include)

set_target_properties(solution10 PROPERTIES
    CUDA_SEPARABLE_COMPILATION ON
)
