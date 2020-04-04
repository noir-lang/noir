#include "./transcript_wrappers.hpp"

namespace transcript {
void StandardTranscript::add_field_element(const std::string& element_name, const barretenberg::fr& element)
{
    add_element(element_name, element.to_buffer());
}

barretenberg::fr StandardTranscript::get_field_element(const std::string& element_name) const
{
    return barretenberg::fr::serialize_from_buffer(&(get_element(element_name))[0]);
}

std::vector<barretenberg::fr> StandardTranscript::get_field_element_vector(const std::string& element_name) const
{
    return barretenberg::fr::from_buffer(get_element(element_name));
}

barretenberg::fr StandardTranscript::get_challenge_field_element(const std::string& challenge_name,
                                                                 const size_t idx) const
{
    return barretenberg::fr::serialize_from_buffer(&(get_challenge(challenge_name, idx))[0]);
}

barretenberg::fr StandardTranscript::get_challenge_field_element_from_map(const std::string& challenge_name,
                                                                          const std::string& challenge_map_name) const
{
    return barretenberg::fr::serialize_from_buffer(&(get_challenge_from_map(challenge_name, challenge_map_name))[0]);
}
} // namespace transcript