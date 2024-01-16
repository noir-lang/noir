#include "slab_allocator.hpp"
#include <barretenberg/common/assert.hpp>
#include <barretenberg/common/log.hpp>
#include <barretenberg/common/mem.hpp>
#include <cstddef>
#include <numeric>
#include <unordered_map>

#define LOGGING 0

/**
 * If we can guarantee that all slabs will be released before the allocator is destroyed, we wouldn't need this.
 * However, there is (and maybe again) cases where a global is holding onto a slab. In such a case you will have
 * issues if the runtime frees the allocator before the slab is released. The effect is subtle, so it's worth
 * protecting against rather than just saying "don't do globals". But you know, don't do globals...
 * (Irony of global slab allocator noted).
 */
namespace {
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
bool allocator_destroyed = false;

// Slabs that are being manually managed by the user.
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
std::unordered_map<void*, std::shared_ptr<void>> manual_slabs;

template <typename... Args> inline void dbg_info(Args... args)
{
#if LOGGING == 1
    info(args...);
#else
    // Suppress warning.
    (void)(sizeof...(args));
#endif
}

/**
 * Allows preallocating memory slabs sized to serve the fact that these slabs of memory follow certain sizing
 * patterns and numbers based on prover system type and circuit size. Without the slab allocator, memory
 * fragmentation prevents proof construction when approaching memory space limits (4GB in WASM).
 *
 * If no circuit_size_hint is given to the constructor, it behaves as a standard memory allocator.
 */
class SlabAllocator {
  private:
    size_t circuit_size_hint_;
    std::map<size_t, std::list<void*>> memory_store;
#ifndef NO_MULTITHREADING
    std::mutex memory_store_mutex;
#endif

  public:
    ~SlabAllocator();
    SlabAllocator() = default;
    SlabAllocator(const SlabAllocator& other) = delete;
    SlabAllocator(SlabAllocator&& other) = delete;
    SlabAllocator& operator=(const SlabAllocator& other) = delete;
    SlabAllocator& operator=(SlabAllocator&& other) = delete;

    void init(size_t circuit_size_hint);

    std::shared_ptr<void> get(size_t size);

    size_t get_total_size();

  private:
    void release(void* ptr, size_t size);
};

SlabAllocator::~SlabAllocator()
{
    allocator_destroyed = true;
    for (auto& e : memory_store) {
        for (auto& p : e.second) {
            aligned_free(p);
        }
    }
}

void SlabAllocator::init(size_t circuit_size_hint)
{
    if (circuit_size_hint <= circuit_size_hint_) {
        return;
    }

    circuit_size_hint_ = circuit_size_hint;

    // Free any existing slabs.
    for (auto& e : memory_store) {
        for (auto& p : e.second) {
            aligned_free(p);
        }
    }
    memory_store.clear();

    dbg_info("slab allocator initing for size: ", circuit_size_hint);

    if (circuit_size_hint == 0ULL) {
        return;
    }

    // Over-allocate because we know there are requests for circuit_size + n. (somewhat arbitrary n = 512)
    size_t overalloc = 512;
    size_t base_size = circuit_size_hint + overalloc;

    std::map<size_t, size_t> prealloc_num;

    // Size comments below assume a base (circuit) size of 2^19, 524288 bytes.

    // /* 0.5 MiB */ prealloc_num[base_size * 1] = 2;        // Batch invert skipped temporary.
    // /*   2 MiB */ prealloc_num[base_size * 4] = 4 +       // Composer base wire vectors.
    //                                             1;        // Miscellaneous.
    // /*   6 MiB */ prealloc_num[base_size * 12] = 2 +      // next_var_index, prev_var_index
    //                                              2;       // real_variable_index, real_variable_tags
    /*  16 MiB */ prealloc_num[base_size * 32] = 11;      // Composer base selector vectors.
    /*  32 MiB */ prealloc_num[base_size * 32 * 2] = 1;   // Miscellaneous.
    /*  50 MiB */ prealloc_num[base_size * 32 * 3] = 1;   // Variables.
    /*  64 MiB */ prealloc_num[base_size * 32 * 4] = 1 +  // SRS monomial points.
                                                     4 +  // Coset-fft wires.
                                                     15 + // Coset-fft constraint selectors.
                                                     8 +  // Coset-fft perm selectors.
                                                     1 +  // Coset-fft sorted poly.
                                                     1 +  // Pippenger point_schedule.
                                                     4;   // Miscellaneous.
    /* 128 MiB */ prealloc_num[base_size * 32 * 8] = 1 +  // Proving key evaluation domain roots.
                                                     2;   // Pippenger point_pairs.

    for (auto& e : prealloc_num) {
        for (size_t i = 0; i < e.second; ++i) {
            auto size = e.first;
            memory_store[size].push_back(aligned_alloc(32, size));
            dbg_info("Allocated memory slab of size: ", size, " total: ", get_total_size());
        }
    }
}

std::shared_ptr<void> SlabAllocator::get(size_t req_size)
{
#ifndef NO_MULTITHREADING
    std::unique_lock<std::mutex> lock(memory_store_mutex);
#endif

    auto it = memory_store.lower_bound(req_size);

    // Can use a preallocated slab that is less than 2 times the requested size.
    if (it != memory_store.end() && it->first < req_size * 2) {
        size_t size = it->first;
        auto* ptr = it->second.back();
        it->second.pop_back();

        if (it->second.empty()) {
            memory_store.erase(it);
        }

        if (req_size >= circuit_size_hint_ && size > req_size + req_size / 10) {
            dbg_info("WARNING: Using memory slab of size: ",
                     size,
                     " for requested ",
                     req_size,
                     " total: ",
                     get_total_size());
        } else {
            dbg_info("Reusing memory slab of size: ", size, " for requested ", req_size, " total: ", get_total_size());
        }

        return { ptr, [this, size](void* p) {
                    if (allocator_destroyed) {
                        aligned_free(p);
                        return;
                    }
                    this->release(p, size);
                } };
    }

    if (req_size > static_cast<size_t>(1024 * 1024)) {
        dbg_info("WARNING: Allocating unmanaged memory slab of size: ", req_size);
    }
    if (req_size % 32 == 0) {
        return { aligned_alloc(32, req_size), aligned_free };
    }
    // NOLINTNEXTLINE(cppcoreguidelines-no-malloc)
    return { malloc(req_size), free };
}

size_t SlabAllocator::get_total_size()
{
    return std::accumulate(memory_store.begin(), memory_store.end(), size_t{ 0 }, [](size_t acc, const auto& kv) {
        return acc + kv.first * kv.second.size();
    });
}

void SlabAllocator::release(void* ptr, size_t size)
{
#ifndef NO_MULTITHREADING
    std::unique_lock<std::mutex> lock(memory_store_mutex);
#endif
    memory_store[size].push_back(ptr);
    // dbg_info("Pooled poly memory of size: ", size, " total: ", get_total_size());
}
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
SlabAllocator allocator;
} // namespace

namespace bb {
void init_slab_allocator(size_t circuit_subgroup_size)
{
    allocator.init(circuit_subgroup_size);
}

// auto init = ([]() {
//     init_slab_allocator(524288);
//     return 0;
// })();

std::shared_ptr<void> get_mem_slab(size_t size)
{
    return allocator.get(size);
}

void* get_mem_slab_raw(size_t size)
{
    auto slab = get_mem_slab(size);
    manual_slabs[slab.get()] = slab;
    return slab.get();
}

void free_mem_slab_raw(void* p)
{
    if (allocator_destroyed) {
        aligned_free(p);
        return;
    }
    manual_slabs.erase(p);
}
} // namespace bb
