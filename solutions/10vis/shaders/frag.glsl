#version 430 core
out vec4 FragColor;

struct Point {
    float x, y, z;
};

layout(std430, binding = 0) buffer PointBuffer {
    Point points[];
};

layout(std140, binding = 1) uniform Config {
    float point_radius;
    int point_count;
};

void main() {
    FragColor = vec4(1.0, 0.5, 0.2, 1.0);
}