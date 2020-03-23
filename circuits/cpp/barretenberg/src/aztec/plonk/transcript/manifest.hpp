#pragma once
#include <string>
#include <vector>

namespace transcript {
class Manifest {
  public:
    struct ManifestEntry {
        std::string name;
        size_t num_bytes;
        bool derived_by_verifier;
    };
    struct RoundManifest {
        RoundManifest(std::initializer_list<ManifestEntry> element_names, const std::string challenge_name, const size_t num_challenges_in)
            : elements(element_names)
            , challenge(challenge_name)
            , num_challenges(num_challenges_in){};

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
    };
    Manifest(std::initializer_list<RoundManifest> _round_manifests)
        : round_manifests(_round_manifests)
        , num_rounds(round_manifests.size()){};

    size_t get_num_rounds() const { return num_rounds; }

    RoundManifest get_round_manifest(const size_t idx) const { return round_manifests[idx]; }

  private:
    std::vector<RoundManifest> round_manifests;
    size_t num_rounds;
};
} // namespace transcript
