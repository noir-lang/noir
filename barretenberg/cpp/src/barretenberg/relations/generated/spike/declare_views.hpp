
#define Spike_DECLARE_VIEWS(index)                                                                                     \
    using Accumulator = typename std::tuple_element<index, ContainerOverSubrelations>::type;                           \
    using View = typename Accumulator::View;                                                                           \
    [[maybe_unused]] auto Spike_first = View(new_term.Spike_first);                                                    \
    [[maybe_unused]] auto Spike_kernel_inputs__is_public = View(new_term.Spike_kernel_inputs__is_public);              \
    [[maybe_unused]] auto Spike_x = View(new_term.Spike_x);
