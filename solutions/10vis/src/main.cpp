#include <iostream>

#include "raymarching.hpp"

int main() {
    auto rm = Raymarching{};

    auto v = std::vector<Point>{
        Point{ 0, 0, 0 },
    };
    rm.run([&v]() -> std::span<Point> { return std::span{v}; });
}