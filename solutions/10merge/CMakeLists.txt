cmake_minimum_required(VERSION 3.18)
project(sph_raymarcher CUDA CXX)

set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} -Wall")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra")

# CUDA
find_package(CUDAToolkit REQUIRED)

# OpenGL/GLFW Dependencies
find_package(OpenGL REQUIRED)
find_package(GLFW3 REQUIRED)

include_directories(
    cuda/include
    opengl/include
    /usr/include/GL
    /usr/include/glm
)

set(CUDA_SOURCES
    cuda/src/kernel.cu
)

set(CPP_SOURCES
    main.cpp
    opengl/src/raymarching.cpp
    opengl/src/glad.c
)

add_executable(sph_raymarcher ${CUDA_SOURCES} ${CPP_SOURCES})

# Claude says I need this
set_target_properties(sph_raymarcher PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

target_link_libraries(sph_raymarcher 
    PRIVATE 
    CUDA::cudart
    OpenGL::GL
    glfw
)

file(GLOB_RECURSE GLSL_FILES "opengl/shaders/*.glsl")
foreach(glsl_file ${GLSL_FILES})
    get_filename_component(fname ${glsl_file} NAME)
    configure_file(${glsl_file} ${CMAKE_CURRENT_BINARY_DIR}/${fname} COPYONLY)
endforeach()
