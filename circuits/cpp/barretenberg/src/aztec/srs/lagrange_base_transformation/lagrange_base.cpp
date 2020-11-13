#include "./lagrange_base.hpp"
#include <iostream>
namespace barretenberg {
namespace lagrange_base {

std::vector<g1::element> g1fft(const g1::element* monomials, size_t size, barretenberg::fr root, size_t offset)
{

    std::vector<g1::element> result(size);
    if (size == 1) {
        result[0] = monomials[0];
        return result;
    }

    auto new_root = root * root;
    auto odd = g1fft(monomials + offset, size / 2, new_root, offset * 2);
    auto even = g1fft(monomials, size / 2, new_root, offset * 2);
    auto current = root;

    for (size_t i = 0; i < size / 2; ++i) {
        auto temp = odd[i] * current;
        g1::element temp2;
        temp2 = even[i] + temp;
        result[i] = temp2;
        temp2 = even[i] - temp;
        result[size / 2 + i] = temp2;
        current *= root;
    }
    return result;
}

void transform_srs(g1::affine_element* monomials, g1::affine_element* lagrange_base_affine, const size_t degree)
{
    barretenberg::evaluation_domain domain(degree);
    barretenberg::fr root = domain.root_inverse;
    std::vector<g1::element> monomials_jac(degree);
    for (size_t i = 0; i < degree; ++i) {
        monomials_jac[i] = g1::element(monomials[i].x, monomials[i].y, g1::one.z );
    }

    auto lagrange_jac = g1fft(&monomials_jac[0], degree, root, 1);
    for (size_t i = 0; i < degree - 1; ++i) {
        lagrange_base_affine[i + 1] = static_cast<g1::affine_element>(lagrange_jac[i] * domain.domain_inverse);
    }
    lagrange_base_affine[0] = static_cast<g1::affine_element>(lagrange_jac[degree-1] * domain.domain_inverse);
}

} // namespace lagrange_base
}