#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <vector>
#include <png.h>

#include "kernel.cuh"
#include "simulation.cuh"
#include "particle_loader.h"

// ─── minimal PNG scatter-plot renderer ───────────────────────────────────────

struct Image {
    int w, h;
    std::vector<uint8_t> buf;   // packed RGB

    Image(int w, int h) : w(w), h(h), buf(w * h * 3, 255) {}

    void set(int x, int y, uint8_t r, uint8_t g, uint8_t b) {
        if (x < 0 || x >= w || y < 0 || y >= h) return;
        int i = (y * w + x) * 3;
        buf[i] = r; buf[i+1] = g; buf[i+2] = b;
    }

    // 2×2 dot so single-pixel particles are visible
    void dot(int cx, int cy, uint8_t r, uint8_t g, uint8_t b) {
        set(cx, cy, r, g, b);  set(cx+1, cy, r, g, b);
        set(cx, cy+1, r, g, b); set(cx+1, cy+1, r, g, b);
    }

    void hline(int x0, int x1, int y, uint8_t r, uint8_t g, uint8_t b) {
        for (int x = x0; x <= x1; x++) set(x, y, r, g, b);
    }

    void vline(int x, int y0, int y1, uint8_t r, uint8_t g, uint8_t b) {
        for (int y = y0; y <= y1; y++) set(x, y, r, g, b);
    }

    void save(const char* path) const {
        FILE* fp = fopen(path, "wb");
        if (!fp) { fprintf(stderr, "Cannot open %s\n", path); return; }

        png_structp png  = png_create_write_struct(PNG_LIBPNG_VER_STRING, nullptr, nullptr, nullptr);
        png_infop   info = png_create_info_struct(png);
        if (setjmp(png_jmpbuf(png))) { fclose(fp); return; }

        png_init_io(png, fp);
        png_set_IHDR(png, info, w, h, 8, PNG_COLOR_TYPE_RGB,
                     PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT, PNG_FILTER_TYPE_DEFAULT);
        png_write_info(png, info);

        std::vector<png_bytep> rows(h);
        for (int i = 0; i < h; i++)
            rows[i] = const_cast<uint8_t*>(&buf[i * w * 3]);
        png_write_image(png, rows.data());
        png_write_end(png, nullptr);
        png_destroy_write_struct(&png, &info);
        fclose(fp);
    }
};

// ─── per-frame render ─────────────────────────────────────────────────────────

template <int N>
void save_frame(const SimulationData<N>& data, const char* path) {
    // Plot bounds (fixed for the Sedov test)
    constexpr float X_MAX = 0.9f;   // radius
    constexpr float Y_MAX = 4.5f;   // density

    constexpr int W = 1280, H = 720;
    // margins: left, right, top, bottom
    constexpr int ML = 50, MR = 20, MT = 20, MB = 40;
    constexpr int PW = W - ML - MR;
    constexpr int PH = H - MT - MB;

    Image img(W, H);

    // light-gray plot background
    for (int y = MT; y < H - MB; y++)
        for (int x = ML; x < W - MR; x++)
            img.set(x, y, 245, 245, 245);

    // horizontal grid lines at integer density values
    for (int d = 1; d <= (int)Y_MAX; d++) {
        int py = MT + (int)((1.0f - d / Y_MAX) * PH);
        img.hline(ML, W - MR, py, 210, 210, 210);
    }

    // border
    img.hline(ML, W - MR, MT,     50, 50, 50);
    img.hline(ML, W - MR, H - MB, 50, 50, 50);
    img.vline(ML,     MT, H - MB, 50, 50, 50);
    img.vline(W - MR, MT, H - MB, 50, 50, 50);

    // particles — matplotlib C0 blue
    for (int i = 0; i < N; i++) {
        float radius  = mag(data.pos[i]);
        float density = data.density[i];

        int px = ML + (int)(radius  / X_MAX * PW);
        int py = MT + (int)((1.0f - density / Y_MAX) * PH);
        img.dot(px, py, 31, 119, 180);
    }

    img.save(path);
}

// ─── main ─────────────────────────────────────────────────────────────────────

int main() {
    constexpr int   N          = 250047;
    constexpr float total_time = 10e-2f;

    std::filesystem::create_directories("../frames");

    auto* host = new SimulationData<N>();
    ParticleFile pf = load_particles("../springel_sedov_smeared.0000");
    copy_to_particle_data(pf, *host);
    host->smoothing_length = 0.03984126984126984;

    Simulation sim { host, false };

    float time  = 0.0f;
    int   frame = 0;
    while (time < total_time) {
        auto result = sim.step_and_fetch();
        time += result.delta_time;

        char path[64];
        snprintf(path, sizeof(path), "../frames/frame_%06d.png", frame);
        save_frame(*host, path);

        printf("[%3d%%] t=%.5f  -> %s\n", (int)(time / total_time * 100), time, path);
        frame++;
    }

    delete host;

    printf("Encoding video...\n");
    int ret = std::system(
        "ffmpeg -y -framerate 30"
        " -i ../frames/frame_%06d.png"
        " -c:v libx264 -pix_fmt yuv420p -crf 18"
        " ../output.mp4"
    );
    if (ret == 0) printf("Video saved to ../output.mp4\n");
    else          fprintf(stderr, "ffmpeg failed (exit code %d)\n", ret);

    return 0;
}
