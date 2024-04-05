
```bash
AFL_AUTORESUME=1 ccargo afl fuzz -M main-$HOSTNAME -i in -o out ../../target/debug/parser-fuzz-target
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer9 -i in -o out ../../target/debug/parser-fuzz-target 
AFL_AUTORESUME=1 ccargo afl fuzz -S fuzzer10 -i in -o out ../../target/debug/parser-fuzz-target 
```

