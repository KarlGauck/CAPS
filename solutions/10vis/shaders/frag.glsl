#version 430 core
out vec4 FragColor;

struct Point {
    float x, y, z;
};

layout(std430, binding = 0) buffer PointBuffer {
    Point points[];
};

void main() {
    FragColor = vec4(1.0, 0.5, 0.2, 1.0);
}