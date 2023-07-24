#include "./factories/crs_factory.hpp"

namespace barretenberg::srs {

void init_crs_factory(std::vector<barretenberg::g1::affine_element> const& points,
                      barretenberg::g2::affine_element const g2_point);

void init_crs_factory(std::string crs_path);

std::shared_ptr<barretenberg::srs::factories::CrsFactory> get_crs_factory();
} // namespace barretenberg::srs