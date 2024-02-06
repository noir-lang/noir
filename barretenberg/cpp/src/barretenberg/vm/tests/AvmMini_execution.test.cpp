#include "barretenberg/vm/avm_trace/AvmMini_execution.hpp"
#include "AvmMini_common.test.hpp"
#include "barretenberg/common/utils.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_deserialization.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_helper.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <cstdint>
#include <gtest/gtest.h>
#include <string>
#include <utility>

using namespace bb;
namespace {
void gen_proof_and_validate(std::vector<uint8_t> const& bytecode,
                            std::vector<Row>&& trace,
                            std::vector<FF> const& calldata)
{
    auto circuit_builder = AvmMiniCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));
    EXPECT_TRUE(circuit_builder.check_circuit());

    auto composer = AvmMiniComposer();
    auto verifier = composer.create_verifier(circuit_builder);

    auto proof = avm_trace::Execution::run_and_prove(bytecode, calldata);

    EXPECT_TRUE(verifier.verify_proof(proof));
}
} // namespace

namespace tests_avm {
using namespace avm_trace;
using bb::utils::hex_to_bytes;

class AvmMiniExecutionTests : public ::testing::Test {
  public:
    AvmMiniTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        srs::init_crs_factory("../srs_db/ignition");
        trace_builder = AvmMiniTraceBuilder(); // Clean instance for every run.
    };
};

// Basic positive test with an ADD and RETURN opcode.
// Parsing, trace generation and proving is verified.
TEST_F(AvmMiniExecutionTests, basicAddReturn)
{
    std::string bytecode_hex = "00"        // ADD
                               "01"        // U8
                               "00000007"  // addr a 7
                               "00000009"  // addr b 9
                               "00000001"  // addr c 1
                               "34"        // RETURN
                               "00000000"  // ret offset 0
                               "00000000"; // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    // 2 instructions
    EXPECT_EQ(instructions.size(), 2);

    // ADD
    EXPECT_EQ(instructions.at(0).op_code, OpCode::ADD);

    auto operands = instructions.at(0).operands;
    EXPECT_EQ(operands.size(), 4);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U8);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 7);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 9);
    EXPECT_EQ(std::get<uint32_t>(operands.at(3)), 1);

    // RETURN
    EXPECT_EQ(instructions.at(1).op_code, OpCode::RETURN);

    operands = instructions.at(1).operands;
    EXPECT_EQ(operands.size(), 2);
    EXPECT_EQ(std::get<uint32_t>(operands.at(0)), 0);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 0);

    auto trace = Execution::gen_trace(instructions);

    gen_proof_and_validate(bytecode, std::move(trace), {});
}

// Positive test for SET and SUB opcodes
TEST_F(AvmMiniExecutionTests, setAndSubOpcodes)
{
    std::string bytecode_hex = "27"        // SET 39 = 0x27
                               "02"        // U16
                               "B813"      // val 47123
                               "000000AA"  // dst_offset 170
                               "27"        // SET 39 = 0x27
                               "02"        // U16
                               "9103"      // val 37123
                               "00000033"  // dst_offset 51
                               "01"        // SUB
                               "02"        // U16
                               "000000AA"  // addr a
                               "00000033"  // addr b
                               "00000001"  // addr c 1
                               "34"        // RETURN
                               "00000000"  // ret offset 0
                               "00000000"; // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_EQ(instructions.size(), 4);

    // SET
    EXPECT_EQ(instructions.at(0).op_code, OpCode::SET);

    auto operands = instructions.at(0).operands;
    EXPECT_EQ(operands.size(), 3);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U16);
    EXPECT_EQ(std::get<uint16_t>(operands.at(1)), 47123);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 170);

    // SET
    EXPECT_EQ(instructions.at(1).op_code, OpCode::SET);

    operands = instructions.at(1).operands;
    EXPECT_EQ(operands.size(), 3);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U16);
    EXPECT_EQ(std::get<uint16_t>(operands.at(1)), 37123);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 51);

    // SUB
    EXPECT_EQ(instructions.at(2).op_code, OpCode::SUB);

    operands = instructions.at(2).operands;
    EXPECT_EQ(operands.size(), 4);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U16);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 170);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 51);
    EXPECT_EQ(std::get<uint32_t>(operands.at(3)), 1);

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == 1; });
    EXPECT_EQ(row->avmMini_ic, 10000); // 47123 - 37123 = 10000

    gen_proof_and_validate(bytecode, std::move(trace), {});
}

// Positive test for multiple MUL opcodes
// We compute 5^12 based on U64 multiplications
// 5 is stored at offset 0 and 1 at offset 1
// Repeat 12 times a multiplication of value
// at offset 0 (5) with value at offset 1 and store
// the result at offset 1.
TEST_F(AvmMiniExecutionTests, powerWithMulOpcodes)
{
    std::string bytecode_hex = "27"        // SET 39 = 0x27
                               "04"        // U64
                               "00000000"  // val 5 higher 32 bits
                               "00000005"  // val 5 lower 32 bits
                               "00000000"  // dst_offset 0
                               "27"        // SET 39 = 0x27
                               "04"        // U64
                               "00000000"  // val 1 higher 32 bits
                               "00000001"  // val 1 lower 32 bits
                               "00000001"; // dst_offset 1

    std::string const mul_hex = "02"        // MUL
                                "04"        // U64
                                "00000000"  // addr a
                                "00000001"  // addr b
                                "00000001"; // addr c 1

    std::string const ret_hex = "34"        // RETURN
                                "00000000"  // ret offset 0
                                "00000000"; // ret size 0

    uint8_t num = 12;
    while (num-- > 0) {
        bytecode_hex.append(mul_hex);
    }

    bytecode_hex.append(ret_hex);

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_EQ(instructions.size(), 15);

    // MUL first pos
    EXPECT_EQ(instructions.at(2).op_code, OpCode::MUL);

    auto operands = instructions.at(2).operands;
    EXPECT_EQ(operands.size(), 4);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U64);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 0);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 1);
    EXPECT_EQ(std::get<uint32_t>(operands.at(3)), 1);

    // MUL last pos
    EXPECT_EQ(instructions.at(13).op_code, OpCode::MUL);

    operands = instructions.at(13).operands;
    EXPECT_EQ(operands.size(), 4);
    EXPECT_EQ(std::get<AvmMemoryTag>(operands.at(0)), AvmMemoryTag::U64);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 0);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 1);
    EXPECT_EQ(std::get<uint32_t>(operands.at(3)), 1);

    // RETURN
    EXPECT_EQ(instructions.at(14).op_code, OpCode::RETURN);
    operands = instructions.at(14).operands;

    EXPECT_EQ(operands.size(), 2);
    EXPECT_EQ(std::get<uint32_t>(operands.at(0)), 0);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 0);

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the multiplication selector and pc = 13
    auto row = std::ranges::find_if(
        trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == 1 && r.avmMini_pc == 13; });
    EXPECT_EQ(row->avmMini_ic, 244140625); // 5^12 = 244140625

    gen_proof_and_validate(bytecode, std::move(trace), {});
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
TEST_F(AvmMiniExecutionTests, simpleInternalCall)
{
    std::string bytecode_hex = "27"       // SET 39 = 0x27
                               "03"       // U32
                               "0D3D2518" // val 222111000 = 0xD3D2518
                               "00000004" // dst_offset 4
                               "25"       // INTERNALCALL 37
                               "00000004" // jmp_dest
                               "00"       // ADD
                               "03"       // U32
                               "00000004" // addr a 4
                               "00000007" // addr b 7
                               "00000009" // addr c9
                               "34"       // RETURN
                               "00000000" // ret offset 0
                               "00000000" // ret size 0
                               "27"       // SET 39 = 0x27
                               "03"       // U32
                               "075BCD15" // val 123456789 = 0x75BCD15
                               "00000007" // dst_offset 7
                               "26"       // INTERNALRETURN 38
        ;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_EQ(instructions.size(), 6);

    // We test parsing step for INTERNALCALL and INTERNALRETURN.

    // INTERNALCALL
    EXPECT_EQ(instructions.at(1).op_code, OpCode::INTERNALCALL);
    EXPECT_EQ(instructions.at(1).operands.size(), 1);
    EXPECT_EQ(std::get<uint32_t>(instructions.at(1).operands.at(0)), 4);

    // INTERNALRETURN
    EXPECT_EQ(instructions.at(5).op_code, OpCode::INTERNALRETURN);

    auto trace = Execution::gen_trace(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 4, 5, 2, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).avmMini_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the addition selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == 1; });
    EXPECT_EQ(row->avmMini_ic, 345567789);

    gen_proof_and_validate(bytecode, std::move(trace), {});
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
TEST_F(AvmMiniExecutionTests, nestedInternalCalls)
{
    auto internalCallHex = [](std::string const& dst_offset) {
        return "25"
               "000000" +
               dst_offset;
    };

    auto setHex = [](std::string const& val, std::string const& dst_offset) {
        return "2701" // SET U8
               + val + "000000" + dst_offset;
    };

    const std::string tag_address_arguments = "01"        // U8
                                              "00000002"  // addr a 2
                                              "00000003"  // addr b 3
                                              "00000002"; // addr c 2

    const std::string return_hex = "34"        // RETURN
                                   "00000000"  // ret offset 0
                                   "00000000"; // ret size 0

    const std::string internal_ret_hex = "26";
    const std::string add_hex = "00";
    const std::string mul_hex = "02";

    const std::string bytecode_f1 = add_hex + tag_address_arguments + internal_ret_hex;
    const std::string bytecode_f2 = mul_hex + tag_address_arguments + internal_ret_hex;
    const std::string bytecode_g =
        internalCallHex("06") + setHex("11", "03") + internalCallHex("04") + internal_ret_hex;

    std::string bytecode_hex = setHex("04", "02") + setHex("07", "03") + internalCallHex("08") + return_hex +
                               bytecode_f2 + bytecode_f1 + bytecode_g;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_EQ(instructions.size(), 12);

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

    auto trace = Execution::gen_trace(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 2, 8, 6, 7, 9, 10, 4, 5, 11, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).avmMini_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the multiplication selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == 1; });
    EXPECT_EQ(row->avmMini_ic, 187);
    EXPECT_EQ(row->avmMini_pc, 4);

    gen_proof_and_validate(bytecode, std::move(trace), {});
}

// Positive test with JUMP and CALLDATACOPY
// We test bytecode which first invoke CALLDATACOPY on a FF array of two values.
// Then, a JUMP call skips a SUB opcode to land to a DIV operation and RETURN.
// Calldata: [13, 156]
// Bytecode layout: CALLDATACOPY  JUMP  SUB  DIV  RETURN
//                        0         1    2    3     4
TEST_F(AvmMiniExecutionTests, jumpAndCalldatacopy)
{
    std::string bytecode_hex = "1F"       // CALLDATACOPY 31 (no in_tag)
                               "00000000" // cd_offset
                               "00000002" // copy_size
                               "0000000A" // dst_offset // M[10] = 13, M[11] = 156
                               "23"       // JUMP 35
                               "00000003" // jmp_dest (DIV located at 3)
                               "01"       // SUB
                               "06"       // FF
                               "0000000B" // addr 11
                               "0000000A" // addr 10
                               "00000001" // addr c 1 (If executed would be 156 - 13 = 143)
                               "03"       // DIV
                               "06"       // FF
                               "0000000B" // addr 11
                               "0000000A" // addr 10
                               "00000001" // addr c 1 (156 / 13 = 12)
                               "34"       // RETURN
                               "00000000" // ret offset 0
                               "00000000" // ret size 0
        ;

    auto bytecode = hex_to_bytes(bytecode_hex);
    auto instructions = Deserialization::parse(bytecode);

    EXPECT_EQ(instructions.size(), 5);

    // We test parsing steps for CALLDATACOPY and JUMP.

    // CALLDATACOPY
    EXPECT_EQ(instructions.at(0).op_code, OpCode::CALLDATACOPY);
    EXPECT_EQ(instructions.at(0).operands.size(), 3);

    auto operands = instructions.at(0).operands;
    EXPECT_EQ(std::get<uint32_t>(operands.at(0)), 0);
    EXPECT_EQ(std::get<uint32_t>(operands.at(1)), 2);
    EXPECT_EQ(std::get<uint32_t>(operands.at(2)), 10);

    // JUMP
    EXPECT_EQ(instructions.at(1).op_code, OpCode::JUMP);
    EXPECT_EQ(instructions.at(1).operands.size(), 1);
    EXPECT_EQ(std::get<uint32_t>(instructions.at(1).operands.at(0)), 3);

    auto trace = Execution::gen_trace(instructions, std::vector<FF>{ 13, 156 });

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 3, 4 };

    for (size_t i = 0; i < 4; i++) {
        EXPECT_EQ(trace.at(i + 1).avmMini_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the division selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == 1; });
    EXPECT_EQ(row->avmMini_ic, 12);

    // Find the first row enabling the subtraction selector.
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == 1; });
    // It must have failed as subtraction was "jumped over".
    EXPECT_EQ(row, trace.end());

    gen_proof_and_validate(bytecode, std::move(trace), std::vector<FF>{ 13, 156 });
}

// Negative test detecting an invalid opcode byte.
TEST_F(AvmMiniExecutionTests, invalidOpcode)
{
    std::string bytecode_hex = "00"        // ADD
                               "02"        // U16
                               "00000007"  // addr a 7
                               "00000009"  // addr b 9
                               "00000001"  // addr c 1
                               "AB"        // Invalid opcode byte
                               "00000000"  // ret offset 0
                               "00000000"; // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Invalid opcode");
}

// Negative test detecting an invalid memmory instruction tag.
TEST_F(AvmMiniExecutionTests, invalidInstructionTag)
{
    std::string bytecode_hex = "00"        // ADD
                               "00"        // Wrong type
                               "00000007"  // addr a 7
                               "00000009"  // addr b 9
                               "00000001"  // addr c 1
                               "34"        // RETURN
                               "00000000"  // ret offset 0
                               "00000000"; // ret size 0

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Instruction tag is invalid");
}

// Negative test detecting SET opcode with instruction memory tag set to FF.
TEST_F(AvmMiniExecutionTests, ffInstructionTagSetOpcode)
{
    std::string bytecode_hex = "00"        // ADD
                               "05"        // U128
                               "00000007"  // addr a 7
                               "00000009"  // addr b 9
                               "00000001"  // addr c 1
                               "27"        // SET 39 = 0x27
                               "06"        // tag FF
                               "00002344"; //

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Instruction tag for SET opcode is invalid");
}

// Negative test detecting SET opcode without any operand.
TEST_F(AvmMiniExecutionTests, SetOpcodeNoOperand)
{
    std::string bytecode_hex = "00"       // ADD
                               "05"       // U128
                               "00000007" // addr a 7
                               "00000009" // addr b 9
                               "00000001" // addr c 1
                               "27";      // SET 39 = 0x27

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand for SET opcode is missing");
}

// Negative test detecting an incomplete instruction: missing instruction tag
TEST_F(AvmMiniExecutionTests, truncatedInstructionNoTag)
{
    std::string bytecode_hex = "00"       // ADD
                               "02"       // U16
                               "00000007" // addr a 7
                               "00000009" // addr b 9
                               "00000001" // addr c 1
                               "01";      // SUB

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand is missing");
}

// Negative test detecting an incomplete instruction: instruction tag present but an operand is missing
TEST_F(AvmMiniExecutionTests, truncatedInstructionNoOperand)
{
    std::string bytecode_hex = "00"        // ADD
                               "02"        // U16
                               "00000007"  // addr a 7
                               "00000009"  // addr b 9
                               "00000001"  // addr c 1
                               "01"        // SUB
                               "04"        // U64
                               "AB2373E7"  // addr a
                               "FFFFFFBB"; // addr b and missing address for c = a-b

    auto bytecode = hex_to_bytes(bytecode_hex);
    EXPECT_THROW_WITH_MESSAGE(Deserialization::parse(bytecode), "Operand is missing");
}

} // namespace tests_avm