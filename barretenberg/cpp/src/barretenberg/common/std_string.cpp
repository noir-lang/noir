#include "std_string.hpp"
#include <algorithm>
#include <cctype>
#include <iostream>
#include <locale>
#include <sstream>
#include <vector>

namespace bb::detail {
std::vector<std::string> split(const std::string& str, char delimiter)
{
    std::vector<std::string> result;
    std::istringstream iss(str);
    std::string token;

    while (std::getline(iss, token, delimiter)) {
        result.push_back(token);
    }

    return result;
}

// trim from start (in place)
void ltrim(std::string& s)
{
    s.erase(s.begin(), std::find_if(s.begin(), s.end(), [](unsigned char ch) { return !std::isspace(ch); }));
}

// trim from end (in place)
void rtrim(std::string& s)
{
    s.erase(std::find_if(s.rbegin(), s.rend(), [](unsigned char ch) { return !std::isspace(ch); }).base(), s.end());
}

// trim from both ends (in place)
void trim(std::string& s)
{
    rtrim(s);
    ltrim(s);
}
std::vector<std::string> split_and_trim(const std::string& str, char delimiter)
{
    std::vector<std::string> ret = split(str, delimiter);
    for (std::string& part : ret) {
        trim(part);
    }
    return ret;
}
} // namespace bb::detail