#pragma once

namespace aztec3::circuits::apps {

template <class T> struct isMap {
    static constexpr bool value = false;
};

template <class KeyType, class ValueType> struct isMap<std::map<KeyType, ValueType>> {
    static constexpr bool value = true;
};

} // namespace aztec3::circuits::apps