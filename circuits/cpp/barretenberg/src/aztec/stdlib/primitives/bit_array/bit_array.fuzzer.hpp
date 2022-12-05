#include <numeric/random/engine.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include "../../../rollup/constants.hpp"

#define MAX_ARRAY_SIZE 128

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
 * @brief The class parametrizing ByteArray fuzzing instructions, execution, etc
 *
 */
template <typename Composer> class BitArrayFuzzBase {
  private:
    typedef plonk::stdlib::bit_array<Composer> bit_array_t;
    typedef plonk::stdlib::byte_array<Composer> byte_array_t;
    template <size_t NumBytes, size_t NumWords>
    static std::vector<uint8_t> to_vector(std::array<plonk::stdlib::uint32<Composer>, NumWords>& a32)
    {
        /* Convert array of uint32_t to vector of uint8_t */
        std::vector<uint8_t> v(NumBytes);
        for (size_t i = 0; i < a32.size(); i++) {
            const uint32_t u32 = htonl(static_cast<uint32_t>(a32[i].get_value()));
            memcpy(v.data() + (i * 4), &u32, 4);
        }

        return v;
    }

    template <size_t NumWords>
    static std::vector<uint8_t> vector_via_populate_uint32_array(bit_array_t& bit_array, const size_t offset = 0)
    {
        /* NumWords can never be 0 because this case has explicit specialization */
        static_assert(NumWords != 0);

        constexpr size_t NumBits = NumWords * 32;
        constexpr size_t NumBytes = NumWords * 4;

        if (offset > bit_array.size() || (bit_array.size() - offset) % 32 != 0) {
            /* If the offset is out-of-bounds, or the output would not
             * be a multiple of 32 bits, return the whole buffer.
             */
            return static_cast<byte_array_t>(bit_array).get_value();
        } else if (bit_array.size() - offset == NumBits) {
            std::array<plonk::stdlib::uint32<Composer>, NumWords> a32;
            bit_array.template populate_uint32_array<NumWords>(offset, a32);
            return to_vector<NumBytes, NumWords>(a32);
        } else {
            /* Template recursion: iterate from NumWords..0 */
            return vector_via_populate_uint32_array<NumWords - 1>(bit_array, offset);
        }
    }

    template <> static std::vector<uint8_t> vector_via_populate_uint32_array<0>(bit_array_t&, const size_t)
    {
        return {};
    }

    template <size_t NumBytes>
    static std::vector<uint8_t> bit_array_to_a32(bit_array_t& bit_array, const bool cast_or_populate)
    {
        /* NumBytes can never be 0 because this case has explicit specialization */
        static_assert(NumBytes != 0);

        /* Must be a multiple of uint32_t for this to work */
        static_assert(NumBytes % 4 == 0);

        constexpr size_t NumWords = NumBytes / 4;
        constexpr size_t NumBits = NumBytes * 8;

        if (bit_array.size() % 32 != 0) {
            /* Bit array is not a multiple of 32 bits; cannot convert to array of uint32_t.
             * Return vector instead.
             */
            return static_cast<byte_array_t>(bit_array).get_value();
        } else if (bit_array.size() == NumBits) {
            std::array<plonk::stdlib::uint32<Composer>, NumWords> a32;

            /* Switch between two different methods to retrieve the uint32 array */
            if (cast_or_populate) {
                a32 = static_cast<decltype(a32)>(bit_array);
            } else {
                bit_array.template populate_uint32_array<NumWords>(0, a32);
            }

            return to_vector<NumBytes, NumWords>(a32);
        } else {
            /* Template recursion: iterate from NumBytes..0 */
            return bit_array_to_a32<NumBytes - 4>(bit_array, cast_or_populate);
        }
    }

    template <> std::vector<uint8_t> bit_array_to_a32<0>(bit_array_t&, const bool) { return {}; }

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
        enum OPCODE { CONSTANT, GET_BIT, SET_BIT, SLICE, SET, RANDOMSEED, _LAST };
        struct Element {
          public:
            std::array<uint8_t, MAX_ARRAY_SIZE> data;
            uint16_t size;

            uint16_t real_size(void) const { return std::min(size, static_cast<uint16_t>(MAX_ARRAY_SIZE)); }
            std::string as_string(void) const { return from_to<decltype(data), std::string>(data, real_size()); }
        };
        struct GetBitArgs {
            uint8_t in;
            uint8_t out;
            uint32_t bit;
        };
        struct SetBitArgs {
            uint8_t in;
            uint32_t bit;
            uint8_t value;
        };
        struct SliceArgs {
            uint8_t in;
            uint8_t out;
            uint16_t offset;
        };
        struct TwoArgs {
            uint8_t in;
            uint8_t out;
        };

        union ArgumentContents {
            uint32_t randomseed;
            Element element;
            GetBitArgs getBitArgs;
            SetBitArgs setBitArgs;
            SliceArgs sliceArgs;
            TwoArgs twoArgs;
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
            uint8_t in1, out, value;
            uint32_t bit;
            uint16_t offset;
            // Depending on instruction
            switch (instruction_opcode) {
            case OPCODE::CONSTANT:
                // Return instruction
                {
                    std::array<uint8_t, MAX_ARRAY_SIZE> data;
                    for (size_t i = 0; i < MAX_ARRAY_SIZE; i++) {
                        data[i] = rng.next() & 0xFF;
                    }

                    const uint16_t size = rng.next() & 0xFFFF;
                    return { .id = instruction_opcode, .arguments.element = { .data = data, .size = size } };
                }
                break;
            case OPCODE::GET_BIT:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                bit = static_cast<uint32_t>(rng.next() & 0xffffffff);
                return { .id = instruction_opcode, .arguments.getBitArgs = { .in = in1, .out = out, .bit = bit } };
            case OPCODE::SET_BIT:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                bit = static_cast<uint32_t>(rng.next() & 0xffffffff);
                value = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.setBitArgs = { .in = in1, .bit = bit, .value = value } };
            case OPCODE::SLICE:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                offset = static_cast<uint16_t>(rng.next() & 0xffff);
                return { .id = instruction_opcode, .arguments.sliceArgs = { .in = in1, .out = out, .offset = offset } };
            case OPCODE::SET:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.twoArgs = { .in = in1, .out = out } };
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
#define PUT_RANDOM_FOUR_BYTES_IF_LUCKY(variable)                                                                       \
    if (rng.next() & 1) {                                                                                              \
        variable = rng.next() & 0xffffffff;                                                                            \
    }
            // Depending on instruction type...
            switch (instruction.id) {
            case OPCODE::CONSTANT:
                break;
            case OPCODE::GET_BIT:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.getBitArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.getBitArgs.out)
                PUT_RANDOM_FOUR_BYTES_IF_LUCKY(instruction.arguments.getBitArgs.bit)
                break;
            case OPCODE::SET_BIT:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.setBitArgs.in)
                PUT_RANDOM_FOUR_BYTES_IF_LUCKY(instruction.arguments.setBitArgs.bit)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.setBitArgs.value)
                break;
            case OPCODE::SLICE:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.out)
                PUT_RANDOM_TWO_BYTES_IF_LUCKY(instruction.arguments.sliceArgs.offset)
                break;
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
        static constexpr size_t CONSTANT = MAX_ARRAY_SIZE + sizeof(uint16_t);
        static constexpr size_t GET_BIT = 6;
        static constexpr size_t SET_BIT = 6;
        static constexpr size_t SLICE = 4;
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
                std::array<uint8_t, MAX_ARRAY_SIZE> data;
                std::copy_n(Data, data.size(), data.begin());

                uint16_t size;
                memcpy(&size, Data + MAX_ARRAY_SIZE, sizeof(uint16_t));

                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.element = { .data = data, .size = size } };
            }
            if constexpr (opcode == Instruction::OPCODE::GET_BIT) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.getBitArgs = {
                                        .in = *Data, .out = *(Data + 1), .bit = *((uint32_t*)(Data + 2)) } };
            }
            if constexpr (opcode == Instruction::OPCODE::SET_BIT) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.setBitArgs = {
                                        .in = *Data, .bit = *((uint32_t*)(Data + 1)), .value = *(Data + 5) } };
            }
            if constexpr (opcode == Instruction::OPCODE::SLICE) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.sliceArgs = {
                                        .in = *Data, .out = *(Data + 1), .offset = *((uint16_t*)(Data + 2)) } };
            }
            if constexpr (opcode == Instruction::OPCODE::SET) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
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
                memcpy(Data + 1, instruction.arguments.element.data.data(), MAX_ARRAY_SIZE);
                memcpy(Data + 1 + MAX_ARRAY_SIZE, &instruction.arguments.element.size, sizeof(uint16_t));
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::GET_BIT) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.getBitArgs.in;
                *(Data + 2) = instruction.arguments.getBitArgs.out;
                *((uint32_t*)(Data + 3)) = instruction.arguments.getBitArgs.bit;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SET_BIT) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.setBitArgs.in;
                *((uint32_t*)(Data + 2)) = instruction.arguments.setBitArgs.bit;
                *(Data + 6) = instruction.arguments.setBitArgs.value;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SLICE) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.sliceArgs.in;
                *(Data + 2) = instruction.arguments.sliceArgs.out;
                *((uint16_t*)(Data + 3)) = instruction.arguments.sliceArgs.offset;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SET) {
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
      public:
        std::vector<uint8_t> reference_value;

        bit_array_t bit_array{ nullptr, std::vector<uint8_t>{} };

        static bool get_bit(const std::vector<uint8_t>& v, size_t bit, const bool rev = true)
        {
            const size_t pos = rev ? v.size() - 1 - (bit / 8) : (bit / 8);
            bit = rev ? bit % 8 : 7 - (bit % 8);
            return static_cast<bool>(v[
                                         /* Byte */ pos] &
                                     /* Bit */ (1 << bit));
        }
        static void set_bit(std::vector<uint8_t>& v, size_t bit, const bool value, const bool rev = true)
        {
            const size_t pos = rev ? v.size() - 1 - (bit / 8) : (bit / 8);
            bit = rev ? bit % 8 : 7 - (bit % 8);
            if (value) {
                v[
                    /* Byte */ pos] |=
                    /* Bit */ (1 << bit);
            } else {
                v[
                    /* Byte */ pos] &=
                    /* Bit */ ~(1 << bit);
            }
        }

        static std::vector<uint8_t> get_value(bit_array_t& bit_array)
        {
            const uint8_t which = VarianceRNG.next() % 4;

            switch (which) {
            case 0:
                return from_to<std::string, std::vector<uint8_t>>(bit_array.get_witness_as_string());
            case 1: {
                const auto bits = bit_array.get_bits();
                std::vector<uint8_t> ret((bits.size() + 7) / 8, 0);
                for (size_t i = 0; i < bits.size(); i++) {
                    set_bit(ret, i, bits[i].get_value());
                }
                return ret;
            }
            case 2:
                return static_cast<byte_array_t>(bit_array).get_value();
            case 3:
                return bit_array_to_a32<MAX_ARRAY_SIZE>(bit_array, static_cast<bool>(VarianceRNG.next() % 2));
                static_assert(MAX_ARRAY_SIZE % 32 == 0);
                if (bit_array.size() == MAX_ARRAY_SIZE / 32) {
                    std::array<uint32_t, MAX_ARRAY_SIZE> a32;
                    const auto a32_ =
                        static_cast<std::array<plonk::stdlib::uint32<Composer>, MAX_ARRAY_SIZE>>(bit_array);
                    for (size_t i = 0; i < a32_.size(); i++) {
                        a32[i] = static_cast<uint32_t>(a32_[i].get_value());
                    }
                    return from_to<std::array<uint32_t, MAX_ARRAY_SIZE>, std::vector<uint8_t>>(a32);
                } else {
                    return static_cast<byte_array_t>(bit_array).get_value();
                }
            default:
                abort();
            }
        }
        static std::vector<uint8_t> v32_to_v8(const std::vector<uint32_t>& v)
        {
            return from_to<std::vector<uint32_t>, std::vector<uint8_t>>(v);
        }
        static const std::vector<uint8_t>& bool_to_vector(const bool& b)
        {
            static const std::vector<uint8_t> false_{ 0 };
            static const std::vector<uint8_t> true_{ 1 };
            return b ? true_ : false_;
        }
        ExecutionHandler() = default;
        ExecutionHandler(std::vector<uint8_t>& r, bit_array_t& s)
            : reference_value(r)
            , bit_array(s)
        {}
        ExecutionHandler(std::vector<uint8_t> r, bit_array_t s)
            : reference_value(r)
            , bit_array(s)
        {}
        ExecutionHandler(bit_array_t s)
            : reference_value(get_value(s))
            , bit_array(s)
        {}

        ExecutionHandler get_bit(Composer* composer, const size_t bit) const
        {
            if (bit >= this->reference_value.size() * 8) {
                return ExecutionHandler(this->reference_value, this->bit_array);
            } else {
                const bool is_set_ref = get_bit(this->reference_value, bit);
                const bool is_set_ba = this->bit_array[bit].get_value();

                return ExecutionHandler(bool_to_vector(is_set_ref), bit_array_t(composer, bool_to_vector(is_set_ba)));
            }
        }
        /* Modifies the buffer at hand, so does not produce a return value */
        void set_bit(const size_t bit, const bool value)
        {
            if (bit < this->reference_value.size() * 8) {
                set_bit(this->reference_value, bit, value);
                this->bit_array[bit] = value;
            }
        }
        /* output = input[offset:] (where offset denotes bits) */
        ExecutionHandler slice(Composer* composer, const size_t offset)
        {
            static_assert(MAX_ARRAY_SIZE % 4 == 0);
            const auto v_ba = vector_via_populate_uint32_array<MAX_ARRAY_SIZE / 4>(this->bit_array, offset);

            std::vector<uint8_t> v_ref;

            const auto& ref = this->reference_value;
            const size_t ref_num_bits = ref.size() * 8;
            const size_t out_num_bits = ref_num_bits - offset;
            const size_t out_num_bytes = out_num_bits / 8;
            if (offset > ref_num_bits || out_num_bits % 32 != 0) {
                v_ref = ref;
            } else {
                v_ref.resize(out_num_bytes);
                for (size_t i = 0; i < out_num_bits; i++) {
                    set_bit(v_ref, i, get_bit(ref, i + offset, false), false);
                }
            }

            return ExecutionHandler(v_ref, bit_array_t(composer, v_ba));
        }

        /* Explicit re-instantiation using the various bit_array constructors */
        ExecutionHandler set(Composer* composer)
        {
            const uint8_t which = VarianceRNG.next() % 6;

            const auto& ref = this->reference_value;

            switch (which) {
            case 0:
                /* Construct via bit_array */
                return ExecutionHandler(ref, bit_array_t(this->bit_array));
            case 1:
                /* Construct via std::string */
                return ExecutionHandler(ref, bit_array_t(composer, this->bit_array.get_witness_as_string()));
            case 2:
                /* Construct via std::vector<uint8_t> */
                return ExecutionHandler(ref,
                                        bit_array_t(composer, static_cast<byte_array_t>(this->bit_array).get_value()));
            case 3:
                /* Construct via byte_array */
                return ExecutionHandler(ref, bit_array_t(static_cast<byte_array_t>(this->bit_array)));
            case 4:
                if (this->bit_array.size() % 32 != 0) {
                    return ExecutionHandler(ref, bit_array_t(this->bit_array));
                } else {
                    const auto v = this->bit_array.to_uint32_vector();

                    if (v.size() == 1 && static_cast<bool>(VarianceRNG.next() % 2)) {
                        /* Construct via uint32<ComposerContext> */
                        return ExecutionHandler(ref, bit_array_t(v[0]));
                    } else {
                        /* Construct via std::vector<uint32<ComposerContext>> */
                        return ExecutionHandler(ref, bit_array_t(v));
                    }
                }
            case 5: {
                /* Create a bit_array with gibberish.
                 *
                 * The purpose of this is to ascertain that no gibberish
                 * values are retained in the re-assigned value
                 */
                const size_t gibberish_size = VarianceRNG.next() % (MAX_ARRAY_SIZE * 2);
                std::vector<uint8_t> gibberish(gibberish_size);
                for (size_t i = 0; i < gibberish_size; i++) {
                    gibberish[i] = static_cast<uint8_t>(VarianceRNG.next() % 0xFF);
                }
                auto ba = bit_array_t(composer, gibberish);

                /* Construct via assignment */
                ba = this->bit_array;

                return ExecutionHandler(ref, ba);
            } break;
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
            (void)composer;
            stack.push_back(bit_array_t(composer, instruction.arguments.element.as_string()));
            return 0;
        }
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
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.getBitArgs.in % stack.size();
            size_t output_index = instruction.arguments.getBitArgs.out;
            const uint32_t bit = instruction.arguments.getBitArgs.bit;
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
         * @brief Execute the SET_BIT instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SET_BIT(Composer* composer,
                                             std::vector<ExecutionHandler>& stack,
                                             Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.setBitArgs.in % stack.size();
            const uint32_t bit = instruction.arguments.setBitArgs.bit;
            const bool value = static_cast<bool>(instruction.arguments.setBitArgs.value % 2);
            stack[first_index].set_bit(bit, value);
            return 0;
        };
        /**
         * @brief Execute the SLICE instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_SLICE(Composer* composer,
                                           std::vector<ExecutionHandler>& stack,
                                           Instruction& instruction)
        {
            (void)composer;
            if (stack.size() == 0) {
                return 1;
            }
            size_t first_index = instruction.arguments.sliceArgs.in % stack.size();
            size_t output_index = instruction.arguments.sliceArgs.out;
            const uint16_t offset = instruction.arguments.sliceArgs.offset;
            ExecutionHandler result;
            result = stack[first_index].slice(composer, offset);
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
            (void)composer;
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
    inline static bool postProcess(Composer* composer, std::vector<BitArrayFuzzBase::ExecutionHandler>& stack)
    {
        (void)composer;
        for (size_t i = 0; i < stack.size(); i++) {
            auto element = stack[i];
            const auto other = from_to<std::string, std::vector<uint8_t>>(element.bit_array.get_witness_as_string());
            if (other != element.reference_value) {
                printf("Other (as bytes):\n");
                for (size_t i = 0; i < other.size(); i++) {
                    printf("%02X ", other[i]);
                }
                printf("\n");
                printf("Reference value (as bytes):\n");
                for (size_t i = 0; i < element.reference_value.size(); i++) {
                    printf("%02X ", element.reference_value[i]);
                }
                printf("\n");
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
    using FuzzerClass = BitArrayFuzzBase<waffle::StandardComposer>;
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
    using FuzzerClass = BitArrayFuzzBase<waffle::StandardComposer>;
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
    RunWithComposers<BitArrayFuzzBase, FuzzerComposerTypes>(Data, Size, VarianceRNG);
    return 0;
}
