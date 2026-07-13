#pragma once

#include <vector>
#include <optional>
#include <string>
#include <thread>
#include <mutex>
#include <latch>
#include <functional>

#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <GL/gl.h>
#include <GL/glext.h>

// ==========================================
// GPU STRUCTS
// ==========================================

struct Point {
    float x, y, z;
};

struct alignas(16) ConfigData {
    float point_radius;
    int point_count;
    float epsilon;
    float _pad[1]; // 16 bytes alignment
};

// ==========================================
// RAYMARCHING CLASS
// ==========================================

enum class RaymarchingError {
    GLFW_INIT_FAIL,
    OPEN_WINDOW_FAIL,
    GLAD_INIT_FAIL,

    WINDOW_NOT_OPEN,
    SHADER_LOAD_FAIL,
    NOT_INITIALIZED,

    POINT_BUF_MAP_FAILED,
};

using RMError = std::optional<RaymarchingError>;

class Raymarching {
public:
    Raymarching();
    ~Raymarching();

    Raymarching(const Raymarching&) = delete;
    Raymarching& operator=(const Raymarching&) = delete;

    // Runs the main render thread.
    // This blocks until the window is closed.
    RMError run(std::function<std::span<Point>()>);

    float point_radius = .5f;
    float epsilon = 0.001f;

private:

    bool _is_initialized;
    RMError initialize();
    void cleanup();

    RMError init_gl_libs();
    RMError init_gl();

    RMError display();

    void log(std::string);

    GLFWwindow *_window;

    void set_data(std::vector<Point>&);

    // async stuff
    std::thread _data_thread;
    std::mutex _access_points_mutex;
    std::mutex _stop_data_thread_mutex;
    bool _stop_data_thread{ false };

    std::function<std::span<Point>()> _get_points;
    void data_loop_async();
    std::vector<Point> _points;
    RMError sync_points();

    // GL stuff
    GLuint _points_ssbo;
    size_t _points_ssbo_capacity{0};

    GLuint _config_ubo;

    GLuint load_shader(std::string path, GLenum shader_type);
    GLuint load_shaders(std::string vert, std::string frag);
};