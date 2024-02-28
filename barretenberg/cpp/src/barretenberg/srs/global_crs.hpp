#include "./factories/crs_factory.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace bb::srs {

// Initializes the crs using files
void init_crs_factory(std::string crs_path);
void init_grumpkin_crs_factory(std::string crs_path);

// Initializes the crs using memory buffers
void init_grumpkin_crs_factory(std::vector<curve::Grumpkin::AffineElement> const& points);
void init_crs_factory(std::vector<bb::g1::affine_element> const& points, bb::g2::affine_element const g2_point);

std::shared_ptr<factories::CrsFactory<curve::BN254>> get_bn254_crs_factory();
std::shared_ptr<factories::CrsFactory<curve::Grumpkin>> get_grumpkin_crs_factory();

template <typename Curve> std::shared_ptr<factories::CrsFactory<Curve>> get_crs_factory();

} // namespace bb::srs