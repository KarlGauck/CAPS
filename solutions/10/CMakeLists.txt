cmake_minimum_required(VERSION 3.18)
project(solution10 CUDA CXX)

set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CXX_STANDARD 17)

find_package(CUDAToolkit REQUIRED)
find_package(PNG REQUIRED)

add_executable(solution10 src/main.cu src/kernel.cu)
target_include_directories(solution10 PRIVATE include)
set_target_properties(solution10 PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

add_executable(visualize src/visualize.cu src/kernel.cu)
target_include_directories(visualize PRIVATE include)
target_link_libraries(visualize PRIVATE PNG::PNG)
set_target_properties(visualize PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

add_executable(test_neighbour_search src/test_neighbour_search.cu)
target_include_directories(test_neighbour_search PRIVATE include)
set_target_properties(test_neighbour_search PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

add_executable(test_conservation src/test_conservation.cu)
target_include_directories(test_conservation PRIVATE include)
set_target_properties(test_conservation PROPERTIES CUDA_SEPARABLE_COMPILATION ON)
