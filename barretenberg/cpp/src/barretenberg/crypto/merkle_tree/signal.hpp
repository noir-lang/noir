#pragma once
#include <atomic>
#include <cstdint>

namespace bb::crypto::merkle_tree {
/**
 * @brief Used in parallel insertions in the IndexedTree. Workers signal to other following workes as they move up
 * the level of the tree.
 *
 */
class Signal {
  public:
    explicit Signal(uint32_t initial_level = 1)
        : signal_(std::make_unique<std::atomic<uint32_t>>(initial_level)){};
    ~Signal() = default;
    Signal(const Signal& other)
        : signal_(std::make_unique<std::atomic<uint32_t>>(other.signal_->load()))
    {}
    Signal(Signal&& other) = default;
    Signal& operator=(const Signal& other)
    {
        if (this != &other) {
            signal_->store(other.signal_->load());
        }
        return *this;
    }
    Signal& operator=(Signal&& other) = default;

    /**
     * @brief Causes the thread to wait until the required level has been signalled
     * @param level The required level
     *
     */
    void wait_for_level(uint32_t level = 0)
    {
        uint32_t current_level = signal_->load();
        while (current_level > level) {
            signal_->wait(current_level);
            current_level = signal_->load();
        }
    }

    /**
     * @brief Signals that the given level has been passed
     * @param level The level to be signalled
     *
     */
    void signal_level(uint32_t level = 0)
    {
        signal_->store(level);
        signal_->notify_all();
    }

    void signal_decrement(uint32_t delta = 1)
    {
        signal_->fetch_sub(delta);
        signal_->notify_all();
    }

  private:
    // apple clang has issues if this cannot be move-constructed, so we wrap in unique_ptr
    std::unique_ptr<std::atomic<uint32_t>> signal_;
};
} // namespace bb::crypto::merkle_tree
