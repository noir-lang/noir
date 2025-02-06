/*
    ## Creating a Noir Library

    Noir libraries let you encapsulate reusable code that can be shared across multiple Noir projects.
    Follow these steps to create your own library.

    ### 1. Project Setup

    Create a new project with a `Nargo.toml` file at the root.
    In this file, specify that your project is a library by setting the type to `lib` under the `[lib]` section.
    For example:

    ```toml
    [package]
    name = "my_noir_library"
    version = "0.1.0"
    authors = ["Your Name <you@example.com>"]
    edition = "2021"

    [lib]
    type = "lib"
    ```

    This tells Nargo (Noir's build tool) to compile your project as a library rather than an executable.

    ### 2. Create the Library File

    Add a source file named `lib.nr` at the root of your project.
    This file is the entry point for your library code.

    #### Example `lib.nr`:

    // A public function that adds two Field values.
    // This function is available to any project that imports this library.
    pub fn add(a: Field, b: Field) -> Field {
        a + b
    }

    // A public structure example.
    pub struct MyStruct {
        pub value: Field,
    }

    // A private helper function used only within this library.
    fn helper_multiply(x: Field) -> Field {
        x * 2
    }

    **Public Items:** Use the `pub` keyword to expose functions, types, or variables to users of your library.
    **Private Items:** Omit `pub` for items that should remain internal.

    ### 3. Build Your Library

    With `Nargo.toml` and `lib.nr` in place, build your library by running:

    ```bash
    nargo build
    ```

    This command compiles your Noir library, preparing it for use as a dependency in other projects.

    ### 4. Importing Your Library in Other Projects

    To use your library in another Noir project, add it as a dependency in the consuming project's `Nargo.toml`.
    For example, if your library is stored locally:

    ```toml
    [dependencies]
    my_noir_library = { path = "../my_noir_library" }
    ```

    Then, in your Noir source file, import and use your library's public items:

    // Import the 'add' function from your library.
    use my_noir_library::add;

    fn main() {
        let result = add(5, 10);
        // Continue with your logic...
    }

    ## Additional Considerations

    - **Documentation:** Provide inline comments and external docs to explain usage of your library's functions and types.
    - **Testing:** Consider adding tests in a dedicated directory (e.g., `tests/`) using Noir test cases.
    - **Versioning and Publishing:** Use semantic versioning to manage updates. When ready, publish your library to a repository or registry.
*/

/* --- Library Code --- */

// Public function: adds two Field elements.
pub fn add(a: Field, b: Field) -> Field {
    a + b
}

// Public structure example.
pub struct MyStruct {
    pub value: Field,
}

// Private helper function: multiplies a Field by 2.
fn helper_multiply(x: Field) -> Field {
    x * 2
}
