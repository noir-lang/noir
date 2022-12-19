#pragma once
namespace honk {
template <class Fr> class Transcript {
  public:
    // Fr add() // data to the transcript
    Fr get_challenge() { return Fr::random_element(); };
    Fr get_challenge_equals_one() { return Fr::one(); };
    // std::array<...> data
};
}; // namespace honk
