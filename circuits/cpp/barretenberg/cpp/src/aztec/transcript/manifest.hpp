#pragma once
#include <string>
#include <vector>

namespace transcript {
/**
 * Composers used Manifest to define the structure of the protocol:
 * 1. What data is used in each round of the protocols
 * 2. Which information is used to create challenges
 * */
class Manifest {
  public:
    /**
     * ManifestEntry describes one piece of data that is used
     * in a particular round of the protocol
     * */
    struct ManifestEntry {
        std::string name;
        size_t num_bytes;
        bool derived_by_verifier;
        int challenge_map_index = 0;
    };

    /**
     * The RoundManifest describes the data used in one round of the protocol
     * and the challenge(s) created from that data.
     * */
    struct RoundManifest {

        /**
         * @param element_names Data used in the round.
         * @param challenge_name The name of the challenge (alpha, beta, etc..)
         * @param num_challenges_in The number of challenges to generate (sometimes we need more than one, e.g in
         * permutation_widget)
         * @param map_challenges_in Whether to put elements in a challenge_map in the transcript.
         * */
        RoundManifest(std::vector<ManifestEntry> element_names,
                      const std::string challenge_name,
                      const size_t num_challenges_in,
                      bool map_challenges_in = false)
            : elements(element_names)
            , challenge(challenge_name)
            , num_challenges(num_challenges_in)
            , map_challenges(map_challenges_in)
        {}

        /**
         * Checks if there is an element in the list with such name.
         *
         * @param element_name The name to search for.
         *
         * @return true if found, false if not.
         * */
        bool includes_element(const std::string& element_name)
        {
            for (auto ele : elements) {
                if (element_name == ele.name) {
                    return true;
                }
            }
            return false;
        }

        std::vector<ManifestEntry> elements;
        std::string challenge;
        size_t num_challenges;
        bool map_challenges;
    };

    // TODO(luke): needed only in development; can be deleted when appropriate
    Manifest() = default;
    Manifest(std::vector<RoundManifest> _round_manifests)
        : round_manifests(_round_manifests)
        , num_rounds(round_manifests.size()){};

    size_t get_num_rounds() const { return num_rounds; }

    RoundManifest get_round_manifest(const size_t idx) const { return round_manifests[idx]; }

    std::vector<RoundManifest> get_round_manifests() const { return round_manifests; }

  private:
    std::vector<RoundManifest> round_manifests;
    size_t num_rounds;
}; // namespace transcript
} // namespace transcript
