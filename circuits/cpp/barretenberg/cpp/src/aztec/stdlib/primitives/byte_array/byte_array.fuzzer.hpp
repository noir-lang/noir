#include <numeric/random/engine.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/safe_uint/safe_uint.hpp>

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
template <typename Composer> class ByteArrayFuzzBase {
  private:
    typedef plonk::stdlib::byte_array<Composer> byte_array_t;
    typedef plonk::stdlib::field_t<Composer> field_t;
    typedef plonk::stdlib::safe_uint_t<Composer> suint_t;

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
        enum OPCODE { CONSTANT, REVERSE, SLICE, GET_BIT, SET_BIT, ADD, SET, RANDOMSEED, _LAST };
        struct Element {
          public:
            std::array<uint8_t, MAX_ARRAY_SIZE> data;
            uint16_t size;

            uint16_t real_size(void) const { return std::min(size, static_cast<uint16_t>(MAX_ARRAY_SIZE)); }
            std::vector<uint8_t> as_vector(void) const
            {
                return from_to<decltype(data), std::vector<uint8_t>>(data, real_size());
            }
            std::string as_string(void) const { return from_to<decltype(data), std::string>(data, real_size()); }
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
        struct SliceArgs {
            uint8_t in;
            uint8_t out;
            uint16_t offset;
            uint16_t length;
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

        union ArgumentContents {
            uint32_t randomseed;
            Element element;
            TwoArgs twoArgs;
            ThreeArgs threeArgs;
            SliceArgs sliceArgs;
            GetBitArgs getBitArgs;
            SetBitArgs setBitArgs;
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
            uint8_t in1, in2, out, value;
            uint16_t offset, length;
            uint32_t bit;
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
            case OPCODE::REVERSE:
            case OPCODE::SET:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.twoArgs = { .in = in1, .out = out } };
                break;
            case OPCODE::SLICE:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                offset = static_cast<uint16_t>(rng.next() & 0xffff);
                length = static_cast<uint16_t>(rng.next() & 0xffff);
                return { .id = instruction_opcode,
                         .arguments.sliceArgs = { .in = in1, .out = out, .offset = offset, .length = length } };
            case OPCODE::GET_BIT:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                bit = static_cast<uint32_t>(rng.next() & 0xffffffff);
                return { .id = instruction_opcode, .arguments.getBitArgs = { .in = in1, .out = out, .bit = bit } };
            case OPCODE::ADD:
                // For two-input-one-output instructions we just randomly pick each argument and generate an instruction
                // accordingly
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                in2 = static_cast<uint8_t>(rng.next() & 0xff);
                out = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.threeArgs = { .in1 = in1, .in2 = in2, .out = out } };
            case OPCODE::SET_BIT:
                in1 = static_cast<uint8_t>(rng.next() & 0xff);
                bit = static_cast<uint32_t>(rng.next() & 0xffffffff);
                value = static_cast<uint8_t>(rng.next() & 0xff);
                return { .id = instruction_opcode, .arguments.setBitArgs = { .in = in1, .bit = bit, .value = value } };
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
            case OPCODE::REVERSE:
            case OPCODE::SET:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.twoArgs.out)
                break;
            case OPCODE::SLICE:
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.in)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.sliceArgs.out)
                PUT_RANDOM_TWO_BYTES_IF_LUCKY(instruction.arguments.sliceArgs.offset)
                PUT_RANDOM_TWO_BYTES_IF_LUCKY(instruction.arguments.sliceArgs.length)
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
            case OPCODE::ADD:
                // Randomly sample each of the arguments with 50% probability
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in1)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.in2)
                PUT_RANDOM_BYTE_IF_LUCKY(instruction.arguments.threeArgs.out)
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
        static constexpr size_t REVERSE = 2;
        static constexpr size_t SLICE = 6;
        static constexpr size_t GET_BIT = 6;
        static constexpr size_t SET_BIT = 6;
        static constexpr size_t ADD = 3;
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
            if constexpr (opcode == Instruction::OPCODE::REVERSE || opcode == Instruction::OPCODE::SET) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.twoArgs = { .in = *Data, .out = *(Data + 1) } };
            }
            if constexpr (opcode == Instruction::OPCODE::SLICE) {
                return Instruction{ .id = static_cast<typename Instruction::OPCODE>(opcode),
                                    .arguments.sliceArgs = { .in = *Data,
                                                             .out = *(Data + 1),
                                                             .offset = *((uint16_t*)(Data + 2)),
                                                             .length = *((uint16_t*)(Data + 4)) } };
            }
            if constexpr (opcode == Instruction::OPCODE::ADD) {
                return { .id = static_cast<typename Instruction::OPCODE>(opcode),
                         .arguments.threeArgs = { .in1 = *Data, .in2 = *(Data + 1), .out = *(Data + 2) } };
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
                memcpy(Data, instruction.arguments.element.data.data(), MAX_ARRAY_SIZE);
                memcpy(Data + MAX_ARRAY_SIZE, &instruction.arguments.element.size, sizeof(uint16_t));
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::REVERSE ||
                          instruction_opcode == Instruction::OPCODE::SET) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.twoArgs.in;
                *(Data + 2) = instruction.arguments.twoArgs.out;
            }
            if constexpr (instruction_opcode == Instruction::OPCODE::SLICE) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.sliceArgs.in;
                *(Data + 2) = instruction.arguments.sliceArgs.out;
                *((uint16_t*)(Data + 3)) = instruction.arguments.sliceArgs.offset;
                *((uint16_t*)(Data + 5)) = instruction.arguments.sliceArgs.length;
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
            if constexpr (instruction_opcode == Instruction::OPCODE::ADD) {
                *Data = instruction.id;
                *(Data + 1) = instruction.arguments.threeArgs.in1;
                *(Data + 2) = instruction.arguments.threeArgs.in2;
                *(Data + 3) = instruction.arguments.threeArgs.out;
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

        byte_array_t byte_array{ nullptr, std::vector<uint8_t>{} };

        static std::vector<uint8_t> get_value(const byte_array_t& byte_array)
        {
            /* Based on the PRNG, alternate between retrieving an std::vector
             * and a string.
             * These should be functionally equivalent.
             */
            if (static_cast<bool>(VarianceRNG.next() % 2)) {
                return byte_array.get_value();
            } else {
                return from_to<std::string, std::vector<uint8_t>>(byte_array.get_string());
            }
        }
        static const std::vector<uint8_t>& bool_to_vector(const bool& b)
        {
            static const std::vector<uint8_t> false_{ 0 };
            static const std::vector<uint8_t> true_{ 1 };
            return b ? true_ : false_;
        }
        std::optional<field_t> to_field_t(std::optional<size_t> max_msb = std::nullopt) const
        {
            const auto& ref = this->reference_value;

            if (ref.size() > 32) {
                /* Cannot construct via field if size is larger than field */
                return std::nullopt;
            } else if (ref.size() == 32) {
                uint64_t u0, u1, u2, u3;
                memcpy(&u3, ref.data(), 8);
                memcpy(&u2, ref.data() + 8, 8);
                memcpy(&u1, ref.data() + 16, 8);
                memcpy(&u0, ref.data() + 24, 8);
                const uint256_t u256{ htonll(u0), htonll(u1), htonll(u2), htonll(u3) };
                if (max_msb != std::nullopt && u256.get_msb() >= max_msb) {
                    return std::nullopt;
                }
                if (u256 >= field_t::modulus) {
                    return std::nullopt;
                }
            }

            return static_cast<field_t>(this->byte_array);
        }
        ExecutionHandler() = default;
        ExecutionHandler(std::vector<uint8_t>& r, byte_array_t& s)
            : reference_value(r)
            , byte_array(s)
        {}
        ExecutionHandler(std::vector<uint8_t> r, byte_array_t s)
            : reference_value(r)
            , byte_array(s)
        {}
        ExecutionHandler(byte_array_t s)
            : reference_value(get_value(s))
            , byte_array(s)
        {}

        ExecutionHandler reverse() const
        {
            auto reversed = this->reference_value;
            std::reverse(reversed.begin(), reversed.end());

            return ExecutionHandler(reversed, this->byte_array.reverse());
        }
        ExecutionHandler slice(const size_t offset, const size_t length) const
        {
            if (offset > this->reference_value.size()) {
                /* Offset is beyond buffer bounds; cannot comply.
                 * Return the whole buffer.
                 */
                return ExecutionHandler(this->reference_value, this->byte_array.slice(0));
            } else if (offset + length > this->reference_value.size()) {
                /* Offset is valid but range is not.
                 * Return data from the offset to the end of the buffer.
                 */
                return ExecutionHandler(
                    std::vector<uint8_t>(this->reference_value.data() + offset,
                                         this->reference_value.data() + this->reference_value.size()),
                    this->byte_array.slice(offset));
            } else {
                return ExecutionHandler(std::vector<uint8_t>(this->reference_value.data() + offset,
                                                             this->reference_value.data() + offset + length),
                                        this->byte_array.slice(offset, length));
            }
        }
        ExecutionHandler get_bit(Composer* composer, const size_t bit) const
        {
            if (bit >= this->reference_value.size() * 8) {
                return ExecutionHandler(this->reference_value, this->byte_array);
            } else {
                const bool is_set_ref = this->reference_value[
                                            /* Byte */ this->reference_value.size() - 1 - (bit / 8)] &
                                        /* Bit */ (1 << (bit % 8));
                const bool is_set_ba = this->byte_array.get_bit(bit).get_value();

                return ExecutionHandler(bool_to_vector(is_set_ref), byte_array_t(composer, bool_to_vector(is_set_ba)));
            }
        }
        /* Modifies the buffer at hand, so does not produce a return value */
        void set_bit(const size_t bit, const bool value)
        {
            if (bit < this->reference_value.size() * 8) {
                if (value) {
                    this->reference_value[
                        /* Byte */ this->reference_value.size() - 1 - (bit / 8)] |=
                        /* Bit */ (1 << (bit % 8));
                } else {
                    this->reference_value[
                        /* Byte */ this->reference_value.size() - 1 - (bit / 8)] &=
                        /* Bit */ ~(1 << (bit % 8));
                }
                this->byte_array.set_bit(bit, value);
            }
        }
        ExecutionHandler operator+(const ExecutionHandler& other)
        {
            if (this->reference_value.size() + other.reference_value.size() > (MAX_ARRAY_SIZE * 3)) {
                return ExecutionHandler(this->reference_value, this->byte_array.slice(0));
            } else {
                const auto other_ref = other.reference_value;
                this->reference_value.insert(this->reference_value.end(), other_ref.begin(), other_ref.end());

                return ExecutionHandler(std::vector<uint8_t>(this->reference_value),
                                        this->byte_array.write(other.byte_array));
            }
        }
        /* Explicit re-instantiation using the various bit_array constructors */
        ExecutionHandler set(Composer* composer)
        {
            const auto& ref = this->reference_value;

            switch (VarianceRNG.next() % 8) {
            case 0:
                /* Construct via byte_array */
                return ExecutionHandler(ref, byte_array_t(this->byte_array));
            case 1:
                /* Construct via std::string */
                return ExecutionHandler(ref, byte_array_t(composer, this->byte_array.get_string()));
            case 2:
                /* Construct via std::vector<uint8_t> */
                return ExecutionHandler(ref, byte_array_t(composer, this->byte_array.get_value()));
            case 3:
                /* Construct via bytes_t */
                return ExecutionHandler(ref, byte_array_t(composer, this->byte_array.bytes()));
            case 4:
                /* Construct via bytes_t move constructor */
                return ExecutionHandler(ref, byte_array_t(composer, std::move(this->byte_array.bytes())));
            case 5: {
                const auto field = to_field_t();

                if (field == std::nullopt) {
                    return ExecutionHandler(ref, byte_array_t(this->byte_array));
                } else {
                    /* Pick a number ref.size()..32 */
                    const size_t num_bytes = ref.size() + (VarianceRNG.next() % (32 - ref.size() + 1));
                    if (num_bytes > 32)
                        abort(); /* Should never happen */

                    const size_t excess_bytes = num_bytes - ref.size();

                    /* Construct new reference value */
                    std::vector<uint8_t> new_ref(excess_bytes, 0);
                    new_ref.insert(new_ref.end(), ref.begin(), ref.end());

                    /* Construct via field_t */
                    return ExecutionHandler(new_ref, byte_array_t(*field, num_bytes));
                }
            }
            case 6: {
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
                auto ba = byte_array_t(composer, gibberish);

                /* Construct via assignment */
                ba = this->byte_array;

                return ExecutionHandler(ref, ba);
            } break;
            case 7: {
                static_assert(suint_t::MAX_BIT_NUM > 0);
                const auto field = to_field_t(
                    /* One bit must be reserved */
                    suint_t::MAX_BIT_NUM - 1);

                if (field == std::nullopt) {
                    return ExecutionHandler(ref, byte_array_t(this->byte_array));
                } else {
                    /* Test the suint constructor.
                     *
                     * byte_array -> field -> suint -> byte_array
                     */
                    return ExecutionHandler(ref, byte_array_t(suint_t(*field, suint_t::MAX_BIT_NUM), ref.size()));
                }
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
            if (static_cast<bool>(VarianceRNG.next() % 2)) {
                stack.push_back(byte_array_t(composer, instruction.arguments.element.as_vector()));
            } else {
                stack.push_back(byte_array_t(composer, instruction.arguments.element.as_string()));
            }
            return 0;
        }
        /**
         * @brief Execute the REVERSE instruction
         *
         * @param composer
         * @param stack
         * @param instruction
         * @return if everything is ok, 1 if we should stop execution, since an expected error was encountered
         */
        static inline size_t execute_REVERSE(Composer* composer,
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
            result = stack[first_index].reverse();
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
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
            size_t first_index = instruction.arguments.sliceArgs.in % stack.size();
            size_t output_index = instruction.arguments.sliceArgs.out;
            const uint16_t offset = instruction.arguments.sliceArgs.offset;
            const uint16_t length = instruction.arguments.sliceArgs.length;
            ExecutionHandler result;
            result = stack[first_index].slice(offset, length);
            // If the output index is larger than the number of elements in stack, append
            if (output_index >= stack.size()) {
                stack.push_back(result);
            } else {
                stack[output_index] = result;
            }
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
         * @brief Execute the ADD (append) instruction
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
    inline static bool postProcess(Composer* composer, std::vector<ByteArrayFuzzBase::ExecutionHandler>& stack)
    {
        (void)composer;
        for (size_t i = 0; i < stack.size(); i++) {
            auto element = stack[i];
            if (element.byte_array.get_value() != element.reference_value) {
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
    using FuzzerClass = ByteArrayFuzzBase<waffle::StandardComposer>;
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
    using FuzzerClass = ByteArrayFuzzBase<waffle::StandardComposer>;
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
    RunWithComposers<ByteArrayFuzzBase, FuzzerComposerTypes>(Data, Size, VarianceRNG);
    return 0;
}
