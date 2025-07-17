# Collection of bugs found in Noir compiler and standard library

This list is a collection of bugs. Its goal is to help developers, auditors and security researchers to find and fix bugs in the Noir compiler and standard library.

## List of bugs

### STDLIB

| NoirVD-STDLIB-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-STDLIB-001 | U128 | decode_ascii function didn't validate input bytes | Soundness | Overflow of 64-bit limbs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024 | [Description](decriptions/noir_stdlib.md) |
| NoirVD-STDLIB-002 | U128 | unconstrained_div function had infinite loop on division by zero | Completeness | Denial of service | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-STDLIB-003 | U128 | unconstrained_div function failed for large inputs | Completeness | Assertion failure for legitimate inputs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-STDLIB-004 | U128 | wrapping_mul function had a bug in high limb calculation | Soundness | Incorrect multiplication results | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-STDLIB-005 | schnorr | verify_signature_noir function had a bug with signature equal to zero | Soundness | Bypass signature check | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6218 | https://github.com/noir-lang/noir/pull/6226  |  |
| NoirVD-STDLIB-006 | schnorr | verify_signature_noir function had a bug in public key checking | Soundness | Bypass signature check | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6266 | https://github.com/noir-lang/noir/pull/6270  |  |
| NoirVD-STDLIB-007 | elliptic curves | Noir allows to use points not on the curve | Soundness | Invalid curve attack | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6217 | This issue will not be fixed  |  |

### Frontend

| NoirVD-Frontend-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-Frontend-001 | Frontend | Inconsistent handling of signed and negative numbers | Completeness | Unable to compile legit code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8023 https://github.com/noir-lang/noir/issues/8051  |    |  |


### SSA
| NoirVD-SSA-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-SSA-001 | SSA | incorrect radix decomposition | Soundness | Presented in issue | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6244 | https://github.com/noir-lang/noir/pull/6278  |  |
| NoirVD-SSA-002 | SSA remove_bit_shifts optimization | shr underflow | Completeness | Assertion failure for legitimate inputs | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7412 | https://github.com/noir-lang/noir/pull/7509  |  |
| NoirVD-SSA-003 | SSA brillig arrays | Overflow in indices | Completeness | Assertion failure for legitimate inputs | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7759 | https://github.com/noir-lang/noir/pull/7827  |  |
| NoirVD-SSA-004 | SSA fields truncation | Improper fields truncation | Completeness | Assertion failure for legitimate inputs, example in the issue | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7962| https://github.com/noir-lang/noir/pull/8010  |  |
| NoirVD-SSA-005 | SSA field modulo op | Possibility to take field mod field in Brillig | Soundness |  | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/8083 | https://github.com/noir-lang/noir/pull/8105  |  |
| NoirVD-SSA-006 | SSA passes | Out of Bounds Check | Completeness | Assertion failure only in Brillig | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/7975 https://github.com/noir-lang/noir/issues/7952 https://github.com/noir-lang/noir/issues/7965  | https://github.com/noir-lang/noir/issues/7995  |  |
| NoirVD-SSA-007 | SSA passes | Incorrect flattening of CFG | Soundness | ACIR and Brillig return different results | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/7961  | https://github.com/noir-lang/noir/issues/8040  |  |
| NoirVD-SSA-008 | SSA passes | Error in constant folding | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/7964  | https://github.com/noir-lang/noir/issues/8019  |  |
| NoirVD-SSA-009 | SSA passes | Incorrect handling of `i1` | Completeness | Assertion failure only in ACIR | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/7973 https://github.com/noir-lang/noir/issues/8198  | https://github.com/noir-lang/noir/issues/8072 https://github.com/noir-lang/noir/issues/8215  |  |
| NoirVD-SSA-010 | SSA passes | Negative loop bounds not handled | Completeness | Compiler crash on valid code  | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8009  | https://github.com/noir-lang/noir/issues/8094  |  |
| NoirVD-SSA-011 | SSA passes | Negative loop bounds skipped | Soundness | Invalid results | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8011  | https://github.com/noir-lang/noir/issues/8103  |  |
| NoirVD-SSA-012 | SSA passes | Shared Brillig entry points | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8069  | https://github.com/noir-lang/noir/issues/8099   |  |
| NoirVD-SSA-013 | SSA passes | Unrolling with `break` | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8073  | https://github.com/noir-lang/noir/issues/8090   |  |
| NoirVD-SSA-014 | SSA passes | Inlining with recursion | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8078 https://github.com/noir-lang/noir/issues/8081  | https://github.com/noir-lang/noir/issues/8127   |  |
| NoirVD-SSA-015 | SSA passes | Global instructions | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8174 https://github.com/noir-lang/noir/issues/8199  | https://github.com/noir-lang/noir/issues/8185 https://github.com/noir-lang/noir/issues/8200   |  |
| NoirVD-SSA-016 | SSA passes | Handling unused parameters | Completeness | Assertion failure only in Brillig | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8229 https://github.com/noir-lang/noir/issues/8230 https://github.com/noir-lang/noir/issues/8231 https://github.com/noir-lang/noir/issues/8233  | https://github.com/noir-lang/noir/issues/8239   |  |
| NoirVD-SSA-017 | SSA passes | Handling unused parameters | Soundness | ACIR and Brillig return different values | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8236  |   |  |
| NoirVD-SSA-018 | SSA passes | Global array ownership | Soundness | Brillig returns incorrect value | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8259  |   |  |
| NoirVD-SSA-019 | SSA passes | Handling side effects | Completeness | Assertion failure only in ACIR | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8261  |   |  |
| NoirVD-SSA-020 | SSA passes | Handling array offsets during optimization | Soundness | Both ACIR and Brillig return incorrect value | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8262  |   |  |

### ACIR Generation

| NoirVD-ACIR-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-ACIR-001 | ACIR generation | Casting `Field` to `u128` failed | Completeness | Compiler crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8024 https://github.com/noir-lang/noir/issues/8175  | https://github.com/noir-lang/noir/issues/8180    |  |
| NoirVD-ACIR-002 | ACIR generation | Handling constant overflows | Completeness | Compilation failure or crash on valid code | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8195 https://github.com/noir-lang/noir/issues/8272 https://github.com/noir-lang/noir/issues/8274 | https://github.com/noir-lang/noir/issues/8197 https://github.com/noir-lang/noir/issues/8294   |  |
| NoirVD-ACIR-003 | ACIR generation | Handling constant zero | Completeness | Assertion failure only in ACIR | ast_fuzzer | Yes | Yes | @aakoshh | https://github.com/noir-lang/noir/issues/8235  | https://github.com/noir-lang/noir/issues/8243   |  |

### Brillig

| NoirVD-Brillig-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-Brillig-001 | brillig | Brillig VM allowed division by zero for field operations | Soundness | Could break someone's ECDSA implementation | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/6266 | https://github.com/noir-lang/noir/pull/6270  |  |
