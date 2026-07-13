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
};

float scene_dist(vec3 p) {
    float min_dist = 1e10;
    for (int i = 0; i < point_count; i++) {
        vec3 scene_point = vec3(points[i].x, points[i].y, points[i].z);
        float d = length(p - scene_point) - point_radius;
        if (d < min_dist) min_dist = d;
    }
    return min_dist;
}

void main() {
    vec2 uv = (gl_FragCoord.xy * 2.0 - vec2(512.0)) / 512.0;
    
    vec3 ray_origin = vec3(0.0, 0.0, -5.0);
    vec3 ray_dir = normalize(vec3(uv, 1.0));
    
    float t = 0.0;
    for (int i = 0; i < 128; i++) {
        float d = scene_dist(ray_origin + ray_dir * t);
        if (d < epsilon) {
            FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            return;
        }
        t += d;
        if (t > 20.0) break;
    }
    
    FragColor = vec4(0.1, 0.1, 0.1, 1.0);
}