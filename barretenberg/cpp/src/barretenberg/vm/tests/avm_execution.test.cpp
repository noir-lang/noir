#include "barretenberg/vm/avm_trace/avm_execution.hpp"
#include "avm_common.test.hpp"
#include "barretenberg/common/utils.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_deserialization.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"

namespace tests_avm {

using namespace bb::avm_trace;
using namespace testing;

using bb::utils::hex_to_bytes;

class AvmExecutionTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
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

    auto trace = Execution::gen_trace(instructions);
    validate_trace(std::move(trace), true);
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_sub == 1; });
    EXPECT_EQ(row->avm_main_ic, 10000); // 47123 - 37123 = 10000
    validate_trace(std::move(trace), true);
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the multiplication selector and pc = 13
    auto row = std::ranges::find_if(
        trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_mul == 1 && r.avm_main_pc == 13; });
    EXPECT_EQ(row->avm_main_ic, 244140625); // 5^12 = 244140625

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 4, 5, 2, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).avm_main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the addition selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_add == 1; });
    EXPECT_EQ(row->avm_main_ic, 345567789);

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 2, 8, 6, 7, 9, 10, 4, 5, 11, 3 };

    for (size_t i = 0; i < 6; i++) {
        EXPECT_EQ(trace.at(i + 1).avm_main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the multiplication selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_mul == 1; });
    EXPECT_EQ(row->avm_main_ic, 187);
    EXPECT_EQ(row->avm_main_pc, 4);

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions, std::vector<FF>{ 13, 156 });

    // Expected sequence of PCs during execution
    std::vector<FF> pc_sequence{ 0, 1, 3, 4 };

    for (size_t i = 0; i < 4; i++) {
        EXPECT_EQ(trace.at(i + 1).avm_main_pc, pc_sequence.at(i));
    }

    // Find the first row enabling the fdiv selector.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_fdiv == 1; });
    EXPECT_EQ(row->avm_main_ic, 12);

    // Find the first row enabling the subtraction selector.
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_sub == 1; });
    // It must have failed as subtraction was "jumped over".
    EXPECT_EQ(row, trace.end());

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the MOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_mov == 1; });
    EXPECT_EQ(row->avm_main_ia, 19);
    EXPECT_EQ(row->avm_main_ic, 19);

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the CMOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_cmov == 1; });
    EXPECT_EQ(row->avm_main_ia, 3);
    EXPECT_EQ(row->avm_main_ib, 4);
    EXPECT_EQ(row->avm_main_ic, 3);
    EXPECT_EQ(row->avm_main_id, 5);

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the MOV selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_mov == 1; });
    EXPECT_EQ(row->avm_main_ia, 255);
    EXPECT_EQ(row->avm_main_ic, 255);

    validate_trace(std::move(trace));
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

    auto trace = Execution::gen_trace(instructions);

    // Find the first row enabling the cast selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_cast == 1; });
    EXPECT_EQ(row->avm_main_ic, 19); // 0XB813 --> 0X13 = 19

    validate_trace(std::move(trace));
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
