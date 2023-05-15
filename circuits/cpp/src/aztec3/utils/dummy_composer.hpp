#pragma once
#include "aztec3/utils/circuit_errors.hpp"

#include <iostream>
#include <string>
#include <utility>
#include <vector>

namespace aztec3::utils {

class DummyComposer {
  public:
    std::vector<CircuitError> failure_msgs;
    // method that created this composer instance. Useful for logging.
    std::string method_name;

    explicit DummyComposer(std::string method_name) : method_name(std::move(method_name)) {}

    void do_assert(bool const& assertion, std::string const& msg, CircuitErrorCode error_code)
    {
        if (!assertion) {
#ifdef __wasm__
            info("Error(", error_code, "): ", msg);
#endif
            failure_msgs.push_back(CircuitError{ error_code, msg });
        }
    }

    [[nodiscard]] bool failed() const { return !failure_msgs.empty(); }

    CircuitError get_first_failure()
    {
        if (failed()) {
            return failure_msgs[0];
        }
        return CircuitError::no_error();
    }

    uint8_t* alloc_and_serialize_first_failure()
    {
        CircuitError const failure = get_first_failure();
        if (failure.code == CircuitErrorCode::NO_ERROR) {
            return nullptr;
        }
        info(this->method_name, ": composer.get_first_failure() = ", failure_msgs[0]);


        // serialize circuit failure to bytes vec
        std::vector<uint8_t> circuit_failure_vec;
        write(circuit_failure_vec, failure);

        // copy to output buffer
        auto* raw_failure_buf = static_cast<uint8_t*>(malloc(circuit_failure_vec.size()));
        memcpy(raw_failure_buf, (void*)circuit_failure_vec.data(), circuit_failure_vec.size());
        return raw_failure_buf;
    }
};

}  // namespace aztec3::utils
