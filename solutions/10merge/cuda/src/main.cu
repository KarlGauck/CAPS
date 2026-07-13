#include <cstdio>
#include <iostream>
#include <chrono>
#include "kernel.cuh"
#include "simulation.cuh"
#include "particle_loader.h"

int main() {
    constexpr int N = 250047; // 63^3: the +0.5 periodic-boundary layer is excluded at load
    constexpr float total_time = 10e-2;
    float time = 0;

    auto* host = new SimulationData<N>();
    ParticleFile pf = load_particles("../springel_sedov_smeared.0000");
    copy_to_particle_data(pf, *host);
    host->smoothing_length =  0.03984126984126984;

    Simulation sim { host, false };

    auto wall_start = std::chrono::steady_clock::now();

    while (time < total_time) {
        float dt = sim.step();
        time += dt;

        std::cout << "[" << int(time/total_time * 100) << "%] Time: " << time << " / " << total_time << " (dt = " << dt << ")" << std::endl;
    }

    auto wall_end = std::chrono::steady_clock::now();
    double elapsed = std::chrono::duration<double>(wall_end - wall_start).count();
    std::cout << "Sim loop: " << elapsed << " s" << std::endl;

    sim.fetch_all();

    write_distribution_csv(*(sim.host_data), "../resulting_distribution.csv");

    return 0;
}
