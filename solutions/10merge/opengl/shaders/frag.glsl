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
    float epsilon;
    float res_x;
    float res_y;
};

layout(std140, binding = 2) uniform Camera {
    vec3 position;
    vec3 forward;
    vec3 right;
    vec3 up;
};

float scene_dist(vec3 p) {
    float min_dist = 1e10;
    for (int i = 0; i < point_count; i++) {
        vec3 pt = vec3(points[i].x, points[i].y, points[i].z);
        float d = length(p - pt) - point_radius;
        if (d < min_dist) min_dist = d;
    }
    return min_dist;
}

void main() {
    vec2 res = vec2(res_x, res_y);
    vec2 uv = (gl_FragCoord.xy * 2.0 - res) / res;
    
    vec3 rd = normalize(forward + uv.x * right + uv.y * up);
    
    float t = 0.0;
    for (int i = 0; i < 128; i++) {
        float d = scene_dist(position + rd * t);
        if (d < epsilon) {
            FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            return;
        }
        t += d;
        if (t > 20.0) break;
    }
    
    FragColor = vec4(0.1, 0.1, 0.1, 1.0);
}