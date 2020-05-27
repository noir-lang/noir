#pragma once
#include "polynomial.hpp"

namespace barretenberg {

// Highly optimised read / write of polynomials in little endian montgomery form.
template <typename B> inline void read(B& buf, polynomial& p)
{
    p = polynomial();
    uint32_t size;
    ::read(buf, size);
    p.resize_unsafe(size);
    memcpy(&p[0], buf, size * sizeof(fr));

    if (!is_little_endian()) {
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t i = 0; i < size; ++i) {
            fr& c = p[i];
            c.data[3] = __builtin_bswap64(c.data[3]);
            c.data[2] = __builtin_bswap64(c.data[2]);
            c.data[1] = __builtin_bswap64(c.data[1]);
            c.data[0] = __builtin_bswap64(c.data[0]);
        }
    }
    buf += size * sizeof(fr);
}

inline void write(uint8_t*& buf, polynomial const& p)
{
    auto size = p.get_size();
    ::write(buf, static_cast<uint32_t>(size));
    memcpy(&buf[0], &p[0], size * sizeof(fr));
    buf += size * sizeof(fr);
}

inline void write(std::vector<uint8_t>& buf, polynomial const& p)
{
    auto size = p.get_size();
    ::write(buf, static_cast<uint32_t>(size));
    auto len = (size * sizeof(fr));
    buf.resize(buf.size() + len);
    auto ptr = &*buf.end() - len;
    memcpy(ptr, &p[0], len);
}

inline void read(std::istream& is, polynomial& p)
{
    p = polynomial();
    uint32_t size;
    ::read(is, size);
    p.resize_unsafe(size);
    is.read((char*)&p[0], (std::streamsize)(size * sizeof(fr)));

    if (!is_little_endian()) {
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t i = 0; i < size; ++i) {
            fr& c = p[i];
            c.data[3] = __builtin_bswap64(c.data[3]);
            c.data[2] = __builtin_bswap64(c.data[2]);
            c.data[1] = __builtin_bswap64(c.data[1]);
            c.data[0] = __builtin_bswap64(c.data[0]);
        }
    }
}

inline void write(std::ostream& os, polynomial const& p)
{
    auto size = p.get_size();
    auto len = size * sizeof(fr);
    ::write(os, static_cast<uint32_t>(size));
    os.write((char*)&p[0], (std::streamsize)len);
}

} // namespace barretenberg
