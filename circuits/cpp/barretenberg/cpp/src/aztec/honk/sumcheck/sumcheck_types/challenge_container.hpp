#pragma once
#include <array>
#include <stddef.h>
#include "../transcript.hpp"

namespace honk {
namespace sumcheck {
template <class Fr, class Transcript, class Univariate> class ChallengeContainer {
  public:
    Fr get_constraint_separator_challenge() { return transcript.get_challenge(); }; // these are powers of a challenge
    // Fr get_constraint_bliding_base(){return transcript.get_challenge(1);} // will get element zeta as well

    Transcript transcript; // TODO(cody):really a pointer to such a thing?
    ChallengeContainer(Transcript transcript)
        : transcript(transcript){};
    Fr get_sumcheck_round_challenge(Univariate univariate_restriction)
    {
        return transcript.get_challenge();
    }; // this is u_l

    Fr get_challenge_equals_one() { return transcript.get_challenge_equals_one(); };
};
} // namespace sumcheck
} // namespace honk
