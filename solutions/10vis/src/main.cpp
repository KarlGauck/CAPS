#include <iostream>

#include "raymarching.hpp"

int main() {
    auto rm = Raymarching{};

    auto v = std::vector<Point>{
        Point{ 0, 0, 0 },
        Point{ 1, 0, 0 },
        Point{ 0, 0, 1 },
        Point{ 1, 0, 1 },
    };
    rm.run([&v]() -> std::span<Point> { return std::span{v}; });
}