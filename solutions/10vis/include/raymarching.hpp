#pragma once

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>
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

#define POINT_SSBO_BUFFER_BINDING 0
struct Point {
    float x, y, z;
};

#define CONFIG_UBO_BINDING 1
struct alignas(16) ConfigData {
    float point_radius;
    int point_count;
    float epsilon;
    float res_x;
    float res_y;
    float _pad[3]; // 16 bytes alignment
};

#define CAMERA_UBO_BINDING 2
struct alignas(16) CameraData {
    glm::vec3 position;
    float _pad1;
    glm::vec3 forward;
    float _pad2;
    glm::vec3 right;
    float _pad3;
    glm::vec3 up;
    float _pad4;
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

    float point_radius = .05f;
    float epsilon = 0.001f;

    void process_input(GLFWwindow* window);
    void mouse_callback(GLFWwindow* window, double xpos, double ypos);
    void sync_camera();
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);

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

    // Camera state
    glm::vec3 _camera_pos{ 0.0f, 0.0f, -5.0f };
    float _camera_yaw = -90.0f;
    float _camera_pitch = 0.0f;
    float _last_x = 256.0f;
    float _last_y = 256.0f;
    bool _first_mouse = true;
    float _camera_speed = 0.05f;
    float _mouse_sensitivity = 0.1f;

    GLuint _camera_ubo;
};