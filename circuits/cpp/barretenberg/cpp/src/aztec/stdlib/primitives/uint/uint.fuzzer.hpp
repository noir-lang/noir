#include <numeric/random/engine.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/bool/bool.hpp>
#include "../../../rollup/constants.hpp"
// This is a global variable, so that the execution handling class could alter it and signal to the input tester that
// the input should fail
bool circuit_should_fail = false;

#define HAVOC_TESTING

#include <common/fuzzer.hpp>
FastRandom VarianceRNG(0);

// Enable this definition, when you want to find out the instructions that caused a failure
//#define SHOW_INFORMATION 1

#define OPERATION_TYPE_SIZE 1

#define ELEMENT_SIZE (sizeof(fr) + 1)
#define TWO_IN_ONE_OUT 3
#define THREE_IN_ONE_OUT 4
#define SLICE_ARGS_SIZE 6

/**
 * @brief The class parametrizing Uint fuzzing instructions, execution, etc
 *
 */
template <typename Composer> class UintFuzzBase {
  private:
    typedef plonk::stdlib::bool_t<Composer> bool_t;
    typedef plonk::stdlib::uint<Composer, uint8_t> uint_8_t;
    typedef plonk::stdlib::uint<Composer, uint16_t> uint_16_t;
    typedef plonk::stdlib::uint<Composer, uint32_t> uint_32_t;
    typedef plonk::stdlib::uint<Composer, uint64_t> uint_64_t;
    typedef plonk::stdlib::field_t<Composer> field_t;
    typedef plonk::stdlib::byte_array<Composer> byte_array_t;

    template <class From, class To> static To from_to(const From& in, const std::optional<size_t> size = std::nullopt)
    {
        return To(in.data(), in.data() + (size ? *size : in.size()));
    }

  public:
    /**
     * @brief A class representing a single fuzzing instruction
     *
     */
    class Instruction {
      public:
        enum OPCODE {
            CONSTANT,
            ADD,
            SUBTRACT,
            MULTIPLY,
            DIVIDE,
            MODULO,
            AND,
            OR,
            XOR,
            GET_BIT,
            SHL,
            SHR,
            ROL,
            ROR,
            NOT,
            SET,
            RANDOMSEED,
            _LAST
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
        struct BitArgs {
            uint8_t in;
            uint8_t out;
            uint64_t bit;
        };
        union ArgumentContents {
            uint32_t randomseed;
            uint64_t element;
            TwoArgs twoArgs;
            ThreeArgs threeArgs;
            BitArgs bitArgs;
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
            uint8_t in1, in2, out;
            uint32_t bit;
            // Depending on instruction
            switch (instruction_opcode) {
            case OPCODE::CONSTANT:
                return { .id = instruction_opcode, .arguments.element = rng.next() };
                break;
            case OPCODE::ADD:
            case OPCODE::SUBTRACT:
            case OPCODE::MULTIPLY:
            case OPCODE::DIVIDE:
            case OPCODE::MODULO:
            case OPCODE::AND:
            case OPCODE::OR:
            case OPCODE::XOR:
                // For two-input-one-output instructions we just randomly pick each argument and generate an instruction
                // accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.threeArgs = { .in1 = in1, .in2 = in2, .out = out } };
                break;
            case OPCODE::GET_BIT:
            case OPCODE::SHL:
            case OPCODE::SHR:
            case OPCODE::ROL:
            case OPCODE::ROR:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                bit = static_cast<uint32_t>(rng.next() & 0xffffffff);
                return { .id = instruction_opcode, .arguments.bitArgs = { .in = in1, .out = out, .bit = bit } };
            case OPCODE::NOT:
            case OPCODE::SET:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.twoArgs = { .in = in1, .out = out } };
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
            (void)rng;
            (void)havoc_config;
#define PUT_RANDOM_BYTE_IF_LUCKY(variable)                                                                             \
    if (rng.next() & 1) {                                                                                              \
        variable = rng.next() & 0xff;                                                                                  \
    }
#define PUT_RANDOM_TWO_BYTES_IF_LUCKY(variable)                                                                        \
    if (rng.next() & 1) {                                                                                              \
        variable = rng.next() & 0xffff;                                                                                \
    }
#define PUT_RANDOM_EIGHT_BYTES_IF_LUCKY(variable)                                                                      \
    if (rng.next() & 1) {                                                                                              \
        variable = rng.next() & 0xffffffff;                                                                            \
        variable <<= 32;                                                                                               \
        variable += rng.next() & 0xffffffff;                                                                           \
    }
            // Depending on instruction type...
            switch (instruction.id) {
            case OPCODE::CONSTANT:
                break;
            case OPCODE::ADD:
            case OPCODE::SUBTRACT:
            case OPCODE::MULTIPLY:
            case OPCODE::DIVIDE:
            case OPCODE::MODULO:
            case OPCODE::AND:
            case OPCODE::OR:
            case OPCODE::XOR:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.out)
                break;
            case OPCODE::GET_BIT:
            case OPCODE::SHL:
            case OPCODE::SHR:
            case OPCODE::ROL:
            case OPCODE::ROR:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.bitArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.bitArgs.out)
                PUT_RANDOM_EIGHT_BYTES_IF_LUCKY(instruction.arguments.bitArgs.bit)
            case OPCODE::NOT:
            case OPCODE::SET:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.out)
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
        static constexpr size_t CONSTANT = sizeof(uint64_t);
        static constexpr size_t ADD = 3;
        static constexpr size_t SUBTRACT = 3;
        static constexpr size_t MULTIPLY = 3;
        static constexpr size_t DIVIDE = 3;
        static constexpr size_t MODULO = 3;
        static constexpr size_t AND = 3;
        static constexpr size_t OR = 3;
        static constexpr size_t XOR = 3;
        static constexpr size_t GET_BIT = 10;
        static constexpr size_t SHL = 10;
        static constexpr size_t SHR = 10;
        static constexpr size_t ROL = 10;
        static constexpr size_t ROR = 10;
        static constexpr size_t NOT = 2;
        static constexpr size_t SET = 2;
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
            if constexpr (opcode == Instruction::OPCODE::CONSTANT) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.element = *((uint64_t*)Data) };
            }
            if constexpr (opcode == Instruction::OPCODE::ADD || opcode == Instruction::OPCODE::SUBTRACT ||
                          opcode == Instruction::OPCODE::MULTIPLY || opcode == Instruction::OPCODE::DIVIDE ||
                          opcode == Instruction::OPCODE::MODULO || opcode == Instruction::OPCODE::AND ||
                          opcode == Instruction::OPCODE::OR || opcode == Instruction::OPCODE::XOR) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.threeArgs = { .in1 = *Data, .in2 = *(Data + 1), .out = *(Data + 2) } };
            }
            if constexpr (opcode == Instruction::OPCODE::GET_BIT || opcode == Instruction::OPCODE::SHL ||
                          opcode == Instruction::OPCODE::SHR || opcode == Instruction::OPCODE::ROL ||
                          opcode == Instruction::OPCODE::ROR) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.bitArgs = {
                                        .in = *Data, .out = *(Data + 1), .bit = *((uint64_t*)(Data + 2)) } };
            }
            if constexpr (opcode == Instruction::OPCODE::NOT || opcode == Instruction::OPCODE::SET) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.twoArgs = { .in = *Data, .out = *(Data + 1) } };
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
            if constexpr (instruction_opcode == Instruction::OPCODE::CONSTANT) {
                *Data = instruction.id;
                memcpy(Data + 1, &instruction.arguments.element, sizeof(uint64_t));
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::ADD ||
                          instruction_opcode == Instruction::OPCODE::SUBTRACT ||
                          instruction_opcode == Instruction::OPCODE::MULTIPLY ||
                          instruction_opcode == Instruction::OPCODE::DIVIDE ||
                          instruction_opcode == Instruction::OPCODE::MODULO ||
                          instruction_opcode == Instruction::OPCODE::AND ||
                          instruction_opcode == Instruction::OPCODE::OR ||
                          instruction_opcode == Instruction::OPCODE::XOR) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.threeArgs.in1;
                *(Data + 2) = instruction.arguments.threeArgs.in2;
                *(Data + 3) = instruction.arguments.threeArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::GET_BIT ||
                          instruction_opcode == Instruction::OPCODE::SHL ||
                          instruction_opcode == Instruction::OPCODE::SHR ||
                          instruction_opcode == Instruction::OPCODE::ROL ||
                          instruction_opcode == Instruction::OPCODE::ROR) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.bitArgs.in;
                *(Data + 2) = instruction.arguments.bitArgs.out;
                *((uint64_t*)(Data + 3)) = instruction.arguments.bitArgs.bit;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::NOT ||
                          instruction_opcode == Instruction::OPCODE::SET) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.twoArgs.in;
                *(Data + 2) = instruction.arguments.twoArgs.out;
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
        template <class T> static T shl(const T v, const size_t bits)
        {
            if (bits >= sizeof(T) * 8) {
                return 0;
            } else {
                return static_cast<T>(v << bits);
            }
        }
        template <class T> static T shr(const T v, const size_t bits)
        {
            if (bits >= sizeof(T) * 8) {
                return 0;
            } else {
                return static_cast<T>(v >> bits);
            }
        }
        template <class T> static T get_bit(const T v, const size_t bit)
        {
            if (bit >= sizeof(T) * 8) {
                return 0;
            } else {
                return (v & (uint64_t(1) << bit)) ? 1 : 0;
            }
        }
        /* wrapper for uint::at which ensures the context of
         * the return value has been set
         */
        template <class T> static bool_t at(const T& v, const size_t bit_index)
        {
            const auto ret = v.at(bit_index);

            if (ret.get_context() != v.get_context()) {
                std::cerr << "Context of return bool_t not set" << std::endl;
                abort();
            }

            return ret;
        }
        template <class T> static T get_bit(Composer* composer, const T& v, const size_t bit)
        {
            return T(composer, std::vector<bool_t>{ at<>(v, bit) });
        }
        template <class T> static std::vector<bool_t> to_bit_vector(const T& v)
        {
            std::vector<bool_t> bits;
            for (size_t i = 0; i < v.get_width(); i++) {
                bits.push_back(at<>(v, i));
            }
            return bits;
        }
        template <class T> static std::array<bool_t, T::width> to_bit_array(const T& v)
        {
            std::array<bool_t, T::width> bits;
            for (size_t i = 0; i < T::width; i++) {
                bits[i] = at<>(v, i);
            }
            return bits;
        }
        template <class T> static uint256_t get_value(const T& v)
        {
            const auto ret = v.get_value();

            if (ret.get_msb() >= T::width) {
                std::cerr << "uint256_t returned by get_value() exceeds type width" << std::endl;
                abort();
            }

            return std::move(ret);
        }
        template <class T> static byte_array_t to_byte_array(const T& v)
        {
            const auto ret = static_cast<byte_array_t>(v);

            static_assert(T::width % 8 == 0);
            if (ret.size() > T::width / 8) {
                std::cerr << "byte_array version of uint exceeds type width" << std::endl;
                abort();
            }

            return ret;
        }
        template <class T> static field_t to_field_t(const T& v)
        {
            auto ret = static_cast<field_t>(v);

            if (static_cast<uint256_t>(ret.get_value()) != v.get_value()) {
                std::cerr << "field_t version of uint differs from its value" << std::endl;
                abort();
            }

            return ret;
        }

      public:
        class Uint {
          public:
            uint_8_t v8;
            uint_16_t v16;
            uint_32_t v32;
            uint_64_t v64;

            Uint() = default;
            Uint(uint_8_t v8, uint_16_t v16, uint_32_t v32, uint_64_t v64)
                : v8(v8)
                , v16(v16)
                , v32(v32)
                , v64(v64)
            {}
            Uint(Composer* composer, const uint64_t v)
                : v8(composer, static_cast<uint8_t>(v & 0xFF))
                , v16(composer, static_cast<uint16_t>(v & 0xFFFF))
                , v32(composer, static_cast<uint32_t>(v & 0xFFFFFFFF))
                , v64(composer, v)
            {}
        };
        class Reference {
          public:
            uint8_t v8;
            uint16_t v16;
            uint32_t v32;
            uint64_t v64;

            Reference() = default;
            Reference(uint8_t v8, uint16_t v16, uint32_t v32, uint64_t v64)
                : v8(v8)
                , v16(v16)
                , v32(v32)
                , v64(v64)
            {}
            Reference(const Uint& u)
                : v8(get_value<>(u.v8))
                , v16(get_value<>(u.v16))
                , v32(get_value<>(u.v32))
                , v64(get_value<>(u.v64))
            {}
        };
        Reference ref;
        Uint uint;

        ExecutionHandler() = default;
        ExecutionHandler(Reference& r, Uint& u)
            : ref(r)
            , uint(u)
        {}
        ExecutionHandler(Reference r, Uint u)
            : ref(r)
            , uint(u)
        {}
        ExecutionHandler(Uint u)
            : ref(u)
            , uint(u)
        {}
        ExecutionHandler operator+(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 + other.ref.v8,
                                       this->ref.v16 + other.ref.v16,
                                       this->ref.v32 + other.ref.v32,
                                       this->ref.v64 + other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* + operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 + other.uint.v8,
                                             this->uint.v16 + other.uint.v16,
                                             this->uint.v32 + other.uint.v32,
                                             this->uint.v64 + other.uint.v64));
            case 1:
                /* += operator */
                {
                    Uint u = uint;

                    u.v8 += other.uint.v8;
                    u.v16 += other.uint.v16;
                    u.v32 += other.uint.v32;
                    u.v64 += other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator-(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 - other.ref.v8,
                                       this->ref.v16 - other.ref.v16,
                                       this->ref.v32 - other.ref.v32,
                                       this->ref.v64 - other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* - operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 - other.uint.v8,
                                             this->uint.v16 - other.uint.v16,
                                             this->uint.v32 - other.uint.v32,
                                             this->uint.v64 - other.uint.v64));
            case 1:
                /* -= operator */
                {
                    Uint u = uint;

                    u.v8 -= other.uint.v8;
                    u.v16 -= other.uint.v16;
                    u.v32 -= other.uint.v32;
                    u.v64 -= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator*(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 * other.ref.v8,
                                       this->ref.v16 * other.ref.v16,
                                       this->ref.v32 * other.ref.v32,
                                       this->ref.v64 * other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* * operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 * other.uint.v8,
                                             this->uint.v16 * other.uint.v16,
                                             this->uint.v32 * other.uint.v32,
                                             this->uint.v64 * other.uint.v64));
            case 1:
                /* *= operator */
                {
                    Uint u = uint;

                    u.v8 *= other.uint.v8;
                    u.v16 *= other.uint.v16;
                    u.v32 *= other.uint.v32;
                    u.v64 *= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator/(const ExecutionHandler& other) const
        {
            const bool divisor_zero =
                other.ref.v8 == 0 || other.ref.v16 == 0 || other.ref.v32 == 0 || other.ref.v64 == 0;
            const Reference ref_result(other.ref.v8 == 0 ? 0 : this->ref.v8 / other.ref.v8,
                                       other.ref.v16 == 0 ? 0 : this->ref.v16 / other.ref.v16,
                                       other.ref.v32 == 0 ? 0 : this->ref.v32 / other.ref.v32,
                                       other.ref.v64 == 0 ? 0 : this->ref.v64 / other.ref.v64);

            if (divisor_zero) {
                circuit_should_fail = true;
            }

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* / operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 / other.uint.v8,
                                             this->uint.v16 / other.uint.v16,
                                             this->uint.v32 / other.uint.v32,
                                             this->uint.v64 / other.uint.v64));
            case 1:
                /* /= operator */
                {
                    Uint u = uint;

                    u.v8 /= other.uint.v8;
                    u.v16 /= other.uint.v16;
                    u.v32 /= other.uint.v32;
                    u.v64 /= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator%(const ExecutionHandler& other) const
        {
            const bool divisor_zero =
                other.ref.v8 == 0 || other.ref.v16 == 0 || other.ref.v32 == 0 || other.ref.v64 == 0;
            const Reference ref_result(other.ref.v8 == 0 ? 0 : this->ref.v8 % other.ref.v8,
                                       other.ref.v16 == 0 ? 0 : this->ref.v16 % other.ref.v16,
                                       other.ref.v32 == 0 ? 0 : this->ref.v32 % other.ref.v32,
                                       other.ref.v64 == 0 ? 0 : this->ref.v64 % other.ref.v64);

            if (divisor_zero) {
                circuit_should_fail = true;
            }

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* % operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 % other.uint.v8,
                                             this->uint.v16 % other.uint.v16,
                                             this->uint.v32 % other.uint.v32,
                                             this->uint.v64 % other.uint.v64));
            case 1:
                /* %= operator */
                {
                    Uint u = uint;

                    u.v8 %= other.uint.v8;
                    u.v16 %= other.uint.v16;
                    u.v32 %= other.uint.v32;
                    u.v64 %= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator&(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 & other.ref.v8,
                                       this->ref.v16 & other.ref.v16,
                                       this->ref.v32 & other.ref.v32,
                                       this->ref.v64 & other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* & operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 & other.uint.v8,
                                             this->uint.v16 & other.uint.v16,
                                             this->uint.v32 & other.uint.v32,
                                             this->uint.v64 & other.uint.v64));
            case 1:
                /* &= operator */
                {
                    Uint u = uint;

                    u.v8 &= other.uint.v8;
                    u.v16 &= other.uint.v16;
                    u.v32 &= other.uint.v32;
                    u.v64 &= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator|(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 | other.ref.v8,
                                       this->ref.v16 | other.ref.v16,
                                       this->ref.v32 | other.ref.v32,
                                       this->ref.v64 | other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* | operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 | other.uint.v8,
                                             this->uint.v16 | other.uint.v16,
                                             this->uint.v32 | other.uint.v32,
                                             this->uint.v64 | other.uint.v64));
            case 1:
                /* |= operator */
                {
                    Uint u = uint;

                    u.v8 |= other.uint.v8;
                    u.v16 |= other.uint.v16;
                    u.v32 |= other.uint.v32;
                    u.v64 |= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler operator^(const ExecutionHandler& other) const
        {
            const Reference ref_result(this->ref.v8 ^ other.ref.v8,
                                       this->ref.v16 ^ other.ref.v16,
                                       this->ref.v32 ^ other.ref.v32,
                                       this->ref.v64 ^ other.ref.v64);

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* ^ operator */
                return ExecutionHandler(ref_result,
                                        Uint(this->uint.v8 ^ other.uint.v8,
                                             this->uint.v16 ^ other.uint.v16,
                                             this->uint.v32 ^ other.uint.v32,
                                             this->uint.v64 ^ other.uint.v64));
            case 1:
                /* ^= operator */
                {
                    Uint u = uint;

                    u.v8 ^= other.uint.v8;
                    u.v16 ^= other.uint.v16;
                    u.v32 ^= other.uint.v32;
                    u.v64 ^= other.uint.v64;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler get_bit(Composer* composer, const size_t bit) const
        {
            return ExecutionHandler(Reference(this->get_bit<uint8_t>(this->ref.v8, bit),
                                              this->get_bit<uint16_t>(this->ref.v16, bit),
                                              this->get_bit<uint32_t>(this->ref.v32, bit),
                                              this->get_bit<uint64_t>(this->ref.v64, bit)),
                                    Uint(this->get_bit<uint_8_t>(composer, this->uint.v8, bit),
                                         this->get_bit<uint_16_t>(composer, this->uint.v16, bit),
                                         this->get_bit<uint_32_t>(composer, this->uint.v32, bit),
                                         this->get_bit<uint_64_t>(composer, this->uint.v64, bit)));
        }
        ExecutionHandler shl(const size_t bits) const
        {
            const Reference ref_result(shl<uint8_t>(this->ref.v8, bits),
                                       shl<uint16_t>(this->ref.v16, bits),
                                       shl<uint32_t>(this->ref.v32, bits),
                                       shl<uint64_t>(this->ref.v64, bits));

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* << operator */
                return ExecutionHandler(
                    ref_result,
                    Uint(
                        this->uint.v8 << bits, this->uint.v16 << bits, this->uint.v32 << bits, this->uint.v64 << bits));
            case 1:
                /* <<= operator */
                {
                    Uint u = uint;

                    u.v8 <<= bits;
                    u.v16 <<= bits;
                    u.v32 <<= bits;
                    u.v64 <<= bits;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler shr(const size_t bits) const
        {
            const Reference ref_result(shr<uint8_t>(this->ref.v8, bits),
                                       shr<uint16_t>(this->ref.v16, bits),
                                       shr<uint32_t>(this->ref.v32, bits),
                                       shr<uint64_t>(this->ref.v64, bits));

            switch (VarianceRNG.next() % 2) {
            case 0:
                /* >> operator */
                return ExecutionHandler(
                    ref_result,
                    Uint(
                        this->uint.v8 >> bits, this->uint.v16 >> bits, this->uint.v32 >> bits, this->uint.v64 >> bits));
            case 1:
                /* >>= operator */
                {
                    Uint u = uint;

                    u.v8 >>= bits;
                    u.v16 >>= bits;
                    u.v32 >>= bits;
                    u.v64 >>= bits;

                    return ExecutionHandler(ref_result, u);
                }
            default:
                abort();
            }
        }
        ExecutionHandler rol(const size_t bits) const
        {
            return ExecutionHandler(Reference(std::rotl(this->ref.v8, static_cast<int>(bits % 8)),
                                              std::rotl(this->ref.v16, static_cast<int>(bits % 16)),
                                              std::rotl(this->ref.v32, static_cast<int>(bits % 32)),
                                              std::rotl(this->ref.v64, static_cast<int>(bits % 64))),
                                    Uint(this->uint.v8.rol(bits),
                                         this->uint.v16.rol(bits),
                                         this->uint.v32.rol(bits),
                                         this->uint.v64.rol(bits)));
        }
        ExecutionHandler ror(const size_t bits) const
        {
            return ExecutionHandler(Reference(std::rotr(this->ref.v8, static_cast<int>(bits % 8)),
                                              std::rotr(this->ref.v16, static_cast<int>(bits % 16)),
                                              std::rotr(this->ref.v32, static_cast<int>(bits % 32)),
                                              std::rotr(this->ref.v64, static_cast<int>(bits % 64))),
                                    Uint(this->uint.v8.ror(bits),
                                         this->uint.v16.ror(bits),
                                         this->uint.v32.ror(bits),
                                         this->uint.v64.ror(bits)));
        }
        ExecutionHandler not_(void) const
        {
            return ExecutionHandler(Reference(~this->ref.v8, ~this->ref.v16, ~this->ref.v32, ~this->ref.v64),
                                    Uint(~this->uint.v8, ~this->uint.v16, ~this->uint.v32, ~this->uint.v64));
        }
        /* Explicit re-instantiation using the various constructors */
        ExecutionHandler set(Composer* composer) const
        {
            switch (VarianceRNG.next() % 7) {
            case 0:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(this->uint.v8),
                                             uint_16_t(this->uint.v16),
                                             uint_32_t(this->uint.v32),
                                             uint_64_t(this->uint.v64)));
            case 1:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(composer, get_value<>(this->uint.v8)),
                                             uint_16_t(composer, get_value<>(this->uint.v16)),
                                             uint_32_t(composer, get_value<>(this->uint.v32)),
                                             uint_64_t(composer, get_value<>(this->uint.v64))));
            case 2:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(this->to_field_t(this->uint.v8)),
                                             uint_16_t(this->to_field_t(this->uint.v16)),
                                             uint_32_t(this->to_field_t(this->uint.v32)),
                                             uint_64_t(this->to_field_t(this->uint.v64))));
            case 3:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(this->to_byte_array(this->uint.v8)),
                                             uint_16_t(this->to_byte_array(this->uint.v16)),
                                             uint_32_t(this->to_byte_array(this->uint.v32)),
                                             uint_64_t(this->to_byte_array(this->uint.v64))));
            case 4:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(composer, this->to_bit_vector(this->uint.v8)),
                                             uint_16_t(composer, this->to_bit_vector(this->uint.v16)),
                                             uint_32_t(composer, this->to_bit_vector(this->uint.v32)),
                                             uint_64_t(composer, this->to_bit_vector(this->uint.v64))));
            case 5:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(composer, this->to_bit_array(this->uint.v8)),
                                             uint_16_t(composer, this->to_bit_array(this->uint.v16)),
                                             uint_32_t(composer, this->to_bit_array(this->uint.v32)),
                                             uint_64_t(composer, this->to_bit_array(this->uint.v64))));
            case 6:
                return ExecutionHandler(this->ref,
                                        Uint(uint_8_t(composer, this->ref.v8),
                                             uint_16_t(composer, this->ref.v16),
                                             uint_32_t(composer, this->ref.v32),
                                             uint_64_t(composer, this->ref.v64)));
            default:
                abort();
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
            stack.push_back(Uint(composer, instruction.arguments.element));
            return 0;
        }
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

            ExecutionHandler result;
            result = stack[first_index] + stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
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

            ExecutionHandler result;
            result = stack[first_index] - stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
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

            ExecutionHandler result;
            result = stack[first_index] * stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
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

            ExecutionHandler result;
            result = stack[first_index] / stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the modulo operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_MODULO(Composer* composer,
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

            ExecutionHandler result;
            result = stack[first_index] % stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the and operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_AND(Composer* composer,
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

            ExecutionHandler result;
            result = stack[first_index] & stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the or operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_OR(Composer* composer,
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

            ExecutionHandler result;
            result = stack[first_index] | stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the xor operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_XOR(Composer* composer,
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

            ExecutionHandler result;
            result = stack[first_index] ^ stack[second_index];
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the GET_BIT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_GET_BIT(Composer* composer,
                                             std::vector<ExecutionHandler>& stack,
                                             Instruction& instruction)
        {
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.bitArgs.in % stack.size();
            size_t output_index = instruction.arguments.bitArgs.out;
            const uint64_t bit = instruction.arguments.bitArgs.bit;
            ExecutionHandler result;
            result = stack[first_index].get_bit(composer, bit);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the left-shift operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SHL(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.bitArgs.in % stack.size();
            size_t output_index = instruction.arguments.bitArgs.out;
            const uint64_t bit = instruction.arguments.bitArgs.bit;
            ExecutionHandler result;
            result = stack[first_index].shl(bit);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the right-shift operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SHR(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.bitArgs.in % stack.size();
            size_t output_index = instruction.arguments.bitArgs.out;
            const uint64_t bit = instruction.arguments.bitArgs.bit;
            ExecutionHandler result;
            result = stack[first_index].shr(bit);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the left-rotate operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ROL(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.bitArgs.in % stack.size();
            size_t output_index = instruction.arguments.bitArgs.out;
            const uint64_t bit = instruction.arguments.bitArgs.bit;
            ExecutionHandler result;
            result = stack[first_index].rol(bit);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the right-rotate operator instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_ROR(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.bitArgs.in % stack.size();
            size_t output_index = instruction.arguments.bitArgs.out;
            const uint64_t bit = instruction.arguments.bitArgs.bit;
            ExecutionHandler result;
            result = stack[first_index].ror(bit);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
        /**
         * @brief Execute the NOT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_NOT(Composer* composer,
                                         std::vector<ExecutionHandler>& stack,
                                         Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t output_index = instruction.arguments.twoArgs.out;

            ExecutionHandler result;
            result = stack[first_index].not_();
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
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
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.twoArgs.in % stack.size();
            size_t output_index = instruction.arguments.twoArgs.out;

            ExecutionHandler result;
            result = stack[first_index].set(composer);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
            return 0;
        };
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
    /**
     * @brief Check that the resulting values are equal to expected
     *
     * @tparam Composer
     * @param composer
     * @param stack
     * @return true
     * @return false
     */
    inline static bool postProcess(Composer* composer, std::vector<UintFuzzBase::ExecutionHandler>& stack)
    {
        (void)composer;
        for (size_t i = 0; i < stack.size(); i++) {
            auto element = stack[i];
            if (element.uint.v8.get_value() != element.ref.v8) {
                std::cerr << "Failed at " << i << " with actual u8 value " << static_cast<size_t>(element.ref.v8)
                          << " and value in uint " << element.uint.v8.get_value() << std::endl;
                return false;
            }
            if (element.uint.v16.get_value() != element.ref.v16) {
                std::cerr << "Failed at " << i << " with actual u16 value " << static_cast<size_t>(element.ref.v16)
                          << " and value in uint " << element.uint.v16.get_value() << std::endl;
                return false;
            }
            if (element.uint.v32.get_value() != element.ref.v32) {
                std::cerr << "Failed at " << i << " with actual u32 value " << static_cast<size_t>(element.ref.v32)
                          << " and value in uint " << element.uint.v32.get_value() << std::endl;
                return false;
            }
            if (element.uint.v64.get_value() != element.ref.v64) {
                std::cerr << "Failed at " << i << " with actual u64 value " << static_cast<size_t>(element.ref.v64)
                          << " and value in uint " << element.uint.v64.get_value() << std::endl;
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
    using FuzzerClass = UintFuzzBase<waffle::StandardComposer>;
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
    using FuzzerClass = UintFuzzBase<waffle::StandardComposer>;
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
    RunWithComposers<UintFuzzBase, FuzzerComposerTypes>(Data, Size, VarianceRNG);
    return 0;
}
