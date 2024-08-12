#pragma once

#include "barretenberg/vm/avm/generated/relations/poseidon2.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"

#include <array>
#include <cstdint>
#include <vector>

namespace bb::avm_trace {

class AvmPoseidon2TraceBuilder {
  public:
    struct Poseidon2TraceEntry {
        uint32_t clk = 0;
        std::array<FF, 4> input;
        std::array<FF, 4> output;
        std::array<FF, 4> first_ext;
        std::array<std::array<FF, 4>, 64> interm_round_vals;
        uint32_t input_addr;
        uint32_t output_addr;
    };

    AvmPoseidon2TraceBuilder() = default;
    void reset();
    // Finalize the trace
    std::vector<Poseidon2TraceEntry> finalize();

    std::array<FF, 4> poseidon2_permutation(std::array<FF, 4> const& input,
                                            uint32_t clk,
                                            uint32_t input_addr,
                                            uint32_t output_addr);

  private:
    std::vector<Poseidon2TraceEntry> poseidon2_trace;
};

template <typename DestRow> void merge_into(DestRow& dest, const AvmPoseidon2TraceBuilder::Poseidon2TraceEntry& src)
{
    // Mem Stuff
    dest.poseidon2_a_0 = src.input[0];
    dest.poseidon2_a_1 = src.input[1];
    dest.poseidon2_a_2 = src.input[2];
    dest.poseidon2_a_3 = src.input[3];
    dest.poseidon2_b_0 = src.output[0];
    dest.poseidon2_b_1 = src.output[1];
    dest.poseidon2_b_2 = src.output[2];
    dest.poseidon2_b_3 = src.output[3];
    dest.poseidon2_input_addr = src.input_addr;
    dest.poseidon2_output_addr = src.output_addr;
    dest.poseidon2_mem_addr_read_a = src.input_addr;
    dest.poseidon2_mem_addr_read_b = src.input_addr + 1;
    dest.poseidon2_mem_addr_read_c = src.input_addr + 2;
    dest.poseidon2_mem_addr_read_d = src.input_addr + 3;
    dest.poseidon2_mem_addr_write_a = src.output_addr;
    dest.poseidon2_mem_addr_write_b = src.output_addr + 1;
    dest.poseidon2_mem_addr_write_c = src.output_addr + 2;
    dest.poseidon2_mem_addr_write_d = src.output_addr + 3;
    dest.poseidon2_sel_poseidon_perm = FF(1);
    // First Ext Round
    dest.poseidon2_EXT_LAYER_6 = src.first_ext[0];
    dest.poseidon2_EXT_LAYER_5 = src.first_ext[1];
    dest.poseidon2_EXT_LAYER_7 = src.first_ext[2];
    dest.poseidon2_EXT_LAYER_4 = src.first_ext[3];
    // Full rounds
    dest.poseidon2_T_0_6 = src.interm_round_vals[0][0];
    dest.poseidon2_T_0_5 = src.interm_round_vals[0][1];
    dest.poseidon2_T_0_7 = src.interm_round_vals[0][2];
    dest.poseidon2_T_0_4 = src.interm_round_vals[0][3];
    dest.poseidon2_T_1_6 = src.interm_round_vals[1][0];
    dest.poseidon2_T_1_5 = src.interm_round_vals[1][1];
    dest.poseidon2_T_1_7 = src.interm_round_vals[1][2];
    dest.poseidon2_T_1_4 = src.interm_round_vals[1][3];
    dest.poseidon2_T_2_6 = src.interm_round_vals[2][0];
    dest.poseidon2_T_2_5 = src.interm_round_vals[2][1];
    dest.poseidon2_T_2_7 = src.interm_round_vals[2][2];
    dest.poseidon2_T_2_4 = src.interm_round_vals[2][3];
    dest.poseidon2_T_3_6 = src.interm_round_vals[3][0];
    dest.poseidon2_T_3_5 = src.interm_round_vals[3][1];
    dest.poseidon2_T_3_7 = src.interm_round_vals[3][2];
    dest.poseidon2_T_3_4 = src.interm_round_vals[3][3];
    // Partial rounds
    dest.poseidon2_B_4_0 = src.interm_round_vals[4][0];
    dest.poseidon2_B_4_1 = src.interm_round_vals[4][1];
    dest.poseidon2_B_4_2 = src.interm_round_vals[4][2];
    dest.poseidon2_B_4_3 = src.interm_round_vals[4][3];
    dest.poseidon2_B_5_0 = src.interm_round_vals[5][0];
    dest.poseidon2_B_5_1 = src.interm_round_vals[5][1];
    dest.poseidon2_B_5_2 = src.interm_round_vals[5][2];
    dest.poseidon2_B_5_3 = src.interm_round_vals[5][3];
    dest.poseidon2_B_6_0 = src.interm_round_vals[6][0];
    dest.poseidon2_B_6_1 = src.interm_round_vals[6][1];
    dest.poseidon2_B_6_2 = src.interm_round_vals[6][2];
    dest.poseidon2_B_6_3 = src.interm_round_vals[6][3];
    dest.poseidon2_B_7_0 = src.interm_round_vals[7][0];
    dest.poseidon2_B_7_1 = src.interm_round_vals[7][1];
    dest.poseidon2_B_7_2 = src.interm_round_vals[7][2];
    dest.poseidon2_B_7_3 = src.interm_round_vals[7][3];
    dest.poseidon2_B_8_0 = src.interm_round_vals[8][0];
    dest.poseidon2_B_8_1 = src.interm_round_vals[8][1];
    dest.poseidon2_B_8_2 = src.interm_round_vals[8][2];
    dest.poseidon2_B_8_3 = src.interm_round_vals[8][3];
    dest.poseidon2_B_9_0 = src.interm_round_vals[9][0];
    dest.poseidon2_B_9_1 = src.interm_round_vals[9][1];
    dest.poseidon2_B_9_2 = src.interm_round_vals[9][2];
    dest.poseidon2_B_9_3 = src.interm_round_vals[9][3];
    dest.poseidon2_B_10_0 = src.interm_round_vals[10][0];
    dest.poseidon2_B_10_1 = src.interm_round_vals[10][1];
    dest.poseidon2_B_10_2 = src.interm_round_vals[10][2];
    dest.poseidon2_B_10_3 = src.interm_round_vals[10][3];
    dest.poseidon2_B_11_0 = src.interm_round_vals[11][0];
    dest.poseidon2_B_11_1 = src.interm_round_vals[11][1];
    dest.poseidon2_B_11_2 = src.interm_round_vals[11][2];
    dest.poseidon2_B_11_3 = src.interm_round_vals[11][3];
    dest.poseidon2_B_12_0 = src.interm_round_vals[12][0];
    dest.poseidon2_B_12_1 = src.interm_round_vals[12][1];
    dest.poseidon2_B_12_2 = src.interm_round_vals[12][2];
    dest.poseidon2_B_12_3 = src.interm_round_vals[12][3];
    dest.poseidon2_B_13_0 = src.interm_round_vals[13][0];
    dest.poseidon2_B_13_1 = src.interm_round_vals[13][1];
    dest.poseidon2_B_13_2 = src.interm_round_vals[13][2];
    dest.poseidon2_B_13_3 = src.interm_round_vals[13][3];
    dest.poseidon2_B_14_0 = src.interm_round_vals[14][0];
    dest.poseidon2_B_14_1 = src.interm_round_vals[14][1];
    dest.poseidon2_B_14_2 = src.interm_round_vals[14][2];
    dest.poseidon2_B_14_3 = src.interm_round_vals[14][3];
    dest.poseidon2_B_15_0 = src.interm_round_vals[15][0];
    dest.poseidon2_B_15_1 = src.interm_round_vals[15][1];
    dest.poseidon2_B_15_2 = src.interm_round_vals[15][2];
    dest.poseidon2_B_15_3 = src.interm_round_vals[15][3];
    dest.poseidon2_B_16_0 = src.interm_round_vals[16][0];
    dest.poseidon2_B_16_1 = src.interm_round_vals[16][1];
    dest.poseidon2_B_16_2 = src.interm_round_vals[16][2];
    dest.poseidon2_B_16_3 = src.interm_round_vals[16][3];
    dest.poseidon2_B_17_0 = src.interm_round_vals[17][0];
    dest.poseidon2_B_17_1 = src.interm_round_vals[17][1];
    dest.poseidon2_B_17_2 = src.interm_round_vals[17][2];
    dest.poseidon2_B_17_3 = src.interm_round_vals[17][3];
    dest.poseidon2_B_18_0 = src.interm_round_vals[18][0];
    dest.poseidon2_B_18_1 = src.interm_round_vals[18][1];
    dest.poseidon2_B_18_2 = src.interm_round_vals[18][2];
    dest.poseidon2_B_18_3 = src.interm_round_vals[18][3];
    dest.poseidon2_B_19_0 = src.interm_round_vals[19][0];
    dest.poseidon2_B_19_1 = src.interm_round_vals[19][1];
    dest.poseidon2_B_19_2 = src.interm_round_vals[19][2];
    dest.poseidon2_B_19_3 = src.interm_round_vals[19][3];
    dest.poseidon2_B_20_0 = src.interm_round_vals[20][0];
    dest.poseidon2_B_20_1 = src.interm_round_vals[20][1];
    dest.poseidon2_B_20_2 = src.interm_round_vals[20][2];
    dest.poseidon2_B_20_3 = src.interm_round_vals[20][3];
    dest.poseidon2_B_21_0 = src.interm_round_vals[21][0];
    dest.poseidon2_B_21_1 = src.interm_round_vals[21][1];
    dest.poseidon2_B_21_2 = src.interm_round_vals[21][2];
    dest.poseidon2_B_21_3 = src.interm_round_vals[21][3];
    dest.poseidon2_B_22_0 = src.interm_round_vals[22][0];
    dest.poseidon2_B_22_1 = src.interm_round_vals[22][1];
    dest.poseidon2_B_22_2 = src.interm_round_vals[22][2];
    dest.poseidon2_B_22_3 = src.interm_round_vals[22][3];
    dest.poseidon2_B_23_0 = src.interm_round_vals[23][0];
    dest.poseidon2_B_23_1 = src.interm_round_vals[23][1];
    dest.poseidon2_B_23_2 = src.interm_round_vals[23][2];
    dest.poseidon2_B_23_3 = src.interm_round_vals[23][3];
    dest.poseidon2_B_24_0 = src.interm_round_vals[24][0];
    dest.poseidon2_B_24_1 = src.interm_round_vals[24][1];
    dest.poseidon2_B_24_2 = src.interm_round_vals[24][2];
    dest.poseidon2_B_24_3 = src.interm_round_vals[24][3];
    dest.poseidon2_B_25_0 = src.interm_round_vals[25][0];
    dest.poseidon2_B_25_1 = src.interm_round_vals[25][1];
    dest.poseidon2_B_25_2 = src.interm_round_vals[25][2];
    dest.poseidon2_B_25_3 = src.interm_round_vals[25][3];
    dest.poseidon2_B_26_0 = src.interm_round_vals[26][0];
    dest.poseidon2_B_26_1 = src.interm_round_vals[26][1];
    dest.poseidon2_B_26_2 = src.interm_round_vals[26][2];
    dest.poseidon2_B_26_3 = src.interm_round_vals[26][3];
    dest.poseidon2_B_27_0 = src.interm_round_vals[27][0];
    dest.poseidon2_B_27_1 = src.interm_round_vals[27][1];
    dest.poseidon2_B_27_2 = src.interm_round_vals[27][2];
    dest.poseidon2_B_27_3 = src.interm_round_vals[27][3];
    dest.poseidon2_B_28_0 = src.interm_round_vals[28][0];
    dest.poseidon2_B_28_1 = src.interm_round_vals[28][1];
    dest.poseidon2_B_28_2 = src.interm_round_vals[28][2];
    dest.poseidon2_B_28_3 = src.interm_round_vals[28][3];
    dest.poseidon2_B_29_0 = src.interm_round_vals[29][0];
    dest.poseidon2_B_29_1 = src.interm_round_vals[29][1];
    dest.poseidon2_B_29_2 = src.interm_round_vals[29][2];
    dest.poseidon2_B_29_3 = src.interm_round_vals[29][3];
    dest.poseidon2_B_30_0 = src.interm_round_vals[30][0];
    dest.poseidon2_B_30_1 = src.interm_round_vals[30][1];
    dest.poseidon2_B_30_2 = src.interm_round_vals[30][2];
    dest.poseidon2_B_30_3 = src.interm_round_vals[30][3];
    dest.poseidon2_B_31_0 = src.interm_round_vals[31][0];
    dest.poseidon2_B_31_1 = src.interm_round_vals[31][1];
    dest.poseidon2_B_31_2 = src.interm_round_vals[31][2];
    dest.poseidon2_B_31_3 = src.interm_round_vals[31][3];
    dest.poseidon2_B_32_0 = src.interm_round_vals[32][0];
    dest.poseidon2_B_32_1 = src.interm_round_vals[32][1];
    dest.poseidon2_B_32_2 = src.interm_round_vals[32][2];
    dest.poseidon2_B_32_3 = src.interm_round_vals[32][3];
    dest.poseidon2_B_33_0 = src.interm_round_vals[33][0];
    dest.poseidon2_B_33_1 = src.interm_round_vals[33][1];
    dest.poseidon2_B_33_2 = src.interm_round_vals[33][2];
    dest.poseidon2_B_33_3 = src.interm_round_vals[33][3];
    dest.poseidon2_B_34_0 = src.interm_round_vals[34][0];
    dest.poseidon2_B_34_1 = src.interm_round_vals[34][1];
    dest.poseidon2_B_34_2 = src.interm_round_vals[34][2];
    dest.poseidon2_B_34_3 = src.interm_round_vals[34][3];
    dest.poseidon2_B_35_0 = src.interm_round_vals[35][0];
    dest.poseidon2_B_35_1 = src.interm_round_vals[35][1];
    dest.poseidon2_B_35_2 = src.interm_round_vals[35][2];
    dest.poseidon2_B_35_3 = src.interm_round_vals[35][3];
    dest.poseidon2_B_36_0 = src.interm_round_vals[36][0];
    dest.poseidon2_B_36_1 = src.interm_round_vals[36][1];
    dest.poseidon2_B_36_2 = src.interm_round_vals[36][2];
    dest.poseidon2_B_36_3 = src.interm_round_vals[36][3];
    dest.poseidon2_B_37_0 = src.interm_round_vals[37][0];
    dest.poseidon2_B_37_1 = src.interm_round_vals[37][1];
    dest.poseidon2_B_37_2 = src.interm_round_vals[37][2];
    dest.poseidon2_B_37_3 = src.interm_round_vals[37][3];
    dest.poseidon2_B_38_0 = src.interm_round_vals[38][0];
    dest.poseidon2_B_38_1 = src.interm_round_vals[38][1];
    dest.poseidon2_B_38_2 = src.interm_round_vals[38][2];
    dest.poseidon2_B_38_3 = src.interm_round_vals[38][3];
    dest.poseidon2_B_39_0 = src.interm_round_vals[39][0];
    dest.poseidon2_B_39_1 = src.interm_round_vals[39][1];
    dest.poseidon2_B_39_2 = src.interm_round_vals[39][2];
    dest.poseidon2_B_39_3 = src.interm_round_vals[39][3];
    dest.poseidon2_B_40_0 = src.interm_round_vals[40][0];
    dest.poseidon2_B_40_1 = src.interm_round_vals[40][1];
    dest.poseidon2_B_40_2 = src.interm_round_vals[40][2];
    dest.poseidon2_B_40_3 = src.interm_round_vals[40][3];
    dest.poseidon2_B_41_0 = src.interm_round_vals[41][0];
    dest.poseidon2_B_41_1 = src.interm_round_vals[41][1];
    dest.poseidon2_B_41_2 = src.interm_round_vals[41][2];
    dest.poseidon2_B_41_3 = src.interm_round_vals[41][3];
    dest.poseidon2_B_42_0 = src.interm_round_vals[42][0];
    dest.poseidon2_B_42_1 = src.interm_round_vals[42][1];
    dest.poseidon2_B_42_2 = src.interm_round_vals[42][2];
    dest.poseidon2_B_42_3 = src.interm_round_vals[42][3];
    dest.poseidon2_B_43_0 = src.interm_round_vals[43][0];
    dest.poseidon2_B_43_1 = src.interm_round_vals[43][1];
    dest.poseidon2_B_43_2 = src.interm_round_vals[43][2];
    dest.poseidon2_B_43_3 = src.interm_round_vals[43][3];
    dest.poseidon2_B_44_0 = src.interm_round_vals[44][0];
    dest.poseidon2_B_44_1 = src.interm_round_vals[44][1];
    dest.poseidon2_B_44_2 = src.interm_round_vals[44][2];
    dest.poseidon2_B_44_3 = src.interm_round_vals[44][3];
    dest.poseidon2_B_45_0 = src.interm_round_vals[45][0];
    dest.poseidon2_B_45_1 = src.interm_round_vals[45][1];
    dest.poseidon2_B_45_2 = src.interm_round_vals[45][2];
    dest.poseidon2_B_45_3 = src.interm_round_vals[45][3];
    dest.poseidon2_B_46_0 = src.interm_round_vals[46][0];
    dest.poseidon2_B_46_1 = src.interm_round_vals[46][1];
    dest.poseidon2_B_46_2 = src.interm_round_vals[46][2];
    dest.poseidon2_B_46_3 = src.interm_round_vals[46][3];
    dest.poseidon2_B_47_0 = src.interm_round_vals[47][0];
    dest.poseidon2_B_47_1 = src.interm_round_vals[47][1];
    dest.poseidon2_B_47_2 = src.interm_round_vals[47][2];
    dest.poseidon2_B_47_3 = src.interm_round_vals[47][3];
    dest.poseidon2_B_48_0 = src.interm_round_vals[48][0];
    dest.poseidon2_B_48_1 = src.interm_round_vals[48][1];
    dest.poseidon2_B_48_2 = src.interm_round_vals[48][2];
    dest.poseidon2_B_48_3 = src.interm_round_vals[48][3];
    dest.poseidon2_B_49_0 = src.interm_round_vals[49][0];
    dest.poseidon2_B_49_1 = src.interm_round_vals[49][1];
    dest.poseidon2_B_49_2 = src.interm_round_vals[49][2];
    dest.poseidon2_B_49_3 = src.interm_round_vals[49][3];
    dest.poseidon2_B_50_0 = src.interm_round_vals[50][0];
    dest.poseidon2_B_50_1 = src.interm_round_vals[50][1];
    dest.poseidon2_B_50_2 = src.interm_round_vals[50][2];
    dest.poseidon2_B_50_3 = src.interm_round_vals[50][3];
    dest.poseidon2_B_51_0 = src.interm_round_vals[51][0];
    dest.poseidon2_B_51_1 = src.interm_round_vals[51][1];
    dest.poseidon2_B_51_2 = src.interm_round_vals[51][2];
    dest.poseidon2_B_51_3 = src.interm_round_vals[51][3];
    dest.poseidon2_B_52_0 = src.interm_round_vals[52][0];
    dest.poseidon2_B_52_1 = src.interm_round_vals[52][1];
    dest.poseidon2_B_52_2 = src.interm_round_vals[52][2];
    dest.poseidon2_B_52_3 = src.interm_round_vals[52][3];
    dest.poseidon2_B_53_0 = src.interm_round_vals[53][0];
    dest.poseidon2_B_53_1 = src.interm_round_vals[53][1];
    dest.poseidon2_B_53_2 = src.interm_round_vals[53][2];
    dest.poseidon2_B_53_3 = src.interm_round_vals[53][3];
    dest.poseidon2_B_54_0 = src.interm_round_vals[54][0];
    dest.poseidon2_B_54_1 = src.interm_round_vals[54][1];
    dest.poseidon2_B_54_2 = src.interm_round_vals[54][2];
    dest.poseidon2_B_54_3 = src.interm_round_vals[54][3];
    dest.poseidon2_B_55_0 = src.interm_round_vals[55][0];
    dest.poseidon2_B_55_1 = src.interm_round_vals[55][1];
    dest.poseidon2_B_55_2 = src.interm_round_vals[55][2];
    dest.poseidon2_B_55_3 = src.interm_round_vals[55][3];
    dest.poseidon2_B_56_0 = src.interm_round_vals[56][0];
    dest.poseidon2_B_56_1 = src.interm_round_vals[56][1];
    dest.poseidon2_B_56_2 = src.interm_round_vals[56][2];
    dest.poseidon2_B_56_3 = src.interm_round_vals[56][3];
    dest.poseidon2_B_57_0 = src.interm_round_vals[57][0];
    dest.poseidon2_B_57_1 = src.interm_round_vals[57][1];
    dest.poseidon2_B_57_2 = src.interm_round_vals[57][2];
    dest.poseidon2_B_57_3 = src.interm_round_vals[57][3];
    dest.poseidon2_B_58_0 = src.interm_round_vals[58][0];
    dest.poseidon2_B_58_1 = src.interm_round_vals[58][1];
    dest.poseidon2_B_58_2 = src.interm_round_vals[58][2];
    dest.poseidon2_B_58_3 = src.interm_round_vals[58][3];
    dest.poseidon2_B_59_0 = src.interm_round_vals[59][0];
    dest.poseidon2_B_59_1 = src.interm_round_vals[59][1];
    dest.poseidon2_B_59_2 = src.interm_round_vals[59][2];
    dest.poseidon2_B_59_3 = src.interm_round_vals[59][3];
    // Full rounds
    dest.poseidon2_T_60_6 = src.interm_round_vals[60][0];
    dest.poseidon2_T_60_5 = src.interm_round_vals[60][1];
    dest.poseidon2_T_60_7 = src.interm_round_vals[60][2];
    dest.poseidon2_T_60_4 = src.interm_round_vals[60][3];

    dest.poseidon2_T_61_6 = src.interm_round_vals[61][0];
    dest.poseidon2_T_61_5 = src.interm_round_vals[61][1];
    dest.poseidon2_T_61_7 = src.interm_round_vals[61][2];
    dest.poseidon2_T_61_4 = src.interm_round_vals[61][3];

    dest.poseidon2_T_62_6 = src.interm_round_vals[62][0];
    dest.poseidon2_T_62_5 = src.interm_round_vals[62][1];
    dest.poseidon2_T_62_7 = src.interm_round_vals[62][2];
    dest.poseidon2_T_62_4 = src.interm_round_vals[62][3];

    dest.poseidon2_T_63_6 = src.interm_round_vals[63][0];
    dest.poseidon2_T_63_5 = src.interm_round_vals[63][1];
    dest.poseidon2_T_63_7 = src.interm_round_vals[63][2];
    dest.poseidon2_T_63_4 = src.interm_round_vals[63][3];
}

} // namespace bb::avm_trace
