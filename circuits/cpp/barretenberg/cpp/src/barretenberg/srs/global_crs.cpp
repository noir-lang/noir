#include "./global_crs.hpp"
#include "./factories/file_crs_factory.hpp"
#include "./factories/mem_crs_factory.hpp"
#include "barretenberg/common/throw_or_abort.hpp"

namespace {
std::shared_ptr<barretenberg::srs::factories::CrsFactory> crs_factory;
}

namespace barretenberg::srs {

void init_crs_factory(std::vector<g1::affine_element> const& points, g2::affine_element const g2_point)
{
    crs_factory = std::make_shared<factories::MemCrsFactory>(points, g2_point);
}

void init_crs_factory(std::string crs_path)
{
    crs_factory = std::make_shared<factories::FileCrsFactory>(crs_path);
}

std::shared_ptr<factories::CrsFactory> get_crs_factory()
{
    if (!crs_factory) {
        throw_or_abort("You need to initalize the global CRS with a call to init_crs_factory(...)!");
    }
    return crs_factory;
}
} // namespace barretenberg::srs