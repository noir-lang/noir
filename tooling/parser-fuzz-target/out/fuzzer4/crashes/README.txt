Command line used to find this crash:

/Users/michaelklein/.local/share/afl.rs/rustc-1.73.0-cc66ad4/afl.rs-0.15.4/afl/bin/afl-fuzz -c0 -S fuzzer4 -i in -o out ../../target/debug/parser-fuzz-target

If you can't reproduce a bug outside of afl-fuzz, be sure to set the same
memory limit. The limit used for this fuzzing session was 0 B.

Need a tool to minimize test cases before investigating the crashes or sending
them to a vendor? Check out the afl-tmin that comes with the fuzzer!

Found any cool bugs in open-source tools using afl-fuzz? If yes, please post
to https://github.com/AFLplusplus/AFLplusplus/issues/286 once the issues
 are fixed :)

