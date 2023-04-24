#pragma once
#include <iostream>
#include <string>
#include <vector>

namespace aztec3::utils {

class DummyComposer {
  public:
    std::vector<std::string> failure_msgs;

    void do_assert(bool const& assertion, std::string const& msg)
    {
        if (!assertion) {
            failure_msgs.push_back(msg);
        }
    }

    bool failed() { return failure_msgs.size() > 0; }

    std::string get_first_failure()
    {
        if (failed()) {
            return failure_msgs[0];
        }
        return "";
    }
};

} // namespace aztec3::utils
