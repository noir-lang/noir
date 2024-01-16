#include <string>
#include <vector>

namespace bb::detail {
std::vector<std::string> split(const std::string& str, char delimiter);
// trim from start (in place)
void ltrim(std::string& s);
// trim from end (in place)
void rtrim(std::string& s);
// trim from both ends (in place)
void trim(std::string& s);

// Used to extract variables from a macro #__VA_ARGS__
std::vector<std::string> split_and_trim(const std::string& str, char delimiter);
} // namespace bb::detail