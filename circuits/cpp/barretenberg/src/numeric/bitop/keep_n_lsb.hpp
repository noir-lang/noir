#include <stddef.h>

namespace numeric {

template <typename T> inline T keep_n_lsb(T input, size_t num_bits)
{
    return num_bits >= sizeof(T) * 8 ? input : input & (((T)1 << num_bits) - 1);
}

}