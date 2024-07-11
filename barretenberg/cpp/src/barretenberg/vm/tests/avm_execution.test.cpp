#include "barretenberg/vm/avm_trace/avm_execution.hpp"

#include <cstdint>
#include <memory>
#include <sys/types.h>

#include "avm_common.test.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/utils.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_deserialization.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/aztec_constants.hpp"
#include "barretenberg/vm/avm_trace/fixed_gas.hpp"

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;
using namespace testing;

using bb::utils::hex_to_bytes;

class AvmExecutionTests : public ::testing::Test {
  public:
    std::vector<FF> public_inputs_vec;
    VmPublicInputs public_inputs;

    AvmExecutionTests()
        : public_inputs_vec(PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH){};

  protected:
    const FixedGasTable& GAS_COST_TABLE = FixedGasTable::get();

    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        srs::init_crs_factory("../srs_db/ignition");
        public_inputs_vec.at(DA_START_GAS_LEFT_PCPI_OFFSET) = DEFAULT_INITIAL_DA_GAS;
        public_inputs_vec.at(L2_START_GAS_LEFT_PCPI_OFFSET) = DEFAULT_INITIAL_L2_GAS;
        public_inputs = Execution::convert_public_inputs(public_inputs_vec);
    };

    /**
     * @brief Generate the execution trace pertaining to the supplied instructions.
     *
     * @param instructions A vector of the instructions to be executed.
     * @return The trace as a vector of Row.
     */
    std::vector<Row> gen_trace_from_instr(std::vector<Instruction> const& instructions) const
    {
        std::vector<FF> calldata{};
        return Execution::gen_trace(instructions, calldata, public_inputs_vec);
    }

    void feed_output(uint32_t output_offset, FF const& value, FF const& side_effect_counter, FF const& metadata)
    {
        std::get<KERNEL_OUTPUTS_VALUE>(public_inputs)[output_offset] = value;
        std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs)[output_offset] = side_effect_counter;
        std::get<KERNEL_OUTPUTS_METADATA>(public_inputs)[output_offset] = metadata;
    };
};

// Basic positive test with an ADD and RETURN opcode.
// Parsing, trace generation and proving is verified.
TEST_F(AvmExecutionTests, basicAddReturn)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) +      // opcode ADD
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "00000007"                 // addr a 7
                               "00000009"                 // addr b 9
                               "00000001"                 // addr c 1
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // 2 instructions
    ASSERT_THAT(instructions, SizeIs(2));

    // ADD
    EXPECT_THAT(instructions.at(0),
                AllOf(Field(&Instruction::op_code, OpCode::ADD),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U8),
                                        VariantWith<uint32_t>(7),
                                        VariantWith<uint32_t>(9),
                                        VariantWith<uint32_t>(1)))));

    // RETURN
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::RETURN),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(0), VariantWith<uint32_t>(0)))));

    auto trace = gen_trace_from_instr(instructions);
    validate_trace(std::move(trace), public_inputs, {}, {}, true);
}

// Positive test for SET and SUB opcodes
TEST_F(AvmExecutionTests, setAndSubOpcodes)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "02"                       // U16
                               "B813"                     // val 47123
                               "000000AA"                 // dst_offset 170
                               + to_hex(OpCode::SET) +    // opcode SET
                               "00"                       // Indirect flag
                               "02"                       // U16
                               "9103"                     // val 37123
                               "00000033"                 // dst_offset 51
                               + to_hex(OpCode::SUB) +    // opcode SUB
                               "00"                       // Indirect flag
                               "02"                       // U16
                               "000000AA"                 // addr a
                               "00000033"                 // addr b
                               "00000001"                 // addr c 1
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(4));

    // SET
    EXPECT_THAT(instructions.at(0),
                AllOf(Field(&Instruction::op_code, OpCode::SET),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U16),
                                        VariantWith<uint16_t>(47123),
                                        VariantWith<uint32_t>(170)))));

    // SET
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::SET),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U16),
                                        VariantWith<uint16_t>(37123),
                                        VariantWith<uint32_t>(51)))));

    // SUB
    EXPECT_THAT(instructions.at(2),
                AllOf(Field(&Instruction::op_code, OpCode::SUB),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U16),
                                        VariantWith<uint32_t>(170),
                                        VariantWith<uint32_t>(51),
                                        VariantWith<uint32_t>(1)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == 1; });
    EXPECT_EQ(row->main_ic, 10000); // 47123 - 37123 = 10000
    validate_trace(std::move(trace), public_inputs, {}, {}, true);
}

// Positive test for multiple MUL opcodes
// We compute 5^12 based on U64 multiplications
// 5 is stored at offset 0 and 1 at offset 1
// Repeat 12 times a multiplication of value
// at offset 0 (5) with value at offset 1 and store
// the result at offset 1.
TEST_F(AvmExecutionTests, powerWithMulOpcodes)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +   // opcode SET
                               "00"                    // Indirect flag
                               "04"                    // U64
                               "00000000"              // val 5 higher 32 bits
                               "00000005"              // val 5 lower 32 bits
                               "00000000"              // dst_offset 0
                               + to_hex(OpCode::SET) + // opcode SET
                               "00"                    // Indirect flag
                               "04"                    // U64
                               "00000000"              // val 1 higher 32 bits
                               "00000001"              // val 1 lower 32 bits
                               "00000001";             // dst_offset 1

    std::string const mul_hex = to_hex(OpCode::MUL) + // opcode MUL
                                "00"                  // Indirect flag
                                "04"                  // U64
                                "00000000"            // addr a
                                "00000001"            // addr b
                                "00000001";           // addr c 1

    std::string const ret_hex = to_hex(OpCode::RETURN) + // opcode RETURN
                                "00"                     // Indirect flag
                                "00000000"               // ret offset 0
                                "00000000";              // ret size 0

    for (int i = 0; i < 12; i++) {
        bytecode_hex.append(mul_hex);
    }

    bytecode_hex.append(ret_hex);

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(15));

    // MUL first pos
    EXPECT_THAT(instructions.at(2),
                AllOf(Field(&Instruction::op_code, OpCode::MUL),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U64),
                                        VariantWith<uint32_t>(0),
                                        VariantWith<uint32_t>(1),
                                        VariantWith<uint32_t>(1)))));

    // MUL last pos
    EXPECT_THAT(instructions.at(13),
                AllOf(Field(&Instruction::op_code, OpCode::MUL),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U64),
                                        VariantWith<uint32_t>(0),
                                        VariantWith<uint32_t>(1),
                                        VariantWith<uint32_t>(1)))));

    // RETURN
    EXPECT_THAT(instructions.at(14),
                AllOf(Field(&Instruction::op_code, OpCode::RETURN),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(0), VariantWith<uint32_t>(0)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the multiplication selector and pc = 13
    auto row = std::ranges::find_if(
        trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == 1 && r.main_pc == 13; });
    EXPECT_EQ(row->main_ic, 244140625); // 5^12 = 244140625

    validate_trace(std::move(trace), public_inputs);
}

// Positive test about a single internal_call and internal_return
// Code of internal routine is SET U32 value 123456789 at memory address 7
// The bytecode execution is:
// SET U32 val. 222111000 at memory address 4
// CALL internal routine
// ADD M[4] with M[7] and output in M[9]
// Internal routine bytecode is at the end.
// Bytecode layout: SET INTERNAL_CALL ADD RETURN SET INTERNAL_RETURN
//                   0        1        2     3    4         5
TEST_F(AvmExecutionTests, simpleInternalCall)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +            // opcode SET
                               "00"                             // Indirect flag
                               "03"                             // U32
                               "0D3D2518"                       // val 222111000 = 0xD3D2518
                               "00000004"                       // dst_offset 4
                               + to_hex(OpCode::INTERNALCALL) + // opcode INTERNALCALL
                               "00000004"                       // jmp_dest
                               + to_hex(OpCode::ADD) +          // opcode ADD
                               "00"                             // Indirect flag
                               "03"                             // U32
                               "00000004"                       // addr a 4
                               "00000007"                       // addr b 7
                               "00000009"                       // addr c9
                               + to_hex(OpCode::RETURN) +       // opcode RETURN
                               "00"                             // Indirect flag
                               "00000000"                       // ret offset 0
                               "00000000"                       // ret size 0
                               + to_hex(OpCode::SET) +          // opcode SET
                               "00"                             // Indirect flag
                               "03"                             // U32
                               "075BCD15"                       // val 123456789 = 0x75BCD15
                               "00000007"                       // dst_offset 7
                               + to_hex(OpCode::INTERNALRETURN) // opcode INTERNALRETURN
        ;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_THAT(instructions, SizeIs(6));

    // We test parsing step for INTERNALCALL and INTERNALRETURN.

    // INTERNALCALL
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::INTERNALCALL),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint32_t>(4)))));

    // INTERNALRETURN
    EXPECT_EQ(instructions.at(5).op_code, OpCode::INTERNALRETURN);

    auto trace = gen_trace_from_instr(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 4, 5, 2, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the addition selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_add == 1; });
    EXPECT_EQ(row->main_ic, 345567789);

    validate_trace(std::move(trace), public_inputs);
}

// Positive test with some nested internall calls
// We use the following functions (internal calls):
// F1: ADD(2,3,2)  M[2] = M[2] + M[3]
// F2: MUL(2,3,2)  M[2] = M[2] * M[3]
// G: F1 SET(17,3) F2  where SET(17,3) means M[3] = 17
// MAIN: SET(4,2) SET(7,3) G
// Whole execution should compute: (4 + 7) * 17 = 187
// Bytecode layout: SET(4,2) SET(7,3) INTERNAL_CALL_G RETURN BYTECODE(F2) BYTECODE(F1) BYTECODE(G)
//                     0         1            2          3         4           6            8
// BYTECODE(F1): ADD(2,3,2) INTERNAL_RETURN
// BYTECODE(F2): MUL(2,3,2) INTERNAL_RETURN
// BYTECODE(G): INTERNAL_CALL(6) SET(17,3) INTERNAL_CALL(4) INTERNAL_RETURN
TEST_F(AvmExecutionTests, nestedInternalCalls)
{
    auto internalCallInstructionHex = [](std::string const& dst_offset) {
        return to_hex(OpCode::INTERNALCALL) // opcode INTERNALCALL
               + "000000" + dst_offset;
    };

    auto setInstructionHex = [](std::string const& val, std::string const& dst_offset) {
        return to_hex(OpCode::SET) // opcode SET
               + "00"              // Indirect flag
               + "01"              // U8
               + val + "000000" + dst_offset;
    };

    const std::string tag_address_arguments = "00"        // Indirect Flag
                                              "01"        // U8
                                              "00000002"  // addr a 2
                                              "00000003"  // addr b 3
                                              "00000002"; // addr c 2

    const std::string return_instruction_hex = to_hex(OpCode::RETURN) // opcode RETURN
                                               + "00"                 // Indirect flag
                                                 "00000000"           // ret offset 0
                                                 "00000000";          // ret size 0

    const std::string bytecode_f1 = to_hex(OpCode::ADD) + tag_address_arguments + to_hex(OpCode::INTERNALRETURN);
    const std::string bytecode_f2 = to_hex(OpCode::MUL) + tag_address_arguments + to_hex(OpCode::INTERNALRETURN);
    const std::string bytecode_g = internalCallInstructionHex("06") + setInstructionHex("11", "03") +
                                   internalCallInstructionHex("04") + to_hex(OpCode::INTERNALRETURN);

    std::string bytecode_hex = setInstructionHex("04", "02") + setInstructionHex("07", "03") +
                               internalCallInstructionHex("08") + return_instruction_hex + bytecode_f2 + bytecode_f1 +
                               bytecode_g;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(12));

    // Expected sequence of opcodes
    std::vector<OpCode> const opcode_sequence{ OpCode::SET,          OpCode::SET,
                                               OpCode::INTERNALCALL, OpCode::RETURN,
                                               OpCode::MUL,          OpCode::INTERNALRETURN,
                                               OpCode::ADD,          OpCode::INTERNALRETURN,
                                               OpCode::INTERNALCALL, OpCode::SET,
                                               OpCode::INTERNALCALL, OpCode::INTERNALRETURN };

    for (size_t i = 0; i < 12; i++) {
        EXPECT_EQ(instructions.at(i).op_code, opcode_sequence.at(i));
    }

    auto trace = gen_trace_from_instr(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 2, 8, 6, 7, 9, 10, 4, 5, 11, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the multiplication selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == 1; });
    EXPECT_EQ(row->main_ic, 187);
    EXPECT_EQ(row->main_pc, 4);

    validate_trace(std::move(trace), public_inputs);
}

// Positive test with JUMP and CALLDATACOPY
// We test bytecode which first invoke CALLDATACOPY on a FF array of two values.
// Then, a JUMP call skips a SUB opcode to land to a FDIV operation and RETURN.
// Calldata: [13, 156]
// Bytecode layout: CALLDATACOPY  JUMP  SUB  FDIV  RETURN
//                        0         1    2    3     4
TEST_F(AvmExecutionTests, jumpAndCalldatacopy)
{
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY (no in tag)
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000002"                     // copy_size
                               "0000000A"                     // dst_offset // M[10] = 13, M[11] = 156
                               + to_hex(OpCode::JUMP) +       // opcode JUMP
                               "00000003"                     // jmp_dest (FDIV located at 3)
                               + to_hex(OpCode::SUB) +        // opcode SUB
                               "00"                           // Indirect flag
                               "06"                           // FF
                               "0000000B"                     // addr 11
                               "0000000A"                     // addr 10
                               "00000001"                     // addr c 1 (If executed would be 156 - 13 = 143)
                               + to_hex(OpCode::FDIV) +       // opcode FDIV
                               "00"                           // Indirect flag
                               "0000000B"                     // addr 11
                               "0000000A"                     // addr 10
                               "00000001"                     // addr c 1 (156 / 13 = 12)
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000000"                     // ret offset 0
                               "00000000"                     // ret size 0
        ;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(5));

    // We test parsing steps for CALLDATACOPY and JUMP.

    // CALLDATACOPY
    EXPECT_THAT(instructions.at(0),
                AllOf(Field(&Instruction::op_code, OpCode::CALLDATACOPY),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<uint32_t>(0),
                                        VariantWith<uint32_t>(2),
                                        VariantWith<uint32_t>(10)))));

    // JUMP
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::JUMP),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint32_t>(3)))));

    std::vector<FF> returndata{};
    auto trace = Execution::gen_trace(instructions, returndata, std::vector<FF>{ 13, 156 }, public_inputs_vec);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 3, 4 };

    for (size_t i = 0; i < 4; i++) {
        EXPECT_EQ(trace.at(i + 1).main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the fdiv selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fdiv == 1; });
    EXPECT_EQ(row->main_ic, 12);

    // Find the first row enabling the subtraction selector.
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == 1; });
    // It must have failed as subtraction was "jumped over".
    EXPECT_EQ(row, trace.end());

    validate_trace(std::move(trace), public_inputs, { 13, 156 });
}

// Positive test for JUMPI.
// We invoke CALLDATACOPY on a FF array of one value which will serve as the conditional value
// for JUMPI ans set this value at memory offset 10.
// Then, we set value 20 (UINT16) at memory offset 101.
// Then, a JUMPI call is performed. Depending of the conditional value, the next opcode (ADD) is
// omitted or not, i.e., we jump to the subsequent opcode MUL.
// Bytecode layout: CALLDATACOPY  SET  JUMPI  ADD   MUL  RETURN
//                        0        1     2     3     4      5
// We test this bytecode with two calldatacopy values: 9873123 and 0.
TEST_F(AvmExecutionTests, jumpiAndCalldatacopy)
{
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY (no in tag)
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000001"                     // copy_size
                               "0000000A"                     // dst_offset 10
                               + to_hex(OpCode::SET) +        // opcode SET
                               "00"                           // Indirect flag
                               "02"                           // U16
                               "0014"                         // val 20
                               "00000065"                     // dst_offset 101
                               + to_hex(OpCode::JUMPI) +      // opcode JUMPI
                               "00"                           // Indirect flag
                               "00000004"                     // jmp_dest (MUL located at 4)
                               "0000000A"                     // cond_offset 10
                               + to_hex(OpCode::ADD) +        // opcode ADD
                               "00"                           // Indirect flag
                               "02"                           // U16
                               "00000065"                     // addr 101
                               "00000065"                     // addr 101
                               "00000065"                     // output addr 101
                               + to_hex(OpCode::MUL) +        // opcode MUL
                               "00"                           // Indirect flag
                               "02"                           // U16
                               "00000065"                     // addr 101
                               "00000065"                     // addr 101
                               "00000066"                     // output of MUL addr 102
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000000"                     // ret offset 0
                               "00000000"                     // ret size 0
        ;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(6));

    // We test parsing of JUMPI.

    // JUMPI
    EXPECT_THAT(
        instructions.at(2),
        AllOf(Field(&Instruction::op_code, OpCode::JUMPI),
              Field(&Instruction::operands,
                    ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(4), VariantWith<uint32_t>(10)))));

    std::vector<FF> returndata{};
    auto trace_jump = Execution::gen_trace(instructions, returndata, std::vector<FF>{ 9873123 }, public_inputs_vec);
    auto trace_no_jump = Execution::gen_trace(instructions, returndata, std::vector<FF>{ 0 }, public_inputs_vec);

    // Expected sequence of PCs during execution with jump
    std::vector<FF> pc_sequence_jump{ 0, 1, 2, 4, 5 };
    // Expected sequence of PCs during execution without jump
    std::vector<FF> pc_sequence_no_jump{ 0, 1, 2, 3, 4, 5 };

    for (size_t i = 0; i < 5; i++) {
        EXPECT_EQ(trace_jump.at(i + 1).main_pc, pc_sequence_jump.at(i));
    }

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace_no_jump.at(i + 1).main_pc, pc_sequence_no_jump.at(i));
    }

    // JUMP CASE
    // Find the first row enabling the MUL opcode
    auto row = std::ranges::find_if(trace_jump.begin(), trace_jump.end(), [](Row r) { return r.main_sel_op_mul == 1; });
    EXPECT_EQ(row->main_ic, 400); // 400 = 20 * 20

    // Find the first row enabling the addition selector.
    row = std::ranges::find_if(trace_jump.begin(), trace_jump.end(), [](Row r) { return r.main_sel_op_add == 1; });
    // It must have failed as addition was "jumped over".
    EXPECT_EQ(row, trace_jump.end());

    // NO JUMP CASE
    // Find the first row enabling the MUL opcode
    row =
        std::ranges::find_if(trace_no_jump.begin(), trace_no_jump.end(), [](Row r) { return r.main_sel_op_mul == 1; });
    EXPECT_EQ(row->main_ic, 1600); // 800 = (20 + 20) * (20 + 20)

    // traces validation
    validate_trace(std::move(trace_jump), public_inputs, { 9873123 });
    validate_trace(std::move(trace_no_jump), public_inputs, { 0 });
}

// Positive test with MOV.
TEST_F(AvmExecutionTests, movOpcode)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "13"                       // val 19
                               "000000AB"                 // dst_offset 171
                               + to_hex(OpCode::MOV) +    // opcode MOV
                               "00"                       // Indirect flag
                               "000000AB"                 // src_offset 171
                               "00000021"                 // dst_offset 33
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(3));

    // SET
    EXPECT_THAT(instructions.at(0),
                AllOf(Field(&Instruction::op_code, OpCode::SET),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U8),
                                        VariantWith<uint8_t>(19),
                                        VariantWith<uint32_t>(171)))));

    // MOV
    EXPECT_THAT(
        instructions.at(1),
        AllOf(Field(&Instruction::op_code, OpCode::MOV),
              Field(&Instruction::operands,
                    ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(171), VariantWith<uint32_t>(33)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the MOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mov == 1; });
    EXPECT_EQ(row->main_ia, 19);
    EXPECT_EQ(row->main_ic, 19);

    validate_trace(std::move(trace), public_inputs);
}

// Positive test with CMOV.
TEST_F(AvmExecutionTests, cmovOpcode)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "03"                       // val 3
                               "00000010"                 // a_offset 16
                               + to_hex(OpCode::SET) +    // opcode SET
                               "00"                       // Indirect flag
                               "02"                       // U16
                               "0004"                     // val 4
                               "00000011"                 // b_offset 17
                               + to_hex(OpCode::SET) +    // opcode SET
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000005"                 // val 5
                               "00000020"                 // cond_offset 32
                               + to_hex(OpCode::CMOV) +   // opcode CMOV
                               "00"                       // Indirect flag
                               "00000010"                 // a_offset 16
                               "00000011"                 // b_offset 17
                               "00000020"                 // cond_offset 32
                               "00000012"                 // dst_offset 18
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(5));

    // CMOV
    EXPECT_THAT(instructions.at(3),
                AllOf(Field(&Instruction::op_code, OpCode::CMOV),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<uint32_t>(16),
                                        VariantWith<uint32_t>(17),
                                        VariantWith<uint32_t>(32),
                                        VariantWith<uint32_t>(18)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the CMOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_cmov == 1; });
    EXPECT_EQ(row->main_ia, 3);
    EXPECT_EQ(row->main_ib, 4);
    EXPECT_EQ(row->main_ic, 3);
    EXPECT_EQ(row->main_id, 5);

    validate_trace(std::move(trace), public_inputs);
}

// Positive test with indirect MOV.
TEST_F(AvmExecutionTests, indMovOpcode)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "0000000A"                 // val 10
                               "00000001"                 // dst_offset 1
                               + to_hex(OpCode::SET) +    // opcode SET
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "0000000B"                 // val 11
                               "00000002"                 // dst_offset 2
                               + to_hex(OpCode::SET) +    // opcode SET
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "FF"                       // val 255
                               "0000000A"                 // dst_offset 10
                               + to_hex(OpCode::MOV) +    // opcode MOV
                               "01"                       // Indirect flag
                               "00000001"                 // src_offset 1 --> direct offset 10
                               "00000002"                 // dst_offset 2 --> direct offset 11
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(5));

    // MOV
    EXPECT_THAT(instructions.at(3),
                AllOf(Field(&Instruction::op_code, OpCode::MOV),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(1), VariantWith<uint32_t>(1), VariantWith<uint32_t>(2)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the MOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mov == 1; });
    EXPECT_EQ(row->main_ia, 255);
    EXPECT_EQ(row->main_ic, 255);

    validate_trace(std::move(trace), public_inputs);
}

// Positive test for SET and CAST opcodes
TEST_F(AvmExecutionTests, setAndCastOpcodes)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "02"                       // U16
                               "B813"                     // val 47123
                               "00000011"                 // dst_offset 17
                               + to_hex(OpCode::CAST) +   // opcode CAST
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "00000011"                 // addr a
                               "00000012"                 // addr casted a
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(3));

    // SUB
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::CAST),
                      Field(&Instruction::operands,
                            ElementsAre(VariantWith<uint8_t>(0),
                                        VariantWith<AvmMemoryTag>(AvmMemoryTag::U8),
                                        VariantWith<uint32_t>(17),
                                        VariantWith<uint32_t>(18)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the cast selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_cast == 1; });
    EXPECT_EQ(row->main_ic, 19); // 0XB813 --> 0X13 = 19

    validate_trace(std::move(trace), public_inputs);
}

// Positive test with TO_RADIX_LE.
TEST_F(AvmExecutionTests, toRadixLeOpcode)
{
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000001"                     // copy_size
                               "00000001"                     // dst_offset
                               + to_hex(OpCode::SET) +        // opcode SET for indirect src
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000001"                     // value 1 (i.e. where the src from calldata is copied)
                               "00000011"                     // dst_offset 17
                               + to_hex(OpCode::SET) +        // opcode SET for indirect dst
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000005"                     // value 5 (i.e. where the dst will be written to)
                               "00000015"                     // dst_offset 21
                               + to_hex(OpCode::TORADIXLE) +  // opcode TO_RADIX_LE
                               "03"                           // Indirect flag
                               "00000011"                     // src_offset 17 (indirect)
                               "00000015"                     // dst_offset 21 (indirect)
                               "00000002"                     // radix: 2 (i.e. perform bitwise decomposition)
                               "00000100"                     // limbs: 256
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000005"                     // ret offset 0
                               "00000100";                    // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata = std::vector<FF>();
    auto trace =
        Execution::gen_trace(instructions, returndata, std::vector<FF>{ FF::modulus - FF(1) }, public_inputs_vec);

    // Find the first row enabling the TORADIXLE selector
    // Expected output is bitwise decomposition of MODULUS - 1..could hardcode the result but it's a bit long
    std::vector<FF> expected_output;
    // Extract each bit.
    for (size_t i = 0; i < 256; i++) {
        FF expected_limb = (FF::modulus - 1) >> i & 1;
        expected_output.emplace_back(expected_limb);
    }
    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, { FF::modulus - FF(1) }, returndata);
}

// // Positive test with SHA256COMPRESSION.
TEST_F(AvmExecutionTests, sha256CompressionOpcode)
{
    std::string bytecode_preamble;
    // Set operations for sha256 state
    // Test vectors taken from noir black_box_solver
    // State = Uint32Array.from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
    for (uint32_t i = 1; i <= 8; i++) {
        bytecode_preamble += to_hex(OpCode::SET) + // opcode SET
                             "00"                  // Indirect flag
                             "03" +                // U32
                             to_hex<uint32_t>(i) + // val i
                             to_hex<uint32_t>(i);  // val i
    }
    // Set operations for sha256 input
    // Test vectors taken from noir black_box_solver
    // Input = Uint32Array.from([1, 2, 3, 4, 5, 6, 7, 8]),
    for (uint32_t i = 1; i <= 16; i++) {
        bytecode_preamble += to_hex(OpCode::SET) +    // opcode SET
                             "00"                     // Indirect flag
                             "03" +                   // U32
                             to_hex<uint32_t>(i) +    // val i
                             to_hex<uint32_t>(i + 8); // val i
    }
    std::string bytecode_hex = bytecode_preamble       // Initial SET operations to store state and input
                               + to_hex(OpCode::SET) + // opcode SET for indirect dst (output)
                               "00"                    // Indirect flag
                               "03"                    // U32
                               "00000100"              // value 256 (i.e. where the dst will be written to)
                               "00000024"              // dst_offset 36
                               + to_hex(OpCode::SET) + // opcode SET for indirect state
                               "00"                    // Indirect flag
                               "03"                    // U32
                               "00000001"              // value 1 (i.e. where the state will be read from)
                               "00000022"              // dst_offset 34
                               + to_hex(OpCode::SET) + // opcode SET for indirect input
                               "00"                    // Indirect flag
                               "03"                    // U32
                               "00000009"              // value 9 (i.e. where the input will be read from)
                               "00000023"              // dst_offset 35
                               + to_hex(OpCode::SHA256COMPRESSION) + // opcode SHA256COMPRESSION
                               "07"                                  // Indirect flag (first 3 operands indirect)
                               "00000024"                            // output offset (indirect 36)
                               "00000022"                            // state offset (indirect 34)
                               "00000023"                            // input offset (indirect 35)
                               + to_hex(OpCode::RETURN) +            // opcode RETURN
                               "00"                                  // Indirect flag
                               "00000100"                            // ret offset 256
                               "00000008";                           // ret size 8

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> calldata = std::vector<FF>();
    std::vector<FF> returndata = std::vector<FF>();
    // Test vector output taken from noir black_box_solver
    // Uint32Array.from([1862536192, 526086805, 2067405084, 593147560, 726610467, 813867028,
    // 4091010797,3974542186]),
    std::vector<FF> expected_output = { 1862536192, 526086805, 2067405084,    593147560,
                                        726610467,  813867028, 4091010797ULL, 3974542186ULL };
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with SHA256
TEST_F(AvmExecutionTests, sha256Opcode)
{

    // Test vectors taken from noir black_box_solver
    // Uint8Array.from([0x61, 0x62, 0x63]),
    // Uint8Array.from([
    //   0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03,
    //   0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    // ]),
    std::vector<FF> expected_output = {
        FF(0xba), FF(0x78), FF(0x16), FF(0xbf), FF(0x8f), FF(0x01), FF(0xcf), FF(0xea), FF(0x41), FF(0x41), FF(0x40),
        FF(0xde), FF(0x5d), FF(0xae), FF(0x22), FF(0x23), FF(0xb0), FF(0x03), FF(0x61), FF(0xa3), FF(0x96), FF(0x17),
        FF(0x7a), FF(0x9c), FF(0xb4), FF(0x10), FF(0xff), FF(0x61), FF(0xf2), FF(0x00), FF(0x15), FF(0xad),
    };
    std::string bytecode_hex = to_hex(OpCode::SET) +      // Initial SET operations to store state and input
                               "00"                       // Indirect Flag
                               "01"                       // U8
                               "61"                       // val 97
                               "00000001"                 // dst_offset 1
                               + to_hex(OpCode::SET) +    // opcode SET for indirect src (input)
                               "00"                       // Indirect flag
                               "01"                       // U8
                               "62"                       // value 98 (i.e. where the src will be read from)A
                               "00000002"                 // input_offset 2
                               + to_hex(OpCode::SET) +    // opcode SET for indirect src (input)
                               "00"                       // Indirect flag
                               "01"                       // U32
                               "63"                       // value 99 (i.e. where the src will be read from)
                               "00000003"                 // input_offset 36
                               + to_hex(OpCode::SET) +    // opcode SET for indirect src (input)
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000001"                 // value 1 (i.e. where the src will be read from)
                               "00000024"                 // input_offset 36
                               + to_hex(OpCode::SET) +    //
                               "00"                       // Indirect flag
                               "03"                       // U8
                               "00000003"                 // value 3 (i.e. where the length parameter is stored)
                               "00000025"                 // input_offset 37
                               + to_hex(OpCode::SET) +    // opcode SET for indirect dst (output)
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000100"                 // value 256 (i.e. where the ouput will be written to)
                               "00000023"                 // dst_offset 35
                               + to_hex(OpCode::SHA256) + // opcode SHA256
                               "03"                       // Indirect flag (first 2 operands indirect)
                               "00000023"                 // output offset (indirect 35)
                               "00000024"                 // input offset (indirect 36)
                               "00000025"                 // length offset 37
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000100"                 // ret offset 256
                               "00000020";                // ret size 32

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata = std::vector<FF>();
    std::vector<FF> calldata = std::vector<FF>();
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with POSEIDON2_PERM.
TEST_F(AvmExecutionTests, poseidon2PermutationOpCode)
{
    // Test vectors taken from barretenberg/permutation/test
    std::vector<FF> calldata{ FF(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")),
                              FF(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")),
                              FF(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")),
                              FF(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")) };

    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALL DATA COPY
                               "00"                           // Indirect Flag
                               "00000000"                     // cd_offset
                               "00000004"                     // copy_size
                               "00000001"                     // dst_offset 1
                               + to_hex(OpCode::SET) +        // opcode SET for indirect src (input)
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000001"                     // value 1 (i.e. where the src will be read from)
                               "00000024"                     // dst_offset 36
                               + to_hex(OpCode::SET) +        // opcode SET for indirect dst (output)
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000009"                     // value 9 (i.e. where the ouput will be written to)
                               "00000023"                     // dst_offset 35
                               + to_hex(OpCode::POSEIDON2) +  // opcode POSEIDON2
                               "03"                           // Indirect flag (first 2 operands indirect)
                               "00000024"                     // input offset (indirect 36)
                               "00000023"                     // output offset (indirect 35)
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000009"                     // ret offset 256
                               "00000004";                    // ret size 8

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata = std::vector<FF>();
    std::vector<FF> expected_output = {
        FF(std::string("0x2bf1eaf87f7d27e8dc4056e9af975985bccc89077a21891d6c7b6ccce0631f95")),
        FF(std::string("0x0c01fa1b8d0748becafbe452c0cb0231c38224ea824554c9362518eebdd5701f")),
        FF(std::string("0x018555a8eb50cf07f64b019ebaf3af3c925c93e631f3ecd455db07bbb52bbdd3")),
        FF(std::string("0x0cbea457c91c22c6c31fd89afd2541efc2edf31736b9f721e823b2165c90fd41"))
    };
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with Keccakf1600.
TEST_F(AvmExecutionTests, keccakf1600OpCode)
{

    // Test vectors taken noir/noir-repo/acvm-repo/blackbox_solver/src/hash.rs
    std::vector<uint64_t> state = {
        0xF1258F7940E1DDE7LLU, 0x84D5CCF933C0478ALLU, 0xD598261EA65AA9EELLU, 0xBD1547306F80494DLLU,
        0x8B284E056253D057LLU, 0xFF97A42D7F8E6FD4LLU, 0x90FEE5A0A44647C4LLU, 0x8C5BDA0CD6192E76LLU,
        0xAD30A6F71B19059CLLU, 0x30935AB7D08FFC64LLU, 0xEB5AA93F2317D635LLU, 0xA9A6E6260D712103LLU,
        0x81A57C16DBCF555FLLU, 0x43B831CD0347C826LLU, 0x01F22F1A11A5569FLLU, 0x05E5635A21D9AE61LLU,
        0x64BEFEF28CC970F2LLU, 0x613670957BC46611LLU, 0xB87C5A554FD00ECBLLU, 0x8C3EE88A1CCF32C8LLU,
        0x940C7922AE3A2614LLU, 0x1841F924A2C509E4LLU, 0x16F53526E70465C2LLU, 0x75F644E97F30A13BLLU,
        0xEAF1FF7B5CECA249LLU,
    };
    std::vector<FF> expected_output = {
        FF(0x2D5C954DF96ECB3CLLU), FF(0x6A332CD07057B56DLLU), FF(0x093D8D1270D76B6CLLU), FF(0x8A20D9B25569D094LLU),
        FF(0x4F9C4F99E5E7F156LLU), FF(0xF957B9A2DA65FB38LLU), FF(0x85773DAE1275AF0DLLU), FF(0xFAF4F247C3D810F7LLU),
        FF(0x1F1B9EE6F79A8759LLU), FF(0xE4FECC0FEE98B425LLU), FF(0x68CE61B6B9CE68A1LLU), FF(0xDEEA66C4BA8F974FLLU),
        FF(0x33C43D836EAFB1F5LLU), FF(0xE00654042719DBD9LLU), FF(0x7CF8A9F009831265LLU), FF(0xFD5449A6BF174743LLU),
        FF(0x97DDAD33D8994B40LLU), FF(0x48EAD5FC5D0BE774LLU), FF(0xE3B8C8EE55B7B03CLLU), FF(0x91A0226E649E42E9LLU),
        FF(0x900E3129E7BADD7BLLU), FF(0x202A9EC5FAA3CCE8LLU), FF(0x5B3402464E1C3DB6LLU), FF(0x609F4E62A44C1059LLU),
        FF(0x20D06CD26A8FBF5CLLU),
    };

    std::string bytecode_preamble;
    // Set operations for keccak state
    for (uint32_t i = 0; i < 25; i++) {
        bytecode_preamble += to_hex(OpCode::SET) +        // opcode SET
                             "00"                         // Indirect flag
                             "04" +                       // U64
                             to_hex<uint64_t>(state[i]) + // val i
                             to_hex<uint32_t>(i + 1);     // dst offset
    }

    // We use calldatacopy twice because we need to set up 4 inputs
    std::string bytecode_hex = bytecode_preamble +             // Initial SET operations to store state and input
                               to_hex(OpCode::SET) +           // opcode SET for indirect src (input)
                               "00"                            // Indirect flag
                               "03"                            // U32
                               "00000001"                      // value 1 (i.e. where the src will be read from)
                               "00000024"                      // input_offset 36
                               + to_hex(OpCode::SET) +         //
                               "00"                            // Indirect flag
                               "03"                            // U32
                               "00000019"                      // value 25 (i.e. where the length parameter is stored)
                               "00000025"                      // input_offset 37
                               + to_hex(OpCode::SET) +         // opcode SET for indirect dst (output)
                               "00"                            // Indirect flag
                               "03"                            // U32
                               "00000100"                      // value 256 (i.e. where the ouput will be written to)
                               "00000023"                      // dst_offset 35
                               + to_hex(OpCode::KECCAKF1600) + // opcode KECCAKF1600
                               "03"                            // Indirect flag (first 2 operands indirect)
                               "00000023"                      // output offset (indirect 35)
                               "00000024"                      // input offset (indirect 36)
                               "00000025"                      // length offset 37
                               + to_hex(OpCode::RETURN) +      // opcode RETURN
                               "00"                            // Indirect flag
                               "00000100"                      // ret offset 256
                               "00000019";                     // ret size 25

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> calldata = std::vector<FF>();
    std::vector<FF> returndata = std::vector<FF>();
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with Keccak.
TEST_F(AvmExecutionTests, keccakOpCode)
{

    // Test vectors from keccak256_test_cases in noir/noir-repo/acvm-repo/blackbox_solver/
    // Input: Uint8Array.from([0xbd]),
    // Output: Uint8Array.from([
    //   0x5a, 0x50, 0x2f, 0x9f, 0xca, 0x46, 0x7b, 0x26, 0x6d, 0x5b, 0x78, 0x33, 0x65, 0x19, 0x37, 0xe8, 0x05, 0x27,
    //   0x0c, 0xa3, 0xf3, 0xaf, 0x1c, 0x0d, 0xd2, 0x46, 0x2d, 0xca, 0x4b, 0x3b, 0x1a, 0xbf,
    // ]),
    std::vector<FF> expected_output = {
        FF(0x5a), FF(0x50), FF(0x2f), FF(0x9f), FF(0xca), FF(0x46), FF(0x7b), FF(0x26), FF(0x6d), FF(0x5b), FF(0x78),
        FF(0x33), FF(0x65), FF(0x19), FF(0x37), FF(0xe8), FF(0x05), FF(0x27), FF(0x0c), FF(0xa3), FF(0xf3), FF(0xaf),
        FF(0x1c), FF(0x0d), FF(0xd2), FF(0x46), FF(0x2d), FF(0xca), FF(0x4b), FF(0x3b), FF(0x1a), FF(0xbf)
    };
    std::string bytecode_hex = to_hex(OpCode::SET) +      // Initial SET operations to store state and input
                               "00"                       // Indirect Flag
                               "01"                       // U8
                               "BD"                       // val 189
                               "00000001"                 // dst_offset 1
                               + to_hex(OpCode::SET) +    // opcode SET for indirect src (input)
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000001"                 // value 1 (i.e. where the src will be read from)
                               "00000024"                 // input_offset 36
                               + to_hex(OpCode::SET) +    //
                               "00"                       // Indirect flag
                               "03"                       // U8
                               "00000001"                 // value 1 (i.e. where the length parameter is stored)
                               "00000025"                 // input_offset 37
                               + to_hex(OpCode::SET) +    // opcode SET for indirect dst (output)
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000100"                 // value 256 (i.e. where the ouput will be written to)
                               "00000023"                 // dst_offset 35
                               + to_hex(OpCode::KECCAK) + // opcode KECCAK
                               "03"                       // Indirect flag (first 2 operands indirect)
                               "00000023"                 // output offset (indirect 35)
                               "00000024"                 // input offset (indirect 36)
                               "00000025"                 // length offset 37
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000100"                 // ret offset 256
                               "00000020";                // ret size 32

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> calldata = std::vector<FF>();
    std::vector<FF> returndata = std::vector<FF>();
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with Pedersen.
TEST_F(AvmExecutionTests, pedersenHashOpCode)
{

    // Test vectors from pedersen_hash in noir/noir-repo/acvm-repo/blackbox_solver/
    // input = [1,1]
    // output = 0x1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6
    // hash_index = 5;
    FF expected_output = FF("0x1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6");
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // Calldatacopy
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000002"                     // copy_size
                               "00000000"                     // dst_offset
                               + to_hex(OpCode::SET) +        // opcode SET for direct hash index offset
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000005"                     // value 5
                               "00000002"                     // input_offset 2
                               + to_hex(OpCode::SET) +        // opcode SET for indirect src
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000000"                     // value 0 (i.e. where the src will be read from)
                               "00000004"                     // dst_offset 4
                               + to_hex(OpCode::SET) +        // opcode SET for direct src_length
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000002"                     // value 2
                               "00000005"                     // dst_offset
                               + to_hex(OpCode::PEDERSEN) +   // opcode PEDERSEN
                               "04"                           // Indirect flag (3rd operand indirect)
                               "00000002"                     // hash_index offset (direct)
                               "00000003"                     // dest offset (direct)
                               "00000004"                     // input offset (indirect)
                               "00000005"                     // length offset (direct)
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000003"                     // ret offset 3
                               "00000001";                    // ret size 1

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata = std::vector<FF>();
    std::vector<FF> calldata = { FF(1), FF(1) };
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata[0], expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}
//
// Positive test with EmbeddedCurveAdd
TEST_F(AvmExecutionTests, embeddedCurveAddOpCode)
{
    // TODO: Look for hardcoded test vectors since bb is missing them
    grumpkin::g1::affine_element a = grumpkin::g1::affine_element::random_element();
    auto a_is_inf = a.is_point_at_infinity();
    grumpkin::g1::affine_element b = grumpkin::g1::affine_element::random_element();
    auto b_is_inf = b.is_point_at_infinity();
    grumpkin::g1::affine_element res = a + b;
    auto expected_output = std::vector<FF>{ res.x, res.y, res.is_point_at_infinity() };
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) +   // Calldatacopy
                               "00"                             // Indirect flag
                               "00000000"                       // cd_offset
                               "00000002"                       // copy_size
                               "00000000"                       // dst_offset
                               + to_hex(OpCode::SET) +          // opcode SET for direct src_length
                               "00"                             // Indirect flag
                               "01"                             // U8
                               + to_hex<uint8_t>(a_is_inf) +    //
                               "00000002"                       // dst_offset
                               + to_hex(OpCode::CALLDATACOPY) + // calldatacopy
                               "00"                             // Indirect flag
                               "00000002"                       // cd_offset
                               "00000002"                       // copy_size
                               "00000003"                       // dst_offset
                               + to_hex(OpCode::SET) +          // opcode SET for direct src_length
                               "00"                             // Indirect flag
                               "01"                             // U32
                               + to_hex<uint8_t>(b_is_inf) +    // value 2
                               "00000005"                       // dst_offset
                               + to_hex(OpCode::SET) +          // opcode SET for direct src_length
                               "00"                             // Indirect flag
                               "03"                             // U32
                               "00000007"                       // value
                               "00000006"                       // dst_offset
                               + to_hex(OpCode::ECADD) +        // opcode ECADD
                               "40"                             // Indirect flag (sixth operand indirect)
                               "00000000"                       // hash_index offset (direct)
                               "00000001"                       // dest offset (direct)
                               "00000002"                       // input offset (indirect)
                               "00000003"                       // length offset (direct)
                               "00000004"                       // length offset (direct)
                               "00000005"                       // length offset (direct)
                               "00000006"                       // length offset (direct)
                               + to_hex(OpCode::RETURN) +       // opcode RETURN
                               "00"                             // Indirect flag
                               "00000007"                       // ret offset 3
                               "00000003";                      // ret size 1

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata;
    std::vector<FF> calldata = { a.x, a.y, b.x, b.y };
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test with MSM
TEST_F(AvmExecutionTests, msmOpCode)
{
    grumpkin::g1::affine_element a = grumpkin::g1::affine_element::random_element();
    FF a_is_inf = a.is_point_at_infinity();
    grumpkin::g1::affine_element b = grumpkin::g1::affine_element::random_element();
    FF b_is_inf = b.is_point_at_infinity();

    grumpkin::g1::Fr scalar_a = grumpkin::g1::Fr::random_element();
    FF scalar_a_lo = uint256_t::from_uint128(uint128_t(scalar_a));
    FF scalar_a_hi = uint256_t(scalar_a) >> 128;
    grumpkin::g1::Fr scalar_b = grumpkin::g1::Fr::random_element();
    FF scalar_b_lo = uint256_t::from_uint128(uint128_t(scalar_b));
    FF scalar_b_hi = uint256_t(scalar_b) >> 128;
    auto expected_result = a * scalar_a + b * scalar_b;
    std::vector<FF> expected_output = { expected_result.x, expected_result.y, expected_result.is_point_at_infinity() };
    // Send all the input as Fields and cast them to U8 later
    std::vector<FF> calldata = { FF(a.x),  FF(a.y),     a_is_inf,    FF(b.x),     FF(b.y),
                                 b_is_inf, scalar_a_lo, scalar_a_hi, scalar_b_lo, scalar_b_hi };
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // Calldatacopy...should fix the limit on calldatacopy
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset 0
                               "0000000a"                     // copy_size (10 elements)
                               "00000000"                     // dst_offset 0
                               + to_hex(OpCode::CAST) +       // opcode CAST inf to U8
                               "00"                           // Indirect flag
                               "01"                           // U8 tag field
                               "00000002"                     // a_is_inf
                               "00000002"                     //
                               + to_hex(OpCode::CAST) +       // opcode CAST inf to U8
                               "00"                           // Indirect flag
                               "01"                           // U8 tag field
                               "00000005"                     // b_is_inf
                               "00000005"                     //
                               + to_hex(OpCode::SET) +        // opcode SET for length
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000006"                     // Length of point elements (6)
                               "0000000b"                     // dst offset (11)
                               + to_hex(OpCode::SET) +        // SET Indirects
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000000"                     // points offset
                               "0000000d"                     // dst offset +
                               + to_hex(OpCode::SET) +        // SET Indirects
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "00000006"                     // scalars offset
                               "0000000e" +                   // dst offset
                               to_hex(OpCode::SET) +          // SET Indirects
                               "00"                           // Indirect flag
                               "03"                           // U32
                               "0000000c"                     // output offset
                               "0000000f" +                   // dst offset
                               to_hex(OpCode::MSM) +          // opcode MSM
                               "07"                           // Indirect flag (first 3 indirect)
                               "0000000d"                     // points offset
                               "0000000e"                     // scalars offset
                               "0000000f"                     // output offset
                               "0000000b"                     // length offset
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "0000000c"                     // ret offset 12 (this overwrites)
                               "00000003";                    // ret size 3

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // Assign a vector that we will mutate internally in gen_trace to store the return values;
    std::vector<FF> returndata;
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    EXPECT_EQ(returndata, expected_output);

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

// Positive test for Kernel Input opcodes
TEST_F(AvmExecutionTests, kernelInputOpcodes)
{
    std::string bytecode_hex = to_hex(OpCode::ADDRESS) +            // opcode ADDRESS
                               "00"                                 // Indirect flag
                               "00000001"                           // dst_offset
                               + to_hex(OpCode::STORAGEADDRESS) +   // opcode STORAGEADDRESS
                               "00"                                 // Indirect flag
                               "00000002"                           // dst_offset
                               + to_hex(OpCode::SENDER) +           // opcode SENDER
                               "00"                                 // Indirect flag
                               "00000003"                           // dst_offset
                               + to_hex(OpCode::FUNCTIONSELECTOR) + // opcode TRANSACTIONFEE
                               "00"                                 // Indirect flag
                               "00000004"                           // dst_offset
                               + to_hex(OpCode::TRANSACTIONFEE) +   // opcode TRANSACTIONFEE
                               "00"                                 // Indirect flag
                               "00000005"                           // dst_offset
                               + to_hex(OpCode::CHAINID) +          // opcode CHAINID
                               "00"                                 // Indirect flag
                               "00000006"                           // dst_offset
                               + to_hex(OpCode::VERSION) +          // opcode VERSION
                               "00"                                 // Indirect flag
                               "00000007"                           // dst_offset
                               + to_hex(OpCode::BLOCKNUMBER) +      // opcode BLOCKNUMBER
                               "00"                                 // Indirect flag
                               "00000008"                           // dst_offset
                               + to_hex(OpCode::TIMESTAMP) +        // opcode TIMESTAMP
                               "00"                                 // Indirect flag
                               "00000009"                           // dst_offset
                                                                    // Not in simulator
                               //    + to_hex(OpCode::COINBASE) +       // opcode COINBASE
                               //    "00"                               // Indirect flag
                               //    "00000009"                         // dst_offset
                               + to_hex(OpCode::FEEPERL2GAS) + // opcode FEEPERL2GAS
                               "00"                            // Indirect flag
                               "0000000a"                      // dst_offset
                               + to_hex(OpCode::FEEPERDAGAS) + // opcode FEEPERDAGAS
                               "00"                            // Indirect flag
                               "0000000b"                      // dst_offset
                               + to_hex(OpCode::RETURN) +      // opcode RETURN
                               "00"                            // Indirect flag
                               "00000001"                      // ret offset 1
                               "0000000b";                     // ret size 11

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(12));

    // ADDRESS
    EXPECT_THAT(instructions.at(0),
                AllOf(Field(&Instruction::op_code, OpCode::ADDRESS),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(1)))));

    // STORAGEADDRESS
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::STORAGEADDRESS),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(2)))));

    // SENDER
    EXPECT_THAT(instructions.at(2),
                AllOf(Field(&Instruction::op_code, OpCode::SENDER),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(3)))));

    // FUNCTIONSELECTOR
    EXPECT_THAT(instructions.at(3),
                AllOf(Field(&Instruction::op_code, OpCode::FUNCTIONSELECTOR),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(4)))));

    // TRANSACTIONFEE
    EXPECT_THAT(instructions.at(4),
                AllOf(Field(&Instruction::op_code, OpCode::TRANSACTIONFEE),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(5)))));

    // CHAINID
    EXPECT_THAT(instructions.at(5),
                AllOf(Field(&Instruction::op_code, OpCode::CHAINID),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(6)))));

    // VERSION
    EXPECT_THAT(instructions.at(6),
                AllOf(Field(&Instruction::op_code, OpCode::VERSION),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(7)))));

    // BLOCKNUMBER
    EXPECT_THAT(instructions.at(7),
                AllOf(Field(&Instruction::op_code, OpCode::BLOCKNUMBER),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(8)))));

    // TIMESTAMP
    EXPECT_THAT(instructions.at(8),
                AllOf(Field(&Instruction::op_code, OpCode::TIMESTAMP),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(9)))));

    // COINBASE
    // Not in simulator
    // EXPECT_THAT(instructions.at(8),
    //             AllOf(Field(&Instruction::op_code, OpCode::COINBASE),
    //                   Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0),
    //                   VariantWith<uint32_t>(10)))));

    // FEEPERL2GAS
    EXPECT_THAT(instructions.at(9),
                AllOf(Field(&Instruction::op_code, OpCode::FEEPERL2GAS),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(10)))));

    // FEEPERDAGAS
    EXPECT_THAT(instructions.at(10),
                AllOf(Field(&Instruction::op_code, OpCode::FEEPERDAGAS),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(11)))));

    // Public inputs for the circuit
    std::vector<FF> calldata = {};

    FF sender = 1;
    FF address = 2;
    // NOTE: address doesn't actually exist in public circuit public inputs,
    // so storage address is just an alias of address for now
    FF storage_address = address;
    FF function_selector = 4;
    FF transaction_fee = 5;
    FF chainid = 6;
    FF version = 7;
    FF blocknumber = 8;
    FF timestamp = 9;
    // FF coinbase = 10; // Not in simulator
    FF feeperl2gas = 10;
    FF feeperdagas = 11;

    // The return data for this test should be a the opcodes in sequence, as the opcodes dst address lines up with
    // this array The returndata call above will then return this array
    std::vector<FF> const expected_returndata = {
        address,     storage_address,         sender,      function_selector, transaction_fee, chainid, version,
        blocknumber, /*coinbase,*/ timestamp, feeperl2gas, feeperdagas,
    };

    // Set up public inputs to contain the above values
    // TODO: maybe have a javascript like object construction so that this is readable
    // Reduce the amount of times we have similar code to this
    //
    public_inputs_vec[ADDRESS_SELECTOR] = address;
    public_inputs_vec[STORAGE_ADDRESS_SELECTOR] = storage_address;
    public_inputs_vec[SENDER_SELECTOR] = sender;
    public_inputs_vec[FUNCTION_SELECTOR_SELECTOR] = function_selector;
    public_inputs_vec[TRANSACTION_FEE_OFFSET] = transaction_fee;

    // Global variables
    public_inputs_vec[CHAIN_ID_OFFSET] = chainid;
    public_inputs_vec[VERSION_OFFSET] = version;
    public_inputs_vec[BLOCK_NUMBER_OFFSET] = blocknumber;
    public_inputs_vec[TIMESTAMP_OFFSET] = timestamp;
    // Not in the simulator yet
    // public_inputs_vec[COINBASE_OFFSET] = coinbase;
    // Global variables - Gas
    public_inputs_vec[FEE_PER_DA_GAS_OFFSET] = feeperdagas;
    public_inputs_vec[FEE_PER_L2_GAS_OFFSET] = feeperl2gas;

    std::vector<FF> returndata;
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    // Validate returndata
    EXPECT_EQ(returndata, expected_returndata);

    // Validate that the opcode read the correct value into ia
    // Check address
    auto address_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_address == 1; });
    EXPECT_EQ(address_row->main_ia, address);

    // Check storage address
    auto storage_addr_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_storage_address == 1; });
    EXPECT_EQ(storage_addr_row->main_ia, storage_address);

    // Check sender
    auto sender_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sender == 1; });
    EXPECT_EQ(sender_row->main_ia, sender);

    // Check function selector
    auto function_selector_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_function_selector == 1; });
    EXPECT_EQ(function_selector_row->main_ia, function_selector);

    // Check transactionfee
    auto transaction_fee_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_transaction_fee == 1; });
    EXPECT_EQ(transaction_fee_row->main_ia, transaction_fee);

    // Check chain id
    auto chainid_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_chain_id == 1; });
    EXPECT_EQ(chainid_row->main_ia, chainid);

    // Check version
    auto version_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_version == 1; });
    EXPECT_EQ(version_row->main_ia, version);

    // Check blocknumber
    auto blocknumber_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_block_number == 1; });
    EXPECT_EQ(blocknumber_row->main_ia, blocknumber);

    // Check timestamp
    auto timestamp_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_timestamp == 1; });
    EXPECT_EQ(timestamp_row->main_ia, timestamp);

    // // Check coinbase
    // Not in simulator
    // auto coinbase_row =
    //     std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_coinbase == 1; });
    // EXPECT_EQ(coinbase_row->main_ia, coinbase);

    // Check feeperdagas
    auto feeperdagas_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_da_gas == 1; });
    EXPECT_EQ(feeperdagas_row->main_ia, feeperdagas);

    // Check feeperl2gas
    auto feeperl2gas_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_l2_gas == 1; });
    EXPECT_EQ(feeperl2gas_row->main_ia, feeperl2gas);

    validate_trace(std::move(trace), Execution::convert_public_inputs(public_inputs_vec), calldata, returndata);
}

// Positive test for L2GASLEFT opcode
TEST_F(AvmExecutionTests, l2GasLeft)
{
    std::string bytecode_hex = to_hex(OpCode::SET) +         // opcode SET
                               "00"                          // Indirect flag
                               "03"                          // U32
                               "00000101"                    // val 257
                               "00000011"                    // dst_offset 17
                               + to_hex(OpCode::L2GASLEFT) + // opcode L2GASLEFT
                               "01"                          // Indirect flag
                               "00000011"                    // dst_offset (indirect addr: 17)
                               + to_hex(OpCode::RETURN) +    // opcode RETURN
                               "00"                          // Indirect flag
                               "00000000"                    // ret offset 0
                               "00000000";                   // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(3));

    // L2GASLEFT
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::L2GASLEFT),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(1), VariantWith<uint32_t>(17)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the L2GASLEFT selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_l2gasleft == 1; });

    uint32_t expected_rem_gas = DEFAULT_INITIAL_L2_GAS -
                                static_cast<uint32_t>(GAS_COST_TABLE.at(OpCode::SET).gas_l2_gas_fixed_table) -
                                static_cast<uint32_t>(GAS_COST_TABLE.at(OpCode::L2GASLEFT).gas_l2_gas_fixed_table);

    EXPECT_EQ(row->main_ia, expected_rem_gas);
    EXPECT_EQ(row->main_mem_addr_a, 257); // Resolved direct address: 257

    validate_trace(std::move(trace), public_inputs);
}

// Positive test for DAGASLEFT opcode
TEST_F(AvmExecutionTests, daGasLeft)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) +         // opcode ADD
                               "00"                          // Indirect flag
                               "03"                          // U32
                               "00000007"                    // addr a 7
                               "00000009"                    // addr b 9
                               "00000001"                    // addr c 1
                               + to_hex(OpCode::DAGASLEFT) + // opcode DAGASLEFT
                               "00"                          // Indirect flag
                               "00000027"                    // dst_offset 39
                               + to_hex(OpCode::RETURN) +    // opcode RETURN
                               "00"                          // Indirect flag
                               "00000000"                    // ret offset 0
                               "00000000";                   // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(3));

    // DAGASLEFT
    EXPECT_THAT(instructions.at(1),
                AllOf(Field(&Instruction::op_code, OpCode::DAGASLEFT),
                      Field(&Instruction::operands, ElementsAre(VariantWith<uint8_t>(0), VariantWith<uint32_t>(39)))));

    auto trace = gen_trace_from_instr(instructions);

    // Find the first row enabling the DAGASLEFT selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_dagasleft == 1; });

    uint32_t expected_rem_gas = DEFAULT_INITIAL_DA_GAS -
                                static_cast<uint32_t>(GAS_COST_TABLE.at(OpCode::ADD).gas_da_gas_fixed_table) -
                                static_cast<uint32_t>(GAS_COST_TABLE.at(OpCode::DAGASLEFT).gas_da_gas_fixed_table);

    EXPECT_EQ(row->main_ia, expected_rem_gas);
    EXPECT_EQ(row->main_mem_addr_a, 39);

    validate_trace(std::move(trace), public_inputs);
}

// Should throw whenever the wrong number of public inputs are provided
TEST_F(AvmExecutionTests, ExecutorThrowsWithIncorrectNumberOfPublicInputs)
{
    std::string bytecode_hex = to_hex(OpCode::SENDER) + // opcode SENDER
                               "00"                     // Indirect flag
                               "00000007";              // addr 7

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};
    std::vector<FF> public_inputs_vec = { 1 };

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_THROW_WITH_MESSAGE(Execution::gen_trace(instructions, calldata, returndata, public_inputs_vec),
                              "Public inputs vector is not of PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH");
}

TEST_F(AvmExecutionTests, kernelOutputEmitOpcodes)
{
    // Set values into the first register to emit
    std::string bytecode_hex = to_hex(OpCode::SET) + // opcode Set
                               "00"                  // Indirect flag
                               "03"                  // U32
                               "00000001"            // value 1
                               "00000001"            // dst_offset 1
                               // Cast set to field
                               + to_hex(OpCode::CAST) +               // opcode CAST
                               "00"                                   // Indirect flag
                               "06"                                   // tag field
                               "00000001"                             // dst 1
                               "00000001"                             // dst 1
                               + to_hex(OpCode::EMITNOTEHASH) +       // opcode EMITNOTEHASH
                               "00"                                   // Indirect flag
                               "00000001"                             // src offset 1
                               + to_hex(OpCode::EMITNULLIFIER) +      // opcode EMITNULLIFIER
                               "00"                                   // Indirect flag
                               "00000001"                             // src offset 1
                               + to_hex(OpCode::EMITUNENCRYPTEDLOG) + // opcode EMITUNENCRYPTEDLOG
                               "00"                                   // Indirect flag
                               "00000001"                             // src offset 1
                               "00000002"                             // src size offset
                               + to_hex(OpCode::SENDL2TOL1MSG) +      // opcode SENDL2TOL1MSG
                               "00"                                   // Indirect flag
                               "00000001"                             // src offset 1
                               "00000001"                             // src offset 1
                               + to_hex(OpCode::RETURN) +             // opcode RETURN
                               "00"                                   // Indirect flag
                               "00000000"                             // ret offset 0
                               "00000000";                            // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(7));

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};
    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);

    // CHECK EMIT NOTE HASH
    // Check output data + side effect counters have been set correctly
    auto emit_note_hash_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_note_hash == 1; });
    EXPECT_EQ(emit_note_hash_row->main_ia, 1);
    EXPECT_EQ(emit_note_hash_row->kernel_side_effect_counter, 0);

    // Get the row of the first note hash out
    uint32_t emit_note_hash_out_offset = START_EMIT_NOTE_HASH_WRITE_OFFSET;
    auto emit_note_hash_kernel_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == emit_note_hash_out_offset; });
    EXPECT_EQ(emit_note_hash_kernel_out_row->kernel_kernel_value_out, 1);
    EXPECT_EQ(emit_note_hash_kernel_out_row->kernel_kernel_side_effect_out, 0);
    feed_output(emit_note_hash_out_offset, 1, 0, 0);

    // CHECK EMIT NULLIFIER
    auto emit_nullifier_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_nullifier == 1; });
    EXPECT_EQ(emit_nullifier_row->main_ia, 1);
    EXPECT_EQ(emit_nullifier_row->kernel_side_effect_counter, 1);

    uint32_t emit_nullifier_out_offset = START_EMIT_NULLIFIER_WRITE_OFFSET;
    auto emit_nullifier_kernel_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == emit_nullifier_out_offset; });
    EXPECT_EQ(emit_nullifier_kernel_out_row->kernel_kernel_value_out, 1);
    EXPECT_EQ(emit_nullifier_kernel_out_row->kernel_kernel_side_effect_out, 1);
    feed_output(emit_nullifier_out_offset, 1, 1, 0);

    // CHECK EMIT UNENCRYPTED LOG
    auto emit_log_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_unencrypted_log == 1; });
    EXPECT_EQ(emit_log_row->main_ia, 1);
    EXPECT_EQ(emit_log_row->kernel_side_effect_counter, 2);

    uint32_t emit_log_out_offset = START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET;
    auto emit_log_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == emit_log_out_offset; });
    EXPECT_EQ(emit_log_kernel_out_row->kernel_kernel_value_out, 1);
    EXPECT_EQ(emit_log_kernel_out_row->kernel_kernel_side_effect_out, 2);
    feed_output(emit_log_out_offset, 1, 2, 0);

    // CHECK SEND L2 TO L1 MSG
    auto send_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_l2_to_l1_msg == 1; });
    EXPECT_EQ(send_row->main_ia, 1);
    EXPECT_EQ(send_row->main_ib, 1);
    EXPECT_EQ(send_row->kernel_side_effect_counter, 3);

    auto msg_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET; });
    EXPECT_EQ(msg_out_row->kernel_kernel_value_out, 1);
    EXPECT_EQ(msg_out_row->kernel_kernel_side_effect_out, 3);
    EXPECT_EQ(msg_out_row->kernel_kernel_metadata_out, 1);
    feed_output(START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET, 1, 3, 1);

    validate_trace(std::move(trace), public_inputs);
}

// SLOAD
TEST_F(AvmExecutionTests, kernelOutputStorageLoadOpcodeSimple)
{
    // Sload from a value that has not previously been written to will require a hint to process
    std::string bytecode_hex = to_hex(OpCode::SET) +      // opcode SET
                               "00"                       // Indirect flag
                               "03"                       // U32
                               "00000009"                 // value 9
                               "00000001"                 // dst_offset 1
                               + to_hex(OpCode::CAST) +   // opcode CAST (Cast set to field)
                               "00"                       // Indirect flag
                               "06"                       // tag field
                               "00000001"                 // dst 1
                               "00000001"                 // dst 1
                               + to_hex(OpCode::SLOAD) +  // opcode SLOAD
                               "00"                       // Indirect flag
                               "00000001"                 // slot offset 1
                               "00000001"                 // slot size 1
                               "00000002"                 // write storage value to offset 2
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(4));

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};

    // Generate Hint for Sload operation
    // side effect counter 0 = value 42
    auto execution_hints = ExecutionHints().with_storage_value_hints({ { 0, 42 } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);

    // CHECK SLOAD
    // Check output data + side effect counters have been set correctly
    auto sload_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sload == 1; });
    EXPECT_EQ(sload_row->main_ia, 42); // Read value
    EXPECT_EQ(sload_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sload_row->kernel_side_effect_counter, 0);

    // Get the row of the first read storage read out
    uint32_t sload_out_offset = START_SLOAD_WRITE_OFFSET;
    auto sload_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sload_out_offset; });
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_value_out, 42); // value
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_side_effect_out, 0);
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_metadata_out, 9); // slot
    feed_output(sload_out_offset, 42, 0, 9);
    validate_trace(std::move(trace), public_inputs);
}

// SLOAD
TEST_F(AvmExecutionTests, kernelOutputStorageLoadOpcodeComplex)
{
    // Sload from a value that has not previously been written to will require a hint to process
    std::string bytecode_hex = to_hex(OpCode::SET) + // opcode SET
                               "00"                  // Indirect flag
                               "03"                  // U32
                               "00000009"            // value 9
                               "00000001"            // dst_offset 1
                               // Cast set to field
                               + to_hex(OpCode::CAST) +   // opcode CAST
                               "00"                       // Indirect flag
                               "06"                       // tag field
                               "00000001"                 // dst 1
                               "00000001"                 // dst 1
                               + to_hex(OpCode::SLOAD) +  // opcode SLOAD
                               "00"                       // Indirect flag (second operand indirect - dest offset)
                               "00000001"                 // slot offset 1
                               "00000002"                 // slot size 2
                               "00000002"                 // write storage value to offset 2
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(4));

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};

    // Generate Hint for Sload operation
    // side effect counter 0 = value 42
    auto execution_hints = ExecutionHints().with_storage_value_hints({ { 0, 42 }, { 1, 123 } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);

    // CHECK SLOAD
    // Check output data + side effect counters have been set correctly
    auto sload_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sload == 1; });
    EXPECT_EQ(sload_row->main_ia, 42); // Read value
    EXPECT_EQ(sload_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sload_row->kernel_side_effect_counter, 0);
    sload_row++;
    EXPECT_EQ(sload_row->main_ia, 123); // Read value
    EXPECT_EQ(sload_row->main_ib, 10);  // Storage slot
    EXPECT_EQ(sload_row->kernel_side_effect_counter, 1);

    // Get the row of the first read storage read out
    uint32_t sload_out_offset = START_SLOAD_WRITE_OFFSET;
    auto sload_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sload_out_offset; });
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_value_out, 42); // value
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_side_effect_out, 0);
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_metadata_out, 9); // slot
    sload_kernel_out_row++;
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_value_out, 123); // value
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_side_effect_out, 1);
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_metadata_out, 10); // slot

    feed_output(sload_out_offset, 42, 0, 9);
    feed_output(sload_out_offset + 1, 123, 1, 10);

    validate_trace(std::move(trace), public_inputs);
}

// SSTORE
TEST_F(AvmExecutionTests, kernelOutputStorageStoreOpcodeSimple)
{
    // SSTORE, write 2 elements of calldata to dstOffset 1 and 2.
    std::vector<FF> calldata = { 42, 123, 9, 10 };
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000004"                     // copy_size
                               "00000001"                     // dst_offset, (i.e. where we store the addr)
                               + to_hex(OpCode::SSTORE) +     // opcode SSTORE
                               "00"                           // Indirect flag
                               "00000001"                     // src offset
                               "00000001"                     // size offset 1
                               "00000003"                     // slot offset
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000000"                     // ret offset 0
                               "00000000";                    // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(3));

    std::vector<FF> returndata = {};

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);
    // CHECK SSTORE
    auto sstore_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sstore == 1; });
    EXPECT_EQ(sstore_row->main_ia, 42); // Read value
    EXPECT_EQ(sstore_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sstore_row->kernel_side_effect_counter, 0);

    // Get the row of the first storage write out
    uint32_t sstore_out_offset = START_SSTORE_WRITE_OFFSET;
    auto sstore_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sstore_out_offset; });

    auto value_out = sstore_kernel_out_row->kernel_kernel_value_out;
    auto side_effect_out = sstore_kernel_out_row->kernel_kernel_side_effect_out;
    auto metadata_out = sstore_kernel_out_row->kernel_kernel_metadata_out;
    EXPECT_EQ(value_out, 42); // value
    EXPECT_EQ(side_effect_out, 0);
    EXPECT_EQ(metadata_out, 9); // slot

    feed_output(sstore_out_offset, value_out, side_effect_out, metadata_out);
    validate_trace(std::move(trace), public_inputs, calldata);
}

// SSTORE
TEST_F(AvmExecutionTests, kernelOutputStorageStoreOpcodeComplex)
{
    // SSTORE, write 2 elements of calldata to dstOffset 1 and 2.
    std::vector<FF> calldata = { 42, 123, 9, 10 };
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000004"                     // copy_size
                               "00000001"                     // dst_offset, (i.e. where we store the addr)
                               + to_hex(OpCode::SET) +        // opcode SET (inidirect SSTORE)
                               "00"
                               "03"
                               "00000001"                 // Value
                               "00000010" +               // Dest val
                               to_hex(OpCode::SSTORE) +   // opcode SSTORE
                               "01"                       // Indirect flag
                               "00000010"                 // src offset
                               "00000002"                 // size offset 1
                               "00000003"                 // slot offset
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(4));

    std::vector<FF> returndata = {};

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec);
    // CHECK SSTORE
    auto sstore_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sstore == 1; });
    EXPECT_EQ(sstore_row->main_ia, 42); // Read value
    EXPECT_EQ(sstore_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sstore_row->kernel_side_effect_counter, 0);
    sstore_row++;

    EXPECT_EQ(sstore_row->main_ia, 123); // Read value
    EXPECT_EQ(sstore_row->main_ib, 10);  // Storage slot
    EXPECT_EQ(sstore_row->kernel_side_effect_counter, 1);

    // Get the row of the first storage write out
    uint32_t sstore_out_offset = START_SSTORE_WRITE_OFFSET;
    auto sstore_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sstore_out_offset; });
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_value_out, 42); // value
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_side_effect_out, 0);
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_metadata_out, 9); // slot
    sstore_kernel_out_row++;
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_value_out, 123); // value
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_side_effect_out, 1);
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_metadata_out, 10); // slot

    feed_output(sstore_out_offset, 42, 0, 9);
    feed_output(sstore_out_offset + 1, 123, 1, 10);

    validate_trace(std::move(trace), public_inputs, calldata);
}

// SLOAD and SSTORE
TEST_F(AvmExecutionTests, kernelOutputStorageOpcodes)
{
    // Sload from a value that has not previously been written to will require a hint to process
    std::string bytecode_hex = to_hex(OpCode::SET) + // opcode SET
                               "00"                  // Indirect flag
                               "03"                  // U32
                               "00000009"            // value 9
                               "00000001"            // dst_offset 1
                               // Cast set to field
                               + to_hex(OpCode::CAST) +   // opcode CAST
                               "00"                       // Indirect flag
                               "06"                       // tag field
                               "00000001"                 // dst 1
                               "00000001"                 // dst 1
                               + to_hex(OpCode::SLOAD) +  // opcode SLOAD
                               "00"                       // Indirect flag
                               "00000001"                 // slot offset 1
                               "00000001"                 // size is 1
                               "00000002"                 // write storage value to offset 2
                               + to_hex(OpCode::SSTORE) + // opcode SSTORE
                               "00"                       // Indirect flag
                               "00000002"                 // src offset 2 (since the sload writes to 2)
                               "00000001"                 // size is 1
                               "00000001"                 // slot offset is 1
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(5));

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};

    // Generate Hint for Sload operation
    // side effect counter 0 = value 42
    auto execution_hints = ExecutionHints().with_storage_value_hints({ { 0, 42 } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);

    // CHECK SLOAD
    // Check output data + side effect counters have been set correctly
    auto sload_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sload == 1; });
    EXPECT_EQ(sload_row->main_ia, 42); // Read value
    EXPECT_EQ(sload_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sload_row->kernel_side_effect_counter, 0);

    // Get the row of the first storage read out
    uint32_t sload_out_offset = START_SLOAD_WRITE_OFFSET;
    auto sload_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sload_out_offset; });
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_value_out, 42); // value
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_side_effect_out, 0);
    EXPECT_EQ(sload_kernel_out_row->kernel_kernel_metadata_out, 9); // slot
    feed_output(sload_out_offset, 42, 0, 9);

    // CHECK SSTORE
    auto sstore_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sstore == 1; });
    EXPECT_EQ(sstore_row->main_ia, 42); // Read value
    EXPECT_EQ(sstore_row->main_ib, 9);  // Storage slot
    EXPECT_EQ(sstore_row->kernel_side_effect_counter, 1);

    // Get the row of the first storage write out
    uint32_t sstore_out_offset = START_SSTORE_WRITE_OFFSET;
    auto sstore_kernel_out_row =
        std::ranges::find_if(trace.begin(), trace.end(), [&](Row r) { return r.main_clk == sstore_out_offset; });
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_value_out, 42); // value
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_side_effect_out, 1);
    EXPECT_EQ(sstore_kernel_out_row->kernel_kernel_metadata_out, 9); // slot
    feed_output(sstore_out_offset, 42, 1, 9);

    validate_trace(std::move(trace), public_inputs);
}

TEST_F(AvmExecutionTests, kernelOutputHashExistsOpcodes)
{
    // hash exists from a value that has not previously been written to will require a hint to process
    std::string bytecode_hex = to_hex(OpCode::SET) + // opcode SET
                               "00"                  // Indirect flag
                               "03"                  // U32
                               "00000001"            // value 1
                               "00000001"            // dst_offset 1
                               // Cast set to field
                               + to_hex(OpCode::CAST) +            // opcode CAST
                               "00"                                // Indirect flag
                               "06"                                // tag field
                               "00000001"                          // dst 1
                               "00000001"                          // dst 1
                               + to_hex(OpCode::NOTEHASHEXISTS) +  // opcode NOTEHASHEXISTS
                               "00"                                // Indirect flag
                               "00000001"                          // slot offset 1
                               "00000002"                          // Leaf index offset 2
                               "00000003"                          // write storage value to offset 2 (exists value)
                               + to_hex(OpCode::NULLIFIEREXISTS) + // opcode NULLIFIEREXISTS
                               "00"                                // Indirect flag
                               "00000001"                          // slot offset 1
                               "00000002"                          // Contract offset 2
                               "00000003"                          // value write offset 2 (exists value)
                               + to_hex(OpCode::L1TOL2MSGEXISTS) + // opcode L1TOL2MSGEXISTS
                               "00"                                // Indirect flag
                               "00000001"                          // slot offset 1
                               "00000002"                          // Lead offset 2
                               "00000003"                          // value write offset 2 (exists value)
                               + to_hex(OpCode::RETURN) +          // opcode RETURN
                               "00"                                // Indirect flag
                               "00000000"                          // ret offset 0
                               "00000000";                         // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    ASSERT_THAT(instructions, SizeIs(6));

    std::vector<FF> calldata = {};
    std::vector<FF> returndata = {};

    // Generate Hint for hash exists operation
    auto execution_hints = ExecutionHints().with_storage_value_hints({ { 0, 1 }, { 1, 1 }, { 2, 1 } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);

    // CHECK NOTEHASHEXISTS
    auto note_hash_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_note_hash_exists == 1; });
    EXPECT_EQ(note_hash_row->main_ia, 1); // Read value
    EXPECT_EQ(note_hash_row->main_ib, 1); // Storage slot
    EXPECT_EQ(note_hash_row->kernel_side_effect_counter, 0);

    auto note_hash_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == START_NOTE_HASH_EXISTS_WRITE_OFFSET; });
    EXPECT_EQ(note_hash_out_row->kernel_kernel_value_out, 1); // value
    EXPECT_EQ(note_hash_out_row->kernel_kernel_side_effect_out, 0);
    EXPECT_EQ(note_hash_out_row->kernel_kernel_metadata_out, 1); // exists
    feed_output(START_NOTE_HASH_EXISTS_WRITE_OFFSET, 1, 0, 1);

    // CHECK NULLIFIEREXISTS
    auto nullifier_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_nullifier_exists == 1; });
    EXPECT_EQ(nullifier_row->main_ia, 1); // Read value
    EXPECT_EQ(nullifier_row->main_ib, 1); // Storage slot
    EXPECT_EQ(nullifier_row->kernel_side_effect_counter, 1);

    auto nullifier_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == START_NULLIFIER_EXISTS_OFFSET; });
    EXPECT_EQ(nullifier_out_row->kernel_kernel_value_out, 1); // value
    EXPECT_EQ(nullifier_out_row->kernel_kernel_side_effect_out, 1);
    EXPECT_EQ(nullifier_out_row->kernel_kernel_metadata_out, 1); // exists
    feed_output(START_NULLIFIER_EXISTS_OFFSET, 1, 1, 1);

    // CHECK L1TOL2MSGEXISTS
    auto l1_to_l2_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_l1_to_l2_msg_exists == 1; });
    EXPECT_EQ(l1_to_l2_row->main_ia, 1); // Read value
    EXPECT_EQ(l1_to_l2_row->main_ib, 1); // Storage slot
    EXPECT_EQ(l1_to_l2_row->kernel_side_effect_counter, 2);

    auto msg_out_row = std::ranges::find_if(
        trace.begin(), trace.end(), [&](Row r) { return r.main_clk == START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET; });
    EXPECT_EQ(msg_out_row->kernel_kernel_value_out, 1); // value
    EXPECT_EQ(msg_out_row->kernel_kernel_side_effect_out, 2);
    EXPECT_EQ(msg_out_row->kernel_kernel_metadata_out, 1); // exists
    feed_output(START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET, 1, 2, 1);

    validate_trace(std::move(trace), public_inputs);
}

TEST_F(AvmExecutionTests, opCallOpcodes)
{
    // Calldata for l2_gas, da_gas, contract_address, nested_call_args (4 elements),
    std::vector<FF> calldata = { 17, 10, 34802342, 1, 2, 3, 4 };
    std::string bytecode_preamble;
    // Set up Gas offsets
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for gas offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000000"            // val 0 (address where gas tuple is located)
                         "00000011";           // dst_offset 17
    // Set up contract address offset
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for args offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000002"            // val 2 (where contract address is located)
                         "00000012";           // dst_offset 18
    // Set up args offset
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for ret offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000003"            // val 3 (the start of the args array)
                         "00000013";           // dst_offset 19
    // Set up args size offset
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for ret offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000004"            // val 4 (the length of the args array)
                         "00000014";           // dst_offset 20
    // Set up the ret offset
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for ret offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000100"            // val 256 (the start of where to write the return data)
                         "00000015";           // dst_offset 21
    // Set up the success offset
    bytecode_preamble += to_hex(OpCode::SET) + // opcode SET for ret offset indirect
                         "00"                  // Indirect flag
                         "03"                  // U32
                         "00000102"            // val 258 (write the success flag at ret_offset + ret_size)
                         "00000016";           // dst_offset 22

    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) + // opcode CALLDATACOPY
                               "00"                           // Indirect flag
                               "00000000"                     // cd_offset
                               "00000007"                     // copy_size
                               "00000000"                     // dst_offset
                               + bytecode_preamble            // Load up memory offsets
                               + to_hex(OpCode::CALL) +       // opcode CALL
                               "3f"                           // Indirect flag
                               "00000011"                     // gas offset
                               "00000012"                     // addr offset
                               "00000013"                     // args offset
                               "00000014"                     // args size offset
                               "00000015"                     // ret offset
                               "00000002"                     // ret size
                               "00000016"                     // success offset
                               "00000017"                     // function_selector_offset
                               + to_hex(OpCode::RETURN) +     // opcode RETURN
                               "00"                           // Indirect flag
                               "00000100"                     // ret offset 8
                               "00000003";                    // ret size 3 (extra read is for the success flag)

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    std::vector<FF> returndata = {};

    // Generate Hint for call operation
    auto execution_hints = ExecutionHints().with_externalcall_hints({ {
        .success = 1,
        .return_data = { 9, 8 },
        .l2_gas_used = 0,
        .da_gas_used = 0,
        .end_side_effect_counter = 0,
    } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);
    EXPECT_EQ(returndata, std::vector<FF>({ 9, 8, 1 })); // The 1 represents the success

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}

TEST_F(AvmExecutionTests, opGetContractInstanceOpcodes)
{
    std::string bytecode_hex = to_hex(OpCode::CALLDATACOPY) +        // opcode CALLDATACOPY for addr
                               "00"                                  // Indirect flag
                               "00000000"                            // cd_offset
                               "00000001"                            // copy_size
                               "00000001"                            // dst_offset, (i.e. where we store the addr)
                               + to_hex(OpCode::SET) +               // opcode SET for the indirect dst offset
                               "00"                                  // Indirect flag
                               "03"                                  // U32
                               "00000003"                            // val i
                               "00000002" +                          // dst_offset 2
                               to_hex(OpCode::GETCONTRACTINSTANCE) + // opcode CALL
                               "02"                                  // Indirect flag
                               "00000001"                            // address offset
                               "00000002"                            // dst offset
                               + to_hex(OpCode::RETURN) +            // opcode RETURN
                               "00"                                  // Indirect flag
                               "00000003"                            // ret offset 3
                               "00000006";                           // ret size 6

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    FF address = 10;
    std::vector<FF> calldata = { address };
    std::vector<FF> returndata = {};

    // Generate Hint for call operation
    // Note: opcode does not write 'address' into memory
    auto execution_hints =
        ExecutionHints().with_contract_instance_hints({ { address, { address, 1, 2, 3, 4, 5, 6 } } });

    auto trace = Execution::gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints);
    EXPECT_EQ(returndata, std::vector<FF>({ 1, 2, 3, 4, 5, 6 })); // The first one represents true

    validate_trace(std::move(trace), public_inputs, calldata, returndata);
}
// Negative test detecting an invalid opcode byte.
TEST_F(AvmExecutionTests, invalidOpcode)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) + // opcode ADD
                               "00"                  // Indirect flag
                               "02"                  // U16
                               "00000007"            // addr a 7
                               "00000009"            // addr b 9
                               "00000001"            // addr c 1
                               "AB"                  // Invalid opcode byte
                               "00000000"            // ret offset 0
                               "00000000";           // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Invalid opcode");
}

// Negative test detecting an invalid memmory instruction tag.
TEST_F(AvmExecutionTests, invalidInstructionTag)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) +      // opcode ADD
                               "00"                       // Indirect flag
                               "00"                       // Wrong type
                               "00000007"                 // addr a 7
                               "00000009"                 // addr b 9
                               "00000001"                 // addr c 1
                               + to_hex(OpCode::RETURN) + // opcode RETURN
                               "00"                       // Indirect flag
                               "00000000"                 // ret offset 0
                               "00000000";                // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Instruction tag is invalid");
}

// Negative test detecting SET opcode with instruction memory tag set to FF.
TEST_F(AvmExecutionTests, ffInstructionTagSetOpcode)
{
    std::string bytecode_hex = "00"                    // ADD
                               "00"                    // Indirect flag
                               "05"                    // U128
                               "00000007"              // addr a 7
                               "00000009"              // addr b 9
                               "00000001"              // addr c 1
                               + to_hex(OpCode::SET) + // opcode SET
                               "00"                    // Indirect flag
                               "06"                    // tag FF
                               "00002344";             //

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Instruction tag for SET opcode is invalid");
}

// Negative test detecting SET opcode without any operand.
TEST_F(AvmExecutionTests, SetOpcodeNoOperand)
{
    std::string bytecode_hex = "00"                    // ADD
                               "00"                    // Indirect flag
                               "05"                    // U128
                               "00000007"              // addr a 7
                               "00000009"              // addr b 9
                               "00000001"              // addr c 1
                               + to_hex(OpCode::SET) + // opcode SET
                               "00";                   // Indirect flag

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand for SET opcode is missing");
}

// Negative test detecting an incomplete instruction: missing instruction tag
TEST_F(AvmExecutionTests, truncatedInstructionNoTag)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) +  // opcode ADD
                               "00"                   // Indirect flag
                               "02"                   // U16
                               "00000007"             // addr a 7
                               "00000009"             // addr b 9
                               "00000001"             // addr c 1
                               + to_hex(OpCode::SUB); // opcode SUB

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand is missing");
}

// Negative test detecting an incomplete instruction: instruction tag present but an operand is missing
TEST_F(AvmExecutionTests, truncatedInstructionNoOperand)
{
    std::string bytecode_hex = to_hex(OpCode::ADD) +   // opcode ADD
                               "00"                    // Indirect flag
                               "02"                    // U16
                               "00000007"              // addr a 7
                               "00000009"              // addr b 9
                               "00000001"              // addr c 1
                               + to_hex(OpCode::SUB) + // opcode SUB
                               "00"                    // Indirect flag
                               "04"                    // U64
                               "AB2373E7"              // addr a
                               "FFFFFFBB";             // addr b and missing address for c = a-b

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand is missing");
}

} // namespace tests_avm
