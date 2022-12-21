#pragma once
#include <cstddef> // for size_t, for now
#include <vector>
#include "transcript.hpp"
#include "../flavor/flavor.hpp"

namespace honk::sumcheck {
// TODO(Cody): This is just for present purposes. I think this kind of structure is nice, but for the purpose of getting
// the PoC working we should link this/replace with existing transcript, then refactor later.
//
// TODO(Cody): needs to know number of rounds?
// TODO(Cody): Univariate class should not be provided as a template parameter?
template <class FF, class Transcript, class Univariate> class ChallengeContainer {
  public:
    Transcript transcript; // TODO(Cody):really a pointer to such a thing?
    explicit ChallengeContainer(Transcript transcript)
        : transcript(transcript){};

    FF get_relation_separator_challenge() { return transcript.get_challenge(); }; // these are powers of a challenge

    // FF get_relation_bliding_base(){return transcript.get_challenge(1);} // will get element zeta as well

    FF get_challenge_equals_one() { return transcript.get_challenge_equals_one(); };

    FF get_grand_product_beta_challenge() { return transcript.get_challenge_equals_one(); };

    FF get_grand_product_gamma_challenge() { return transcript.get_challenge_equals_one(); };

    FF get_sumcheck_round_challenge(size_t) // NOLINT(readability-named-parameter)
    {
        return transcript.get_challenge();
    };

    Univariate get_sumcheck_round_univariate(size_t) // NOLINT(readability-named-parameter)
    {
        Univariate result;
        return result;
    };

    std::vector<FF> get_sumcheck_purported_evaluations()
    {
        std::vector<FF> result{ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
        return result;
    };
    // TODO(Cody): Leaving how things are added to the transcript as a black box for now.
};
} // namespace honk::sumcheck
