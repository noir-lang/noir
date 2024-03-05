barretenberg_module(smt_verification common proof_system stdlib_primitives stdlib_sha256 cvc5)

set(CVC5_INCLUDE $ENV{HOME}/cvc5/tmp-lib/include)
set(CVC5_LIB $ENV{HOME}/cvc5/tmp-lib/lib)

target_include_directories(smt_verification              PUBLIC ${CVC5_INCLUDE})
target_include_directories(smt_verification_objects      PUBLIC ${CVC5_INCLUDE})
target_include_directories(smt_verification_tests        PUBLIC ${CVC5_INCLUDE})
target_include_directories(smt_verification_test_objects PUBLIC ${CVC5_INCLUDE})

target_link_directories(smt_verification                 PUBLIC ${CVC5_LIB})
target_link_directories(smt_verification_objects         PUBLIC ${CVC5_LIB})
target_link_directories(smt_verification_tests           PUBLIC ${CVC5_LIB})
target_link_directories(smt_verification_test_objects    PUBLIC ${CVC5_LIB})