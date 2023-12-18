#include "./factories/crs_factory.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace barretenberg::srs {

// Initializes the crs using files
void init_crs_factory(std::string crs_path);
void init_grumpkin_crs_factory(std::string crs_path);

// Initializes the crs using memory buffers
void init_grumpkin_crs_factory(std::vector<curve::Grumpkin::AffineElement> const& points);
void init_crs_factory(std::vector<barretenberg::g1::affine_element> const& points,
                      barretenberg::g2::affine_element const g2_point);

std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> get_crs_factory();
std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::Grumpkin>> get_grumpkin_crs_factory();

} // namespace barretenberg::srs