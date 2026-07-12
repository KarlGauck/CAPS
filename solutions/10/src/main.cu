#include <cstdio>
#include <iostream>
#include "kernel.cuh"
#include "simulation.cuh"
#include "particle_loader.h"

int main() {
    constexpr int N = 262144;
    constexpr float total_time = 3e-2;
    float time = 0;

    auto* host = new SimulationData<N>();
    ParticleFile pf = load_particles("../springel_sedov_smeared.0000");
    copy_to_particle_data(pf, *host);
    host->smoothing_length =  0.03984126984126984;

    Simulation sim { host };

    while (time < total_time) {
        float dt = sim.step();
        time += dt;
    }

    sim.fetch_all();

    write_distribution_csv(*(sim.host_data), "../resulting_distribution.csv");

    return 0;
}
