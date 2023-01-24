#pragma once
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <concepts>

#define PARENS ()

// Rescan macro tokens 256 times
#define EXPAND(arg) EXPAND1(EXPAND1(EXPAND1(EXPAND1(arg))))
#define EXPAND1(arg) EXPAND2(EXPAND2(EXPAND2(EXPAND2(arg))))
#define EXPAND2(arg) EXPAND3(EXPAND3(EXPAND3(EXPAND3(arg))))
#define EXPAND3(arg) EXPAND4(EXPAND4(EXPAND4(EXPAND4(arg))))
#define EXPAND4(arg) arg

#define FOR_EACH(macro, ...) __VA_OPT__(EXPAND(FOR_EACH_HELPER(macro, __VA_ARGS__)))
#define FOR_EACH_HELPER(macro, a1, ...) macro(a1) __VA_OPT__(FOR_EACH_AGAIN PARENS(macro, __VA_ARGS__))
#define FOR_EACH_AGAIN() FOR_EACH_HELPER

#define ALL_POSSIBLE_OPCODES                                                                                           \
    CONSTANT, WITNESS, CONSTANT_WITNESS, ADD, SUBTRACT, MULTIPLY, DIVIDE, ADD_TWO, MADD, MULT_MADD, MSUB_DIV, SQR,     \
        ASSERT_EQUAL, ASSERT_NOT_EQUAL, SQR_ADD, ASSERT_EQUAL, ASSERT_NOT_EQUAL, SQR_ADD, SUBTRACT_WITH_CONSTRAINT,    \
        DIVIDE_WITH_CONSTRAINTS, SLICE, ASSERT_ZERO, ASSERT_NOT_ZERO, COND_NEGATE, ADD_MULTI, ASSERT_VALID,            \
        COND_SELECT, DOUBLE, RANDOMSEED, SELECT_IF_ZERO, SELECT_IF_EQ, REVERSE, GET_BIT, SET_BIT, SET, INVERT, AND,    \
        OR, XOR, MODULO, SHL, SHR, ROL, ROR, NOT

struct HavocSettings {
    size_t GEN_LLVM_POST_MUTATION_PROB; // Controls frequency of additional mutation after structural ones
    size_t GEN_MUTATION_COUNT_LOG; // This is the logarithm of the number of micromutations applied during mutation of a
                                   // testcase
    size_t GEN_STRUCTURAL_MUTATION_PROBABILITY; // The probability of applying a structural mutation
                                                // (DELETION/DUPLICATION/INSERTION/SWAP)
    size_t GEN_VALUE_MUTATION_PROBABILITY;      // The probability of applying a value mutation
    size_t ST_MUT_DELETION_PROBABILITY;         // The probability of applying DELETION mutation
    size_t ST_MUT_DUPLICATION_PROBABILITY;      // The probability of applying DUPLICATION mutation
    size_t ST_MUT_INSERTION_PROBABILITY;        // The probability of applying INSERTION mutation
    size_t ST_MUT_MAXIMUM_DELETION_LOG;         // The logarithm of the maximum of deletions
    size_t ST_MUT_MAXIMUM_DUPLICATION_LOG;      // The logarithm of the maximum of duplication
    size_t ST_MUT_SWAP_PROBABILITY;             // The probability of a SWAP mutation
    size_t VAL_MUT_LLVM_MUTATE_PROBABILITY;     // The probablity of using the LLVM mutator on field element value
    size_t VAL_MUT_MONTGOMERY_PROBABILITY;     // The probability of converting to montgomery form before applying value
                                               // mutations
    size_t VAL_MUT_NON_MONTGOMERY_PROBABILITY; // The probability of not converting to montgomery form before applying
                                               // value mutations
    size_t VAL_MUT_SMALL_ADDITION_PROBABILITY; // The probability of performing small additions
    size_t VAL_MUT_SMALL_MULTIPLICATION_PROBABILITY; // The probability of performing small multiplications
    size_t VAL_MUT_SPECIAL_VALUE_PROBABILITY; // The probability of assigning special values (0,1, p-1, p-2, p-1/2)
    std::vector<size_t> structural_mutation_distribution; // Holds the values to quickly select a structural mutation
                                                          // based on chosen probabilities
    std::vector<size_t> value_mutation_distribution; // Holds the values to quickly select a value mutation based on
                                                     // chosen probabilities
};
#ifdef HAVOC_TESTING

HavocSettings fuzzer_havoc_settings;
#endif
// This is an external function in Libfuzzer used internally by custom mutators
extern "C" size_t LLVMFuzzerMutate(uint8_t* Data, size_t Size, size_t MaxSize);

/**
 * @brief Class for quickly deterministically creating new random values. We don't care about distribution much here.
 *
 */
class FastRandom {
    uint32_t state;

  public:
    FastRandom(uint32_t seed) { reseed(seed); }
    uint32_t next()
    {
        state = static_cast<uint32_t>((uint64_t(state) * uint64_t(363364578) + uint64_t(537)) % uint64_t(3758096939));
        return state;
    }
    void reseed(uint32_t seed)
    {
        if (seed == 0) {
            seed = 1;
        }
        state = seed;
    }
};

/**
 * @brief Concept for a simple PRNG which returns a uint32_t when next is called
 *
 * @tparam T
 */
template <typename T> concept SimpleRng = requires(T a)
{
    {
        a.next()
    }
    ->std::convertible_to<uint32_t>;
};
/**
 * @brief Concept for forcing ArgumentSizes to be size_t
 *
 * @tparam T
 */
template <typename T> concept InstructionArgumentSizes = requires
{
    {
        std::make_tuple(T::CONSTANT,
                        T::WITNESS,
                        T::CONSTANT_WITNESS,
                        T::ADD,
                        T::SUBTRACT,
                        T::MULTIPLY,
                        T::DIVIDE,
                        T::ADD_TWO,
                        T::MADD,
                        T::MULT_MADD,
                        T::MSUB_DIV,
                        T::SQR,
                        T::SQR_ADD,
                        T::SUBTRACT_WITH_CONSTRAINT,
                        T::DIVIDE_WITH_CONSTRAINTS,
                        T::SLICE,
                        T::ASSERT_ZERO,
                        T::ASSERT_NOT_ZERO)
    }
    ->std::same_as<std::tuple<size_t>>;
};

/**
 * @brief Concept for Havoc Configurations
 *
 * @tparam T
 */
template <typename T> concept HavocConfigConstraint = requires
{
    {
        std::make_tuple(T::GEN_MUTATION_COUNT_LOG, T::GEN_STRUCTURAL_MUTATION_PROBABILITY)
    }
    ->std::same_as<std::tuple<size_t>>;
    T::GEN_MUTATION_COUNT_LOG <= 7;
};
/**
 * @brief Concept specifying the class used by the fuzzer
 *
 * @tparam T
 */
template <typename T> concept ArithmeticFuzzHelperConstraint = requires
{
    typename T::ArgSizes;
    typename T::Instruction;
    typename T::ExecutionState;
    typename T::ExecutionHandler;
    InstructionArgumentSizes<typename T::ArgSizes>;
    // HavocConfigConstraint<typename T::HavocConfig>;
};

/**
 * @brief Fuzzer uses only composers with check_circuit function
 *
 * @tparam T
 */
template <typename T> concept CheckableComposer = requires(T a)
{
    {
        a.check_circuit()
    }
    ->std::same_as<bool>;
};

/**
 * @brief The fuzzer can use a postprocessing function that is specific to the type being fuzzed
 *
 * @tparam T Type being tested
 * @tparam Composer
 * @tparam Context The class containing the full context
 */
template <typename T, typename Composer, typename Context>
concept PostProcessingEnabled = requires(Composer composer, Context context)
{
    {
        T::postProcess(&composer, context)
    }
    ->std::same_as<bool>;
};

/**
 * @brief This concept is used when we want to limit the number of executions of certain instructions (for example,
 * divisions and multiplications in bigfield start to bog down the fuzzer)
 *
 * @tparam T
 */
template <typename T> concept InstructionWeightsEnabled = requires
{
    typename T::InstructionWeights;
    T::InstructionWeights::_LIMIT;
};
/**
 * @brief A templated class containing most of the fuzzing logic for a generic Arithmetic class
 *
 * @tparam T
 */
template <typename T> requires ArithmeticFuzzHelperConstraint<T> class ArithmeticFuzzHelper {
  private:
    /**
     * @brief Mutator swapping two instructions together
     *
     * @param instructions
     * @param rng
     */
    inline static void swapTwoInstructions(std::vector<typename T::Instruction>& instructions, FastRandom& rng)
    {
        const size_t instructions_count = instructions.size();
        if (instructions_count <= 2) {
            return;
        }
        const size_t first_element_index = rng.next() % instructions_count;
        size_t second_element_index = rng.next() % instructions_count;
        if (first_element_index == second_element_index) {
            second_element_index = (second_element_index + 1) % instructions_count;
        }
        std::iter_swap(instructions.begin() + static_cast<int>(first_element_index),
                       instructions.begin() + static_cast<int>(second_element_index));
    }

    /**
     * @brief Mutator, deleting a sequence of instructions
     *
     * @param instructions
     * @param rng
     * @param havoc_settings
     */
    inline static void deleteInstructions(std::vector<typename T::Instruction>& instructions,
                                          FastRandom& rng,
                                          HavocSettings& havoc_settings)
    {

        const size_t instructions_count = instructions.size();
        if (instructions_count == 0) {
            return;
        }
        if (rng.next() & 1) {
            instructions.erase(instructions.begin() + (rng.next() % instructions_count));
        } else {
            // We get the logarithm of number of instructions and subtract 1 to delete at most half
            const size_t max_deletion_log =
                std::min(static_cast<size_t>(64 - __builtin_clzll(static_cast<uint64_t>(instructions_count)) - 1),
                         havoc_settings.ST_MUT_MAXIMUM_DELETION_LOG);

            if (max_deletion_log == 0) {
                return;
            }
            const size_t deletion_size = 1 << (rng.next() % max_deletion_log);
            const size_t start = rng.next() % (instructions_count + 1 - deletion_size);
            instructions.erase(instructions.begin() + static_cast<int>(start),
                               instructions.begin() + static_cast<int>(start + deletion_size));
        }
    }
    /**
     * @brief Mutator duplicating an instruction
     *
     * @param instructions
     * @param rng
     * @param havoc_settings
     */
    inline static void duplicateInstruction(std::vector<typename T::Instruction>& instructions,
                                            FastRandom& rng,
                                            HavocSettings& havoc_settings)
    {
        const size_t instructions_count = instructions.size();
        if (instructions_count == 0) {
            return;
        }
        const size_t duplication_size = 1 << (rng.next() % havoc_settings.ST_MUT_MAXIMUM_DUPLICATION_LOG);
        typename T::Instruction chosen_instruction = instructions[rng.next() % instructions_count];
        instructions.insert(
            instructions.begin() + (rng.next() % (instructions_count + 1)), duplication_size, chosen_instruction);
    }
    inline static void insertRandomInstruction(std::vector<typename T::Instruction>& instructions,
                                               FastRandom& rng,
                                               HavocSettings& havoc_settings)
    {
        (void)havoc_settings;
        instructions.insert(instructions.begin() + static_cast<int>(rng.next() % (instructions.size() + 1)),
                            T::Instruction::template generateRandom<FastRandom>(rng));
    }
    /**
     * @brief Mutator for instruction structure
     *
     * @param instructions
     * @param rng
     * @param havoc_settings
     */
    inline static void mutateInstructionStructure(std::vector<typename T::Instruction>& instructions,
                                                  FastRandom& rng,
                                                  HavocSettings& havoc_settings)
    {
        const size_t structural_mutators_count = havoc_settings.structural_mutation_distribution.size();
        const size_t prob_pool = havoc_settings.structural_mutation_distribution[structural_mutators_count - 1];
        const size_t choice = rng.next() % prob_pool;
        if (choice < havoc_settings.structural_mutation_distribution[0]) {
            deleteInstructions(instructions, rng, havoc_settings);
        } else if (choice < havoc_settings.structural_mutation_distribution[1]) {

            duplicateInstruction(instructions, rng, havoc_settings);
        } else if (choice < havoc_settings.structural_mutation_distribution[2]) {
            insertRandomInstruction(instructions, rng, havoc_settings);
        } else {

            swapTwoInstructions(instructions, rng);
        }
    }
    /**
     * @brief Choose a random instruction from the vector and mutate it
     *
     * @param instructions  Vector of instructions
     * @param rng Pseudorandom number generator
     * @param havoc_settings Mutation settings
     */
    inline static void mutateInstructionValue(std::vector<typename T::Instruction>& instructions,
                                              FastRandom& rng,
                                              HavocSettings& havoc_settings)
    {

        const size_t instructions_count = instructions.size();
        if (instructions_count == 0) {
            return;
        }
        const size_t chosen = rng.next() % instructions_count;
        instructions[chosen] =
            T::Instruction::template mutateInstruction<FastRandom>(instructions[chosen], rng, havoc_settings);
    }

    static void mutateInstructionVector(std::vector<typename T::Instruction>& instructions, FastRandom& rng)
    {
#ifdef HAVOC_TESTING
        // If we are testing which havoc settings are best, then we use global parameters
        const size_t mutation_count = 1 << fuzzer_havoc_settings.GEN_MUTATION_COUNT_LOG;
#else
        const size_t mutation_count = 1 << T::HavocConfig::MUTATION_COUNT_LOG;
        HavocSettings fuzzer_havoc_settings;
        // FILL the values
#endif
        for (size_t i = 0; i < mutation_count; i++) {
            uint32_t val = rng.next();
            if ((val % (fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY +
                        fuzzer_havoc_settings.GEN_VALUE_MUTATION_PROBABILITY)) <
                fuzzer_havoc_settings.GEN_STRUCTURAL_MUTATION_PROBABILITY) {
                // mutate structure
                mutateInstructionStructure(instructions, rng, fuzzer_havoc_settings);
            } else {
                // mutate a single instruction vector

                mutateInstructionValue(instructions, rng, fuzzer_havoc_settings);
            }
        }
    }

  public:
    /**
     * @brief Splice two instruction vectors into one randomly
     *
     * @param vecA First instruction vector
     * @param vecB Second instruction vector
     * @param rng PRNG
     * @return Resulting vector of instructions
     */
    static std::vector<typename T::Instruction> crossoverInstructionVector(
        const std::vector<typename T::Instruction>& vecA,
        const std::vector<typename T::Instruction>& vecB,
        FastRandom& rng)
    {
        // Get vector sizes
        const size_t vecA_size = vecA.size();
        const size_t vecB_size = vecB.size();
        // If one of them is empty, just return the other one
        if (vecA_size == 0) {
            return vecB;
        }
        if (vecB_size == 0) {
            return vecA;
        }
        std::vector<typename T::Instruction> result;
        // Choose the size of th resulting vector
        const size_t final_result_size = rng.next() % (vecA_size + vecB_size) + 1;
        size_t indexA = 0, indexB = 0;
        size_t* inIndex = &indexA;
        size_t inSize = vecA_size;
        auto inIterator = vecA.begin();
        size_t current_result_size = 0;
        bool currentlyUsingA = true;
        // What we do is basically pick a sequence from one, follow with a sequence from the other
        while (current_result_size < final_result_size && (indexA < vecA_size || indexB < vecB_size)) {
            // Get the size left
            size_t result_size_left = final_result_size - current_result_size;
            // If we can still read from this vector
            if (*inIndex < inSize) {
                // Get the size left in this vector and in the output vector and pick the lowest
                size_t inSizeLeft = inSize - *inIndex;
                size_t maxExtraSize = std::min(result_size_left, inSizeLeft);
                if (maxExtraSize != 0) {
                    // If not zero, get a random number of elements from input
                    size_t copySize = (rng.next() % maxExtraSize) + 1;
                    result.insert(result.begin() + static_cast<long>(current_result_size),
                                  inIterator + static_cast<long>((*inIndex)),

                                  inIterator + static_cast<long>((*inIndex) + copySize));
                    // Update indexes and sizes
                    *inIndex += copySize;
                    current_result_size += copySize;
                }
            }
            // Switch input vector
            inIndex = currentlyUsingA ? &indexB : &indexA;
            inSize = currentlyUsingA ? vecB_size : vecA_size;
            inIterator = currentlyUsingA ? vecB.begin() : vecA.begin();
            currentlyUsingA = !currentlyUsingA;
        }
        // Return spliced vector
        return result;
    }
    /**
     * @brief Parses a given data buffer into a vector of instructions for testing the arithmetic
     *
     * @param Data Pointer to the data buffer
     * @param Size Data buffer size
     * @return A vector of instructions
     */
    static std::vector<typename T::Instruction> parseDataIntoInstructions(const uint8_t* Data, size_t Size)
    {
        std::vector<typename T::Instruction> fuzzingInstructions;
        uint8_t* pData = (uint8_t*)Data;
        size_t size_left = Size;
        while (size_left != 0) {
            uint8_t chosen_operation = *pData;
            size_left -= 1;
            pData++;
            // If the opcode is enabled (exists and arguments' size is not -1), check if it's the right opcode. If it
            // is, parse it with a designated function
#define PARSE_OPCODE(name)                                                                                             \
    if constexpr (requires { T::ArgSizes::name; })                                                                     \
        if constexpr (T::ArgSizes::name != size_t(-1)) {                                                               \
            if (chosen_operation == T::Instruction::OPCODE::name) {                                                    \
                if (size_left < T::ArgSizes::name) {                                                                   \
                    return fuzzingInstructions;                                                                        \
                }                                                                                                      \
                fuzzingInstructions.push_back(                                                                         \
                    T::Parser::template parseInstructionArgs<T::Instruction::OPCODE::name>(pData));                    \
                size_left -= T::ArgSizes::name;                                                                        \
                pData += T::ArgSizes::name;                                                                            \
                continue;                                                                                              \
            }                                                                                                          \
        }
            // Create handlers for all opcodes that are in ArgsSizes
#define PARSE_ALL_OPCODES(...) FOR_EACH(PARSE_OPCODE, __VA_ARGS__)

            PARSE_ALL_OPCODES(ALL_POSSIBLE_OPCODES)
        }
        return fuzzingInstructions;
    }
    /**
     * @brief Write instructions into the buffer until there are no instructions left or there is no more space
     *
     * @param instructions Vector of fuzzing instructions
     * @param Data Pointer to data buffer
     * @param MaxSize Size of buffer
     * @return How much of the buffer was filled with instructions
     */
    static size_t writeInstructionsToBuffer(std::vector<typename T::Instruction>& instructions,
                                            uint8_t* Data,
                                            size_t MaxSize)
    {
        uint8_t* pData = Data;
        size_t size_left = MaxSize;
        for (auto& instruction : instructions) {
            // If the opcode is enabled and it's this opcode, use a designated function to serialize it
#define WRITE_OPCODE_IF(name)                                                                                          \
    if constexpr (requires { T::ArgSizes::name; })                                                                     \
        if constexpr (T::ArgSizes::name != (size_t)-1) {                                                               \
            if (instruction.id == T::Instruction::OPCODE::name) {                                                      \
                if (size_left >= (T::ArgSizes::name + 1)) {                                                            \
                    T::Parser::template writeInstruction<T::Instruction::OPCODE::name>(instruction, pData);            \
                    size_left -= (T::ArgSizes::name + 1);                                                              \
                    pData += (T::ArgSizes::name + 1);                                                                  \
                } else {                                                                                               \
                    return MaxSize - size_left;                                                                        \
                }                                                                                                      \
                continue;                                                                                              \
            }                                                                                                          \
        }
            // Create handlers for all opcodes that are in ArgsSizes
#define WRITE_ALL_OPCODES(...) FOR_EACH(WRITE_OPCODE_IF, __VA_ARGS__)

            WRITE_ALL_OPCODES(ALL_POSSIBLE_OPCODES)
        }
        return MaxSize - size_left;
    }

    /**
     * @brief Execute instructions in a loop
     *
     * @tparam Composer composer used
     * @param instructions
     */
    template <typename Composer>
    inline static void executeInstructions(
        std::vector<typename T::Instruction>& instructions) requires CheckableComposer<Composer>
    {
        typename T::ExecutionState state;
        Composer composer = Composer();
        circuit_should_fail = false;
        size_t total_instruction_weight = 0;
        (void)total_instruction_weight;
        for (auto& instruction : instructions) {
            // If instruction enabled and this is it, delegate to the handler
#define EXECUTE_OPCODE_IF(name)                                                                                        \
    if constexpr (requires { T::ArgSizes::name; })                                                                     \
        if constexpr (T::ArgSizes::name != size_t(-1)) {                                                               \
            if (instruction.id == T::Instruction::OPCODE::name) {                                                      \
                if constexpr (InstructionWeightsEnabled<T>) {                                                          \
                    if (!((total_instruction_weight + T::InstructionWeights::name) > T::InstructionWeights::_LIMIT)) { \
                        total_instruction_weight += T::InstructionWeights::name;                                       \
                        if (T::ExecutionHandler::execute_##name(&composer, state, instruction)) {                      \
                            return;                                                                                    \
                        }                                                                                              \
                    } else {                                                                                           \
                        return;                                                                                        \
                    }                                                                                                  \
                } else {                                                                                               \
                                                                                                                       \
                    if (T::ExecutionHandler::execute_##name(&composer, state, instruction)) {                          \
                        return;                                                                                        \
                    }                                                                                                  \
                }                                                                                                      \
            }                                                                                                          \
        }
#define EXECUTE_ALL_OPCODES(...) FOR_EACH(EXECUTE_OPCODE_IF, __VA_ARGS__)

            EXECUTE_ALL_OPCODES(ALL_POSSIBLE_OPCODES)
        }
        bool final_value_check = true;
        // If there is a posprocessing function, use it
        if constexpr (PostProcessingEnabled<T, Composer, std::vector<typename T::ExecutionHandler>>) {
            final_value_check = T::postProcess(&composer, state);
#ifdef SHOW_INFORMATION
            if (!final_value_check) {
                std::cerr << "Final value check failed" << std::endl;
            }
#endif
        }
        bool check_result = composer.check_circuit() && final_value_check;
        // If the circuit is correct, but it should fail, abort
        if (check_result && circuit_should_fail) {
            abort();
        }
        // If the circuit is incorrect, but there's no reason, abort
        if ((!check_result) && (!circuit_should_fail)) {
            if (!final_value_check) {
                std::cerr << "Final value check failed" << std::endl;
            } else {
                std::cerr << "Circuit failed" << std::endl;
            }

            abort();
        }
    }

    /**
     * @brief Interpret the data buffer as a series of arithmetic instructions and mutate it accordingly
     *
     * @param Data Pointer to the buffer
     * @param Size The initial filled size
     * @param MaxSize   The size of the buffer
     * @return size_t The new length of data in the buffer
     */
    static size_t MutateInstructionBuffer(uint8_t* Data, size_t Size, size_t MaxSize, FastRandom& rng)
    {
        // Parse the vector
        std::vector<typename T::Instruction> instructions = parseDataIntoInstructions(Data, Size);
        // Mutate the vector of instructions
        mutateInstructionVector(instructions, rng);
        // Serialize the vector of instructions back to buffer
        return writeInstructionsToBuffer(instructions, Data, MaxSize);
    }
};

template <template <typename> class Fuzzer, typename Composer>
constexpr void RunWithComposer(const uint8_t* Data, const size_t Size, FastRandom& VarianceRNG)
{
    VarianceRNG.reseed(0);
    auto instructions = ArithmeticFuzzHelper<Fuzzer<Composer>>::parseDataIntoInstructions(Data, Size);
    ArithmeticFuzzHelper<Fuzzer<Composer>>::template executeInstructions<Composer>(instructions);
}

template <template <typename> class Fuzzer, uint64_t Composers>
constexpr void RunWithComposers(const uint8_t* Data, const size_t Size, FastRandom& VarianceRNG)
{
    if (Composers & 1) {
        RunWithComposer<Fuzzer, waffle::StandardComposer>(Data, Size, VarianceRNG);
    }
    if (Composers & 2) {
        RunWithComposer<Fuzzer, waffle::TurboComposer>(Data, Size, VarianceRNG);
    }
}
