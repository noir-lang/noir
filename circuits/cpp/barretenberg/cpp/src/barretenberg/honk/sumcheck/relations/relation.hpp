#pragma once

namespace proof_system::honk::sumcheck {

template <typename FF> struct RelationParameters {
    FF beta;
    FF gamma;
    FF public_input_delta;
};
} // namespace proof_system::honk::sumcheck
