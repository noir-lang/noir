#include "./global_crs.hpp"
#include "./factories/file_crs_factory.hpp"
#include "./factories/mem_bn254_crs_factory.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/srs/factories/mem_grumpkin_crs_factory.hpp"

namespace {
// TODO(#637): As a PoC we have two global variables for the two CRS but this could be improved to avoid duplication.
std::shared_ptr<bb::srs::factories::CrsFactory<bb::curve::BN254>> crs_factory;
std::shared_ptr<bb::srs::factories::CrsFactory<bb::curve::Grumpkin>> grumpkin_crs_factory;
} // namespace

namespace bb::srs {

// Initializes the crs using the memory buffers
void init_crs_factory(std::vector<g1::affine_element> const& points, g2::affine_element const g2_point)
{
    crs_factory = std::make_shared<factories::MemBn254CrsFactory>(points, g2_point);
}

// Initializes crs from a file path this we use in the entire codebase
void init_crs_factory(std::string crs_path)
{
    if (crs_factory != nullptr) {
        return;
    }
#ifdef WASMTIME_ENV_HACK
    static_cast<void>(crs_path);
    // We only need this codepath in wasmtime because the SRS cannot be loaded in our usual ways
    // and we don't need a real CRS for our purposes.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/837): make this a real SRS.
    std::cout << "WASMTIME_ENV_HACK: started generating fake bn254 curve" << std::endl;
    std::vector<g1::affine_element> points;
    // 2**19 points
    points.reserve(1 << 19);
    for (int i = 0; i < (1 << 19); i++) {
        points.push_back(g1::affine_element::random_element());
    }
    init_crs_factory(points, g2::affine_element{ fq::random_element(), fq::random_element() });
    std::cout << "WASMTIME_ENV_HACK: finished generating fake bn254 curve" << std::endl;
#else
    crs_factory = std::make_shared<factories::FileCrsFactory<curve::BN254>>(crs_path);
#endif
}

// Initializes the crs using the memory buffers
void init_grumpkin_crs_factory(std::vector<curve::Grumpkin::AffineElement> const& points)
{
    grumpkin_crs_factory = std::make_shared<factories::MemGrumpkinCrsFactory>(points);
}

void init_grumpkin_crs_factory(std::string crs_path)
{
    if (grumpkin_crs_factory != nullptr) {
        return;
    }
#ifdef WASMTIME_ENV_HACK
    // We only need this codepath in wasmtime because the SRS cannot be loaded in our usual ways
    // and we don't need a real CRS for our purposes.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/837): make this a real SRS.
    static_cast<void>(crs_path);
    std::cout << "WASMTIME_ENV_HACK: started generating fake grumpkin curve" << std::endl;
    std::vector<curve::Grumpkin::AffineElement> points;
    // 2**18 points
    points.reserve(1 << 18);
    for (int i = 0; i < (1 << 18); i++) {
        points.push_back(curve::Grumpkin::AffineElement::random_element());
    }
    std::cout << "WASMTIME_ENV_HACK: finished generating fake grumpkin curve" << std::endl;
    init_grumpkin_crs_factory(points);
#else
    grumpkin_crs_factory = std::make_shared<factories::FileCrsFactory<curve::Grumpkin>>(crs_path);
#endif
}

std::shared_ptr<factories::CrsFactory<curve::BN254>> get_crs_factory()
{
    if (!crs_factory) {
        throw_or_abort("You need to initalize the global CRS with a call to init_crs_factory(...)!");
    }
    return crs_factory;
}

std::shared_ptr<factories::CrsFactory<curve::Grumpkin>> get_grumpkin_crs_factory()
{
    if (!grumpkin_crs_factory) {
        throw_or_abort("You need to initalize the global CRS with a call to init_grumpkin_crs_factory(...)!");
    }
    return grumpkin_crs_factory;
}
} // namespace bb::srs