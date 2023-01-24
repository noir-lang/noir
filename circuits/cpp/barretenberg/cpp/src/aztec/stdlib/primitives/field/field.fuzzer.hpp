#include <numeric/uint256/uint256.hpp>
#include <numeric/random/engine.hpp>
#include <stdlib/primitives/field/field.hpp>
#include "../../../rollup/constants.hpp"
#include <stdlib/primitives/bool/bool.hpp>
#include <ecc/curves/bn254/fr.hpp>

// This is a global variable, so that the execution handling class could alter it and signal to the input tester
// that the input should fail
bool circuit_should_fail = false;

#define HAVOC_TESTING
//#define DISABLE_DIVISION 1
#include <common/fuzzer.hpp>
FastRandom VarianceRNG(0);

//#define DISABLE_DIVISION
// Enable this definition, when you want to find out the instructions that caused a failure
//#define SHOW_INFORMATION 1

#ifdef SHOW_INFORMATION
#define PRINT_SINGLE_ARG_INSTRUCTION(first_index, vector, operation_name, preposition)                                 \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].field.get_value() << ") at " << first_index << " " << preposition             \
                  << std::flush;                                                                                       \
    }

#define PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, vector, operation_name, preposition)                      \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].field.get_value() << ") at " << first_index << " " << preposition << " "      \
                  << (vector[second_index].field.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].field.get_value() << ") at " << second_index << std::flush;                  \
    }

#define PRINT_THREE_ARG_INSTRUCTION(                                                                                   \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)                        \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].field.get_value() << ") at " << first_index << " " << preposition1 << " "     \
                  << (vector[second_index].field.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].field.get_value() << ") at " << second_index << " " << preposition2 << " "   \
                  << (vector[third_index].field.is_constant() ? "constant(" : "witness(")                              \
                  << vector[third_index].field.get_value() << ") at " << third_index << std::flush;                    \
    }
#define PRINT_TWO_ARG_ONE_VALUE_INSTRUCTION(                                                                           \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)                        \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].field.get_value() << ") at " << first_index << " " << preposition1 << " "     \
                  << (vector[second_index].field.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].field.get_value() << ") at " << second_index << " " << preposition2 << " "   \
                  << third_index << std::flush;                                                                        \
    }

#define PRINT_TWO_ARG_TWO_VALUES_INSTRUCTION(                                                                          \
    first_index, second_index, value1, value2, vector, operation_name, preposition1, preposition2, preposition3)       \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].field.get_value() << ") at " << first_index << " " << preposition1 << " "     \
                  << (vector[second_index].field.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].field.get_value() << ") at " << second_index << " " << preposition2 << " "   \
                  << value1 << preposition3 << value2 << std::flush;                                                   \
    }

#define PRINT_SLICE(first_index, lsb, msb, vector)                                                                     \
    {                                                                                                                  \
        std::cout << "Slice:"                                                                                          \
                  << " " << (vector[first_index].field.is_constant() ? "constant(" : "witness(")                       \
                  << vector[first_index].field.get_value() << ") at " << first_index << " "                            \
                  << "(" << (size_t)lsb << ":" << (size_t)msb << ")" << std::flush;                                    \
    }

#define PRINT_RESULT(prefix, action, index, value)                                                                     \
    {                                                                                                                  \
        std::cout << "  result(" << value.field.get_value() << ")" << action << index << std::endl << std::flush;      \
    }

#else

#define PRINT_SINGLE_ARG_INSTRUCTION(first_index, vector, operation_name, preposition)
#define PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, vector, operation_name, preposition)

#define PRINT_TWO_ARG_ONE_VALUE_INSTRUCTION(                                                                           \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)
#define PRINT_TWO_ARG_TWO_VALUES_INSTRUCTION(                                                                          \
    first_index, second_index, value1, value2, vector, operation_name, preposition1, preposition2, preposition3)

#define PRINT_THREE_ARG_INSTRUCTION(                                                                                   \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)
#define PRINT_RESULT(prefix, action, index, value)

#define PRINT_SLICE(first_index, lsb, msb, vector)
#endif

#define OPERATION_TYPE_SIZE 1

#define ELEMENT_SIZE (sizeof(fr) + 1)
#define TWO_IN_ONE_OUT 3
#define THREE_IN_ONE_OUT 4
#define SLICE_ARGS_SIZE 6

#define MSUB_DIV_MINIMUM_MUL_PAIRS 1
#define MSUB_DIV_MAXIMUM_MUL_PAIRS 8
#define MSUB_DIV_MINIMUM_SUBTRACTED_ELEMENTS 0
#define MSUB_DIV_MAXIMUM_SUBTRACTED_ELEMENTS 8
#define MULT_MADD_MINIMUM_MUL_PAIRS 1
#define MULT_MADD_MAXIMUM_MUL_PAIRS 8
#define MULT_MADD_MINIMUM_ADDED_ELEMENTS 0
#define MULT_MADD_MAXIMUM_ADDED_ELEMENTS 8
#define SQR_ADD_MINIMUM_ADDED_ELEMENTS 0
#define SQR_ADD_MAXIMUM_ADDED_ELEMENTS 8
/**
 * @brief The class parametrizing Field fuzzing instructions, execution, etc
 *
 */
template <typename Composer> class FieldBase {
  private:
    typedef plonk::stdlib::bool_t<Composer> bool_t;
    typedef plonk::stdlib::field_t<Composer> field_t;
    typedef plonk::stdlib::witness_t<Composer> witness_t;
    typedef plonk::stdlib::public_witness_t<Composer> public_witness_t;

  public:
    /**
     * @brief A class representing a single fuzzing instruction
     *
     */
    class Instruction {
      public:
        enum OPCODE {
            CONSTANT,
            WITNESS,
            CONSTANT_WITNESS,
            ADD,
            SUBTRACT,
            MULTIPLY,
#ifndef DISABLE_DIVISION
            DIVIDE,
#endif
            ADD_TWO,
            MADD,
            SQR,
            ASSERT_EQUAL,
            ASSERT_NOT_EQUAL,
            ASSERT_ZERO,
            ASSERT_NOT_ZERO,
            SLICE,
            RANDOMSEED,
            COND_NEGATE,
            COND_SELECT,
            SELECT_IF_ZERO,
            SELECT_IF_EQ,
            SET,
            INVERT,
            _LAST
        };
        Instruction& operator=(const Instruction& other) = default;

        struct Element {
            Element() = default;
            Element(const Element& other) = default;
            Element(const Element&& other) { value = std::move(other.value); };
            Element(fr in)
                : value(in){};
            Element& operator=(const Element& other) = default;
            fr value;
        };
        struct SingleArg {
            uint8_t in;
        };
        struct TwoArgs {
            uint8_t in;
            uint8_t out;
        };
        struct ThreeArgs {
            uint8_t in1;
            uint8_t in2;
            uint8_t out;
        };
        struct FourArgs {
            uint8_t in1;
            uint8_t in2;
            uint8_t in3;
            uint8_t out;
        };
        struct FiveArgs {
            uint8_t in1;
            uint8_t in2;
            uint8_t qbs;
            uint8_t rbs;
            uint8_t out;
        };
        struct MultAddArgs {
            uint8_t input_index;
            uint8_t output_index;
        };
        struct MultOpArgs {
            uint8_t divisor_index;
            uint8_t output_index;
        };

        struct SliceArgs {
            uint8_t in1;
            uint8_t lsb;
            uint8_t msb;
            uint8_t out1;
            uint8_t out2;
            uint8_t out3;
        };
        union ArgumentContents {
            ArgumentContents() { element = Element(fr(0)); }
            ArgumentContents& operator=(const ArgumentContents& other) = default;
            uint32_t randomseed;
            Element element;
            SingleArg singleArg;
            TwoArgs twoArgs;
            ThreeArgs threeArgs;
            FourArgs fourArgs;
            FiveArgs fiveArgs;
            SliceArgs sliceArgs;
            MultOpArgs multOpArgs;
            MultAddArgs multAddArgs;
        };
        // The type of instruction
        OPCODE id;
        // Instruction arguments
        ArgumentContents arguments;
        /**
         * @brief Generate a random instruction
         *
         * @tparam T PRNG class type
         * @param rng PRNG used
         * @return A random instruction
         */
        template <typename T> inline static Instruction generateRandom(T& rng) requires SimpleRng<T>
        {
            // Choose which instruction we are going to generate
            OPCODE instruction_opcode = static_cast<OPCODE>(rng.next() % (OPCODE::_LAST));
            uint8_t in1, in2, in3, lsb, msb, out, out1, out2, out3, mask_size;
            uint256_t mask, temp;
            Instruction instr;

            // Depending on instruction
            switch (instruction_opcode) {
            case OPCODE::CONSTANT:
            case OPCODE::WITNESS:
            case OPCODE::CONSTANT_WITNESS:
                // If it's a constant or witness, it just pushes data onto the stack to be acted upon
                // Generate a random field element
                for (size_t i = 0; i < (sizeof(uint256_t) >> 1); i++) {
                    *(((uint16_t*)&temp) + i) = static_cast<uint16_t>(rng.next() & 0xffff);
                }
                // We want small values, too. If we generate randomly, we aren't going to have them, so we also
                // apply a random mask, which randomizes the logarithm of maximum value
                mask_size = static_cast<uint8_t>(rng.next() & 0xff);
                mask = (uint256_t(1) << mask_size) - 1;
                // Choose the bit range
                // Return instruction
                return { .id = instruction_opcode, .arguments.element = Element(temp & mask) };
                break;
            case OPCODE::ASSERT_ZERO:
            case OPCODE::ASSERT_NOT_ZERO:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.singleArg = { .in = in1 } };
                break;
            case OPCODE::SQR:
            case OPCODE::ASSERT_EQUAL:
            case OPCODE::ASSERT_NOT_EQUAL:
            case OPCODE::SET:
            case OPCODE::INVERT:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.twoArgs = { .in = in1, .out = out } };
                break;
            case OPCODE::ADD:
            case OPCODE::SUBTRACT:
            case OPCODE::MULTIPLY:
#ifndef DISABLE_DIVISION
            case OPCODE::DIVIDE:
#endif
            case OPCODE::COND_NEGATE:
                // For two-input-one-output instructions we just randomly pick each argument and generate an instruction
                // accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.threeArgs = { .in1 = in1, .in2 = in2, .out = out } };
                break;
            case OPCODE::ADD_TWO:
            case OPCODE::MADD:
            case OPCODE::COND_SELECT:
            case OPCODE::SELECT_IF_ZERO:
            case OPCODE::SELECT_IF_EQ:
                // For three-input-one-output instructions we just randomly pick each argument and generate an
                // instruction accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                in3 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode,
                         .arguments.fourArgs{ .in1 = in1, .in2 = in2, .in3 = in3, .out = out } };
                break;
            case OPCODE::SLICE:
                // For the slice instruction we just randomly pick each argument and generate an instruction
                // accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                lsb = static_cast<uint8_t>(rng.next() & 0xff);
                msb = static_cast<uint8_t>(rng.next() & 0xff);
                out1 = static_cast<uint8_t>(rng.next() & 0xff);
                out2 = static_cast<uint8_t>(rng.next() & 0xff);
                out3 = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode,
                         .arguments.sliceArgs = {
                             .in1 = in1, .lsb = lsb, .msb = msb, .out1 = out1, .out2 = out2, .out3 = out3 } };
            case OPCODE::RANDOMSEED:
                return { .id = instruction_opcode, .arguments.randomseed = rng.next() };
                break;
            default:
                abort(); // We have missed some instructions, it seems
                break;
            }
        }

        /**
         * @brief Mutate the value of a field element
         *
         * @tparam T PRNG class
         * @param e Initial element value
         * @param rng PRNG
         * @param havoc_config Mutation configuration
         * @return Mutated element
         */
        template <typename T>
        inline static fr mutateFieldElement(fr e, T& rng, HavocSettings& havoc_config) requires SimpleRng<T>
        {
            // With a certain probability, we apply changes to the Montgomery form, rather than the plain form. This
            // has merit, since the computation is performed in montgomery form and comparisons are often performed
            // in it, too. Libfuzzer comparison tracing logic can then be enabled in Montgomery form
            bool convert_to_montgomery = (rng.next() % (havoc_config.VAL_MUT_MONTGOMERY_PROBABILITY +
                                                        havoc_config.VAL_MUT_NON_MONTGOMERY_PROBABILITY)) <
                                         havoc_config.VAL_MUT_MONTGOMERY_PROBABILITY;
            uint256_t value_data;
            // Conversion at the start
#define MONT_CONVERSION                                                                                                \
    if (convert_to_montgomery) {                                                                                       \
        value_data = uint256_t(e.to_montgomery_form());                                                                \
    } else {                                                                                                           \
        value_data = uint256_t(e);                                                                                     \
    }
            // Inverse conversion at the end
#define INV_MONT_CONVERSION                                                                                            \
    if (convert_to_montgomery) {                                                                                       \
        e = fr(value_data).from_montgomery_form();                                                                     \
    } else {                                                                                                           \
        e = fr(value_data);                                                                                            \
    }

            // Pick the last value from the mutation distrivution vector
            const size_t mutation_type_count = havoc_config.value_mutation_distribution.size();
            // Choose mutation
            const size_t choice = rng.next() % havoc_config.value_mutation_distribution[mutation_type_count - 1];
            if (choice < havoc_config.value_mutation_distribution[0]) {
                // Delegate mutation to libfuzzer (bit/byte mutations, autodictionary, etc)
                MONT_CONVERSION
                LLVMFuzzerMutate((uint8_t*)&value_data, sizeof(uint256_t), sizeof(uint256_t));
                INV_MONT_CONVERSION
            } else if (choice < havoc_config.value_mutation_distribution[1]) {
                // Small addition/subtraction
                if (convert_to_montgomery) {
                    e = e.to_montgomery_form();
                }
                if (rng.next() & 1) {
                    value_data = e + fr(rng.next() & 0xff);
                } else {
                    value_data = e - fr(rng.next() & 0xff);
                }
                if (convert_to_montgomery) {
                    e = e.from_montgomery_form();
                }
            } else {
                // Substitute field element with a special value
                MONT_CONVERSION
                switch (rng.next() % 9) {
                case 0:
                    e = fr::zero();
                    break;
                case 1:
                    e = fr::one();
                    break;
                case 2:
                    e = -fr::one();
                    break;
                case 3:
                    e = fr::one().sqrt().second;
                    break;
                case 4:
                    e = fr::one().sqrt().second.invert();
                    break;
                case 5:
                    e = fr::get_root_of_unity(8);
                    break;
                case 6:
                    e = fr(2);
                    break;
                case 7:
                    e = fr((fr::modulus - 1) / 2);
                    break;
                case 8:
                    e = fr((fr::modulus));
                    break;
                default:
                    abort();
                    break;
                }
                INV_MONT_CONVERSION
            }
            // Return instruction
            return e;
        }
        /**
         * @brief Mutate a single instruction
         *
         * @tparam T PRNG class
         * @param instruction The instruction
         * @param rng PRNG
         * @param havoc_config Mutation configuration
         * @return Mutated instruction
         */
        template <typename T>
        inline static Instruction mutateInstruction(Instruction instruction,
                                                    T& rng,
                                                    HavocSettings& havoc_config) requires SimpleRng<T>
        {
#define PUT_RANDOM_BYTE_IF_LUCKY(variable)                                                                             \
    if (rng.next() & 1) {                                                                                              \
        variable = rng.next() & 0xff;                                                                                  \
    }
            // Depending on instruction type...
            switch (instruction.id) {
            case OPCODE::CONSTANT:
            case OPCODE::WITNESS:
            case OPCODE::CONSTANT_WITNESS:
                // If it represents pushing a value on the stack with a 50% probability randomly sample a bit_range
                // Maybe mutate the value
                if (rng.next() & 1) {
                    instruction.arguments.element.value =
                        mutateFieldElement(instruction.arguments.element.value, rng, havoc_config);
                }
                break;
            case OPCODE::ASSERT_ZERO:
            case OPCODE::ASSERT_NOT_ZERO:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.singleArg.in)
                break;
            case OPCODE::SQR:
            case OPCODE::ASSERT_EQUAL:
            case OPCODE::ASSERT_NOT_EQUAL:
            case OPCODE::SET:
            case OPCODE::INVERT:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.out)
                break;
            case OPCODE::ADD:
#ifndef DISABLE_DIVISION
            case OPCODE::DIVIDE:
#endif
            case OPCODE::MULTIPLY:
            case OPCODE::SUBTRACT:
            case OPCODE::COND_NEGATE:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.out)
                break;
            case OPCODE::ADD_TWO:
            case OPCODE::MADD:
            case OPCODE::COND_SELECT:
            case OPCODE::SELECT_IF_ZERO:
            case OPCODE::SELECT_IF_EQ:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in3)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.out)
                break;
            case OPCODE::SLICE:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.lsb)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.msb)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.out1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.out2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.out3)
                break;
            case OPCODE::RANDOMSEED:
                instruction.arguments.randomseed = rng.next();
                break;
            default:
                abort(); // New instruction encountered
                break;
            }
            // Return mutated instruction
            return instruction;
        }
    };
    // We use argsizes to both specify the size of data needed to parse the instruction and to signal that the
    // instruction is enabled (if it is -1,it's disabled )
    class ArgSizes {
      public:
        static constexpr size_t CONSTANT = sizeof(fr);
        static constexpr size_t WITNESS = sizeof(fr);
        static constexpr size_t CONSTANT_WITNESS = sizeof(fr);
        static constexpr size_t SQR = 2;
        static constexpr size_t ASSERT_EQUAL = 2;
        static constexpr size_t ASSERT_NOT_EQUAL = 2;
        static constexpr size_t ASSERT_ZERO = 1;
        static constexpr size_t ASSERT_NOT_ZERO = 1;
        static constexpr size_t ADD = 3;
        static constexpr size_t SUBTRACT = 3;
        static constexpr size_t MULTIPLY = 3;
        static constexpr size_t ADD_TWO = 4;
#ifndef DISABLE_DIVISION
        static constexpr size_t DIVIDE = 3;
#else
        static constexpr size_t DIVIDE = static_cast<size_t>(-1);
#endif
        static constexpr size_t MADD = 4;
        static constexpr size_t SUBTRACT_WITH_CONSTRAINT = static_cast<size_t>(-1);
        static constexpr size_t DIVIDE_WITH_CONSTRAINTS = static_cast<size_t>(-1);
        static constexpr size_t SLICE = 6;
        static constexpr size_t RANDOMSEED = sizeof(uint32_t);
        static constexpr size_t COND_NEGATE = 3;
        static constexpr size_t COND_SELECT = 4;
        static constexpr size_t SELECT_IF_ZERO = 4;
        static constexpr size_t SELECT_IF_EQ = 4;
        static constexpr size_t SET = 2;
        static constexpr size_t INVERT = 2;
    };

    /**
     * @brief Optional subclass that governs limits on the use of certain instructions, since some of them can be too
     * slow
     *
     */
    class InstructionWeights {
      public:
        static constexpr size_t CONSTANT = 1;
        static constexpr size_t WITNESS = 1;
        static constexpr size_t CONSTANT_WITNESS = 1;
        static constexpr size_t ADD = 1;
        static constexpr size_t SUBTRACT = 1;
        static constexpr size_t MULTIPLY = 2;
        static constexpr size_t SQR = 2;
        static constexpr size_t ASSERT_EQUAL = 2;
        static constexpr size_t ASSERT_NOT_EQUAL = 2;
        static constexpr size_t ASSERT_ZERO = 2;
        static constexpr size_t ASSERT_NOT_ZERO = 2;
        static constexpr size_t ADD_TWO = 1;
#ifndef DISABLE_DIVISION
        static constexpr size_t DIVIDE = 16;
#endif
        static constexpr size_t MADD = 2;
        static constexpr size_t SUBTRACT_WITH_CONSTRAINT = 0;
        static constexpr size_t DIVIDE_WITH_CONSTRAINTS = 0;
        static constexpr size_t SLICE = 1;
        static constexpr size_t RANDOMSEED = 0;
        static constexpr size_t COND_NEGATE = 0;
        static constexpr size_t COND_SELECT = 0;
        static constexpr size_t SELECT_IF_ZERO = 0;
        static constexpr size_t SELECT_IF_EQ = 0;
        static constexpr size_t SET = 0;
        static constexpr size_t INVERT = 0;
        static constexpr size_t _LIMIT = 64;
    };
    /**
     * @brief Parser class handles the parsing and writing the instructions back to data buffer
     *
     */
    class Parser {
      public:
        /**
         * @brief Parse a single instruction from data
         *
         * @tparam opcode The opcode we are parsing
         * @param Data Pointer to arguments in buffer
         * @return Parsed instructiong
         */
        template <typename Instruction::OPCODE opcode> inline static Instruction parseInstructionArgs(uint8_t* Data)
        {
            if constexpr (opcode == Instruction::OPCODE::CONSTANT || opcode == Instruction::OPCODE::WITNESS ||
                          opcode == Instruction::OPCODE::CONSTANT_WITNESS) {
                Instruction instr;
                instr.id = static_cast<typename Instruction::OPCODE>(opcode);
                // instr.arguments.element.value = fr::serialize_from_buffer(Data+1);
                instr.arguments.element.value = fr::serialize_from_buffer(Data);
                return instr;
            };
            if constexpr (opcode == Instruction::OPCODE::ASSERT_ZERO ||
                          opcode == Instruction::OPCODE::ASSERT_NOT_ZERO) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.singleArg = { .in = *Data } };
            }
            if constexpr (opcode == Instruction::OPCODE::SQR || opcode == Instruction::OPCODE::ASSERT_EQUAL ||
                          opcode == Instruction::OPCODE::ASSERT_NOT_EQUAL || opcode == Instruction::OPCODE::SET ||
                          opcode == Instruction::OPCODE::INVERT) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.twoArgs = { .in = *Data, .out = *(Data + 1) } };
            }
            if constexpr (opcode == Instruction::OPCODE::ADD || opcode == Instruction::OPCODE::MULTIPLY ||
#ifndef DISABLE_DIVISION
                          opcode == Instruction::OPCODE::DIVIDE ||
#endif
                          opcode == Instruction::OPCODE::SUBTRACT || opcode == Instruction::OPCODE::COND_NEGATE) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.threeArgs = { .in1 = *Data, .in2 = *(Data + 1), .out = *(Data + 2) } };
            }
            if constexpr (opcode == Instruction::OPCODE::MADD || opcode == Instruction::OPCODE::ADD_TWO ||
                          opcode == Instruction::OPCODE::COND_SELECT || opcode == Instruction::OPCODE::SELECT_IF_ZERO ||
                          opcode == Instruction::OPCODE::SELECT_IF_EQ) {

                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.fourArgs = {
                             .in1 = *Data, .in2 = *(Data + 1), .in3 = *(Data + 2), .out = *(Data + 3) } };
            }
            if constexpr (opcode == Instruction::OPCODE::SLICE) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.sliceArgs = { .in1 = *Data,
                                                             .lsb = *(Data + 1),
                                                             .msb = *(Data + 2),
                                                             .out1 = *(Data + 3),
                                                             .out2 = *(Data + 4),
                                                             .out3 = *(Data + 5) } };
            }
            if constexpr (opcode == Instruction::OPCODE::RANDOMSEED) {
                uint32_t randomseed;
                memcpy(&randomseed, Data, sizeof(uint32_t));
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.randomseed = randomseed };
            };
        }
        /**
         * @brief Write a single instruction to buffer
         *
         * @tparam instruction_opcode Instruction type
         * @param instruction instruction
         * @param Data Pointer to the data buffer (needs to have enough space for the instruction)
         */
        template <typename Instruction::OPCODE instruction_opcode>
        inline static void writeInstruction(Instruction& instruction, uint8_t* Data)
        {
            if constexpr (instruction_opcode == Instruction::OPCODE::CONSTANT ||
                          instruction_opcode == Instruction::OPCODE::WITNESS ||
                          instruction_opcode == Instruction::OPCODE::CONSTANT_WITNESS) {
                *Data = instruction.id;
                memcpy(Data + 1,
                       &instruction.arguments.element.value.data[0],
                       sizeof(instruction.arguments.element.value.data));
            }

            if constexpr (instruction_opcode == Instruction::OPCODE::ASSERT_ZERO ||
                          instruction_opcode == Instruction::OPCODE::ASSERT_NOT_ZERO) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.singleArg.in;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SQR ||
                          instruction_opcode == Instruction::OPCODE::ASSERT_EQUAL ||
                          instruction_opcode == Instruction::OPCODE::ASSERT_NOT_EQUAL ||
                          instruction_opcode == Instruction::OPCODE::SET ||
                          instruction_opcode == Instruction::OPCODE::INVERT) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.twoArgs.in;
                *(Data + 2) = instruction.arguments.twoArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::ADD ||
#ifndef DISABLE_DIVISION
                          instruction_opcode == Instruction::OPCODE::DIVIDE ||
#endif
                          instruction_opcode == Instruction::OPCODE::MULTIPLY ||
                          instruction_opcode == Instruction::OPCODE::SUBTRACT ||
                          instruction_opcode == Instruction::OPCODE::COND_NEGATE) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.threeArgs.in1;
                *(Data + 2) = instruction.arguments.threeArgs.in2;
                *(Data + 3) = instruction.arguments.threeArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::ADD_TWO ||
                          instruction_opcode == Instruction::OPCODE::MADD ||
                          instruction_opcode == Instruction::OPCODE::COND_SELECT ||
                          instruction_opcode == Instruction::OPCODE::SELECT_IF_ZERO ||
                          instruction_opcode == Instruction::OPCODE::SELECT_IF_EQ) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.fourArgs.in1;
                *(Data + 2) = instruction.arguments.fourArgs.in2;
                *(Data + 3) = instruction.arguments.fourArgs.in3;
                *(Data + 4) = instruction.arguments.fourArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SLICE) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.sliceArgs.in1;
                *(Data + 2) = instruction.arguments.sliceArgs.lsb;
                *(Data + 3) = instruction.arguments.sliceArgs.msb;
                *(Data + 4) = instruction.arguments.sliceArgs.out1;
                *(Data + 5) = instruction.arguments.sliceArgs.out2;
                *(Data + 6) = instruction.arguments.sliceArgs.out3;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::RANDOMSEED) {

                *Data = instruction.id;
                memcpy(Data + 1, &instruction.arguments.randomseed, sizeof(uint32_t));
            }
        }
    };
    /**
     * @brief This class implements the execution of safeuint with an oracle to detect discrepancies
     *
     */
    class ExecutionHandler {
      private:
        template <class T>
        ExecutionHandler construct_via_cast(const std::optional<uint256_t> max = std::nullopt,
                                            const std::optional<T> value = std::nullopt) const
        {
            const auto base_u256 = static_cast<uint256_t>(this->base);

            if (max != std::nullopt && base_u256 > *max) {
                return ExecutionHandler(this->base, field_t(this->field));
            }

            field_t new_field;

            if (value == std::nullopt) {
                /* Construct via casting to uint256_t, then T */
                new_field = field_t(static_cast<T>(static_cast<uint256_t>(this->base)));
                new_field.context = this->field.context;
            } else {
                new_field = field_t(*value);
            }

            const auto& ref = new_field;
            return ExecutionHandler(this->base, ref);
        }
        static bool_t construct_predicate(Composer* composer, const bool predicate)
        {
            /* The context field of a predicate can be nullptr;
             * in that case, the function that handles the predicate
             * will use the context of another input parameter
             */
            const bool predicate_has_ctx = static_cast<bool>(VarianceRNG.next() % 2);

            return bool_t(predicate_has_ctx ? composer : nullptr, predicate);
        }
        field_t f() const
        {
            const bool reconstruct = static_cast<bool>(VarianceRNG.next() % 2);

            if (!reconstruct) {
                return this->field;
            }

            return field_t(this->field);
        }
        void assert_equal(field_t f) const
        {
            switch (VarianceRNG.next() % 2) {
            case 0:
                this->f().assert_equal(f);
                break;
            case 1:
                this->f().assert_is_in_set({ f });
                break;
            default:
                abort();
            }
        }

      public:
        fr base;
        field_t field;
        ExecutionHandler() = default;
        ExecutionHandler(fr a, field_t b)
            : base(a)
            , field(b)
        {}
        ExecutionHandler(fr a, field_t& b)
            : base(a)
            , field(b)
        {}
        ExecutionHandler(fr& a, field_t& b)
            : base(a)
            , field(b)
        {}
        ExecutionHandler operator+(const ExecutionHandler& other)
        {
            const auto b = this->base + other.base;

            switch (VarianceRNG.next() % 3) {
            case 0:
                /* Invoke the + operator */
                return ExecutionHandler(b, this->f() + other.f());
            case 1:
                /* Invoke the += operator */
                {
                    auto f = this->f();
                    return ExecutionHandler(b, f += other.f());
                }
                break;
            case 2:
                /* Use accumulate() to compute the sum */
                return ExecutionHandler(b, field_t::accumulate({ this->f(), other.f() }));
            default:
                abort();
            }
        }
        ExecutionHandler operator-(const ExecutionHandler& other)
        {
            const auto b = this->base - other.base;

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* Invoke the - operator */
                return ExecutionHandler(b, this->f() - other.f());
            case 1:
                /* Invoke the -= operator */
                {
                    auto f = this->f();
                    return ExecutionHandler(b, f -= other.f());
                }
                break;
            default:
                abort();
            }
        }
        ExecutionHandler operator*(const ExecutionHandler& other)
        {
            const auto b = this->base * other.base;

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* Invoke the * operator */
                return ExecutionHandler(b, this->f() * other.f());
            case 1:
                /* Invoke the *= operator */
                {
                    auto f = this->f();
                    return ExecutionHandler(b, f *= other.f());
                }
                break;
            default:
                abort();
            }
        }
        ExecutionHandler sqr() { return ExecutionHandler(this->base.sqr(), this->f().sqr()); }
        ExecutionHandler operator/(const ExecutionHandler& other)
        {
            if (other.f().get_value() == 0) {
                circuit_should_fail = true;
            }

            const auto b = this->base / other.base;

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* Invoke the / operator */
                return ExecutionHandler(b, this->f() / other.f());
            case 1:
                /* Invoke the /= operator */
                {
                    auto f = this->f();
                    return ExecutionHandler(b, f /= other.f());
                }
                break;
            default:
                abort();
            }
        }
        ExecutionHandler add_two(const ExecutionHandler& other1, const ExecutionHandler& other2)
        {
            switch (VarianceRNG.next() % 2) {
            case 0:
                return ExecutionHandler(this->base + other1.base + other2.base,
                                        this->f().add_two(other1.f(), other2.f()));
            case 1:
                return ExecutionHandler(this->base + other1.base + other2.base,
                                        field_t::accumulate({ this->f(), other1.f(), other2.f() }));
            default:
                abort();
            }
        }

        ExecutionHandler madd(const ExecutionHandler& other1, const ExecutionHandler& other2)
        {

            return ExecutionHandler(this->base * other1.base + other2.base, this->f().madd(other1.f(), { other2.f() }));
        }
        std::array<ExecutionHandler, 3> slice(uint8_t lsb, uint8_t msb)
        {
            const auto msb_plus_one = uint32_t(msb) + 1;
            const auto hi_mask = ((uint256_t(1) << (256 - uint32_t(msb))) - 1);
            const auto hi_base = (uint256_t(this->base) >> msb_plus_one) & hi_mask;

            const auto lo_mask = (uint256_t(1) << lsb) - 1;
            const auto lo_base = (uint256_t)(this->base) & lo_mask;

            const auto slice_mask = ((uint256_t(1) << (uint32_t(msb - lsb) + 1)) - 1);
            const auto slice_base = (uint256_t(this->base) >> lsb) & slice_mask;

            auto lo_slice_hi_suint_array = this->f().slice(msb, lsb);
            return std::array<ExecutionHandler, 3>{ ExecutionHandler(lo_base, std::move(lo_slice_hi_suint_array[0])),
                                                    ExecutionHandler(slice_base, std::move(lo_slice_hi_suint_array[1])),
                                                    ExecutionHandler(hi_base, std::move(lo_slice_hi_suint_array[2])) };
        }
        void assert_equal(ExecutionHandler& other)
        {
            if (other.f().is_constant()) {
                if (this->f().is_constant()) {
                    // Assert equal does nothing in this case
                    return;
                } else {
                    auto to_add = field_t(this->f().context, uint256_t(this->base - other.base));
                    this->assert_equal(other.f() + to_add);
                }
            } else {
                if (this->f().is_constant()) {
                    auto to_add = field_t(this->f().context, uint256_t(this->base - other.base));
                    auto new_el = other.f() + to_add;
                    this->assert_equal(new_el);

                } else {
                    auto to_add = field_t(this->f().context, uint256_t(this->base - other.base));
                    this->assert_equal(other.f() + to_add);
                }
            }
        }

        void assert_not_equal(ExecutionHandler& other)
        {
            if (this->base == other.base) {
                return;
            } else {
                this->f().assert_not_equal(other.f());
            }
        }

        void assert_zero()
        {
            if (!this->base.is_zero()) {
                circuit_should_fail = true;
            }
            this->f().assert_is_zero();
        }
        void assert_not_zero()
        {
            if (this->base.is_zero()) {
                circuit_should_fail = true;
            }
            this->f().assert_is_not_zero();
        }

        ExecutionHandler conditional_negate(Composer* composer, const bool predicate)
        {
            return ExecutionHandler(predicate ? -(this->base) : this->base,
                                    this->f().conditional_negate(construct_predicate(composer, predicate)));
        }

        ExecutionHandler conditional_select(Composer* composer, ExecutionHandler& other, const bool predicate)
        {
            return ExecutionHandler(
                predicate ? other.base : this->base,
                field_t(composer).conditional_assign(construct_predicate(composer, predicate), other.f(), this->f()));
        }

        ExecutionHandler select_if_zero(Composer* composer, ExecutionHandler& other1, ExecutionHandler& other2)
        {
            return ExecutionHandler(other2.base.is_zero() ? other1.base : this->base,
                                    field_t(composer).conditional_assign(other2.f().is_zero(), other1.f(), this->f()));
        }

        ExecutionHandler select_if_eq(Composer* composer, ExecutionHandler& other1, ExecutionHandler& other2)
        {
            return ExecutionHandler(
                other1.base == other2.base ? other1.base : this->base,
                field_t(composer).conditional_assign(other1.f() == other2.f(), other1.f(), this->f()));
        }
        /* Explicit re-instantiation using the various constructors */
        ExecutionHandler set(Composer* composer)
        {
            (void)composer;

            switch (VarianceRNG.next() % 9) {
            case 0:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via bit_array" << std::endl;
#endif
                return ExecutionHandler(this->base, field_t(this->field));
            case 1:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via int" << std::endl;
#endif
                return construct_via_cast<int>(std::numeric_limits<int>::max());
            case 2:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via unsigned int" << std::endl;
#endif
                return construct_via_cast<unsigned int>(std::numeric_limits<unsigned int>::max());
            case 3:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via unsigned long" << std::endl;
#endif
                return construct_via_cast<unsigned long>(std::numeric_limits<unsigned long>::max());
            case 4:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via uint256_t" << std::endl;
#endif
                return construct_via_cast<uint256_t>();
            case 5:
#ifdef SHOW_INFORMATION
                std::cout << "Construct via fr" << std::endl;
#endif
                return construct_via_cast<fr>(fr::modulus - 1);
            case 6:
#if 1
                /* Disabled because casting to bool_t can fail.
                 * Test for this issue:
                 *
                 * TEST(stdlib_field, test_construct_via_bool_t)
                 * {
                 *     waffle::StandardComposer composer = waffle::StandardComposer();
                 *     field_t a(witness_t(&composer, fr(uint256_t{0xf396b678452ebf15, 0x82ae10893982638b,
                 * 0xdf185a29c65fbf80, 0x1d18b2de99e48308}))); field_t b = a - a; field_t c(static_cast<bool_t>(b));
                 *     std::cout << c.get_value() << std::endl;
                 * }
                 *
                 * According to Rumata this is because the input value needs to be normalized
                 * first.
                 *
                 * Enable this again once this is resolved.
                 */
                return ExecutionHandler(this->base, field_t(this->field));
#else
                if (static_cast<uint256_t>(this->base) > 1) {
                    return ExecutionHandler(this->base, field_t(this->field));
                } else {
#ifdef SHOW_INFORMATION
                    std::cout << "Construct via bool_t" << std::endl;
#endif
                    /* Construct via bool_t */
                    return ExecutionHandler(this->base, field_t(static_cast<bool_t>(this->field)));
                }
#endif
            case 7:
#ifdef SHOW_INFORMATION
                std::cout << "Reproduce via accumulate()" << std::endl;
#endif
                return ExecutionHandler(this->base, field_t::accumulate({ this->f() }));
            case 8: {
#ifdef SHOW_INFORMATION
                std::cout << "Reproduce via decompose_into_bits()" << std::endl;
#endif
                const size_t min_num_bits = static_cast<uint256_t>(this->base).get_msb() + 1;
                if (min_num_bits > 256)
                    abort(); /* Should never happen */

                const size_t num_bits = min_num_bits + (VarianceRNG.next() % (256 - min_num_bits + 1));
                if (num_bits > 256)
                    abort(); /* Should never happen */

                /* XXX this gives: Range error at gate 559 */
                // const auto bits = this->f().decompose_into_bits(num_bits);
                const auto bits = this->f().decompose_into_bits();

                std::vector<fr> frs(bits.size());
                for (size_t i = 0; i < bits.size(); i++) {
                    frs[i] = bits[i].get_value() ? fr(uint256_t(1) << i) : 0;
                }

                switch (VarianceRNG.next() % 2) {
                case 0: {
                    const fr field_from_bits = std::accumulate(frs.begin(), frs.end(), fr(0));
                    return ExecutionHandler(this->base, field_t(composer, field_from_bits));
                }
                case 1: {
                    std::vector<field_t> fields;
                    for (const auto& fr : frs) {
                        fields.push_back(field_t(composer, fr));
                    }
                    /* This is a good opportunity to test
                     * field_t::accumulate with many elements
                     */
                    return ExecutionHandler(this->base, field_t::accumulate(fields));
                }
                default:
                    abort();
                }
            }
            default:
                abort();
            }
        }
        ExecutionHandler invert(void) const
        {
            if (this->base == 0) {
                return ExecutionHandler(this->base, this->f());
            } else {
                return ExecutionHandler(fr(1) / this->base, this->f().invert());
            }
        }

        /**
         * @brief Execute the constant instruction (push constant safeuint to the stack)
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return 0 if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_CONSTANT(Composer* composer,
                                              std::vector<ExecutionHandler>& stack,
                                              Instruction& instruction)
        {
            (void)composer;
            stack.push_back(ExecutionHandler(instruction.arguments.element.value,
                                             field_t(composer, instruction.arguments.element.value)));
#ifdef SHOW_INFORMATION
            std::cout << "Pushed constant value " << instruction.arguments.element.value << " to position "
                      << stack.size() - 1 << std::endl;
#endif
            return 0;
        }

        /**
         * @brief Execute the witness instruction (push witness safeuit to the stack)
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_WITNESS(Composer* composer,
                                             std::vector<ExecutionHandler>& stack,
                                             Instruction& instruction)
        {

            // THis is strange
            stack.push_back(ExecutionHandler(instruction.arguments.element.value,
                                             witness_t(composer, instruction.arguments.element.value)));

#ifdef SHOW_INFORMATION
            std::cout << "Pushed witness value " << instruction.arguments.element.value << " to position "
                      << stack.size() - 1 << std::endl;
#endif
            return 0;
        }

        /**
         * @brief Execute the constant_witness instruction (push a safeuint witness equal to the constant to the
         * stack)
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return 0 if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_CONSTANT_WITNESS(Composer* composer,
                                                      std::vector<ExecutionHandler>& stack,
                                                      Instruction& instruction)
        {
            auto v = field_t(witness_t(composer, instruction.arguments.element.value));
            v.convert_constant_to_witness(composer);
            stack.push_back(ExecutionHandler(instruction.arguments.element.value, std::move(v)));
#ifdef SHOW_INFORMATION
            std::cout << "Pushed constant witness value " << instruction.arguments.element.value << " to position "
                      << stack.size() - 1 << std::endl;
#endif
            return 0;
        }
        /**
         * @brief Execute the multiply instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_MULTIPLY(Composer* composer,
                                              std::vector<ExecutionHandler>& stack,
                                              Instruction& instruction)
        {

            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.threeArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.threeArgs.in2 % stack.size();
            size_t output_index = instruction.arguments.threeArgs.out;

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "Multiplying", "*")

            ExecutionHandler result;
            result = stack[first_index] * stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the addition operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ADD(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.threeArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.threeArgs.in2 % stack.size();
            size_t output_index = instruction.arguments.threeArgs.out;

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "Adding", "+")

            ExecutionHandler result;
            result = stack[first_index] + stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };

        /**
         * @brief Execute the SQR  instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SQR(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t output_index = instruction.arguments.twoArgs.out;

            PRINT_SINGLE_ARG_INSTRUCTION(first_index, stack, "Squaring", "squared")

            ExecutionHandler result;
            result = stack[first_index].sqr();
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };

        /**
         * @brief Execute the ASSERT_EQUAL  instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ASSERT_EQUAL(Composer* composer,
                                                  std::vector<ExecutionHandler>& stack,
                                                  Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t second_index = instruction.arguments.twoArgs.out % stack.size();

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "ASSERT_EQUAL", "== something + ")
#ifdef SHOW_INFORMATION
            std::cout << std::endl;
#endif

            stack[first_index].assert_equal(stack[second_index]);
            return 0;
        };

        /**
         * @brief Execute the ASSERT_NOT_EQUAL  instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ASSERT_NOT_EQUAL(Composer* composer,
                                                      std::vector<ExecutionHandler>& stack,
                                                      Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t second_index = instruction.arguments.twoArgs.out % stack.size();

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "ASSERT_NOT_EQUAL", "!=")
#ifdef SHOW_INFORMATION
            std::cout << std::endl;
#endif

            stack[first_index].assert_not_equal(stack[second_index]);
            return 0;
        };

        /**
         * @brief Execute the ASSERT_ZERO  instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ASSERT_ZERO(Composer* composer,
                                                 std::vector<ExecutionHandler>& stack,
                                                 Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t index = instruction.arguments.singleArg.in % stack.size();

            // Handler for the case that should be discovered through an ASSERT
            if (stack[index].f().is_constant() && !stack[index].base.is_zero()) {
                return 0;
            }
            PRINT_SINGLE_ARG_INSTRUCTION(index, stack, "ASSERT_ZERO", "!")
#ifdef SHOW_INFORMATION
            std::cout << std::endl;
#endif

            stack[index].assert_zero();
            return 0;
        };

        /**
         * @brief Execute the ASSERT_NOT_ZERO  instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ASSERT_NOT_ZERO(Composer* composer,
                                                     std::vector<ExecutionHandler>& stack,
                                                     Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t index = instruction.arguments.singleArg.in % stack.size();
            // Handler for the case that should be discovered through an ASSERT
            if (stack[index].f().is_constant() && stack[index].base.is_zero()) {
                return 0;
            }

            PRINT_SINGLE_ARG_INSTRUCTION(index, stack, "ASSERT_NOT_ZERO", "!!")
#ifdef SHOW_INFORMATION
            std::cout << std::endl;
#endif
            stack[index].assert_not_zero();
            return 0;
        };

        /**
         * @brief Execute the subtraction operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SUBTRACT(Composer* composer,
                                              std::vector<ExecutionHandler>& stack,
                                              Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.threeArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.threeArgs.in2 % stack.size();
            size_t output_index = instruction.arguments.threeArgs.out;

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "Subtracting", "-")

            ExecutionHandler result;
            result = stack[first_index] - stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the division operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_DIVIDE(Composer* composer,
                                            std::vector<ExecutionHandler>& stack,
                                            Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.threeArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.threeArgs.in2 % stack.size();
            size_t output_index = instruction.arguments.threeArgs.out;

            PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, stack, "Dividing", "/")

            ExecutionHandler result;
            if (fr((uint256_t(stack[second_index].f().get_value()) % fr::modulus)) == 0) {
                return 0; // This is not handled by field
            }
            // TODO: FIX THIS. I can't think of an elegant fix for this field issue right now
            // if (fr((stack[first_index].field.get_value() % fr::modulus).lo) == 0) {
            //     return 0;
            // }
            result = stack[first_index] / stack[second_index];
            // If the output index is larger than the number of elements .in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the ADD_TWO instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
        size_t
         */
        static inline size_t execute_ADD_TWO(Composer* composer,
                                             std::vector<ExecutionHandler>& stack,
                                             Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t third_index = instruction.arguments.fourArgs.in3 % stack.size();
            size_t output_index = instruction.arguments.fourArgs.out;
            PRINT_THREE_ARG_INSTRUCTION(first_index, second_index, third_index, stack, "ADD_TWO:", "+", "+")

            ExecutionHandler result;
            result = stack[first_index].add_two(stack[second_index], stack[third_index]);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the MADD instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
        size_t
         */
        static inline size_t execute_MADD(Composer* composer,
                                          std::vector<ExecutionHandler>& stack,
                                          Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t third_index = instruction.arguments.fourArgs.in3 % stack.size();
            size_t output_index = instruction.arguments.fourArgs.out;
            PRINT_THREE_ARG_INSTRUCTION(first_index, second_index, third_index, stack, "MADD:", "*", "+")

            ExecutionHandler result;
            result = stack[first_index].madd(stack[second_index], stack[third_index]);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };

        /**
         * @brief Execute the slice instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
        size_t
         */
        static inline size_t execute_SLICE(Composer* composer,
                                           std::vector<ExecutionHandler>& stack,
                                           Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.sliceArgs.in1 % stack.size();
            uint8_t lsb = instruction.arguments.sliceArgs.lsb;
            uint8_t msb = instruction.arguments.sliceArgs.msb;
            size_t second_index = instruction.arguments.sliceArgs.out1;
            size_t third_index = instruction.arguments.sliceArgs.out2;
            size_t output_index = instruction.arguments.sliceArgs.out3;
            PRINT_SLICE(first_index, lsb, msb, stack)
            // Check assert conditions
            if ((lsb > msb) || (msb > 252) ||
                (static_cast<uint256_t>(stack[first_index].f().get_value()) >=
                 (static_cast<uint256_t>(1) << rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH))) {
                return 0;
            }
            PRINT_SLICE(first_index, lsb, msb, stack)
            auto slices = stack[first_index].slice(lsb, msb);
            std::array<std::pair<ExecutionHandler, size_t>, 3> what_where = { std::make_pair(slices[0], second_index),
                                                                              std::make_pair(slices[1], third_index),
                                                                              std::make_pair(slices[2], output_index) };
            for (auto& x : what_where) {
                auto suints_count = stack.size();
                if (x.second >= suints_count) {

                    PRINT_RESULT("\t", "pushed to ", stack.size(), x.first)
                    stack.push_back(x.first);
                } else {

                    PRINT_RESULT("\t", "saved to ", x.second, x.first)
                    stack[x.second] = x.first;
                }
            }

            return 0;
        }

        /**
         * @brief Execute the RANDOMSEED instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_RANDOMSEED(Composer* composer,
                                                std::vector<ExecutionHandler>& stack,
                                                Instruction& instruction)
        {
            (void)composer;
            (void)stack;

            VarianceRNG.reseed(instruction.arguments.randomseed);
            return 0;
        };
        /**
         * @brief Execute the COND_NEGATE instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_COND_NEGATE(Composer* composer,
                                                 std::vector<ExecutionHandler>& stack,
                                                 Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.threeArgs.in1 % stack.size();
            size_t output_index = instruction.arguments.threeArgs.out % stack.size();
            bool predicate = instruction.arguments.threeArgs.in2 % 2;

            ExecutionHandler result;
            result = stack[first_index].conditional_negate(composer, predicate);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };

        /**
         * @brief Execute the COND_SELECT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_COND_SELECT(Composer* composer,
                                                 std::vector<ExecutionHandler>& stack,
                                                 Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t output_index = instruction.arguments.fourArgs.out % stack.size();
            bool predicate = instruction.arguments.fourArgs.in3 % 2;

            ExecutionHandler result;
            result = stack[first_index].conditional_select(composer, stack[second_index], predicate);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the SELECT_IF_ZERO instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SELECT_IF_ZERO(Composer* composer,
                                                    std::vector<ExecutionHandler>& stack,
                                                    Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t third_index = instruction.arguments.fourArgs.in3 % stack.size();
            size_t output_index = instruction.arguments.fourArgs.out % stack.size();

            ExecutionHandler result;
            result = stack[first_index].select_if_zero(composer, stack[second_index], stack[third_index]);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the SELECT_IF_EQ instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SELECT_IF_EQ(Composer* composer,
                                                  std::vector<ExecutionHandler>& stack,
                                                  Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t third_index = instruction.arguments.fourArgs.in3 % stack.size();
            size_t output_index = instruction.arguments.fourArgs.out % stack.size();

            ExecutionHandler result;
            result = stack[first_index].select_if_eq(composer, stack[second_index], stack[third_index]);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {

                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the SET instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SET(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t output_index = instruction.arguments.twoArgs.out;

            PRINT_SINGLE_ARG_INSTRUCTION(first_index, stack, "Instantiating", "")

            ExecutionHandler result;
            result = stack[first_index].set(composer);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {
                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the INVERT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_INVERT(Composer* composer,
                                            std::vector<ExecutionHandler>& stack,
                                            Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t output_index = instruction.arguments.twoArgs.out;

            PRINT_SINGLE_ARG_INSTRUCTION(first_index, stack, "invert", "")

            ExecutionHandler result;
            result = stack[first_index].invert();
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                PRINT_RESULT("", "pushed to ", stack.size(), result)
                stack.push_back(result);
            } else {
                PRINT_RESULT("", "saved to ", output_index, result)
                stack[output_index] = result;
            }
            return 0;
        };
    };

    /** For field execution state is just a vector of ExecutionHandler objects
     *
     * */
    typedef std::vector<ExecutionHandler> ExecutionState;
    /**
     * @brief Check that the resulting values are equal to expected
     *
     * @tparam Composer
     * @param composer
     * @param stack
     * @return true
     * @return false
     */
    inline static bool postProcess(Composer* composer, std::vector<FieldBase::ExecutionHandler>& stack)
    {
        (void)composer;
        for (size_t i = 0; i < stack.size(); i++) {
            auto element = stack[i];
            if (fr((uint256_t(element.field.get_value()) % fr::modulus)) != element.base) {
                std::cerr << "Failed at " << i << " with actual value " << element.base << " and value in field "
                          << element.field.get_value() << std::endl;
                return false;
            }
        }
        return true;
    }
};

#ifdef HAVOC_TESTING

extern "C" int LLVMFuzzerInitialize(int* argc, char*** argv)
{
    (void)argc;
    (void)argv;
    // These are the settings, optimized for the safeuint class (under them, fuzzer reaches maximum expected
    // coverage in 40 seconds)
    fuzzer_havoc_settings = HavocSettings{
        .GEN_LLVM_POST_MUTATION_PROB = 30,          // Out of 200
        .GEN_MUTATION_COUNT_LOG = 5,                // -Fully checked
        .GEN_STRUCTURAL_MUTATION_PROBABILITY = 300, // Fully  checked
        .GEN_VALUE_MUTATION_PROBABILITY = 700,      // Fully checked
        .ST_MUT_DELETION_PROBABILITY = 100,         // Fully checked
        .ST_MUT_DUPLICATION_PROBABILITY = 80,       // Fully checked
        .ST_MUT_INSERTION_PROBABILITY = 120,        // Fully checked
        .ST_MUT_MAXIMUM_DELETION_LOG = 6,           // 2 because of limit
        .ST_MUT_MAXIMUM_DUPLICATION_LOG = 2,        // -Fully checked
        .ST_MUT_SWAP_PROBABILITY = 50,              // Fully checked
        .VAL_MUT_LLVM_MUTATE_PROBABILITY = 250,     // Fully checked
        .VAL_MUT_MONTGOMERY_PROBABILITY = 130,      // Fully checked
        .VAL_MUT_NON_MONTGOMERY_PROBABILITY = 50,   // Fully checked
        .VAL_MUT_SMALL_ADDITION_PROBABILITY = 110,  // Fully checked
        .VAL_MUT_SPECIAL_VALUE_PROBABILITY = 130    // Fully checked

    };
    /**
     * @brief This is used, when we need to determine the probabilities of various mutations. Left here for
     * posterity
     *
     */
    /*
    std::random_device rd;
    std::uniform_int_distribution<uint64_t> dist(0, ~(uint64_t)(0));
    srandom(static_cast<unsigned int>(dist(rd)));

    fuzzer_havoc_settings =
        HavocSettings{ .GEN_MUTATION_COUNT_LOG = static_cast<size_t>((random() % 8) + 1),
                       .GEN_STRUCTURAL_MUTATION_PROBABILITY = static_cast<size_t>(random() % 100),
                       .GEN_VALUE_MUTATION_PROBABILITY = static_cast<size_t>(random() % 100),
                       .ST_MUT_DELETION_PROBABILITY = static_cast<size_t>(random() % 100),
                       .ST_MUT_DUPLICATION_PROBABILITY = static_cast<size_t>(random() % 100),
                       .ST_MUT_INSERTION_PROBABILITY = static_cast<size_t>((random() % 99) + 1),
                       .ST_MUT_MAXIMUM_DELETION_LOG = static_cast<size_t>((random() % 8) + 1),
                       .ST_MUT_MAXIMUM_DUPLICATION_LOG = static_cast<size_t>((random() % 8) + 1),
                       .ST_MUT_SWAP_PROBABILITY = static_cast<size_t>(random() % 100),
                       .VAL_MUT_LLVM_MUTATE_PROBABILITY = static_cast<size_t>(random() % 100),
                       .VAL_MUT_MONTGOMERY_PROBABILITY = static_cast<size_t>(random() % 100),
                       .VAL_MUT_NON_MONTGOMERY_PROBABILITY = static_cast<size_t>(random() % 100),
                       .VAL_MUT_SMALL_ADDITION_PROBABILITY = static_cast<size_t>(random() % 100),
                       .VAL_MUT_SPECIAL_VALUE_PROBABILITY = static_cast<size_t>(random() % 100)

        };
    while (fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY == 0 &&
           fuzzer_havoc_settings.GEN_VALUE_MUTATION_PROBABILITY == 0) {
        fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY = static_cast<size_t>(random() % 8);
        fuzzer_havoc_settings.GEN_VALUE_MUTATION_PROBABILITY = static_cast<size_t>(random() % 8);
    }
    */

    // fuzzer_havoc_settings.GEN_LLVM_POST_MUTATION_PROB = static_cast<size_t>(((random() % (20 - 1)) + 1) * 10);
    /**
     * @brief Write mutation settings to log
     *
     */
    /*
    std::cerr << "CUSTOM MUTATOR SETTINGS:" << std::endl
              << "################################################################" << std::endl
              << "GEN_LLVM_POST_MUTATION_PROB: " << fuzzer_havoc_settings.GEN_LLVM_POST_MUTATION_PROB << std::endl
              << "GEN_MUTATION_COUNT_LOG: " << fuzzer_havoc_settings.GEN_MUTATION_COUNT_LOG << std::endl
              << "GEN_STRUCTURAL_MUTATION_PROBABILITY: " <<
    fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY
              << std::endl
              << "GEN_VALUE_MUTATION_PROBABILITY: " << fuzzer_havoc_settings.GEN_VALUE_MUTATION_PROBABILITY <<
    std::endl
              << "ST_MUT_DELETION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_DELETION_PROBABILITY << std::endl
              << "ST_MUT_DUPLICATION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_DUPLICATION_PROBABILITY <<
    std::endl
              << "ST_MUT_INSERTION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_INSERTION_PROBABILITY << std::endl
              << "ST_MUT_MAXIMUM_DELETION_LOG: " << fuzzer_havoc_settings.ST_MUT_MAXIMUM_DELETION_LOG << std::endl
              << "ST_MUT_MAXIMUM_DUPLICATION_LOG: " << fuzzer_havoc_settings.ST_MUT_MAXIMUM_DUPLICATION_LOG <<
    std::endl
              << "ST_MUT_SWAP_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_SWAP_PROBABILITY << std::endl
              << "VAL_MUT_LLVM_MUTATE_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_LLVM_MUTATE_PROBABILITY
              << std::endl
              << "VAL_MUT_MONTGOMERY_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_MONTGOMERY_PROBABILITY <<
    std::endl
              << "VAL_MUT_NON_MONTGOMERY_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_NON_MONTGOMERY_PROBABILITY
              << std::endl
              << "VAL_MUT_SMALL_ADDITION_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_SMALL_ADDITION_PROBABILITY
              << std::endl
              << "VAL_MUT_SMALL_MULTIPLICATION_PROBABILITY: "
              << fuzzer_havoc_settings.VAL_MUT_SMALL_MULTIPLICATION_PROBABILITY << std::endl
              << "VAL_MUT_SPECIAL_VALUE_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_SPECIAL_VALUE_PROBABILITY
              << std::endl;
    */
    std::vector<size_t> structural_mutation_distribution;
    std::vector<size_t> value_mutation_distribution;
    size_t temp = 0;
    temp += fuzzer_havoc_settings.ST_MUT_DELETION_PROBABILITY;
    structural_mutation_distribution.push_back(temp);
    temp += fuzzer_havoc_settings.ST_MUT_DUPLICATION_PROBABILITY;
    structural_mutation_distribution.push_back(temp);
    temp += fuzzer_havoc_settings.ST_MUT_INSERTION_PROBABILITY;
    structural_mutation_distribution.push_back(temp);
    temp += fuzzer_havoc_settings.ST_MUT_SWAP_PROBABILITY;
    structural_mutation_distribution.push_back(temp);
    fuzzer_havoc_settings.structural_mutation_distribution = structural_mutation_distribution;

    temp = 0;
    temp += fuzzer_havoc_settings.VAL_MUT_LLVM_MUTATE_PROBABILITY;
    value_mutation_distribution.push_back(temp);
    temp += fuzzer_havoc_settings.VAL_MUT_SMALL_ADDITION_PROBABILITY;
    value_mutation_distribution.push_back(temp);

    temp += fuzzer_havoc_settings.VAL_MUT_SPECIAL_VALUE_PROBABILITY;
    value_mutation_distribution.push_back(temp);
    fuzzer_havoc_settings.value_mutation_distribution = value_mutation_distribution;
    return 0;
}
#endif
#ifndef DISABLE_CUSTOM_MUTATORS
/**
 * @brief Custom mutator. Since we know the structure, this is more efficient than basic
 *
 */
extern "C" size_t LLVMFuzzerCustomMutator(uint8_t* Data, size_t Size, size_t MaxSize, unsigned int Seed)
{
    using FuzzerClass = FieldBase<waffle::StandardComposer>;
    auto fast_random = FastRandom(Seed);
    auto size_occupied = ArithmeticFuzzHelper<FuzzerClass>::MutateInstructionBuffer(Data, Size, MaxSize, fast_random);
    if ((fast_random.next() % 200) < fuzzer_havoc_settings.GEN_LLVM_POST_MUTATION_PROB) {
        size_occupied = LLVMFuzzerMutate(Data, size_occupied, MaxSize);
    }
    return size_occupied;
}

/**
 * @brief Custom crossover that parses the buffers as instructions and then splices them
 *
 */
extern "C" size_t LLVMFuzzerCustomCrossOver(const uint8_t* Data1,
                                            size_t Size1,
                                            const uint8_t* Data2,
                                            size_t Size2,
                                            uint8_t* Out,
                                            size_t MaxOutSize,
                                            unsigned int Seed)
{
    using FuzzerClass = FieldBase<waffle::StandardComposer>;
    auto fast_random = FastRandom(Seed);
    auto vecA = ArithmeticFuzzHelper<FuzzerClass>::parseDataIntoInstructions(Data1, Size1);
    auto vecB = ArithmeticFuzzHelper<FuzzerClass>::parseDataIntoInstructions(Data2, Size2);
    auto vecC = ArithmeticFuzzHelper<FuzzerClass>::crossoverInstructionVector(vecA, vecB, fast_random);
    return ArithmeticFuzzHelper<FuzzerClass>::writeInstructionsToBuffer(vecC, Out, MaxOutSize);
}

#endif

/**
 * @brief Fuzzer entry function
 *
 */
extern "C" size_t LLVMFuzzerTestOneInput(const uint8_t* Data, size_t Size)
{
    RunWithComposers<FieldBase, FuzzerComposerTypes>(Data, Size, VarianceRNG);
    return 0;
}
