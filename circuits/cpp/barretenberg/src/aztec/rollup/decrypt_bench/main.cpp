#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <chrono>

int main(int, char**)
{

    constexpr size_t num_points = 1024;

    std::vector<grumpkin::g1::element> points;
    for (size_t i = 0; i < num_points; ++i) {
        points.emplace_back(grumpkin::g1::element::random_element());
    }
    grumpkin::g1::element::batch_normalize(&points[0], num_points);

    std::vector<grumpkin::g1::affine_element> affine_points;
    for (size_t i = 0; i < num_points; ++i) {
        affine_points.emplace_back(points[i]);
    }
    const grumpkin::fr exponent = grumpkin::fr::random_element();

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();

    std::vector<grumpkin::g1::element> expected;
    for (const auto& point : points) {
        expected.emplace_back((point * exponent).normalize());
    }

    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "regular mul operations: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();

    const auto result = grumpkin::g1::element::batch_mul_with_endomorphism(affine_points, exponent);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "batched mul operations: " << diff.count() << "ms" << std::endl;
}