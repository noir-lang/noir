#include <numeric/uint256/uint256.hpp>
#include <numeric/random/engine.hpp>
#include <stdlib/primitives/safe_uint/safe_uint.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

// This is a global variable, so that the execution handling class could alter it and signal to the input tester that
// the input should fail
bool circuit_should_fail = false;

#define HAVOC_TESTING

#include <common/fuzzer.hpp>
FastRandom VarianceRNG(0);

// Enable this definition, when you want to find out the instructions that caused a failure
//#define SHOW_INFORMATION 1

#ifdef SHOW_INFORMATION
#define PRINT_TWO_ARG_INSTRUCTION(first_index, second_index, vector, operation_name, preposition)                      \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].suint.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].suint.get_value() << ":" << vector[first_index].suint.current_max << ") at "  \
                  << first_index << " " << preposition << " "                                                          \
                  << (vector[second_index].suint.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].suint.get_value() << ":" << vector[second_index].suint.current_max           \
                  << ") at " << second_index << std::flush;                                                            \
    }

#define PRINT_THREE_ARG_INSTRUCTION(                                                                                   \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)                        \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].suint.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].suint.get_value() << ":" << vector[first_index].suint.current_max << ") at "  \
                  << first_index << " " << preposition1 << " "                                                         \
                  << (vector[second_index].suint.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].suint.get_value() << ":" << vector[second_index].suint.current_max           \
                  << ") at " << second_index << " " << preposition2 << " "                                             \
                  << (vector[third_index].suint.is_constant() ? "constant(" : "witness(")                              \
                  << vector[third_index].suint.get_value() << ":" << vector[third_index].suint.current_max << ") at "  \
                  << third_index << std::flush;                                                                        \
    }
#define PRINT_TWO_ARG_ONE_VALUE_INSTRUCTION(                                                                           \
    first_index, second_index, third_index, vector, operation_name, preposition1, preposition2)                        \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].suint.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].suint.get_value() << ":" << vector[first_index].suint.current_max << ") at "  \
                  << first_index << " " << preposition1 << " "                                                         \
                  << (vector[second_index].suint.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].suint.get_value() << ":" << vector[second_index].suint.current_max           \
                  << ") at " << second_index << " " << preposition2 << " " << third_index << std::flush;               \
    }

#define PRINT_TWO_ARG_TWO_VALUES_INSTRUCTION(                                                                          \
    first_index, second_index, value1, value2, vector, operation_name, preposition1, preposition2, preposition3)       \
    {                                                                                                                  \
        std::cout << operation_name << " " << (vector[first_index].suint.is_constant() ? "constant(" : "witness(")     \
                  << vector[first_index].suint.get_value() << ":" << vector[first_index].suint.current_max << ") at "  \
                  << first_index << " " << preposition1 << " "                                                         \
                  << (vector[second_index].suint.is_constant() ? "constant(" : "witness(")                             \
                  << vector[second_index].suint.get_value() << ":" << vector[second_index].suint.current_max           \
                  << ") at " << second_index << " " << preposition2 << " " << value1 << preposition3 << value2         \
                  << std::flush;                                                                                       \
    }

#define PRINT_SLICE(first_index, lsb, msb, vector)                                                                     \
    {                                                                                                                  \
        std::cout << "Slice:"                                                                                          \
                  << " " << (vector[first_index].suint.is_constant() ? "constant(" : "witness(")                       \
                  << vector[first_index].suint.get_value() << ":" << vector[first_index].suint.current_max << ") at "  \
                  << first_index << " "                                                                                \
                  << "(" << (size_t)lsb << ":" << (size_t)msb << ")" << std::flush;                                    \
    }

#define PRINT_RESULT(prefix, action, index, value)                                                                     \
    {                                                                                                                  \
        std::cout << "  result(" << value.suint.get_value() << " : " << value.suint.current_max << ") " << action      \
                  << index << std::endl                                                                                \
                  << std::flush;                                                                                       \
    }

#else

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

/**
 * @brief The class parametrizing SafeUint fuzzing instructions, execution, etc
 *
 */
template <typename Composer> class SafeUintFuzzBase {
  private:
    typedef plonk::stdlib::bool_t<Composer> bool_t;
    typedef plonk::stdlib::field_t<Composer> field_t;
    typedef plonk::stdlib::safe_uint_t<Composer> suint_t;
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
            DIVIDE,
            ADD_TWO,
            MADD,
            SUBTRACT_WITH_CONSTRAINT,
            DIVIDE_WITH_CONSTRAINTS,
            SLICE,
            RANDOMSEED,
            _LAST
        };
        struct Element {
            fr value;
            uint8_t bit_range;
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

        struct SliceArgs {
            uint8_t in1;
            uint8_t lsb;
            uint8_t msb;
            uint8_t out1;
            uint8_t out2;
            uint8_t out3;
        };
        union ArgumentContents {
            uint32_t randomseed;
            Element element;
            ThreeArgs threeArgs;
            FourArgs fourArgs;
            FiveArgs fiveArgs;
            SliceArgs sliceArgs;
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
            uint8_t in1, in2, in3, lsb, msb, out, out1, out2, out3, mask_size, bit_range;
            uint256_t mask, temp;
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
                // We want small values, too. If we generate randomly, we aren't going to have them, so we also apply a
                // random mask, which randomizes the logarithm of maximum value
                mask_size = static_cast<uint8_t>(rng.next() & 0xff);
                mask = (uint256_t(1) << mask_size) - 1;
                // Choose the bit range
                bit_range = static_cast<uint8_t>(rng.next() & 0xff);
                // Return instruction
                return { .id = instruction_opcode,
                         .arguments.element = { .value = fr(temp & mask), .bit_range = bit_range } };

                break;
            case OPCODE::ADD:
            case OPCODE::SUBTRACT:
            case OPCODE::MULTIPLY:
            case OPCODE::DIVIDE:
                // For two-input-one-output instructions we just randomly pick each argument and generate an instruction
                // accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.threeArgs = { .in1 = in1, .in2 = in2, .out = out } };
                break;
            case OPCODE::ADD_TWO:
            case OPCODE::MADD:
            case OPCODE::SUBTRACT_WITH_CONSTRAINT:
                // For three-input-one-output instructions we just randomly pick each argument and generate an
                // instruction accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                in3 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode,
                         .arguments.fourArgs = { .in1 = in1, .in2 = in2, .in3 = in3, .out = out } };
                break;

            case OPCODE::DIVIDE_WITH_CONSTRAINTS:
                // For four-input-one-output instructions we just randomly pick each argument and generate an
                // instruction accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                in3 = static_cast<uint8_t>(rng.next() & 0xff);
                lsb = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode,
                         .arguments.fiveArgs = { .in1 = in1, .in2 = in2, .qbs = in3, .rbs = lsb, .out = out } };

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
                break;
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
            // With a certain probability, we apply changes to the Montgomery form, rather than the plain form. This has
            // merit, since the computation is performed in montgomery form and comparisons are often performed in it,
            // too. Libfuzzer comparison tracing logic can then be enabled in Montgomery form
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
                switch (rng.next() % 8) {
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
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.element.bit_range);
                // Maybe mutate the value
                if (rng.next() & 1) {
                    instruction.arguments.element.value =
                        mutateFieldElement(instruction.arguments.element.value, rng, havoc_config);
                }
                break;
            case OPCODE::ADD:
            case OPCODE::DIVIDE:
            case OPCODE::MULTIPLY:
            case OPCODE::SUBTRACT:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.out)
                break;
            case OPCODE::ADD_TWO:
            case OPCODE::MADD:
            case OPCODE::SUBTRACT_WITH_CONSTRAINT:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.in3)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fourArgs.out)
                break;
            case OPCODE::DIVIDE_WITH_CONSTRAINTS:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fiveArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fiveArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fiveArgs.qbs)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fiveArgs.rbs)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.fiveArgs.out)
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
        static constexpr size_t CONSTANT = sizeof(uint256_t) + 1;
        static constexpr size_t WITNESS = sizeof(uint256_t) + 1;
        static constexpr size_t CONSTANT_WITNESS = sizeof(uint256_t) + 1;
        static constexpr size_t ADD = 3;
        static constexpr size_t SUBTRACT = 3;
        static constexpr size_t MULTIPLY = 3;
        static constexpr size_t ADD_TWO = 4;
        static constexpr size_t DIVIDE = 3;
        static constexpr size_t MADD = 4;
        static constexpr size_t SUBTRACT_WITH_CONSTRAINT = 4;
        static constexpr size_t DIVIDE_WITH_CONSTRAINTS = 5;
        static constexpr size_t SLICE = 6;
        static constexpr size_t RANDOMSEED = sizeof(uint32_t);
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
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.element = { .value = fr::serialize_from_buffer(Data + 1),
                                                           .bit_range = *Data } };
            }
            if constexpr (opcode == Instruction::OPCODE::ADD || opcode == Instruction::OPCODE::MULTIPLY ||
                          opcode == Instruction::OPCODE::DIVIDE || opcode == Instruction::OPCODE::SUBTRACT) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.threeArgs = { .in1 = *Data, .in2 = *(Data + 1), .out = *(Data + 2) } };
            }
            if constexpr (opcode == Instruction::OPCODE::MADD || opcode == Instruction::OPCODE::ADD_TWO ||
                          opcode == Instruction::OPCODE::SUBTRACT_WITH_CONSTRAINT) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.fourArgs = {
                                        .in1 = *Data, .in2 = *(Data + 1), .in3 = *(Data + 2), .out = *(Data + 3) } };
            }
            if constexpr (opcode == Instruction::OPCODE::DIVIDE_WITH_CONSTRAINTS) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.fiveArgs = { .in1 = *Data,
                                                            .in2 = *(Data + 1),
                                                            .qbs = *(Data + 2),
                                                            .rbs = *(Data + 3),
                                                            .out = *(Data + 4) } };
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
                *(Data + 1) = instruction.arguments.element.bit_range;
                fr::serialize_to_buffer(instruction.arguments.element.value, Data + 2);
            }

            if constexpr (instruction_opcode == Instruction::OPCODE::ADD ||
                          instruction_opcode == Instruction::OPCODE::DIVIDE ||
                          instruction_opcode == Instruction::OPCODE::MULTIPLY ||
                          instruction_opcode == Instruction::OPCODE::SUBTRACT) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.threeArgs.in1;
                *(Data + 2) = instruction.arguments.threeArgs.in2;
                *(Data + 3) = instruction.arguments.threeArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::ADD_TWO ||
                          instruction_opcode == Instruction::OPCODE::MADD ||
                          instruction_opcode == Instruction::OPCODE::SUBTRACT_WITH_CONSTRAINT) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.fourArgs.in1;
                *(Data + 2) = instruction.arguments.fourArgs.in2;
                *(Data + 3) = instruction.arguments.fourArgs.in3;
                *(Data + 4) = instruction.arguments.fourArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::DIVIDE_WITH_CONSTRAINTS) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.fiveArgs.in1;
                *(Data + 2) = instruction.arguments.fiveArgs.in2;
                *(Data + 3) = instruction.arguments.fiveArgs.qbs;
                *(Data + 4) = instruction.arguments.fiveArgs.rbs;
                *(Data + 5) = instruction.arguments.fiveArgs.out;
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
        suint_t s() const
        {
            const bool reconstruct = static_cast<bool>(VarianceRNG.next() % 2);

            if (!reconstruct) {
                return this->suint;
            }

            return suint_t(this->suint);
        }

      public:
        // The value that tracks the actual uint value and shouldn't be able to overflow, so it helps detect when suint
        // overflows the modulus
        uint512_t reference_value;

        suint_t suint;
        ExecutionHandler() = default;
        ExecutionHandler(uint512_t& r, suint_t& s)
            : reference_value(r)
            , suint(s)
        {
            // If the reference value overflows the modulus, the circuit is expected to fail
            if (r >= static_cast<uint512_t>(fr::modulus)) {
                circuit_should_fail = true;
            }
        }
        ExecutionHandler(uint512_t r, suint_t s)
            : reference_value(r)
            , suint(s)
        {

            // If the reference value overflows the modulus, the circuit is expected to fail
            if (r >= static_cast<uint512_t>(fr::modulus)) {

                circuit_should_fail = true;
            }
        }
        ExecutionHandler(suint_t s)
            : reference_value(s.get_value())
            , suint(s)
        {}
        ExecutionHandler operator+(const ExecutionHandler& other)
        {
            return ExecutionHandler(this->reference_value + other.reference_value, this->s() + other.s());
        }
        ExecutionHandler subtract(const ExecutionHandler& other, size_t difference_bit_size)
        {
            if ((this->reference_value - other.reference_value) >= (uint512_t(1) << difference_bit_size)) {
                circuit_should_fail = true;
            }
            return ExecutionHandler(this->reference_value - other.reference_value,
                                    this->s().subtract(other.s(), difference_bit_size));
        }
        ExecutionHandler operator-(const ExecutionHandler& other)
        {
            return ExecutionHandler(this->reference_value - other.reference_value, this->s() - other.s());
        }
        ExecutionHandler operator*(const ExecutionHandler& other)
        {
            return ExecutionHandler(this->reference_value * other.reference_value, this->s() * other.s());
        }
        ExecutionHandler divide(const ExecutionHandler& other, size_t quotient_bit_size, size_t remainder_bit_size)
        {
            if (other.s().get_value() == 0) {
                circuit_should_fail = true;
            }
            auto quotient = static_cast<uint512_t>(this->reference_value.lo / other.reference_value.lo);
            auto remainder = this->reference_value.lo - quotient.lo * other.reference_value.lo;
            if ((quotient.lo >= (uint256_t(1) << quotient_bit_size)) ||
                (remainder >= (uint256_t(1) << remainder_bit_size))) {
                circuit_should_fail = true;
            }
            return ExecutionHandler(quotient, this->s().divide(other.s(), quotient_bit_size, remainder_bit_size));
        }
        ExecutionHandler operator/(const ExecutionHandler& other)
        {
            if (other.s().get_value() == 0) {
                circuit_should_fail = true;
            }
            return ExecutionHandler(static_cast<uint512_t>(this->reference_value.lo / other.reference_value.lo),
                                    this->s() / other.s());
        }

        ExecutionHandler add_two(const ExecutionHandler& other1, const ExecutionHandler& other2)
        {

            return ExecutionHandler(this->reference_value + other1.reference_value + other2.reference_value,
                                    this->s().add_two(other1.s(), other2.s()));
        }

        ExecutionHandler madd(const ExecutionHandler& other1, const ExecutionHandler& other2)
        {

            return ExecutionHandler(this->reference_value * other1.reference_value + other2.reference_value,
                                    this->s().madd(other1.s(), other2.s()));
        }

        std::array<ExecutionHandler, 3> slice(uint8_t lsb, uint8_t msb)
        {
            const auto msb_plus_one = uint32_t(msb) + 1;
            const auto hi_mask = ((uint256_t(1) << (256 - uint32_t(msb))) - 1);
            const auto hi_reference = (this->reference_value.lo >> msb_plus_one) & hi_mask;

            const auto lo_mask = (uint256_t(1) << lsb) - 1;
            const auto lo_reference = this->reference_value.lo & lo_mask;

            const auto slice_mask = ((uint256_t(1) << (uint32_t(msb - lsb) + 1)) - 1);
            const auto slice_reference = (this->reference_value.lo >> lsb) & slice_mask;

            auto lo_slice_hi_suint_array = this->s().slice(msb, lsb);
            return std::array<ExecutionHandler, 3>{ ExecutionHandler(lo_reference, lo_slice_hi_suint_array[0]),
                                                    ExecutionHandler(slice_reference, lo_slice_hi_suint_array[1]),
                                                    ExecutionHandler(hi_reference, lo_slice_hi_suint_array[2]) };
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
            stack.push_back(suint_t(instruction.arguments.element.value));
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
            size_t bit_range = static_cast<size_t>(instruction.arguments.element.bit_range);
            // This is handled by an assert
            if ((bit_range > suint_t::MAX_BIT_NUM) ||
                (bit_range > 0 && (uint256_t(instruction.arguments.element.value) >= (uint256_t(1) << bit_range)))) {
                return 1;
            }
            // Bit range ==0 should only work for the 0 value
            if (bit_range == 0 && instruction.arguments.element.value != 0) {
                circuit_should_fail = true;
            }

            stack.push_back(suint_t(witness_t(composer, instruction.arguments.element.value),
                                    instruction.arguments.element.bit_range));
#ifdef SHOW_INFORMATION
            std::cout << "Pushed witness value " << instruction.arguments.element.value << " < 2^" << (size_t)bit_range
                      << " to position " << stack.size() - 1 << std::endl;
#endif
            return 0;
        }

        /**
         * @brief Execute the constant_witness instruction (push a safeuint witness equal to the constant to the stack)
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
            stack.push_back(suint_t::create_constant_witness(composer, instruction.arguments.element.value));
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

            // If the maximum values overflow 256 bits, this is detected by ASSERTS
            if ((static_cast<uint512_t>(stack[first_index].suint.current_max) *
                 static_cast<uint512_t>(stack[second_index].suint.current_max))
                    .hi != 0) {
                // Handled by asserts
                return 1;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index] * stack[second_index];
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                throw err;
            }
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
            try {
                result = stack[first_index] + stack[second_index];
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                throw err;
            }
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

            // Perform ASSERT checks that we've disabled
            if ((static_cast<uint512_t>(1 << (stack[first_index].suint.current_max.get_msb() + 1)) +
                 static_cast<uint512_t>(stack[second_index].suint.current_max)) > suint_t::MAX_VALUE) {
                // We don't want to trigger the throw
                return 0;
            }

            if (stack[first_index].suint.is_constant() && stack[second_index].suint.is_constant() &&
                (static_cast<uint256_t>(stack[first_index].suint.get_value()) <
                 static_cast<uint256_t>(stack[second_index].suint.get_value()))) {
                // This case is handled by assert
                return 0;
            }
            // When we subtract values, there is an ASSERT that checks that the maximum possible result can be
            // constrained. So let's check it beforehand
            if ((stack[first_index].suint.current_max.get_msb() + 1) > suint_t::MAX_BIT_NUM) {
                return 0;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index] - stack[second_index];
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                if (!strncmp(err.what(),
                             "maximum value exceeded in safe_uint minus operator",
                             sizeof("maximum value exceeded in safe_uint minus operator"))) {
                    return 1;
                }
                throw err;
            }
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

            if (stack[first_index].suint.value.is_constant()) {
                return 1;
            }
            // Assert checks
            // The maximum value of the quotient * divisor shouldn't overflow uint256_t
            if ((((uint512_t(1) << (stack[first_index].suint.current_max.get_msb() + 1)) - 1) *
                 stack[second_index].suint.current_max)
                    .hi != 0) {
                return 0;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index] / stack[second_index];
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                if (!strncmp(err.what(),
                             "maximum value exceeded in safe_uint minus operator",
                             sizeof("maximum value exceeded in safe_uint minus operator"))) {
                    return 1;
                }
                throw err;
            }
            // If division of zero by zero passes that is not ok.
            if (stack[first_index].suint.get_value().is_zero() && stack[second_index].suint.get_value().is_zero()) {
                circuit_should_fail = true;
            }
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

            if ((static_cast<uint512_t>(stack[first_index].suint.current_max) +
                 static_cast<uint512_t>(stack[second_index].suint.current_max) +
                 static_cast<uint512_t>(stack[third_index].suint.current_max)) >
                static_cast<uint512_t>(suint_t::MAX_VALUE)) {
                return 1;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index].add_two(stack[second_index], stack[third_index]);
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                throw err;
            }
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

            // If maximums overflow the modulus, then skip this instruction (an assert should handle this)
            if ((static_cast<uint512_t>(stack[first_index].suint.current_max) *
                     static_cast<uint512_t>(stack[second_index].suint.current_max) +
                 static_cast<uint512_t>(stack[third_index].suint.current_max)) >
                static_cast<uint512_t>(suint_t::MAX_VALUE)) {
                return 0;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index].madd(stack[second_index], stack[third_index]);
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                throw err;
            }
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
         * @brief Execute the SUBTRACT_WITH_CONSTRAINT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
        size_t
            */
        static inline size_t execute_SUBTRACT_WITH_CONSTRAINT(Composer* composer,
                                                              std::vector<ExecutionHandler>& stack,
                                                              Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fourArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fourArgs.in2 % stack.size();
            size_t difference_bit_size = instruction.arguments.fourArgs.in3;
            size_t output_index = instruction.arguments.fourArgs.out;
            PRINT_TWO_ARG_ONE_VALUE_INSTRUCTION(
                first_index, second_index, difference_bit_size, stack, "SUBTRACT_WITH_CONSTRAINT:", "-", "<= 2**")

            // If difference bit size is too big, it should be caught by assertion.
            if (difference_bit_size > suint_t::MAX_BIT_NUM) {
                return 0;
            }
            // If both constants, should be handled by assert
            if (stack[first_index].suint.is_constant() && stack[second_index].suint.is_constant()) {
                return 0;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index].subtract(stack[second_index], difference_bit_size);
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "maximum value exceeded in safe_uint subtract",
                             sizeof("maximum value exceeded in safe_uint subtract"))) {
                    return 1;
                }
                throw err;
            }
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
         * @brief Execute the DIVIDE_WITH_CONSTRAINTS instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was
 encountered
            */
        static inline size_t execute_DIVIDE_WITH_CONSTRAINTS(Composer* composer,
                                                             std::vector<ExecutionHandler>& stack,
                                                             Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.fiveArgs.in1 % stack.size();
            size_t second_index = instruction.arguments.fiveArgs.in2 % stack.size();
            size_t quotient_bit_size = instruction.arguments.fiveArgs.qbs;
            size_t remainder_bit_size = instruction.arguments.fiveArgs.rbs;
            size_t output_index = instruction.arguments.fiveArgs.out;
            PRINT_TWO_ARG_TWO_VALUES_INSTRUCTION(first_index,
                                                 second_index,
                                                 quotient_bit_size,
                                                 remainder_bit_size,
                                                 stack,
                                                 "DIVIDE_WITH_CONSTRAINTS:",
                                                 "/",
                                                 "quotient < 2**",
                                                 "remainder < 2**")

            // If difference bit size is too big, it should be caught by assertion.
            if ((quotient_bit_size > suint_t::MAX_BIT_NUM) || (remainder_bit_size > suint_t::MAX_BIT_NUM)) {
                return 0;
            }
            // If both constants, should be handled by assert
            if (stack[first_index].suint.is_constant()) {
                return 0;
            }
            ExecutionHandler result;
            try {
                result = stack[first_index].divide(stack[second_index], quotient_bit_size, remainder_bit_size);
            } catch (std::runtime_error& err) {
                if (!strncmp(err.what(),
                             "exceeded modulus in safe_uint class",
                             sizeof("exceeded modulus in safe_uint class"))) {
                    return 1;
                }
                if (!strncmp(err.what(),
                             "maximum value exceeded in safe_uint minus operator",
                             sizeof("maximum value exceeded in safe_uint minus operator"))) {
                    return 1;
                }
                throw err;
            }
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
                (static_cast<uint256_t>(stack[first_index].suint.get_value()) >=
                 (static_cast<uint256_t>(1) << grumpkin::MAX_NO_WRAP_INTEGER_BIT_LENGTH))) {
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
    };

    typedef std::vector<ExecutionHandler> ExecutionState;
};

#ifdef HAVOC_TESTING

extern "C" int LLVMFuzzerInitialize(int* argc, char*** argv)
{
    (void)argc;
    (void)argv;
    // These are the settings, optimized for the safeuint class (under them, fuzzer reaches maximum expected coverage in
    // 40 seconds)
    fuzzer_havoc_settings = HavocSettings{
        .GEN_LLVM_POST_MUTATION_PROB = 30,          // Out of 200
        .GEN_MUTATION_COUNT_LOG = 5,                // Fully checked
        .GEN_STRUCTURAL_MUTATION_PROBABILITY = 300, // Fully  checked
        .GEN_VALUE_MUTATION_PROBABILITY = 700,      // Fully checked
        .ST_MUT_DELETION_PROBABILITY = 100,         // Fully checked
        .ST_MUT_DUPLICATION_PROBABILITY = 80,       // Fully checked
        .ST_MUT_INSERTION_PROBABILITY = 120,        // Fully checked
        .ST_MUT_MAXIMUM_DELETION_LOG = 6,           // Fully checked
        .ST_MUT_MAXIMUM_DUPLICATION_LOG = 2,        // Fully checked
        .ST_MUT_SWAP_PROBABILITY = 50,              // Fully checked
        .VAL_MUT_LLVM_MUTATE_PROBABILITY = 250,     // Fully checked
        .VAL_MUT_MONTGOMERY_PROBABILITY = 130,      // Fully checked
        .VAL_MUT_NON_MONTGOMERY_PROBABILITY = 50,   // Fully checked
        .VAL_MUT_SMALL_ADDITION_PROBABILITY = 110,  // Fully checked
        .VAL_MUT_SPECIAL_VALUE_PROBABILITY = 130    // Fully checked

    };
    /**
     * @brief This is used, when we need to determine the probabilities of various mutations. Left here for posterity
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
              << "GEN_STRUCTURAL_MUTATION_PROBABILITY: " << fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY
              << std::endl
              << "GEN_VALUE_MUTATION_PROBABILITY: " << fuzzer_havoc_settings.GEN_VALUE_MUTATION_PROBABILITY << std::endl
              << "ST_MUT_DELETION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_DELETION_PROBABILITY << std::endl
              << "ST_MUT_DUPLICATION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_DUPLICATION_PROBABILITY << std::endl
              << "ST_MUT_INSERTION_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_INSERTION_PROBABILITY << std::endl
              << "ST_MUT_MAXIMUM_DELETION_LOG: " << fuzzer_havoc_settings.ST_MUT_MAXIMUM_DELETION_LOG << std::endl
              << "ST_MUT_MAXIMUM_DUPLICATION_LOG: " << fuzzer_havoc_settings.ST_MUT_MAXIMUM_DUPLICATION_LOG << std::endl
              << "ST_MUT_SWAP_PROBABILITY: " << fuzzer_havoc_settings.ST_MUT_SWAP_PROBABILITY << std::endl
              << "VAL_MUT_LLVM_MUTATE_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_LLVM_MUTATE_PROBABILITY
              << std::endl
              << "VAL_MUT_MONTGOMERY_PROBABILITY: " << fuzzer_havoc_settings.VAL_MUT_MONTGOMERY_PROBABILITY << std::endl
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
    using FuzzerClass = SafeUintFuzzBase<waffle::StandardComposer>;
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
    using FuzzerClass = SafeUintFuzzBase<waffle::StandardComposer>;
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
    RunWithComposers<SafeUintFuzzBase, FuzzerComposerTypes>(Data, Size, VarianceRNG);
    return 0;
}
