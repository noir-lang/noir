#pragma once
#include "exec_pipe.hpp"
#include "file_io.hpp"
#include "log.hpp"
#include <barretenberg/ecc/curves/bn254/g1.hpp>
#include <barretenberg/srs/io.hpp>
#include <filesystem>
#include <fstream>
#include <ios>

namespace bb {

std::vector<curve::Grumpkin::AffineElement> get_grumpkin_g1_data(const std::filesystem::path& path, size_t num_points);

}