#pragma once

#include <cstdio>
#include <cstdlib>
#include <vector>
#include <algorithm>
#include "kernel.cuh"

// Runtime-sized particle data read from a file.
// Use copy_to_particle_data<N>() to transfer into a SimulationData<N>.
struct ParticleFile {
    int                count = 0;
    std::vector<Vec3>  pos;
    std::vector<Vec3>  vel;
    std::vector<float> mass;
    std::vector<float> int_e;
};

// Counts particles (non-empty lines) by scanning the file in 64 KB chunks
// without parsing any values — fast even for 250k+ lines.
inline int count_particles(const char* path) {
    FILE* fp = fopen(path, "r");
    if (!fp) {
        fprintf(stderr, "count_particles: cannot open '%s'\n", path);
        std::abort();
    }

    int   count           = 0;
    bool  line_has_data   = false;
    char  buf[1 << 16];
    size_t n;

    while ((n = fread(buf, 1, sizeof(buf), fp)) > 0) {
        for (size_t i = 0; i < n; i++) {
            char c = buf[i];
            if (c == '\n') {
                if (line_has_data) count++;
                line_has_data = false;
            } else if (c != ' ' && c != '\r' && c != '\t') {
                line_has_data = true;
            }
        }
    }
    if (line_has_data) count++;   // last line with no trailing newline

    fclose(fp);
    return count;
}

// Loads particles from a space-separated file:
//   x y z vx vy vz mass epsilon [type_flag]
// Values are read as double and stored as float.
// Extra columns (e.g. type flag) are silently ignored.
// Path is relative to the current working directory.
inline ParticleFile load_particles(const char* path) {
    FILE* fp = fopen(path, "r");
    if (!fp) {
        fprintf(stderr, "load_particles: cannot open '%s'\n", path);
        std::abort();
    }

    ParticleFile pf;
    pf.pos.reserve(1 << 18);   // 262144 — avoids repeated reallocation
    pf.vel.reserve(1 << 18);
    pf.mass.reserve(1 << 18);
    pf.int_e.reserve(1 << 18);

    constexpr double BOUNDARY = 0.5 - 1e-9;
    double x, y, z, vx, vy, vz, m, eps;
    while (fscanf(fp, "%lf %lf %lf %lf %lf %lf %lf %lf", &x, &y, &z, &vx, &vy, &vz, &m, &eps) == 8) {
        // consume any remaining fields on the line (e.g. type flag)
        int c;
        while ((c = fgetc(fp)) != '\n' && c != EOF) {}

        // skip the +0.5 boundary layer — those particles are periodic duplicates of x=-0.5
        if (x > BOUNDARY || y > BOUNDARY || z > BOUNDARY) continue;

        pf.pos.push_back({ (float)x, (float)y, (float)z });
        pf.vel.push_back({ (float)vx, (float)vy, (float)vz });
        pf.mass.push_back((float)m);
        pf.int_e.push_back((float)eps);
    }
    fclose(fp);

    pf.count = (int)pf.pos.size();
    printf("load_particles: read %d particles from '%s'\n", pf.count, path);
    return pf;
}

// Copies min(pf.count, N) particles from pf into pd.
// For large N, pd must be heap-allocated — sizeof(SimulationData<262144>) is ~18 MB.
template <int N>
void copy_to_particle_data(const ParticleFile& pf, SimulationData<N>& pd) {
    int n = std::min(pf.count, N);
    if (pf.count > N)
        fprintf(stderr, "copy_to_particle_data: file has %d particles but "
                        "SimulationData<%d> holds only %d — truncating.\n",
                pf.count, N, N);
    for (int i = 0; i < n; i++) {
        pd.pos[i]   = pf.pos[i];
        pd.vel[i]   = pf.vel[i];
        pd.mass[i]  = pf.mass[i];
        pd.int_e[i] = pf.int_e[i];
    }
}

template <int N>
void write_distribution_csv(const SimulationData<N>& pd, const char* path) {
    FILE* fp = fopen(path, "w");
    if (!fp) {
        fprintf(stderr, "write_distribution_csv: cannot open '%s'\n", path);
        std::abort();
    }
    for (int i = 0; i < N; i++)
        fprintf(fp, "%f,%f,%f,%f\n", mag(pd.pos[i]), pd.density[i], pd.pressure[i], pd.int_e[i]);
    fclose(fp);
}
