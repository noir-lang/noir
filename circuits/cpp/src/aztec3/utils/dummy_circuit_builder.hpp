#pragma once
#include "aztec3/utils/circuit_errors.hpp"

#include <iostream>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace aztec3::utils {

class DummyCircuitBuilder {
  public:
    std::vector<CircuitError> failure_msgs;
    // method that created this builder instance. Useful for logging.
    std::string method_name;

    explicit DummyCircuitBuilder(std::string method_name) : method_name(std::move(method_name)) {}

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

    /**
     * Returns 'value' as a CircuitResult<T>, unless there was an error.
     * If there was an error, return it instead.
     * @tparam T the value type.
     * @param value the value.
     * @return the value, or last error if it exists.
     */
    template <typename T> CircuitResult<T> result_or_error(const T& value)
    {
        CircuitError const failure = get_first_failure();
        if (failure.code != CircuitErrorCode::NO_ERROR) {
            return CircuitResult<T>{ failure };
        }
        return CircuitResult<T>{ value };
    }

    uint8_t* alloc_and_serialize_first_failure()
    {
        CircuitError const failure = get_first_failure();
        if (failure.code == CircuitErrorCode::NO_ERROR) {
            return nullptr;
        }
        info(this->method_name, ": builder.get_first_failure() = ", failure_msgs[0]);


        // serialize circuit failure to bytes vec
        std::vector<uint8_t> circuit_failure_vec;
        serialize::write(circuit_failure_vec, failure);

        // copy to output buffer
        auto* raw_failure_buf = static_cast<uint8_t*>(malloc(circuit_failure_vec.size()));
        memcpy(raw_failure_buf, (void*)circuit_failure_vec.data(), circuit_failure_vec.size());
        return raw_failure_buf;
    }
};

}  // namespace aztec3::utils
