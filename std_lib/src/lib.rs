pub const LIB_NOIR : &'static str = include_str!("lib.nr");

// This lib.rs file is here so that we can leverage the Rust build system
// The contents of src except for Rust files will be copied to a config directory.
// When compiling a Project, the compiler will look for the standard library in this config directory