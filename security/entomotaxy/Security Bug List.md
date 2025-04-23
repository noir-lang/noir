# Collection of bugs found in Noir compiler and standard library

This list is a collection of bugs. Its goal is to help developers, auditors and security researchers to find and fix bugs in the Noir compiler and standard library.

## List of bugs

| NoirVD-ID | Component | Short description | Type | Potential Impact | Mechanism of finding | Found with a tool? | Found internally (yes or no) | Found by | Link to issue | Link to fix | Link to description |
|-----------|-----------|-------------------|------|------------------|----------------------|-------------------|------------------------------|-----------|--------------|------------|---------------------|
| NoirVD-001 | U128 | decode_ascii function didn't validate input bytes | Soundness | Overflow of 64-bit limbs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024 | [Description](security/entomotaxy/decriptions/noir_stdlib.md) |
| NoirVD-002 | U128 | unconstrained_div function had infinite loop on division by zero | Completeness | Denial of service | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](security/entomotaxy/decriptions/noir_stdlib.md) |
| NoirVD-003 | U128 | unconstrained_div function failed for large inputs | Completeness | Assertion failure for legitimate inputs | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](security/entomotaxy/decriptions/noir_stdlib.md) |
| NoirVD-004 | U128 | wrapping_mul function had a bug in high limb calculation | Soundness | Incorrect multiplication results | Manual code review | No | Yes | @Rumata888 | |https://github.com/noir-lang/noir/pull/5024    | [Description](security/entomotaxy/decriptions/noir_stdlib.md) |
