## C++ Standard for Aztec Circuits

### Sticking to the Standard

Read the standards and stick to them! There is some automation to help with this, but **the automation is far from comprehensive**.

Here are the types of automation that should help stick to the standard:
1. The VSCode workspace file `circuits.code-workspace` is configured to automatically format your code nicely when you save a C++ file.
    * It uses `cpp/.clang-format` for this
2. These workspace settings are also configured to warn you (with yellow squiggles) when something does not follow some of the rules.
    * It uses `cpp/.clangd` for this
3. A tidy check is run in CI
    * Job fails if your code would be changed by `./scripts/tidy.sh fix`
4. To perform some auto-tidying of your code, run `./scripts/tidy.sh fix` from `cpp/`
    * **Commit your code first** since tidying will occasionally mess up your code!
    * This will run `clang-tidy` on all C++ source files
        * It uses `cpp/.clang-tidy` * and formats tidied code with `cpp/.clang-format`
    * **Manually review any fixes to your code!**
        * If you disagree with an auto-fix or if it is buggy, use `// NOLINT...` ([more here](https://clang.llvm.org/extra/clang-tidy/#suppressing-undesired-diagnostics))
        * If you believe we should reject an entire class of tidy fixes, consider explicitly omitting from our checks or errors in `./clang-tidy`
            * Discuss with others first
    * _Note:_ tidying takes a while!
    * **You may need to run this multiple times!**
        * An error (with an auto-fix) in one round may prevent certain subsequent fixes
        * A fix in the one round may introduce more potential for fixes

### The Standard

1. **general**
    *  when something is not covered below, fall back to [Google's style guide](https://google.github.io/styleguide/cppguide.html)
    *  all TODOs should mention a username, email or bug number
        ```
        // TODO(dbanks12)
        // TODO(david@aztecprotocol.com)
        // TODO(bug 12345)
        ```
    *  if your editor warns you about a line of code fix it!
        * consider doing so even if you didn't write that code
1. **spacing**
    *  4 spaces except for access-specifiers (`public`/`protected`/`private`) which can use 2
    *  namespaces are not indented
    *  for continued indentation, just be sane
    *  remove trailing spaces at end of line (use a plugin)
    *  include a newline at the end of a file
        ```
        namespace my_namespace {
        class MyClass {
          public:
            // ... public stuff

          protected:
            // ... protected stuff

          private:
            // ... private stuff

            void my_private_function0(int arg0,
                                      int arg1)
            {
                // ...
            }

            void my_private_function1(
                int arg0,
                int arg1)
            {
                // ...
            }
        }
        } // namespace my_namespace

        ```
1. **braces**
    *  functions use curly brace alone on newline
    *  namespaces, classes, structs, enums, ifs, and loops use curely brace on same line
    * examples
        ```
        void my_function()
        {
            // ...
        }

        for (int i = 0; i < max; i++) {
            // ...
        }

        if (something_is_true) {
            // ...
        }

        struct MyStruct {
            // ...
        }
        ```
1.  **naming**
    *  `snake_case` for files, namespaces and local variables/members
    *  `CamelCase` for classes, structs, enums, types
        *  exceptions types can be made for types if trying to mimic a std type's name like `uint` or `field_ct`
    *  `ALL_CAPS` for `constexpr`s, global constants, and macros
    *  do use
        *  Clear names
        *  Descriptive names
    *  don't use
        *  abbreviations
        *  words with letters removed
        *  acronyms
        *  single letter names
            *  Unless writing maths, in which case use your best judgement and follow the naming of a _linked_ paper
1.  `auto`
    *  include `*`, `&`, and/or `const` even when using `auto`
    *  use when type is evident
    *  use when type should be deduced automatically based on expr
    *  use in loops to iterate over members of a container
    *  don't use if it makes type unclear
    *  don't use you need to enforce a type
    * examples
        ```
        auto my_var = my_function_with_unclear_return_type(); // BAD
        auto my_var = get_new_of_type_a(); // GOOD
        ```
1.  `const` and `constexpr`
    *  use `const` whenever possible to express immutability
    *  use `constexpr` whenever a `const` can be computed at compile-time
    *  place `const`/`constexpr` BEFORE the core type as is done in bberg stdlib
    * examples
        ```
        const int my_const = 0;
        constexpr int MY_CONST = 0;
        ```
1.  `namespace` and `using`
    *  never do `using namespace my_namespace;` which causes namespace pollution and reduces readability
        *  [see here for google's corresponding rule](https://clang.llvm.org/extra/clang-tidy/checks/google/build-using-namespace.html)
    *  avoid doing `typedef my::OldType NewType` and instead do `using NewType = my::OldType`
    *  namespaces should exactly match directory structure. If you create a nested namespace, create a nested directory for it
    *  example for directory `aztec3/circuits/abis/private_kernel`:
        ```
        namespace aztec3::circuits::abis::private_kernel {
            // ...
        } // namespace aztec3::circuits::abis::private_kernel
        ```
    *  use`init.hpp` *only* for core/critical renames like `NT/CT` and for toggling core types like `CircuitBuilder`
    *  use unnamed/anonymous namespaces to import and shorten external names into *just this one file*
        *  all of a file's external imports belong in a single anonymous namespace `namespace { ...\n } // namespace` at the very top of the file directly after `#include`s
        *  use `using Rename = old::namespace::prefix::Name;` to import and shorten names from external namespaces
        *  avoid using renames to obscure template params (`using A = A<NT>;`)
        *  never use renames to remove the `std::` prefix
        *  never use renames to remove a `NT::` or `CT::` prefix
    *  `test.cpp` tests must always explicitly import every single name they intend to use
        *  they might want to test over multiple namespaces, native and circuit types, and builder types
    *  avoid calling barretenberg's functions directly and instead go through interface files like `circuit_types` and 
`native_types`
    *  `using` statements should be sorted case according to the `LexicographicNumeric` rules
        *   see the `SortUsingDeclarations` section of the [LLVM Clang Format Style Options document](https://clang.llvm.org/docs/ClangFormatStyleOptions.html)
    *  if your IDE is telling you that an include or name is unused in a file, remove it!
1.  **includes**
    * start every header with `#pragma once`
    * `index.hpp` should include common headers that will be referenced by most cpp/hpp files in the current directory
    * `init.hpp` should inject ONLY critical renames (like `NT`/`CT`) and type toggles (like CircuitBuilder)
        * example `using NT = aztec3::utils::types::NativeTypes;`
    *  avoid including headers via relative paths (`../../other_dir`) unless they are a subdir (`subdir/header.hpp`)
        *  use full path like `aztec3/circuits/hash.hpp`
    * ordering of includes
        * this source file's header
        * essentials (if present)
          * `"index.hpp"`
          * `"init.hpp"`
        * headers nearby
            * headers from this directory (no `/`)
            * headers from this project using relative path (`"private/private_kernel_inputs.hpp"`)
            * Note: headers in this group are sorted in the above order (no `/` first, relative paths second)
        * headers from this project using full path (starts with aztec3: `"aztec3/constants.hpp"`)
        * barretenberg headers
        * `<gtest>` or other third party headers specified in `.clang-format`
        * C++ standard library headers
    * use quotes internal headers
    * use angle braces for std library and external library headers
        * this includes barretenberg
    * each group of includes should be sorted case sensitive alphabetically
    * each group of includes should be newline-separated
    * example:
        ```
        #include "this_file.hpp"

        #include "index.hpp"
        #include "init.hpp"

        #include "my_file_a_in_this_dir.hpp"
        #include "my_file_b_in_this_dir.hpp"
        #include "other_dir/file_a_nearby.hpp"
        #include "other_dir/file_b_nearby.hpp"

        #include "aztec3/file_a_in_project.hpp"
        #include "aztec3/file_b_in_project.hpp"

        #include <barretenberg/file_a.hpp>
        #include <barretenberg/file_b.hpp>

        #include <gtest>

        #include <vector>
        #include <iostream>
        ``` 
1.  **access specifiers**
    *  order them `public`, `protected`, `private`
1.  **struct and array initialization**
    *  use `MyStruct my_inst{};`
        *  will call default constructor if exists and otherwise will value-initialize members to zero (or will call *their* default constructors if they exist)
    *  explicitly initialize struct members with default values: `NT::fr my_fr = 0;`
    *  initialize arrays using `std::array<T, 8> my_arr{};`
        *  this value-initializes all entries to 0 if T has no default constructor, otherwise calls default constructor for each
    *  For arrays of fields `fr`, it is particularly important to use the direct initialization form with empty initializer list
     `
     std::array<fr, VK_TREE_HEIGHT> vk_path{};
     `
     because the default constructor of `fr` is NOT initializing the array field values to 0. For large arrays and performance considerations, it may be useful to not initialize the field array elements using `std::array<fr, VK_TREE_HEIGHT> vk_path();`
1.  **references**
    *  use them whenever possible for function arguments since pass by reference is cheaper
    *  make arg references "const" if they should not be modified inside a function
1.  **avoid C-style coding**
    *  avoid `malloc/free`
    *  use `std::array/vector` instead of `int[]`
    *  use references instead of pointers when possible
    *  if pointers are necessary, use smart pointers (`std::unique_ptr/shared_ptr`) instead of raw pointers
    *  avoid C-style casts (use `static_cast` or `reinterpret_cast`)
1.  **comments**
    * use doxygen docstrings (will include format example)
        ```
        /**
         * @brief Brief description
         * @details more details
         * @tparam mytemplateparam description
         * @param myfunctionarg description
         * @return describe return value
         * @see otherRelevantFunction()
         * @see [mylink](url)
         */
        ```
    *  every file should have a meaningful comment
    *  every class/struct/function/test should have a meaningful comment
        *  class/struct comment might == file comment
    *  comment function preconditions ("arg x must be < 100")
1.  **side-effects**
    *  avoid functions with side effects when it is easy enough to just have pure functions
    *  if a function modifies its arguments, it should be made very clear that this is happening
        *  same with class methods that modify members
    *  function arguments should be `const` when they will not be modified
1. **global state**
    * no
1.  **docs**
    *  every subdir should have a readme
1.  **functions**
    *  use `[[nodiscard]]` if it makes no sense to call this function and discard return value
        ```
        [[nodiscard] int my_function()
        {
            // ...
            return some_int;
        }

        // later can't do
        my_function();

        // can only do
        int capture_ret = my_function();
        ```
    *  if there is a name clash, prefix parameter with underscore like `_myparam`
        ```
        void my_function(int _my_var)
        {
            my_var = _my_var;
        }
        ```
1.  **macros**
    *  avoid macros as much as possible with exceptions for
        *  testing
        *  debug utilities
        *  agreed upon macro infrastructure (like cbinds)
1.  **misc**
    *  use `uintN_t` instead of a primitive type (e.g. `size_t`) when a specific type width must be guaranteed
    *  avoid signed types (`int`, `long`, `char` etc) unless signedness is required
        *  signed types are susceptible to undefined behavior on overflow/underflow
    *  initialize pointers to `nullptr`
    *  constructors with single arguments should be marked `explicit` to prevent unwanted conversions
    *  if a constructor is meant to do nothing, do `A() = default;` instead of `A(){}` ([explanation here](https://clang.llvm.org/extra/clang-tidy/checks/modernize/use-equals-default.html))
        *  definitely don't do `A(){};` (with semicolon) which can't even be auto-fixed by `clang-tidy`
    *  explicitly use `override` when overriding a parent class' member ([explanation here](https://clang.llvm.org/extra/clang-tidy/checks/modernize/use-override.html))
    *  avoid multiple declarations on the same line
        *  do:
            ```
            int a = 0;
            int b = 0;
            ```
        *  dont:
            ```
            int a, b = 0, 0;
            ```
    *  use `std::vector::emplace_back` instead of `push_back`
        *  don't use `std::make_pair` when using `emplace_back` ([unnecessary as explained here](https://clang.llvm.org/extra/clang-tidy/checks/modernize/use-emplace.html))
    *  **no magic numbers** even if there is a comment explaining them


## References

1. [Mike's Draft C++ Standard](https://hackmd.io/@aztec-network/B1r36lhmj?type=view)
2. [Barretenberg's `.clangd`](https://github.com/AztecProtocol/barretenberg/blob/master/cpp/.clangd)
3. [Barretenberg's `.clang-format`](https://github.com/AztecProtocol/barretenberg/blob/master/cpp/.clang-format)
4. [LLVM's Clang Format Style Options](https://clang.llvm.org/docs/ClangFormatStyleOptions.html)
5. [Google's Style Guide](https://google.github.io/styleguide/cppguide.html)