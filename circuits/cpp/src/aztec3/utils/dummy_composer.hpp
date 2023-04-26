#pragma once
#include "aztec3/utils/circuit_errors.hpp"
#include <iostream>
#include <string>
#include <vector>

namespace aztec3::utils {

class DummyComposer {
  public:
    std::vector<CircuitError> failure_msgs;

    void do_assert(bool const& assertion, std::string const& msg, CircuitErrorCode error_code)
    {
        if (!assertion) {
            failure_msgs.push_back(CircuitError{ error_code, msg });
        }
    }

    bool failed() { return failure_msgs.size() > 0; }

    CircuitError get_first_failure()
    {
        if (failed()) {
            return failure_msgs[0];
        }
        return CircuitError::no_error();
    }

    uint8_t* alloc_and_serialize_first_failure()
    {
        CircuitError failure = get_first_failure();
        if (failure.code == CircuitErrorCode::NO_ERROR) {
            return nullptr;
        }

        // serialize circuit failure to bytes vec
        std::vector<uint8_t> circuit_failure_vec;
        write(circuit_failure_vec, failure);

        // copy to output buffer
        auto raw_failure_buf = (uint8_t*)malloc(circuit_failure_vec.size());
        memcpy(raw_failure_buf, (void*)circuit_failure_vec.data(), circuit_failure_vec.size());
        return raw_failure_buf;
    }

    void log_failures_if_any(std::string from_method_name)
    {
        if (failed()) {
            info(from_method_name, ": composer.failed() = ", failed());
            info(from_method_name, ": composer.get_first_failure() = ", get_first_failure());
        }
    }
};

} // namespace aztec3::utils
