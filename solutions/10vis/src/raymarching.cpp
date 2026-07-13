#include "raymarching.hpp"

#include <cstring>

#include <fstream>
#include <iostream>

std::string err_to_str(const RaymarchingError& err) {
    switch (err)
    {
    case RaymarchingError::GLFW_INIT_FAIL: return "glfw init fail";
    case RaymarchingError::OPEN_WINDOW_FAIL: return "open window init fail";
    case RaymarchingError::GLAD_INIT_FAIL: return "glad init fail";
    case RaymarchingError::WINDOW_NOT_OPEN: return "window not open";
    case RaymarchingError::SHADER_LOAD_FAIL: return "shader could not be loaded";
    case RaymarchingError::POINT_BUF_MAP_FAILED: return "point buffer mapping failed";
    default: return "Unknown error";
    }
}

// Static wrapper for GLFW mouse callback
void mouse_callback_wrapper(GLFWwindow* window, double xpos, double ypos) {
    auto* rm = static_cast<Raymarching*>(glfwGetWindowUserPointer(window));
    if (rm) rm->mouse_callback(window, xpos, ypos);
}

// Static wrapper for GLFW framebuffer size callback
void framebuffer_size_callback_wrapper(GLFWwindow* window, int width, int height) {
    auto* rm = static_cast<Raymarching*>(glfwGetWindowUserPointer(window));
    if (rm) rm->framebuffer_size_callback(window, width, height);
}

// ==========================================
// CLASS RAYMARCHING
// ==========================================
// PUBLIC
//
//
Raymarching::Raymarching() : _data_thread{} {
    auto err = initialize();
    if (err) {
        log("Error: " + err_to_str(err.value()));
        return;
    }

    _is_initialized = true;
}
Raymarching::~Raymarching() {
    cleanup();
}

void Raymarching::set_data(std::vector<Point>& points) {
    std::lock_guard m(_access_points_mutex);

    // ...
}

bool should_close_window(GLFWwindow *window) {
    return
        glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS
        || glfwWindowShouldClose(window);
}
RMError Raymarching::run(std::function<std::span<Point>()> get_points) {
    if (!_is_initialized) return RaymarchingError::NOT_INITIALIZED;
    if (!_window) return RaymarchingError::WINDOW_NOT_OPEN;

    // Start data thread
    _get_points = get_points;
    _data_thread = std::thread { &Raymarching::data_loop_async, this };

    log("starting render loop...");

    // Render loop
    while(!should_close_window(_window)){
        process_input(_window);
        sync_camera();
        const auto sp_err = sync_points();
        if (sp_err) return sp_err;

        const auto err = display();
        if (err) return err;

        glfwSwapBuffers(_window);
        glfwPollEvents();
    }

    log("waiting for data thread to complete...");
    {
        std::lock_guard l(_stop_data_thread_mutex);
        _stop_data_thread = true;
    }
    _data_thread.join();

    log("finished rendering");

    return {};
}

// PRIVATE
//
//

RMError Raymarching::initialize() {
    return
        init_gl_libs()
        .or_else([this](){ return init_gl(); });
}
void Raymarching::cleanup() {
    if (!_is_initialized) return;
}

RMError Raymarching::init_gl_libs() {
    if ( !glfwInit() ){
        return RaymarchingError::GLFW_INIT_FAIL;
    }

    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    _window = glfwCreateWindow(512, 512, "raymarching", NULL, NULL);

    if ( !_window ){
        return RaymarchingError::OPEN_WINDOW_FAIL;
    }

    glfwMakeContextCurrent(_window);

    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        return RaymarchingError::GLAD_INIT_FAIL;
    }

    return {};
}

GLuint Raymarching::load_shader(std::string path, GLenum shader_type) {
    std::string shader_code;

    std::ifstream file_stream(path, std::ios::in);
    if(file_stream.is_open()) {
        shader_code = std::string(  std::istreambuf_iterator<char>(file_stream),
                                    std::istreambuf_iterator<char>());
        file_stream.close();
    }
    else{
        log("Could not open " + path);
        return 0;
    }

    const char *c_str_shader_code = shader_code.c_str();

    GLuint shader = glCreateShader(shader_type);
    glShaderSource(shader, 1, &c_str_shader_code , NULL);
    glCompileShader(shader);
    return shader;
}
GLuint Raymarching::load_shaders(std::string vert, std::string frag) {
    GLuint vshader_id = load_shader(vert, GL_VERTEX_SHADER);
    GLuint fshader_id = load_shader(frag, GL_FRAGMENT_SHADER);

    if (!vshader_id || !fshader_id) return 0;

    GLint success = GL_FALSE;

    glGetShaderiv(vshader_id, GL_COMPILE_STATUS, &success);
    if(!success) {
        char shader_err[255];
        glGetShaderInfoLog(vshader_id, 255, NULL, shader_err);
        log("Error at vertex shader compilation: " + std::string(shader_err));
        glDeleteShader(vshader_id);
        return 0;
    }

    glGetShaderiv(fshader_id, GL_COMPILE_STATUS, &success);
    if(!success) {
        char shader_err[255];
        glGetShaderInfoLog(fshader_id, 255, NULL, shader_err);
        log("Error at vertex shader compilation: " + std::string(shader_err));
        glDeleteShader(vshader_id);
        glDeleteShader(fshader_id);
        return 0;
    }

    GLuint program_id = glCreateProgram();
    glAttachShader(program_id, vshader_id);
    glAttachShader(program_id, fshader_id);
    glLinkProgram(program_id);

    glGetProgramiv(program_id, GL_LINK_STATUS, &success);
    if(!success) {
        char link_err[255];
        glGetShaderInfoLog(program_id, 255, NULL, link_err);
        log("Error in linking the shaders: " + std::string(link_err));
        return 0;
    }

    glDeleteShader(vshader_id);
    glDeleteShader(fshader_id);

    return program_id;
}
RMError Raymarching::init_gl() {
    glClearColor(0.4, 0.5, 0.6, 1.0);

    // set correct viewport size

    {
        int width, height;
        glfwGetFramebufferSize(_window, &width, &height);
        glViewport(0, 0, width, height);
    }

    // shaders

    GLuint shader_id = load_shaders("vertex.glsl", "frag.glsl");

    if(!shader_id){
        return RaymarchingError::SHADER_LOAD_FAIL;
    }

    glUseProgram(shader_id);

    //init empty VAO, as we just use the fixed screen triangle in the vert shader
    GLuint vao_id;
    glGenVertexArrays(1, &vao_id);
    glBindVertexArray(vao_id);

    // Point data
    glGenBuffers(1, &_points_ssbo);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, _points_ssbo);
    glBufferData(GL_SHADER_STORAGE_BUFFER, 0, nullptr, GL_DYNAMIC_DRAW);

    glBindBufferBase(GL_SHADER_STORAGE_BUFFER, POINT_SSBO_BUFFER_BINDING, _points_ssbo);

    // Config
    glGenBuffers(1, &_config_ubo);
    glBindBuffer(GL_UNIFORM_BUFFER, _config_ubo);
    glBufferData(GL_UNIFORM_BUFFER, sizeof(ConfigData), nullptr, GL_DYNAMIC_DRAW);

    glBindBufferBase(GL_UNIFORM_BUFFER, CONFIG_UBO_BINDING, _config_ubo);

    // Camera
    glGenBuffers(1, &_camera_ubo);
    glBindBuffer(GL_UNIFORM_BUFFER, _camera_ubo);
    glBufferData(GL_UNIFORM_BUFFER, sizeof(CameraData), nullptr, GL_DYNAMIC_DRAW);
    glBindBufferBase(GL_UNIFORM_BUFFER, CAMERA_UBO_BINDING, _camera_ubo);

    // GLFW stuff
    glfwSetWindowUserPointer(_window, this);
    glfwSetCursorPosCallback(_window, mouse_callback_wrapper);
    glfwSetFramebufferSizeCallback(_window, framebuffer_size_callback_wrapper);
    glfwSetInputMode(_window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);

    log("succesfully initialized OpenGL");

    return {};
}

RMError Raymarching::display() {
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawArrays(GL_TRIANGLES, 0, 3);

    return {};
}
RMError Raymarching::sync_points() {
    if (_points.empty()) return {};

    // update points ssbo buffer

    glBindBuffer(GL_SHADER_STORAGE_BUFFER, _points_ssbo);

    std::lock_guard l(_access_points_mutex);

    const auto size_bytes = _points.size() * sizeof(Point);

    if (_points.size() > _points_ssbo_capacity || _points_ssbo_capacity == 0) {
        _points_ssbo_capacity = _points.size();
        glBufferData(GL_SHADER_STORAGE_BUFFER, size_bytes, _points.data(), GL_DYNAMIC_DRAW);
    } else {
        void* ptr = glMapBufferRange(GL_SHADER_STORAGE_BUFFER, 0, size_bytes,
            GL_MAP_WRITE_BIT | GL_MAP_INVALIDATE_BUFFER_BIT);
        if (!ptr) {
            return RaymarchingError::POINT_BUF_MAP_FAILED;
        }
        std::memcpy(ptr, _points.data(), size_bytes);
        glUnmapBuffer(GL_SHADER_STORAGE_BUFFER);
    }

    // update config ubo

    int width, height;
    glfwGetFramebufferSize(_window, &width, &height);
    float res_x = static_cast<float>(width);
    float res_y = static_cast<float>(height);

    ConfigData cfg {
        point_radius,
        static_cast<int>(_points.size()),
        epsilon,
        res_x, res_y
    };

    glBindBuffer(GL_UNIFORM_BUFFER, _config_ubo);
    glBufferSubData(GL_UNIFORM_BUFFER, 0, sizeof(ConfigData), &cfg);

    return {};
}

void Raymarching::log(std::string msg) {
    std::cout << "[Raymarching] " << msg << "\n";
}

void Raymarching::sync_camera() {
    glm::vec3 forward;
    forward.x = cos(glm::radians(_camera_yaw)) * cos(glm::radians(_camera_pitch));
    forward.y = sin(glm::radians(_camera_pitch));
    forward.z = sin(glm::radians(_camera_yaw)) * cos(glm::radians(_camera_pitch));
    forward = glm::normalize(forward);

    glm::vec3 right = glm::normalize(glm::cross(forward, glm::vec3(0, 1, 0)));
    glm::vec3 up = glm::cross(right, forward);

    // 0s are the padding
    CameraData cam {
        _camera_pos, 0.0f,
        forward, 0.0f,
        right, 0.0f,
        up, 0.0f
    };

    glBindBuffer(GL_UNIFORM_BUFFER, _camera_ubo);
    glBufferSubData(GL_UNIFORM_BUFFER, 0, sizeof(CameraData), &cam);
}

void Raymarching::process_input(GLFWwindow* window) {
    glm::vec3 forward;
    forward.x = cos(glm::radians(_camera_yaw)) * cos(glm::radians(_camera_pitch));
    forward.y = sin(glm::radians(_camera_pitch));
    forward.z = sin(glm::radians(_camera_yaw)) * cos(glm::radians(_camera_pitch));
    forward = glm::normalize(forward);

    glm::vec3 right = glm::normalize(glm::cross(forward, glm::vec3(0, 1, 0)));

    if (glfwGetKey(window, GLFW_KEY_W) == GLFW_PRESS)
        _camera_pos += _camera_speed * forward;
    if (glfwGetKey(window, GLFW_KEY_S) == GLFW_PRESS)
        _camera_pos -= _camera_speed * forward;
    if (glfwGetKey(window, GLFW_KEY_A) == GLFW_PRESS)
        _camera_pos -= _camera_speed * right;
    if (glfwGetKey(window, GLFW_KEY_D) == GLFW_PRESS)
        _camera_pos += _camera_speed * right;
    if (glfwGetKey(window, GLFW_KEY_E) == GLFW_PRESS)
        _camera_pos.y += _camera_speed;
    if (glfwGetKey(window, GLFW_KEY_Q) == GLFW_PRESS)
        _camera_pos.y -= _camera_speed;
}


void Raymarching::framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void Raymarching::mouse_callback(GLFWwindow* window, double xpos, double ypos) {
    if (_first_mouse) {
        _last_x = xpos;
        _last_y = ypos;
        _first_mouse = false;
    }

    float xoffset = xpos - _last_x;
    float yoffset = _last_y - ypos; 
    _last_x = xpos;
    _last_y = ypos;

    xoffset *= _mouse_sensitivity;
    yoffset *= _mouse_sensitivity;

    _camera_yaw += xoffset;
    _camera_pitch += yoffset;

    if (_camera_pitch > 89.0f) _camera_pitch = 89.0f;
    if (_camera_pitch < -89.0f) _camera_pitch = -89.0f;
}

void Raymarching::data_loop_async() {
    while (true) {
        {
            std::lock_guard l(_stop_data_thread_mutex);
            if (_stop_data_thread) return;
        }

        std::lock_guard l(_access_points_mutex);
        const auto points = _get_points();

        _points.clear();
        _points.reserve(points.size());
        _points.append_range(points);
    }
}