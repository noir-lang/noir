# Collection of bugs found in Noir compiler and standard library

This list is a collection of bugs. Its goal is to help developers, auditors and security researchers to find and fix bugs in the Noir compiler and standard library.

## List of bugs

| NoirVD-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-001 | U128 | decode_ascii function didn't validate input bytes | Soundness | Overflow of 64-bit limbs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024 | [Description](decriptions/noir_stdlib.md) |
| NoirVD-002 | U128 | unconstrained_div function had infinite loop on division by zero | Completeness | Denial of service | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-003 | U128 | unconstrained_div function failed for large inputs | Completeness | Assertion failure for legitimate inputs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-004 | U128 | wrapping_mul function had a bug in high limb calculation | Soundness | Incorrect multiplication results | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](decriptions/noir_stdlib.md) |
| NoirVD-005 | schnorr | verify_signature_noir function had a bug with signature equal to zero | Soundness | Bypass signature check | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6218 | https://github.com/noir-lang/noir/pull/6226  |  |
| NoirVD-006 | SSA | incorrect radix decomposition | Soundness | Presented in issue | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6244 | https://github.com/noir-lang/noir/pull/6278  |  |
| NoirVD-007 | schnorr | verify_signature_noir function had a bug in public key checking | Soundness | Bypass signature check | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6266 | https://github.com/noir-lang/noir/pull/6270  |  |
| NoirVD-008 | brillig | Brillig VM allowed division by zero for field operations | Soundness | Could break someone's ECDSA implementation | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/6266 | https://github.com/noir-lang/noir/pull/6270  |  |
| NoirVD-009 | elliptic curves | Noir allows to use points not on the curve | Soundness | Invalid curve attack | Manual code review | No | Yes | @defkit | https://github.com/noir-lang/noir/issues/6217 | This issue will not be fixed  |  |
| NoirVD-010 | SSA remove_bit_shifts optimization | shr underflow | Completeness | Assertion failure for legitimate inputs | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7412 | https://github.com/noir-lang/noir/pull/7509  |  |
| NoirVD-011 | SSA brillig arrays | Overflow in indices | Completeness | Assertion failure for legitimate inputs | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7759 | https://github.com/noir-lang/noir/pull/7827  |  |
| NoirVD-012 | SSA fields truncation | Improper fields truncation | Completeness | Assertion failure for legitimate inputs, example in the issue | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/7962| https://github.com/noir-lang/noir/pull/8010  |  |
| NoirVD-013 | SSA field modulo op | Possibility to take field mod field in Brillig | Soundness |  | ssa_fuzzer | Yes | Yes | @defkit | https://github.com/noir-lang/noir/issues/8083 | https://github.com/noir-lang/noir/pull/8105  |  |
|  | SSA passes | Out of Bounds Check false failure | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #7975 #7952 #7965  | #7995  |  |
|  | SSA passes | Incorrect flattening of CFG | Soundness |  | ast_fuzzer | Yes | Yes | @aakoshh | #7961  | #8040  |  |
|  | SSA passes | Error in constant folding | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #7964  | #8019  |  |
|  | SSA passes | Incorrect handling of `i1` | Soundness |  | ast_fuzzer | Yes | Yes | @aakoshh | #7973 #8198  | #8072 #8215  |  |
|  | SSA passes | Negative loop bounds weren't handled | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8009 #8011  | #8094 #8103  |  |
|  | ACIR generation | Casting `Field` to `u128` failed | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8024 #8175  | #8180    |  |
|  | Frontend | Inconsistent handling of signed and negative numbers | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8023 #8051  |    |  |
|  | SSA passes | Shared Brillig entry points | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8069  | #8099   |  |
|  | SSA passes | Unrolling with `break` | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8073  | #8090   |  |
|  | SSA passes | Inlining with recursion | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8078 #8081  | #8127   |  |
|  | SSA passes | Global instructions | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8174 #8199  | #8185 #8200   |  |
|  | ACIR generation | Handling constant overflows | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8195  | #8197   |  |
|  | SSA passes | Handling unused parameters; constraint failure | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8229 #8230 #8231 #8233  | #8239   |  |
|  | SSA passes | Handling unused parameters; incorrect return value | Soundness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8236  |   |  |
|  | ACIR generation | Handling constant zero | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8235  | #8243   |  |
|  | SSA passes | Global array ownership | Soundness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8259  |   |  |
|  | SSA passes | Handling side effects; constraint failure | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8259  |   |  |
|  | SSA passes | Array offsets; incorrect return value | Soundness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8262  |   |  |
|  | ACIR generation | Handling negative constants; constraint failure | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8272  |   |  |
|  | ACIR generation | Handling constant overflows | Completeness |  | ast_fuzzer | Yes | Yes | @aakoshh | #8274  |   |  |