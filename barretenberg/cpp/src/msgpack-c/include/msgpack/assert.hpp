//
// MessagePack for C++ static resolution routine
//
// Copyright (C) 2022 KONDO Takatoshi
//
//    Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_1_0.txt or copy at
//    http://www.boost.org/LICENSE_1_0.txt)
//

#ifndef MSGPACK_ASSERT_HPP
#define MSGPACK_ASSERT_HPP

#if defined(MSGPACK_NO_BOOST)

#include <cassert>
#define MSGPACK_ASSERT assert

#else  // defined(MSGPACK_NO_BOOST)

#include <boost/assert.hpp>
#define MSGPACK_ASSERT BOOST_ASSERT

#endif // defined(MSGPACK_NO_BOOST)

#ifdef __wasm__
struct AbortStream {
    void operator<< [[noreturn]] (const auto& error) {
        info(error.what());
        std::abort();
    }
};
#define THROW AbortStream() <<
#define try if (true)
#define catch(...) if (false)
#define RETHROW 
#else
#define THROW throw
#define RETHROW THROW
#endif

#endif // MSGPACK_ASSERT_HPP
