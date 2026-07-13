#include <iostream>
#include <vector>
#include <span>

#include "cuda/include/kernel.cuh"
#include "cuda/include/simulation.cuh"
#include "cuda/include/particle_loader.h"
#include "opengl/include/raymarching.hpp"

int main() {
    constexpr int N = 250047; 
    
    auto* host_data = new SimulationData<N>();
    
    ParticleFile pf = load_particles("cuda/springel_sedov_smeared.0000");
    copy_to_particle_data(pf, *host_data);
    host_data->smoothing_length = 0.03984126984126984;

    Simulation<N> sim { host_data, false };

    Raymarching rm;
    rm.point_radius = 0.01f;

    std::cout << "Starting Simulation..." << std::endl;
    
    auto err = rm.run([&]() -> std::span<Point> {
        auto result = sim.step_and_fetch();
        // Vec3 is wonderfully compatible with 3 floats
        return std::span<Point>(reinterpret_cast<Point*>(result.positions), N);
    });

    if (err) {
        std::cerr << "Raymarching error occurred: " << err << std::endl;
    }

    delete host_data;
    return 0;
}
