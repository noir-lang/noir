
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Spike_vm {

template <typename FF> struct SpikeRow {
    FF Spike_first{};
    FF Spike_x{};
};

inline std::string get_relation_label_spike(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class spikeImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS{
        2,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            Spike_DECLARE_VIEWS(0);

            auto tmp = (Spike_x - Spike_first);
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using spike = Relation<spikeImpl<FF>>;

} // namespace bb::Spike_vm