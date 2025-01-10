# Changelog

## [1.0.0-beta.1](https://github.com/noir-lang/noir/compare/v1.0.0-beta.0...v1.0.0-beta.1) (2024-12-20)


### ⚠ BREAKING CHANGES

* **stdlib:** Remove Schnorr ([#6749](https://github.com/noir-lang/noir/issues/6749))
* remove SchnorrVerify opcode (https://github.com/AztecProtocol/aztec-packages/pull/9897)
* several format string fixes and improvements ([#6703](https://github.com/noir-lang/noir/issues/6703))
* remove `ec` module from stdlib ([#6612](https://github.com/noir-lang/noir/issues/6612))
* Disallow `#[export]` on associated methods ([#6626](https://github.com/noir-lang/noir/issues/6626))

### Features

* `nargo test -q` (or `nargo test --format terse`) ([#6776](https://github.com/noir-lang/noir/issues/6776)) ([919149d](https://github.com/noir-lang/noir/commit/919149d3413be5232b33611094687fdb5fd86086))
* `std::hint::black_box` function. ([#6529](https://github.com/noir-lang/noir/issues/6529)) ([237e8fa](https://github.com/noir-lang/noir/commit/237e8fa9ef8621e3da472e64c649849b4281004c))
* Add `(x | 1)` optimization for booleans ([#6795](https://github.com/noir-lang/noir/issues/6795)) ([feec2a1](https://github.com/noir-lang/noir/commit/feec2a1a9851306d7cf682c796ab084dea10b2ec))
* Add `BoundedVec::from_parts` and `BoundedVec::from_parts_unchecked` ([#6691](https://github.com/noir-lang/noir/issues/6691)) ([768aa7c](https://github.com/noir-lang/noir/commit/768aa7ce9ed809fa2c9d368f2b2f625d9689a63f))
* Add `nargo test --format json` ([#6796](https://github.com/noir-lang/noir/issues/6796)) ([eb975ab](https://github.com/noir-lang/noir/commit/eb975ab28fb056cf92859377c02f2bb1a608eda3))
* Add a warning when using unsafe blocks without safety comments ([#6860](https://github.com/noir-lang/noir/issues/6860)) ([5c00a79](https://github.com/noir-lang/noir/commit/5c00a79d2c93056d07330c350bf7b6efbf81d477))
* Add memory report into the CI ([#6630](https://github.com/noir-lang/noir/issues/6630)) ([6acef6d](https://github.com/noir-lang/noir/commit/6acef6d795cd74dee4a21e82c5a912d58b40b06c))
* Allow filtering which SSA passes are printed ([#6636](https://github.com/noir-lang/noir/issues/6636)) ([50f4aa7](https://github.com/noir-lang/noir/commit/50f4aa72e409e7205724f90046d394bb83584e9c))
* Allow ignoring test failures from foreign calls ([#6660](https://github.com/noir-lang/noir/issues/6660)) ([e3a0914](https://github.com/noir-lang/noir/commit/e3a0914c6dede6f54f426ed7d790a0c98a7e0908))
* Better error message when trying to invoke struct function field ([#6661](https://github.com/noir-lang/noir/issues/6661)) ([ea7c04a](https://github.com/noir-lang/noir/commit/ea7c04a8410ed8d2ce8e5a27e3c0784ba3195638))
* **ci:** Initial compilation report on test_programs ([#6731](https://github.com/noir-lang/noir/issues/6731)) ([b3c04f0](https://github.com/noir-lang/noir/commit/b3c04f02d467c71ac9cb5eb6eca20b5bc0a2e47d))
* **cli:** Run command on the package closest to the current directory ([#6752](https://github.com/noir-lang/noir/issues/6752)) ([f45b354](https://github.com/noir-lang/noir/commit/f45b3546bf82fd35eb446f7cbf00b739f287b92a))
* **cli:** Verify `return` against ABI and `Prover.toml` ([#6765](https://github.com/noir-lang/noir/issues/6765)) ([5795a09](https://github.com/noir-lang/noir/commit/5795a099657a268b735a539298dfeefa445db3ff))
* **comptime:** Implement blackbox functions in comptime interpreter ([#6551](https://github.com/noir-lang/noir/issues/6551)) ([10a9f81](https://github.com/noir-lang/noir/commit/10a9f8104e1ebb8fad044927ff130a0e2ce9131b))
* Configurable external check failures ([#6810](https://github.com/noir-lang/noir/issues/6810)) ([73ccd45](https://github.com/noir-lang/noir/commit/73ccd45590222fc82642a6a9aa657c2915fc2c58))
* Flatten nested if-else statements with equivalent conditions ([#6875](https://github.com/noir-lang/noir/issues/6875)) ([1a0a5f6](https://github.com/noir-lang/noir/commit/1a0a5f61231c3b65fa6397ec0769d3e6c5c238a3))
* Improve parser recovery of constructor field with '::' instead of ':' ([#6701](https://github.com/noir-lang/noir/issues/6701)) ([c400543](https://github.com/noir-lang/noir/commit/c400543a9d9747798fd3b27b8508ac0a0668a09c))
* Order attribute execution by their source ordering ([#6326](https://github.com/noir-lang/noir/issues/6326)) ([852155d](https://github.com/noir-lang/noir/commit/852155dc1c4a910bf9cd4e7af334f3856c1c4643))
* **perf:** Track last loads per block in mem2reg and remove them if possible ([#6088](https://github.com/noir-lang/noir/issues/6088)) ([624ae6c](https://github.com/noir-lang/noir/commit/624ae6c6f0fdb077533abf93f8ff94814a7394f4))
* Reduce memory consumption by storing array length as `u32` during SSA ([#6606](https://github.com/noir-lang/noir/issues/6606)) ([6196d05](https://github.com/noir-lang/noir/commit/6196d05bcb7ecd0b84fcc5ccc20d8dab99bc8052))
* Replace `eval_global_as_array_length` with type/interpreter evaluation ([#6469](https://github.com/noir-lang/noir/issues/6469)) ([ddb4673](https://github.com/noir-lang/noir/commit/ddb46733fcf596b5c8508a208b2690df52aa16e3))
* Replace quadratic removal of `rc` instructions ([#6705](https://github.com/noir-lang/noir/issues/6705)) ([7619da5](https://github.com/noir-lang/noir/commit/7619da59fc34cdd6e3b2581ad1668b5131ba4dde))
* Replace quadratic removal of rc instructions (https://github.com/AztecProtocol/aztec-packages/pull/10416) ([66d3275](https://github.com/noir-lang/noir/commit/66d32751311378701b075ee7b2106d61e531ae4f))
* Revert changes to `ValueMerger` and `Instruction::IfElse` ([#6673](https://github.com/noir-lang/noir/issues/6673)) ([f81244c](https://github.com/noir-lang/noir/commit/f81244c6bb29e8869f489d536141eebf6f68f00a))
* Several `nargo test` improvements ([#6728](https://github.com/noir-lang/noir/issues/6728)) ([1b0dd41](https://github.com/noir-lang/noir/commit/1b0dd4149d9249f0ea4fb5e2228c688e0135618f))
* Show printable byte arrays as byte strings in SSA ([#6709](https://github.com/noir-lang/noir/issues/6709)) ([fc11b63](https://github.com/noir-lang/noir/commit/fc11b631a2a1c0054b3b2a9e9fd2b7fa3a285076))
* Simplify `jmpif`s by reversing branches if condition is negated ([#5891](https://github.com/noir-lang/noir/issues/5891)) ([ba7a568](https://github.com/noir-lang/noir/commit/ba7a568430c3477cc39e0ec147b11bdfc95093de))
* **ssa:** Bring back tracking of RC instructions during DIE ([#6783](https://github.com/noir-lang/noir/issues/6783)) ([bc03152](https://github.com/noir-lang/noir/commit/bc03152366f242a6976f6e006e12520989e5e112))
* **ssa:** Deduplicate intrinsics with predicates ([#6615](https://github.com/noir-lang/noir/issues/6615)) ([53f16c7](https://github.com/noir-lang/noir/commit/53f16c7fe75da04c54517b3d3199094b15195ce4))
* **ssa:** Hoist MakeArray instructions during loop invariant code motion  ([#6782](https://github.com/noir-lang/noir/issues/6782)) ([b88db67](https://github.com/noir-lang/noir/commit/b88db67a4fa92f861329105fb732a7b1309620fe))
* **ssa:** Hoisting of array get using known induction variable maximum ([#6639](https://github.com/noir-lang/noir/issues/6639)) ([26d2351](https://github.com/noir-lang/noir/commit/26d235198f9a2fedbe438b3f7b39184554c5e1c1))
* **ssa:** Implement missing brillig constraints SSA check ([#6658](https://github.com/noir-lang/noir/issues/6658)) ([c5a4caf](https://github.com/noir-lang/noir/commit/c5a4caf4e3971b8e9cb73681dcb21db0ba5550fc))
* **ssa:** Option to set the maximum acceptable Brillig bytecode increase in unrolling ([#6641](https://github.com/noir-lang/noir/issues/6641)) ([4ff3081](https://github.com/noir-lang/noir/commit/4ff308128755c95b4d461bbcb7e3a49f16145585))
* **ssa:** Simplify array get from set that writes to the same dynamic index ([#6684](https://github.com/noir-lang/noir/issues/6684)) ([304403f](https://github.com/noir-lang/noir/commit/304403f24e2a15b57bb054c4402a8d7f8d275668))
* Sync from aztec-packages ([#6634](https://github.com/noir-lang/noir/issues/6634)) ([aa143a7](https://github.com/noir-lang/noir/commit/aa143a75d3460785ed88ea7ab3337c880c1153fd))
* Sync from aztec-packages ([#6656](https://github.com/noir-lang/noir/issues/6656)) ([594aad2](https://github.com/noir-lang/noir/commit/594aad21f30614b1733a3ba2b8a2a5f5d7b7e119))
* Sync from aztec-packages ([#6824](https://github.com/noir-lang/noir/issues/6824)) ([b3bca76](https://github.com/noir-lang/noir/commit/b3bca76620229e32c531417c6fa92e4a2c044fa0))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/10307) ([66d3275](https://github.com/noir-lang/noir/commit/66d32751311378701b075ee7b2106d61e531ae4f))
* **test:** Check that `nargo::ops::transform_program` is idempotent ([#6694](https://github.com/noir-lang/noir/issues/6694)) ([9f3e0e6](https://github.com/noir-lang/noir/commit/9f3e0e68f8bc179fbceb606598a9749d3dc1c9e3))
* **tooling:** Skip program transformation when loaded from cache ([#6689](https://github.com/noir-lang/noir/issues/6689)) ([7feb658](https://github.com/noir-lang/noir/commit/7feb6589d98246400497dfa492f39d67bae85977))
* Warn on unnecessary unsafe blocks ([#6867](https://github.com/noir-lang/noir/issues/6867)) ([a97402a](https://github.com/noir-lang/noir/commit/a97402aa3b3a1a7d40f61242e02d3772175846f2))


### Bug Fixes

* Allow empty loop headers ([#6736](https://github.com/noir-lang/noir/issues/6736)) ([332ba79](https://github.com/noir-lang/noir/commit/332ba790287b152f14af8c88d0349323287e59bf))
* Allow multiple `_` parameters, and disallow `_` as an expression you can read from ([#6657](https://github.com/noir-lang/noir/issues/6657)) ([d80a9d7](https://github.com/noir-lang/noir/commit/d80a9d71a8e4081a3aeacf34b2d5c2b0faf87484))
* Always return an array of `u8`s when simplifying `Intrinsic::ToRadix` calls ([#6663](https://github.com/noir-lang/noir/issues/6663)) ([59c0c35](https://github.com/noir-lang/noir/commit/59c0c3562ad75d6c10fa7c6a2f74e3fbf68ec3e6))
* Correct signed integer handling in `noirc_abi` ([#6638](https://github.com/noir-lang/noir/issues/6638)) ([ecaf63d](https://github.com/noir-lang/noir/commit/ecaf63da19a3a76e7c2940721d9044b1980a588f))
* Correct types returned by constant EC operations simplified within SSA ([#6652](https://github.com/noir-lang/noir/issues/6652)) ([eec5970](https://github.com/noir-lang/noir/commit/eec5970658157704dac5c41e6d61b2aa652ce996))
* Detect cycles in globals ([#6859](https://github.com/noir-lang/noir/issues/6859)) ([0d7642c](https://github.com/noir-lang/noir/commit/0d7642cb2071fbee148978a89a0922bfffe5be6a))
* Disable failure persistance in nargo test fuzzing ([#6777](https://github.com/noir-lang/noir/issues/6777)) ([68ff7bd](https://github.com/noir-lang/noir/commit/68ff7bd85cb17f57afd481ec55318ec282a93aa6))
* Disallow `#[export]` on associated methods ([#6626](https://github.com/noir-lang/noir/issues/6626)) ([7b56904](https://github.com/noir-lang/noir/commit/7b56904e56d95b88cefcbf3862e822fd3b1c8730))
* Do not merge expressions that contain output witnesses ([#6757](https://github.com/noir-lang/noir/issues/6757)) ([f9abf72](https://github.com/noir-lang/noir/commit/f9abf724abd674ea4ccb342a770d237c70864ee1))
* Do not warn on unused functions marked with #[export] ([#6625](https://github.com/noir-lang/noir/issues/6625)) ([30f8378](https://github.com/noir-lang/noir/commit/30f8378525b6f8ee305d356388c32761e12ee61c))
* Don't deduplicate binary math of unsigned types ([#6848](https://github.com/noir-lang/noir/issues/6848)) ([ee0754b](https://github.com/noir-lang/noir/commit/ee0754b1c6b36961c180901db59dd593c183de77))
* Don't remove necessary RC instructions in DIE pass ([#6585](https://github.com/noir-lang/noir/issues/6585)) ([440d94d](https://github.com/noir-lang/noir/commit/440d94d8149ede5f211437e9405f65b460cfcbf8))
* Double alias in path ([#6855](https://github.com/noir-lang/noir/issues/6855)) ([82f595b](https://github.com/noir-lang/noir/commit/82f595b960a8fd54bcb5d2a76ea304af5782509b))
* Git dependency trailing slash ([#6725](https://github.com/noir-lang/noir/issues/6725)) ([df71df7](https://github.com/noir-lang/noir/commit/df71df7e875b40529aa9404d45fd391a8857568a))
* Implement `as_field` and `from_field` in the interpreter ([#6829](https://github.com/noir-lang/noir/issues/6829)) ([f037c36](https://github.com/noir-lang/noir/commit/f037c36f6bfcb8efb1950e444b50bc3eff28ffc4))
* Improve type error when indexing a variable of unknown type ([#6744](https://github.com/noir-lang/noir/issues/6744)) ([909b22b](https://github.com/noir-lang/noir/commit/909b22bb20645761aabf8b40622aa79c6a96ed6a))
* LSP auto-import text indent ([#6699](https://github.com/noir-lang/noir/issues/6699)) ([bbe7564](https://github.com/noir-lang/noir/commit/bbe756414612a37371812ace300e77c309791729))
* LSP code action wasn't triggering on beginning or end of identifier ([#6616](https://github.com/noir-lang/noir/issues/6616)) ([1b910bc](https://github.com/noir-lang/noir/commit/1b910bc424d0435479b4104d2ed50557fdaf2bea))
* **LSP:** Use generic self type to narrow down methods to complete ([#6617](https://github.com/noir-lang/noir/issues/6617)) ([454b77b](https://github.com/noir-lang/noir/commit/454b77b3ac4e99b1a272fdc5c36f8babb5781cec))
* Map entry point indexes after all ssa passes ([#6740](https://github.com/noir-lang/noir/issues/6740)) ([1b6e26b](https://github.com/noir-lang/noir/commit/1b6e26b06ceb12abf92fdd49b70e6e2d10852d3b))
* Minimal change to avoid reverting entire PR [#6685](https://github.com/noir-lang/noir/issues/6685) ([#6778](https://github.com/noir-lang/noir/issues/6778)) ([0925a33](https://github.com/noir-lang/noir/commit/0925a332dbaa561aad195c143079588158498dad))
* Optimize array ref counts to copy arrays much less often ([#6685](https://github.com/noir-lang/noir/issues/6685)) ([24cc19e](https://github.com/noir-lang/noir/commit/24cc19ed9f29792c7b056124b2adf87fc6c18e42))
* Optimizer to keep track of changing opcode locations ([#6781](https://github.com/noir-lang/noir/issues/6781)) ([13c41d2](https://github.com/noir-lang/noir/commit/13c41d21f81fb40cdf2a970be119a83d11da9e03))
* Parser would hand on function type with colon in it ([#6764](https://github.com/noir-lang/noir/issues/6764)) ([9d7aadc](https://github.com/noir-lang/noir/commit/9d7aadc63c28b2c61d3524902c8b1038a46ba6f0))
* Prevent hoisting binary instructions which can overflow ([#6672](https://github.com/noir-lang/noir/issues/6672)) ([b4750d8](https://github.com/noir-lang/noir/commit/b4750d8dc13245bad81cbf7bef7010e3794e040a))
* Print ssa blocks without recursion ([#6715](https://github.com/noir-lang/noir/issues/6715)) ([5ccde81](https://github.com/noir-lang/noir/commit/5ccde8196400a9b99e5c3c4d9af0e3136e72b4cb))
* Println("{{}}") was printing "{{}}" instead of "{}" ([#6745](https://github.com/noir-lang/noir/issues/6745)) ([36bca82](https://github.com/noir-lang/noir/commit/36bca82c9a25fdb5f1cd657dace11e46e59491c8))
* Several format string fixes and improvements ([#6703](https://github.com/noir-lang/noir/issues/6703)) ([b70daf4](https://github.com/noir-lang/noir/commit/b70daf423890aa0f885ccf32531fa1583770c23c))
* **ssa:** Don't deduplicate constraints in blocks that are not dominated ([#6627](https://github.com/noir-lang/noir/issues/6627)) ([b024581](https://github.com/noir-lang/noir/commit/b0245811bfd84e0bf3559aa1e2f37ec52d08691e))
* **ssa:** Remove RC tracker in DIE ([#6700](https://github.com/noir-lang/noir/issues/6700)) ([f2607fd](https://github.com/noir-lang/noir/commit/f2607fd0eafac0018c157e101c7ebb1fe4223f73))
* **ssa:** Track all local allocations during flattening ([#6619](https://github.com/noir-lang/noir/issues/6619)) ([6491175](https://github.com/noir-lang/noir/commit/649117570b95b26776150e337c458d478eb48c2e))
* Typo in u128 docs ([#6711](https://github.com/noir-lang/noir/issues/6711)) ([37a4996](https://github.com/noir-lang/noir/commit/37a4996a7e33b9afe78dcb494f6c3e796d852607))
* Use correct type for attribute arguments ([#6640](https://github.com/noir-lang/noir/issues/6640)) ([de3e77a](https://github.com/noir-lang/noir/commit/de3e77a4acaab4bd2edc9a9e5226e54f468fd620))
* Use extension in docs link so it also works on GitHub ([#6787](https://github.com/noir-lang/noir/issues/6787)) ([655a3d3](https://github.com/noir-lang/noir/commit/655a3d3fdcf4f4cdb4381e3ff47d2f822c4b2276))
* Used signed division for signed modulo ([#6635](https://github.com/noir-lang/noir/issues/6635)) ([dace078](https://github.com/noir-lang/noir/commit/dace07849aa28795abb30b3f9d979ffc6b6487e6))


### Miscellaneous Chores

* Remove `ec` module from stdlib ([#6612](https://github.com/noir-lang/noir/issues/6612)) ([1e965bc](https://github.com/noir-lang/noir/commit/1e965bc8b9c4222c7b2ad7502df415781308de7f))
* Remove SchnorrVerify opcode (https://github.com/AztecProtocol/aztec-packages/pull/9897) ([66d3275](https://github.com/noir-lang/noir/commit/66d32751311378701b075ee7b2106d61e531ae4f))
* **stdlib:** Remove Schnorr ([#6749](https://github.com/noir-lang/noir/issues/6749)) ([57ebee0](https://github.com/noir-lang/noir/commit/57ebee03e00b69d1cc8f541b083b001cd517ec35))

## [1.0.0-beta.0](https://github.com/noir-lang/noir/compare/v0.39.0...v1.0.0-beta.0) (2024-11-22)


### ⚠ BREAKING CHANGES

* Require types of globals to be specified ([#6592](https://github.com/noir-lang/noir/issues/6592))
* remove eddsa from stdlib ([#6591](https://github.com/noir-lang/noir/issues/6591))

### Features

* Add `array_refcount` and `slice_refcount` builtins for debugging ([#6584](https://github.com/noir-lang/noir/issues/6584)) ([45eb756](https://github.com/noir-lang/noir/commit/45eb7568d56b2d254453b85f236d554232aa5df9))
* Avoid incrementing reference counts in some cases ([#6568](https://github.com/noir-lang/noir/issues/6568)) ([01c4a9f](https://github.com/noir-lang/noir/commit/01c4a9fb62ffe2190c73f0d5b12933d2eb8f6b5d))
* **ssa:** Loop invariant code motion ([#6563](https://github.com/noir-lang/noir/issues/6563)) ([7216f08](https://github.com/noir-lang/noir/commit/7216f0829dcece948d3243471e6d57380522e997))
* Trait aliases ([#6431](https://github.com/noir-lang/noir/issues/6431)) ([68c32b4](https://github.com/noir-lang/noir/commit/68c32b4ffd9b069fe4b119327dbf4018c17ab9d4))
* Try to inline brillig calls with all constant arguments  ([#6548](https://github.com/noir-lang/noir/issues/6548)) ([e4c66b9](https://github.com/noir-lang/noir/commit/e4c66b91d42b20d17837fe5e7c32c9a83b6ab354))


### Bug Fixes

* Consider prereleases to be compatible with pre-1.0.0 releases ([#6580](https://github.com/noir-lang/noir/issues/6580)) ([013e200](https://github.com/noir-lang/noir/commit/013e2000f1d7e7346b5cac0427732d545f501444))
* Correct type when simplifying `derive_pedersen_generators` ([#6579](https://github.com/noir-lang/noir/issues/6579)) ([efa5cc4](https://github.com/noir-lang/noir/commit/efa5cc4bf173b0ce49f47b1954165a2bdb276792))
* Don't report visibility errors when elaborating comptime value ([#6498](https://github.com/noir-lang/noir/issues/6498)) ([3c361c9](https://github.com/noir-lang/noir/commit/3c361c9f78a5d9de1b1bcb5a839d3bc481f89898))
* Parse a bit more SSA stuff ([#6599](https://github.com/noir-lang/noir/issues/6599)) ([0a6207d](https://github.com/noir-lang/noir/commit/0a6207dde6c744e2853905014e70d33b29b3e53b))
* Preserve newlines between comments when formatting statements ([#6601](https://github.com/noir-lang/noir/issues/6601)) ([d94eb08](https://github.com/noir-lang/noir/commit/d94eb085adf2cdd8f0e80d9cfd712c19c8810974))
* Remove `compiler_version` from new `Nargo.toml` ([#6590](https://github.com/noir-lang/noir/issues/6590)) ([df8f2ee](https://github.com/noir-lang/noir/commit/df8f2eee5c27d3cd4b6128056afdd9bd4a0322fe))


### Miscellaneous Chores

* Remove eddsa from stdlib ([#6591](https://github.com/noir-lang/noir/issues/6591)) ([8e046af](https://github.com/noir-lang/noir/commit/8e046afbbe3fba06c1e177f74aacefdd1bf871b6))
* Require types of globals to be specified ([#6592](https://github.com/noir-lang/noir/issues/6592)) ([8ff4efd](https://github.com/noir-lang/noir/commit/8ff4efda5589d39d31ced31c6575f43133fceebc))
* Switch to 1.0.0-beta versioning ([#6503](https://github.com/noir-lang/noir/issues/6503)) ([44e7dc1](https://github.com/noir-lang/noir/commit/44e7dc1037b047db866af675cd8caa0fc8aee324))

## [0.39.0](https://github.com/noir-lang/noir/compare/v0.38.0...v0.39.0) (2024-11-19)


### ⚠ BREAKING CHANGES

* Remove `recursive` from ACIR format; add them to API and CLI (https://github.com/AztecProtocol/aztec-packages/pull/9479)

### Features

* Avoid unnecessary ssa passes while loop unrolling ([#6509](https://github.com/noir-lang/noir/issues/6509)) ([f81c649](https://github.com/noir-lang/noir/commit/f81c6497ff88e1cc6f3f5c183e679090c6433c65))
* Deduplicate instructions across blocks ([#6499](https://github.com/noir-lang/noir/issues/6499)) ([b65a63d](https://github.com/noir-lang/noir/commit/b65a63d8d898e46cc686baa500f0b8070e45df14))
* Encode static error strings in the ABI (https://github.com/AztecProtocol/aztec-packages/pull/9552) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Parallelize DIE pass (https://github.com/AztecProtocol/aztec-packages/pull/9933) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* **profiler:** Reduce memory in Brillig execution flamegraph ([#6538](https://github.com/noir-lang/noir/issues/6538)) ([1cad7c8](https://github.com/noir-lang/noir/commit/1cad7c887893ebfb5de57a71d7965c8b88158a14))
* Simplify constant MSM calls in SSA ([#6547](https://github.com/noir-lang/noir/issues/6547)) ([f291e37](https://github.com/noir-lang/noir/commit/f291e3702589a5cd043acfded5e187f56ec765cc))
* SSA parser ([#6489](https://github.com/noir-lang/noir/issues/6489)) ([21c9db5](https://github.com/noir-lang/noir/commit/21c9db5f325beef91df024838c4b33ff7f704332))
* **ssa:** Unroll small loops in brillig ([#6505](https://github.com/noir-lang/noir/issues/6505)) ([5d5175e](https://github.com/noir-lang/noir/commit/5d5175e1c076bd651702b6c84a00c85bc4fea860))
* Stop with HeapVector (https://github.com/AztecProtocol/aztec-packages/pull/9810) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9711) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Use a full `BlackBoxFunctionSolver` implementation when execution brillig during acirgen ([#6481](https://github.com/noir-lang/noir/issues/6481)) ([22fc11a](https://github.com/noir-lang/noir/commit/22fc11ace31b515287f62219c0e6d6ed3d1bedd7))


### Bug Fixes

* Allow range checks to be performed within the comptime intepreter ([#6514](https://github.com/noir-lang/noir/issues/6514)) ([852c87a](https://github.com/noir-lang/noir/commit/852c87ae9ecdd441ee4c2ab3e78e86b2da07d8a4))
* Disallow `#[test]` on associated functions ([#6449](https://github.com/noir-lang/noir/issues/6449)) ([35408ab](https://github.com/noir-lang/noir/commit/35408ab303f1018c1e2c38e6ea55430a2c89dc4c))
* Do a shallow follow_bindings before unification ([#6558](https://github.com/noir-lang/noir/issues/6558)) ([32a9ed9](https://github.com/noir-lang/noir/commit/32a9ed9ad19cf81275c31ca77e4970bc1598c112))
* **docs:** Fix broken links in oracles doc ([#6488](https://github.com/noir-lang/noir/issues/6488)) ([aa37cd5](https://github.com/noir-lang/noir/commit/aa37cd5be25412919f466a938260ae1a485ee096))
* Fix poor handling of aliased references in flattening pass causing some values to be zeroed ([#6434](https://github.com/noir-lang/noir/issues/6434)) ([8932dac](https://github.com/noir-lang/noir/commit/8932dac4847c643341320c2893f7e4297c78c621))
* Parse Slice type in SSa ([#6507](https://github.com/noir-lang/noir/issues/6507)) ([34ad666](https://github.com/noir-lang/noir/commit/34ad6669b210173ddf0484b04e47161b2cfbcadf))
* Perform arithmetic simplification through `CheckedCast` ([#6502](https://github.com/noir-lang/noir/issues/6502)) ([72e8de0](https://github.com/noir-lang/noir/commit/72e8de0656c4789f57ff1d3ddecc8901df627aab))
* Set local_module before elaborating each trait ([#6506](https://github.com/noir-lang/noir/issues/6506)) ([1df8c45](https://github.com/noir-lang/noir/commit/1df8c456d6d256f120d6df6ae3e6735cb7eb7dae))
* Take blackbox function outputs into account when merging expressions ([#6532](https://github.com/noir-lang/noir/issues/6532)) ([713df69](https://github.com/noir-lang/noir/commit/713df69aad56fc5aaefd5d140275a3217de4d866))
* **tests:** Use a file lock as well as a mutex to isolate tests cases ([#6508](https://github.com/noir-lang/noir/issues/6508)) ([cfc22cb](https://github.com/noir-lang/noir/commit/cfc22cb0ca133fce49a25c3f055f5a6b8bd9b58e))
* Treat all parameters as possible aliases of each other ([#6477](https://github.com/noir-lang/noir/issues/6477)) ([0262e5b](https://github.com/noir-lang/noir/commit/0262e5b93ab71a420365c6e56d3250b2d1eea659))
* Typing of artifacts (https://github.com/AztecProtocol/aztec-packages/pull/9581) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))


### Miscellaneous Chores

* Remove `recursive` from ACIR format; add them to API and CLI (https://github.com/AztecProtocol/aztec-packages/pull/9479) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))

## [0.38.0](https://github.com/noir-lang/noir/compare/v0.37.0...v0.38.0) (2024-11-08)


### ⚠ BREAKING CHANGES

* Always Check Arithmetic Generics at Monomorphization ([#6329](https://github.com/noir-lang/noir/issues/6329))

### Features

* Always Check Arithmetic Generics at Monomorphization ([#6329](https://github.com/noir-lang/noir/issues/6329)) ([2972db2](https://github.com/noir-lang/noir/commit/2972db20fc00ed0e43b662092f0d0712421d122f))
* Ensure that generated ACIR is solvable ([#6415](https://github.com/noir-lang/noir/issues/6415)) ([b473d99](https://github.com/noir-lang/noir/commit/b473d99b2b70b595596b8392617256dbaf5d5642))
* Nargo command to generate shell completions ([#6413](https://github.com/noir-lang/noir/issues/6413)) ([13856a1](https://github.com/noir-lang/noir/commit/13856a121125b1ccca15919942081a5d157d280e))


### Bug Fixes

* Check infix expression is valid in program input ([#6450](https://github.com/noir-lang/noir/issues/6450)) ([35dedb5](https://github.com/noir-lang/noir/commit/35dedb54a0853ba0fa85038d832a520f9ba01a98))
* Discard optimisation that would change execution ordering or that is related to call outputs ([#6461](https://github.com/noir-lang/noir/issues/6461)) ([b8654f7](https://github.com/noir-lang/noir/commit/b8654f700b218cc09c5381af65df11ead9ffcdaf))
* Don't crash on AsTraitPath with empty path ([#6454](https://github.com/noir-lang/noir/issues/6454)) ([fc72dcd](https://github.com/noir-lang/noir/commit/fc72dcdf3e8eeff73f72756e7ab87dddf2920657))
* Fix Alias and Error kinds ([#6426](https://github.com/noir-lang/noir/issues/6426)) ([3cb259f](https://github.com/noir-lang/noir/commit/3cb259f419cf352f768728b15f849e520fc233cb))
* Let formatter respect newlines between comments ([#6458](https://github.com/noir-lang/noir/issues/6458)) ([fb1a8ca](https://github.com/noir-lang/noir/commit/fb1a8ca67c58d87991358078e6c532b49824fdb8))
* Right shift is not a regular division ([#6400](https://github.com/noir-lang/noir/issues/6400)) ([2247814](https://github.com/noir-lang/noir/commit/2247814f951f5d33257cd123a3bdcba857c9b167))
* **sea:** Mem2reg to treat block input references as alias ([#6452](https://github.com/noir-lang/noir/issues/6452)) ([5310064](https://github.com/noir-lang/noir/commit/53100647bf1dc7917b66c9a7041c06b1e716fbe7))
* **ssa:** Change array_set to not mutate slices coming from function inputs ([#6463](https://github.com/noir-lang/noir/issues/6463)) ([371bd45](https://github.com/noir-lang/noir/commit/371bd45130c9095e5dfb20dc79fbf41c02ed087c))
* **ssa:** Resolve value IDs in terminator before comparing to array ([#6448](https://github.com/noir-lang/noir/issues/6448)) ([66f15ca](https://github.com/noir-lang/noir/commit/66f15caba8466501256a98cee289c49376b27097))
* **tests:** Prevent EOF error while running test programs ([#6455](https://github.com/noir-lang/noir/issues/6455)) ([358e381](https://github.com/noir-lang/noir/commit/358e38107edbc4f40c97b88196456d82f5557e3f))
* Type-check turbofish in trait before function call ([#6416](https://github.com/noir-lang/noir/issues/6416)) ([f8fd813](https://github.com/noir-lang/noir/commit/f8fd813b09ce870364700659e3ea8499ab51105e))

## [0.37.0](https://github.com/noir-lang/noir/compare/v0.36.0...v0.37.0) (2024-10-31)


### ⚠ BREAKING CHANGES

* remove mimc from stdlib ([#6402](https://github.com/noir-lang/noir/issues/6402))
* **avm/brillig:** revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408)
* use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416)
* remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338)
* replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350)
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327))

### Features

* Add capacities to brillig vectors and use them in slice ops ([#6332](https://github.com/noir-lang/noir/issues/6332)) ([c9ff9a3](https://github.com/noir-lang/noir/commit/c9ff9a392f6142c04a3a512722ef6c9f9a7c3439))
* **avm/brillig:** Revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))
* Better LSP hover for functions ([#6376](https://github.com/noir-lang/noir/issues/6376)) ([e92b519](https://github.com/noir-lang/noir/commit/e92b519bdfbd2a149a46745ad2ecffdd0e91f3f1))
* Check trait where clause ([#6325](https://github.com/noir-lang/noir/issues/6325)) ([0de3241](https://github.com/noir-lang/noir/commit/0de3241bd290b1737ff831c30e5a2a0633a53eb3))
* **ci:** Add report of Brillig opcodes executed ([#6396](https://github.com/noir-lang/noir/issues/6396)) ([e04b026](https://github.com/noir-lang/noir/commit/e04b02621e3651ddbb8e314563d614171a8a9933))
* Do not increment reference counts on arrays through references ([#6375](https://github.com/noir-lang/noir/issues/6375)) ([60c770f](https://github.com/noir-lang/noir/commit/60c770f5f2594eea31ac75c852980edefa40d9eb))
* Improve malformed test attribute error ([#6414](https://github.com/noir-lang/noir/issues/6414)) ([8f516d7](https://github.com/noir-lang/noir/commit/8f516d73d2d33988f6cdb9367244c11bc36ede22))
* Let LSP suggest traits in trait bounds ([#6370](https://github.com/noir-lang/noir/issues/6370)) ([e909dcb](https://github.com/noir-lang/noir/commit/e909dcbb06c7b0043ffc79d5b8af99835b0096e5))
* Let the formatter remove lambda block braces for single-statement blocks ([#6335](https://github.com/noir-lang/noir/issues/6335)) ([52f7c0b](https://github.com/noir-lang/noir/commit/52f7c0b67fa2f70848512c87fabcefc4c5426dd1))
* Let the LSP import code action insert into existing use statements ([#6358](https://github.com/noir-lang/noir/issues/6358)) ([308717b](https://github.com/noir-lang/noir/commit/308717b6c44db4b206ad371cd6322478ce68746b))
* LSP auto-import will try to add to existing use statements ([#6354](https://github.com/noir-lang/noir/issues/6354)) ([647f6a4](https://github.com/noir-lang/noir/commit/647f6a4bd3d00fd3b3b3e4ff17dce512287ee5b4))
* Merge and sort imports ([#6322](https://github.com/noir-lang/noir/issues/6322)) ([07ab515](https://github.com/noir-lang/noir/commit/07ab5150857ec6719b132ec91d5f90af0564a046))
* **perf:** Use [u32;16] for message block in sha256 ([#6324](https://github.com/noir-lang/noir/issues/6324)) ([81c612f](https://github.com/noir-lang/noir/commit/81c612f281cddf41d12ea62d9f610eab05ad1973))
* **profiler:** Add Brillig procedure info to debug artifact for more informative profiling ([#6385](https://github.com/noir-lang/noir/issues/6385)) ([f5f65dc](https://github.com/noir-lang/noir/commit/f5f65dc29eb1f71926e6f8ed6681df563a85bd23))
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327)) ([4d87c9a](https://github.com/noir-lang/noir/commit/4d87c9ac78b48b4bd0ae81316df28aab390d004e))
* Reject programs with unconditional recursion ([#6292](https://github.com/noir-lang/noir/issues/6292)) ([00c5c51](https://github.com/noir-lang/noir/commit/00c5c5154b818d0b50802721eae621efb3379a4e))
* Remove 'single use' intermediate variables ([#6268](https://github.com/noir-lang/noir/issues/6268)) ([ec75e8e](https://github.com/noir-lang/noir/commit/ec75e8ec59e0f2a2169aea67372411ede4074d09))
* Remove mimc from stdlib ([#6402](https://github.com/noir-lang/noir/issues/6402)) ([ec03e77](https://github.com/noir-lang/noir/commit/ec03e779f438069e51e973d8f29727e1e0fb5665))
* Sha256 refactoring and benchmark with longer input ([#6318](https://github.com/noir-lang/noir/issues/6318)) ([d606491](https://github.com/noir-lang/noir/commit/d606491a61a9fe2153666f7d0a3ec6cae7bfaecb))
* **ssa:** Various mem2reg reverts to reduce memory and compilation time ([#6307](https://github.com/noir-lang/noir/issues/6307)) ([b820328](https://github.com/noir-lang/noir/commit/b82032888819eac82b2bfce8300c2c8b66507c64))
* Suggest removing `!` from macro call that doesn't return Quoted ([#6384](https://github.com/noir-lang/noir/issues/6384)) ([0232b57](https://github.com/noir-lang/noir/commit/0232b573c418ab74715b7cc1d3e858d993bc1c07))
* Support specifying generics on a struct when calling an associated function ([#6306](https://github.com/noir-lang/noir/issues/6306)) ([eba151e](https://github.com/noir-lang/noir/commit/eba151ecf59c61f7ffc6bec00d455dce84e7b927))
* **test:** Run test matrix on stdlib tests ([#6352](https://github.com/noir-lang/noir/issues/6352)) ([4c39514](https://github.com/noir-lang/noir/commit/4c39514fccf3595de6bdfad755b6ae2d3ef11aa1))


### Bug Fixes

* (formatter) correctly format quote delimiters ([#6377](https://github.com/noir-lang/noir/issues/6377)) ([b42accf](https://github.com/noir-lang/noir/commit/b42accf59c9294131ce2773ac3ebdb20f548ece5))
* (formatter) indent after infix lhs ([#6331](https://github.com/noir-lang/noir/issues/6331)) ([c891ffd](https://github.com/noir-lang/noir/commit/c891ffda9df17eabcaf2035f098d29f97bfc463a))
* (LSP) check visibility of module that re-exports item, if any ([#6371](https://github.com/noir-lang/noir/issues/6371)) ([a4fc6e8](https://github.com/noir-lang/noir/commit/a4fc6e861492ab5ff12ebc5fdbb248f983eab0a2))
* Aliases in path ([#6399](https://github.com/noir-lang/noir/issues/6399)) ([be882f1](https://github.com/noir-lang/noir/commit/be882f11ee661bf19ed6d78a7b3085099d4273e8))
* Allow globals in format strings ([#6382](https://github.com/noir-lang/noir/issues/6382)) ([15c729a](https://github.com/noir-lang/noir/commit/15c729a7f29564092411658be613145b18ddd226))
* Allow type aliases in let patterns ([#6356](https://github.com/noir-lang/noir/issues/6356)) ([91c0842](https://github.com/noir-lang/noir/commit/91c08421fdc5df7edcf502fb7fc1d343bb860b03))
* Always inline `derive_generators` ([#6350](https://github.com/noir-lang/noir/issues/6350)) ([7c98b36](https://github.com/noir-lang/noir/commit/7c98b36305ffdbbaee3947723f248fa718e7a950))
* Better formatting of leading/trailing line/block comments in expression lists ([#6338](https://github.com/noir-lang/noir/issues/6338)) ([3299c25](https://github.com/noir-lang/noir/commit/3299c25cefb6e3eb4b55396b2f842138b658e42f))
* Display every bit in integer tokens ([#6360](https://github.com/noir-lang/noir/issues/6360)) ([b985fdf](https://github.com/noir-lang/noir/commit/b985fdf6e635570b8db3af83d9ec14e7cd749062))
* Distinguish TypePath with and without turbofish ([#6404](https://github.com/noir-lang/noir/issues/6404)) ([0e974c2](https://github.com/noir-lang/noir/commit/0e974c22a1de0f6d38bc7a59280f86222f864698))
* Fix panic in comptime code ([#6361](https://github.com/noir-lang/noir/issues/6361)) ([2f37610](https://github.com/noir-lang/noir/commit/2f376100d3ee7ab519d6ea30153395bb3e7af7b1))
* Formatter didn't format `&gt;>=` well ([#6337](https://github.com/noir-lang/noir/issues/6337)) ([598230d](https://github.com/noir-lang/noir/commit/598230d9427cf988fc6da8fe9e1eb2b7c00a2fa6))
* LSP auto-import would import public item inside private module ([#6366](https://github.com/noir-lang/noir/issues/6366)) ([51eb295](https://github.com/noir-lang/noir/commit/51eb2954e8dfb3da298431a82f36fa72ebbee8eb))
* Make keccak256 work with input lengths greater than 136 bytes ([#6393](https://github.com/noir-lang/noir/issues/6393)) ([07c9322](https://github.com/noir-lang/noir/commit/07c9322332e147c0e8fade5e238552ecbf3e7849))
* Mutable global pattern didn't have a span ([#6328](https://github.com/noir-lang/noir/issues/6328)) ([5a6dae9](https://github.com/noir-lang/noir/commit/5a6dae9a9ee9c3650695a16d18fb8b7ac12180f4))
* Numeric generic doesn't have a default type ([#6405](https://github.com/noir-lang/noir/issues/6405)) ([3a073f7](https://github.com/noir-lang/noir/commit/3a073f7446e3cd78ca963b221e05f341a6041067))
* Remove assumed parent traits ([#6365](https://github.com/noir-lang/noir/issues/6365)) ([83d29f2](https://github.com/noir-lang/noir/commit/83d29f259debe41d0b5cdfb6e63d31733ae4e0c7))
* Slightly better formatting of empty blocks with comments ([#6367](https://github.com/noir-lang/noir/issues/6367)) ([da72979](https://github.com/noir-lang/noir/commit/da729791b7ffcfcd2f58ba1f8bf2c274c04f303e))
* **ssa:** Do not mark an array from a parameter mutable ([#6355](https://github.com/noir-lang/noir/issues/6355)) ([bcd8976](https://github.com/noir-lang/noir/commit/bcd897627c69b1ebcadc8b84abe2922ce3473c56))


### Miscellaneous Chores

* Remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))

## [0.36.0](https://github.com/noir-lang/noir/compare/v0.35.0...v0.36.0) (2024-10-22)


### ⚠ BREAKING CHANGES

* remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107)
* remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245)
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057)
* remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104)
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989)
* **avm:** remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030)
* Integer division is not the inverse of integer multiplication ([#6243](https://github.com/noir-lang/noir/issues/6243))
* kind size checks ([#6137](https://github.com/noir-lang/noir/issues/6137))
* Change tag attributes to require a ' prefix ([#6235](https://github.com/noir-lang/noir/issues/6235))

### Features

* Add `checked_transmute` ([#6262](https://github.com/noir-lang/noir/issues/6262)) ([2618061](https://github.com/noir-lang/noir/commit/2618061ee88e47fb063904d50af7a4eea26d3db9))
* Add more `Type` and `UnresolvedType` methods ([#5994](https://github.com/noir-lang/noir/issues/5994)) ([8236cbd](https://github.com/noir-lang/noir/commit/8236cbdff60c1aaf41fc53142b6f0f9ea2fc2fa8))
* Allow `unconstrained` after visibility ([#6246](https://github.com/noir-lang/noir/issues/6246)) ([f6dfbcf](https://github.com/noir-lang/noir/commit/f6dfbcf057efc95141b36499152dbd0b919a31b3))
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Don't crash LSP when there are errors resolving the workspace ([#6257](https://github.com/noir-lang/noir/issues/6257)) ([7cc7197](https://github.com/noir-lang/noir/commit/7cc7197bf7b2e41c07e8d1979f7e9d45c676d11b))
* Don't suggest private struct fields in LSP ([#6256](https://github.com/noir-lang/noir/issues/6256)) ([2a727b3](https://github.com/noir-lang/noir/commit/2a727b3f7f7fb84ab88b0d08e1ab29ae012a8c4f))
* Handwritten parser ([#6180](https://github.com/noir-lang/noir/issues/6180)) ([c4273a0](https://github.com/noir-lang/noir/commit/c4273a0c8f8b751a3dbe097e070e4e7b2c8ec438))
* **improve:** Remove scan through globals ([#6282](https://github.com/noir-lang/noir/issues/6282)) ([fd91913](https://github.com/noir-lang/noir/commit/fd91913806a49255ba721012c2e302959a82c4f6))
* Inclusive for loop ([#6200](https://github.com/noir-lang/noir/issues/6200)) ([bd861f2](https://github.com/noir-lang/noir/commit/bd861f282144056ecb52954fa9f6fd8db918e093))
* Integrate databus in the private kernels (https://github.com/AztecProtocol/aztec-packages/pull/9028) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* **interpreter:** Comptime derive generators ([#6303](https://github.com/noir-lang/noir/issues/6303)) ([d8767b3](https://github.com/noir-lang/noir/commit/d8767b364f4db9a52c823e7f39f36feac3c90fcd))
* Kind size checks ([#6137](https://github.com/noir-lang/noir/issues/6137)) ([6e40f62](https://github.com/noir-lang/noir/commit/6e40f628a87ab4b5e9e817b7b3a790920dc01683))
* New formatter ([#6300](https://github.com/noir-lang/noir/issues/6300)) ([62404d7](https://github.com/noir-lang/noir/commit/62404d7ff349ddf7551f2efd865adafc5213a742))
* Optimize `Quoted::as_expr` by parsing just once ([#6237](https://github.com/noir-lang/noir/issues/6237)) ([a4fcd00](https://github.com/noir-lang/noir/commit/a4fcd0017e019f05b5a4d6b97c50b75f9e560210))
* Optimize reading a workspace's files ([#6281](https://github.com/noir-lang/noir/issues/6281)) ([b54ed26](https://github.com/noir-lang/noir/commit/b54ed2671c8bc0e198e262883598936b9e49d69e))
* **perf:** Flamegraphs for test program execution benchmarks ([#6253](https://github.com/noir-lang/noir/issues/6253)) ([c186791](https://github.com/noir-lang/noir/commit/c186791636c2afb2d3763bccee956298039feed2))
* **perf:** Follow array sets backwards in array set from get optimization ([#6208](https://github.com/noir-lang/noir/issues/6208)) ([999071b](https://github.com/noir-lang/noir/commit/999071b80e61a37cb994a4e359eabbac27cd53f1))
* Recover from '=' instead of ':' in struct constructor/pattern ([#6236](https://github.com/noir-lang/noir/issues/6236)) ([9a12f31](https://github.com/noir-lang/noir/commit/9a12f31e909bbd4d4f0538704b3f40ea654fabaf))
* Remove byte decomposition in `compute_decomposition` ([#6159](https://github.com/noir-lang/noir/issues/6159)) ([a8bcae2](https://github.com/noir-lang/noir/commit/a8bcae215bf19356226ad052710c94b64da90ffa))
* Show LSP diagnostic related information ([#6277](https://github.com/noir-lang/noir/issues/6277)) ([c8a91a5](https://github.com/noir-lang/noir/commit/c8a91a55d69c54e3ea9b6a16053fa83ce17b1426))
* Slightly improve "unexpected token" error message ([#6279](https://github.com/noir-lang/noir/issues/6279)) ([8232bfa](https://github.com/noir-lang/noir/commit/8232bfaf0a88dcba5a6949489b81d78c3413c5bc))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8934) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9034) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9099) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9275) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* **test:** Fuzz poseidon hases against an external library ([#6273](https://github.com/noir-lang/noir/issues/6273)) ([8d8ea89](https://github.com/noir-lang/noir/commit/8d8ea8963c5e4e23bd387aa729e09d3a9553a698))
* **test:** Fuzz test poseidon2 hash equivalence ([#6265](https://github.com/noir-lang/noir/issues/6265)) ([f61ba03](https://github.com/noir-lang/noir/commit/f61ba037c6726e19be4f894d9447fe396df95417))
* **test:** Fuzz test stdlib hash functions ([#6233](https://github.com/noir-lang/noir/issues/6233)) ([1a2ca46](https://github.com/noir-lang/noir/commit/1a2ca46af0d1c05813dbe28670a2bc39b79e4c9f))
* **test:** Include the PoseidonHasher in the fuzzing ([#6280](https://github.com/noir-lang/noir/issues/6280)) ([afb8a7c](https://github.com/noir-lang/noir/commit/afb8a7cf7b1751a10dd2cdd87817945fa4c1ed1f))
* Trait inheritance ([#6252](https://github.com/noir-lang/noir/issues/6252)) ([d3301a4](https://github.com/noir-lang/noir/commit/d3301a4f5558cf4e173f7d0edc08186ad4fb2eee))
* Visibility for impl functions ([#6179](https://github.com/noir-lang/noir/issues/6179)) ([1b26440](https://github.com/noir-lang/noir/commit/1b26440889379f491315cd9d088537b1898d57c5))
* Visibility for struct fields ([#6221](https://github.com/noir-lang/noir/issues/6221)) ([fc1c7ab](https://github.com/noir-lang/noir/commit/fc1c7ab6ee7be7c9d57fab5b2efe252c613f326b))
* Warn about private types leaking in public functions and struct fields ([#6296](https://github.com/noir-lang/noir/issues/6296)) ([67ac0d6](https://github.com/noir-lang/noir/commit/67ac0d60c3e8b450a9e871f3edb29322ac5045d2))


### Bug Fixes

* Add missing visibility for auto-import names ([#6205](https://github.com/noir-lang/noir/issues/6205)) ([c3cb38a](https://github.com/noir-lang/noir/commit/c3cb38a7c4de6fc321b367eda3fca6d06e76b77a))
* Address inactive public key check in `verify_signature_noir` ([#6270](https://github.com/noir-lang/noir/issues/6270)) ([e4325aa](https://github.com/noir-lang/noir/commit/e4325aace424d5c4552c92cdb360974fdd294048))
* Allow array map on empty arrays ([#6305](https://github.com/noir-lang/noir/issues/6305)) ([51ae1b3](https://github.com/noir-lang/noir/commit/51ae1b324cd73fdb4fe3695b5d483a44b4aff4a9))
* Change tag attributes to require a ' prefix ([#6235](https://github.com/noir-lang/noir/issues/6235)) ([b43dcb2](https://github.com/noir-lang/noir/commit/b43dcb2b30ce090c393990b2192411f9b3dc6a9e))
* Check for Schnorr null signature ([#6226](https://github.com/noir-lang/noir/issues/6226)) ([2430920](https://github.com/noir-lang/noir/commit/24309200f600ad20a51d9f2c6c53849466fccda4))
* Display function name and body when inlining recursion limit hit ([#6291](https://github.com/noir-lang/noir/issues/6291)) ([33a1e7d](https://github.com/noir-lang/noir/commit/33a1e7d2246bdea48dd6fe925d427c7be8c4659d))
* Do not warn on unused self in traits ([#6298](https://github.com/noir-lang/noir/issues/6298)) ([4d524bf](https://github.com/noir-lang/noir/commit/4d524bf34de98449419a025aa53d593bf42e70a7))
* Don't warn on unuse global if it has an abi annotation ([#6258](https://github.com/noir-lang/noir/issues/6258)) ([e13f617](https://github.com/noir-lang/noir/commit/e13f61741d17ed2e03ff26cb858cb3d243e67c88))
* Don't warn on unused struct that has an abi annotation ([#6254](https://github.com/noir-lang/noir/issues/6254)) ([8a31632](https://github.com/noir-lang/noir/commit/8a316324a971a10d46392d7c64125d1d6ac9d557))
* Don't warn twice when referring to private item ([#6216](https://github.com/noir-lang/noir/issues/6216)) ([619c545](https://github.com/noir-lang/noir/commit/619c5451b152d62e01d3c4c1da7e13ff6502f915))
* Enforce correctness of decompositions performed at compile time ([#6278](https://github.com/noir-lang/noir/issues/6278)) ([53252fd](https://github.com/noir-lang/noir/commit/53252fd521ce7818a1d97824be30466590d879f5))
* **frontend:** Do not warn when a nested struct is provided as input to main ([#6239](https://github.com/noir-lang/noir/issues/6239)) ([9dfe223](https://github.com/noir-lang/noir/commit/9dfe223e4dc168351c5cceb9d1abda326141b014))
* Handle dfg databus in SSA normalization ([#6249](https://github.com/noir-lang/noir/issues/6249)) ([9d8bee5](https://github.com/noir-lang/noir/commit/9d8bee5b4e9308a812b1f93c3a48ddd11971ac17))
* Handle nested arrays in calldata ([#6232](https://github.com/noir-lang/noir/issues/6232)) ([0ab8f5e](https://github.com/noir-lang/noir/commit/0ab8f5e3c32af05a3c158562c0fcf9729741e0ab))
* Homogeneous input points for EC ADD ([#6241](https://github.com/noir-lang/noir/issues/6241)) ([f6a7306](https://github.com/noir-lang/noir/commit/f6a7306436ea1a37ec7f3b884721b50467e9a063))
* Integer division is not the inverse of integer multiplication ([#6243](https://github.com/noir-lang/noir/issues/6243)) ([1cd2587](https://github.com/noir-lang/noir/commit/1cd2587bf67143832f76f90c25aecca1a46b1284))
* Panic on composite types within databus ([#6225](https://github.com/noir-lang/noir/issues/6225)) ([29bd125](https://github.com/noir-lang/noir/commit/29bd125314b58e2eac23742ff1de022a97dcc60a))
* Prevent compiler panic when popping from empty slices ([#6274](https://github.com/noir-lang/noir/issues/6274)) ([87137d8](https://github.com/noir-lang/noir/commit/87137d8d93622052dbe1c8a933d542a5c147c15c))
* Reject invalid expression with in CLI parser ([#6287](https://github.com/noir-lang/noir/issues/6287)) ([052aee8](https://github.com/noir-lang/noir/commit/052aee80ff3e1e4fd2ca45310d7bb8b980af126a))
* Remove need for duplicate attributes on each function (https://github.com/AztecProtocol/aztec-packages/pull/9244) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Visibility for impl methods ([#6261](https://github.com/noir-lang/noir/issues/6261)) ([70cbeb4](https://github.com/noir-lang/noir/commit/70cbeb4322a0b11c1c167ab27bf0408d04fe7b7d))


### Miscellaneous Chores

* Remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))


### Code Refactoring

* **avm:** Remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))

## [0.35.0](https://github.com/noir-lang/noir/compare/v0.34.0...v0.35.0) (2024-10-03)


### ⚠ BREAKING CHANGES

* Syncing TypeVariableKind with Kind ([#6094](https://github.com/noir-lang/noir/issues/6094))
* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Infer globals to be u32 when used in a type ([#6083](https://github.com/noir-lang/noir/issues/6083))
* removing implicit numeric generics ([#5837](https://github.com/noir-lang/noir/issues/5837))

### Features

* (LSP) if in runtime code, always suggest functions that return Quoted as macro calls ([#6098](https://github.com/noir-lang/noir/issues/6098)) ([4a160cb](https://github.com/noir-lang/noir/commit/4a160cb99cbd9928c034a7009f398974fc6fdb11))
* (LSP) remove unused imports ([#6129](https://github.com/noir-lang/noir/issues/6129)) ([98bc460](https://github.com/noir-lang/noir/commit/98bc46002cdd8daff1baf0756ecc60dbdf420fd9))
* (LSP) show global value on hover ([#6097](https://github.com/noir-lang/noir/issues/6097)) ([3d9d072](https://github.com/noir-lang/noir/commit/3d9d07210544c9d27051eb5e629585760f48cd1c))
* (LSP) suggest $vars inside `quote { ... }` ([#6114](https://github.com/noir-lang/noir/issues/6114)) ([73245b3](https://github.com/noir-lang/noir/commit/73245b3aae0c65780a102ac6842f06df65e5fc35))
* Add `Expr::as_constructor` ([#5980](https://github.com/noir-lang/noir/issues/5980)) ([76dea7b](https://github.com/noir-lang/noir/commit/76dea7b409baa98236f6433f17c2ce9206dd4ba3))
* Add `Expr::as_for` and `Expr::as_for_range` ([#6039](https://github.com/noir-lang/noir/issues/6039)) ([abcae75](https://github.com/noir-lang/noir/commit/abcae750f022dd7c49cee616edddd7b1cc93f3b8))
* Add `Expr::as_lambda` ([#6048](https://github.com/noir-lang/noir/issues/6048)) ([31130dc](https://github.com/noir-lang/noir/commit/31130dc7aec24a7a7b9f342df94b14f295eb2103))
* Add a `comptime` string type for string handling at compile-time ([#6026](https://github.com/noir-lang/noir/issues/6026)) ([5d2984f](https://github.com/noir-lang/noir/commit/5d2984fce4f55e43cb418e40462d430227b71768))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Allow silencing an unused variable defined via `let` ([#6149](https://github.com/noir-lang/noir/issues/6149)) ([a2bc059](https://github.com/noir-lang/noir/commit/a2bc059f993d3e9ca06a2fe4857ef1c522c97286))
* Allow visibility modifiers in struct definitions ([#6054](https://github.com/noir-lang/noir/issues/6054)) ([199be58](https://github.com/noir-lang/noir/commit/199be584a36d20660ada49473050c5191251d6c5))
* Check unconstrained trait impl method matches ([#6057](https://github.com/noir-lang/noir/issues/6057)) ([aedc983](https://github.com/noir-lang/noir/commit/aedc9832240e55473c504fa2e6e3b3af618bda08))
* Default to outputting witness with file named after package ([#6031](https://github.com/noir-lang/noir/issues/6031)) ([e74b4ae](https://github.com/noir-lang/noir/commit/e74b4ae3ebcf301eedc5d0059bcebd5dced75d72))
* Detect unconstructed structs ([#6061](https://github.com/noir-lang/noir/issues/6061)) ([bcb438b](https://github.com/noir-lang/noir/commit/bcb438b0816fbe08344535545612d32b4730af79))
* Do not double error on import with error ([#6131](https://github.com/noir-lang/noir/issues/6131)) ([9b26650](https://github.com/noir-lang/noir/commit/9b26650f4a45c220484fc187500c7307af9c88d7))
* Expose `derived_generators` and `pedersen_commitment_with_separator` from the stdlib ([#6154](https://github.com/noir-lang/noir/issues/6154)) ([877b806](https://github.com/noir-lang/noir/commit/877b806ee02cb640472c6bb2b1ed7bc76b861a9b))
* Faster LSP by caching file managers ([#6047](https://github.com/noir-lang/noir/issues/6047)) ([c48a4f8](https://github.com/noir-lang/noir/commit/c48a4f83063ff55574d5b4a6277950a9edbc6317))
* Hoist constant allocation outside of loops ([#6158](https://github.com/noir-lang/noir/issues/6158)) ([180bfc9](https://github.com/noir-lang/noir/commit/180bfc99944cd42b3f44048213458d1399687cef))
* Implement `to_be_radix` in the comptime interpreter ([#6043](https://github.com/noir-lang/noir/issues/6043)) ([1550278](https://github.com/noir-lang/noir/commit/1550278f1e96392967b477b9b12be3bb0eea8fd6))
* Implement solver for mov_registers_to_registers ([#6089](https://github.com/noir-lang/noir/issues/6089)) ([4170c55](https://github.com/noir-lang/noir/commit/4170c55019bd27fd51be8a46637514dfe86de53c))
* Implement type paths ([#6093](https://github.com/noir-lang/noir/issues/6093)) ([2174ffb](https://github.com/noir-lang/noir/commit/2174ffb92b5d88e7e0926c91f42bc7f849e8ddc1))
* Let `Module::functions` and `Module::structs` return them in definition order ([#6178](https://github.com/noir-lang/noir/issues/6178)) ([dec9874](https://github.com/noir-lang/noir/commit/dec98747197442f6c2a15e6543c5d453dff4b967))
* Let LSP suggest macro calls too ([#6090](https://github.com/noir-lang/noir/issues/6090)) ([26d275b](https://github.com/noir-lang/noir/commit/26d275b65fa339d877c90d5c6c13ac8ef47189e1))
* Let LSP suggest trait impl methods as you are typing them ([#6029](https://github.com/noir-lang/noir/issues/6029)) ([dfed81b](https://github.com/noir-lang/noir/commit/dfed81b4b39b2f783d6e81a78ee27fba7032e01c))
* LSP autocompletion for `TypePath` ([#6117](https://github.com/noir-lang/noir/issues/6117)) ([3f79d8f](https://github.com/noir-lang/noir/commit/3f79d8f04c5f90c6b21359a3d0960446ebf84b2d))
* **metaprogramming:** Add `#[use_callers_scope]` ([#6050](https://github.com/noir-lang/noir/issues/6050)) ([8c34046](https://github.com/noir-lang/noir/commit/8c340461c3f7054839009c4b1ed5ac8a0dd55e09))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constraints in sha256 ([#6145](https://github.com/noir-lang/noir/issues/6145)) ([164d29e](https://github.com/noir-lang/noir/commit/164d29e4d1960d16fdeafe2cc8ea8144a769f7b2))
* **perf:** Allow array set last uses optimization in return block of Brillig functions ([#6119](https://github.com/noir-lang/noir/issues/6119)) ([5598059](https://github.com/noir-lang/noir/commit/5598059576c6cbc72474aff4b18bc5e4bb9f08e1))
* **perf:** Handle array set optimization across blocks for Brillig functions ([#6153](https://github.com/noir-lang/noir/issues/6153)) ([12cb80a](https://github.com/noir-lang/noir/commit/12cb80a214fd81eb7619413a6d0663369be38512))
* **perf:** Optimize array set from get ([#6207](https://github.com/noir-lang/noir/issues/6207)) ([dfeb1c5](https://github.com/noir-lang/noir/commit/dfeb1c51c564ec345978a9a0efef3e4e96ab638a))
* **perf:** Remove inc_rc instructions for arrays which are never mutably borrowed ([#6168](https://github.com/noir-lang/noir/issues/6168)) ([a195442](https://github.com/noir-lang/noir/commit/a19544247fffaf5d2fe0d6d45013f833576f7c61))
* **perf:** Remove redundant inc rc without instructions between ([#6183](https://github.com/noir-lang/noir/issues/6183)) ([be9dcfe](https://github.com/noir-lang/noir/commit/be9dcfe56d808b1bd5ef552d41274705b2df7062))
* **perf:** Remove unused loads in mem2reg and last stores per function ([#5925](https://github.com/noir-lang/noir/issues/5925)) ([19eef30](https://github.com/noir-lang/noir/commit/19eef30cdbd8a3a4671aabbbe66b5481a5dec3f7))
* **perf:** Remove useless paired RC instructions within a block during DIE ([#6160](https://github.com/noir-lang/noir/issues/6160)) ([59c4118](https://github.com/noir-lang/noir/commit/59c41182faa19d1cb8c9be5c11d50636fc17dad7))
* **perf:** Simplify the cfg after DIE ([#6184](https://github.com/noir-lang/noir/issues/6184)) ([a1b5046](https://github.com/noir-lang/noir/commit/a1b50466bfd8c44d50440e00ecb50e29425471e5))
* Pretty print Quoted token stream ([#6111](https://github.com/noir-lang/noir/issues/6111)) ([cd81f85](https://github.com/noir-lang/noir/commit/cd81f85856a477e208533ebd0915b5901c1bb184))
* Refactor SSA passes to run on individual functions ([#6072](https://github.com/noir-lang/noir/issues/6072)) ([85c502c](https://github.com/noir-lang/noir/commit/85c502c9fa69b151fdff1a97b5a97ad78cb599ab))
* Remove aztec macros ([#6087](https://github.com/noir-lang/noir/issues/6087)) ([9d96207](https://github.com/noir-lang/noir/commit/9d962077630131840f0cb7c211f462b579b0b577))
* Remove orphaned blocks from cfg to improve `simplify_cfg` pass. ([#6198](https://github.com/noir-lang/noir/issues/6198)) ([b4712c5](https://github.com/noir-lang/noir/commit/b4712c5ba50ef38789978522afcd251ffbcf8780))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Remove unnecessary branching in keccak impl ([#6133](https://github.com/noir-lang/noir/issues/6133)) ([9c69dce](https://github.com/noir-lang/noir/commit/9c69dce2250b6fc656af8d9c06d7fac34b35c73a))
* Represent assertions more similarly to function calls ([#6103](https://github.com/noir-lang/noir/issues/6103)) ([3ecd0e2](https://github.com/noir-lang/noir/commit/3ecd0e29441d27bc77c49993495209a70be0d86e))
* Show test output when running via LSP ([#6049](https://github.com/noir-lang/noir/issues/6049)) ([9fb010e](https://github.com/noir-lang/noir/commit/9fb010ef8a93cf25e4d361ee42aa8969e5a46bab))
* Simplify sha256 implementation ([#6142](https://github.com/noir-lang/noir/issues/6142)) ([acdfbbc](https://github.com/noir-lang/noir/commit/acdfbbc4ecc9d213dc885a12952e29e188420dff))
* Skip `remove_enable_side_effects` pass on brillig functions ([#6199](https://github.com/noir-lang/noir/issues/6199)) ([2303615](https://github.com/noir-lang/noir/commit/2303615815a2a60de8ac3dd53349f85201660917))
* **ssa:** Simplify signed casts ([#6166](https://github.com/noir-lang/noir/issues/6166)) ([eec3a61](https://github.com/noir-lang/noir/commit/eec3a6152493e56866ec5338ff52f823c530778e))
* Swap endianness in-place in keccak implementation ([#6128](https://github.com/noir-lang/noir/issues/6128)) ([e3cdebe](https://github.com/noir-lang/noir/commit/e3cdebe515e4dc4ee6e16e01bd8af25135939798))
* Syncing TypeVariableKind with Kind ([#6094](https://github.com/noir-lang/noir/issues/6094)) ([6440e18](https://github.com/noir-lang/noir/commit/6440e183085160d77563b4e735ccaaf199e21693))
* Visibility for globals ([#6161](https://github.com/noir-lang/noir/issues/6161)) ([103b54d](https://github.com/noir-lang/noir/commit/103b54db8a5a81ecf76381fe99320c1e1f606898))
* Visibility for modules ([#6165](https://github.com/noir-lang/noir/issues/6165)) ([fcdbcb9](https://github.com/noir-lang/noir/commit/fcdbcb91afb18771cbb5ee48628e171845f22f5f))
* Visibility for traits ([#6056](https://github.com/noir-lang/noir/issues/6056)) ([5bbd9ba](https://github.com/noir-lang/noir/commit/5bbd9ba9a6d6494fd16813b44036b78c871f6613))
* Visibility for type aliases ([#6058](https://github.com/noir-lang/noir/issues/6058)) ([66d2a07](https://github.com/noir-lang/noir/commit/66d2a07f0fedb04422c218cbe8d6fb080efac994))


### Bug Fixes

* (LSP) make goto and hover work well for attributes ([#6152](https://github.com/noir-lang/noir/issues/6152)) ([c679bc6](https://github.com/noir-lang/noir/commit/c679bc6bbd291b6264820dd497b37279116a1cd2))
* Allow macros to change types on each iteration of a comptime loop ([#6105](https://github.com/noir-lang/noir/issues/6105)) ([0864e7c](https://github.com/noir-lang/noir/commit/0864e7c945089cc06f8cc9e5c7d933c465d8c892))
* Allow providing default implementations of unconstrained trait methods ([#6138](https://github.com/noir-lang/noir/issues/6138)) ([7679bbc](https://github.com/noir-lang/noir/commit/7679bbc10cb2fa480489fe1aad83fe77ec2af7e8))
* Always parse all tokens from quoted token streams ([#6064](https://github.com/noir-lang/noir/issues/6064)) ([23ed74b](https://github.com/noir-lang/noir/commit/23ed74bc94ec4da8dbd35da0ae39b26c7ef601e5))
* Be more lenient with semicolons on interned expressions ([#6062](https://github.com/noir-lang/noir/issues/6062)) ([052c4fe](https://github.com/noir-lang/noir/commit/052c4fe52a4df9d6492f9b0d6b449151b87b18d5))
* Consider constants as used values to keep their rc ops ([#6122](https://github.com/noir-lang/noir/issues/6122)) ([1217005](https://github.com/noir-lang/noir/commit/12170056102ea15698aacc820876fee0bb7d0c68))
* Correct stack trace order in comptime assertion failures ([#6066](https://github.com/noir-lang/noir/issues/6066)) ([04f1636](https://github.com/noir-lang/noir/commit/04f1636ca0ccd741c72fa98d6c26227ea9835b0c))
* Databus panic for fns with empty params (https://github.com/AztecProtocol/aztec-packages/pull/8847) ([d252748](https://github.com/noir-lang/noir/commit/d2527482dafef694be2f389e5b4dbc813234da71))
* Decode databus return values ([#6095](https://github.com/noir-lang/noir/issues/6095)) ([c40eb1f](https://github.com/noir-lang/noir/commit/c40eb1fd8a0ba63b2d122e42b47dfa9dca5bf7b0))
* Disable side-effects for no_predicates functions ([#6027](https://github.com/noir-lang/noir/issues/6027)) ([fc74c55](https://github.com/noir-lang/noir/commit/fc74c55ffed892962413c6fe15af62e1d2e7b785))
* Disambiguate field or int static trait method call ([#6112](https://github.com/noir-lang/noir/issues/6112)) ([5b27ea4](https://github.com/noir-lang/noir/commit/5b27ea4d8031318723cc2b97f76758d401a565a0))
* Do not duplicate constant arrays in brillig ([#6155](https://github.com/noir-lang/noir/issues/6155)) ([68f3022](https://github.com/noir-lang/noir/commit/68f3022fcdaab6e379e43091b3242e6ea51cff26))
* **docs:** Rename recursion.md to recursion.mdx ([#6195](https://github.com/noir-lang/noir/issues/6195)) ([054e48b](https://github.com/noir-lang/noir/commit/054e48b76e7b083feb500d30c54912f9db57c565))
* Don't crash on untyped global used as array length ([#6076](https://github.com/noir-lang/noir/issues/6076)) ([426f295](https://github.com/noir-lang/noir/commit/426f2955cbe4f086581d05eea7d06c47e0491195))
* Ensure to_bytes returns the canonical decomposition ([#6084](https://github.com/noir-lang/noir/issues/6084)) ([b280a79](https://github.com/noir-lang/noir/commit/b280a79cf8a4fd2a97200e5436e0ec7cb7134711))
* Error on `&mut x` when `x` is not mutable ([#6037](https://github.com/noir-lang/noir/issues/6037)) ([57afc7d](https://github.com/noir-lang/noir/commit/57afc7ddd424220106af7b9c6e0715007f6ea8b8))
* Fix canonicalization bug ([#6033](https://github.com/noir-lang/noir/issues/6033)) ([7397772](https://github.com/noir-lang/noir/commit/739777214863de4088162711953f26ca992b356e))
* Fix comptime type formatting ([#6079](https://github.com/noir-lang/noir/issues/6079)) ([e678091](https://github.com/noir-lang/noir/commit/e67809165c277423e25110c3f1f8eff6e8daa0e4))
* Handle multi-byte utf8 characters in formatter ([#6118](https://github.com/noir-lang/noir/issues/6118)) ([b1d0619](https://github.com/noir-lang/noir/commit/b1d061926376965805ef3ece3e32d94df81462a6))
* Handle parenthesized expressions in array length ([#6132](https://github.com/noir-lang/noir/issues/6132)) ([9f0b397](https://github.com/noir-lang/noir/commit/9f0b3971ee41e78241cbea4e3f81bac4edd5897d))
* Ignore compression of blocks after msg.len in sha256_var ([#6206](https://github.com/noir-lang/noir/issues/6206)) ([76eec71](https://github.com/noir-lang/noir/commit/76eec710ff73e5e45fdddcd41ae2cd74e879cfa5))
* Infer globals to be u32 when used in a type ([#6083](https://github.com/noir-lang/noir/issues/6083)) ([78262c9](https://github.com/noir-lang/noir/commit/78262c96d5b116c77e50653f9059da60824db812))
* Initialise databus using return values ([#6074](https://github.com/noir-lang/noir/issues/6074)) ([e17dfa5](https://github.com/noir-lang/noir/commit/e17dfa55719f0cfb1080dd25eeda7b70ed44b60d))
* Let LSP suggest fields and methods in LValue chains ([#6051](https://github.com/noir-lang/noir/issues/6051)) ([5bf6567](https://github.com/noir-lang/noir/commit/5bf6567320629835ef6fa7765ca87e9b38ae4c9a))
* Let token pretty printer handle `+=` and similar token sequences ([#6135](https://github.com/noir-lang/noir/issues/6135)) ([684b6cc](https://github.com/noir-lang/noir/commit/684b6cc7deb3ed7ecbb2cea4663e8e9a3ae075f0))
* **mem2reg:** Remove possibility of underflow ([#6107](https://github.com/noir-lang/noir/issues/6107)) ([aea5cc7](https://github.com/noir-lang/noir/commit/aea5cc789ccf4a4d16b1d238d99474f37920b37e))
* Parse a statement as an expression ([#6040](https://github.com/noir-lang/noir/issues/6040)) ([ab203e4](https://github.com/noir-lang/noir/commit/ab203e4ee902b9137519f9a4261ec368d22f0a25))
* Pass radix directly to the blackbox ([#6164](https://github.com/noir-lang/noir/issues/6164)) ([82b89c4](https://github.com/noir-lang/noir/commit/82b89c421da80b719922416d574c1bbaa73d55b4))
* Preserve generic kind on trait methods ([#6099](https://github.com/noir-lang/noir/issues/6099)) ([1df102a](https://github.com/noir-lang/noir/commit/1df102a1ee0eb39dcbada50e10b226c7f7be0f26))
* Prevent check_can_mutate crashing on undefined variable ([#6044](https://github.com/noir-lang/noir/issues/6044)) ([b3accfc](https://github.com/noir-lang/noir/commit/b3accfc99249ccd198051ecb98cf7962af64a629))
* Revert mistaken stack size change ([#6212](https://github.com/noir-lang/noir/issues/6212)) ([a37117a](https://github.com/noir-lang/noir/commit/a37117aca3340447d807c1cf3ca79ba573ceaf8b))
* **ssa:** Check if result of array set is used in value of another array set ([#6197](https://github.com/noir-lang/noir/issues/6197)) ([594ec91](https://github.com/noir-lang/noir/commit/594ec91de55c4cf191d7cdc94a00bb16711cd430))
* **ssa:** RC correctness issue  ([#6134](https://github.com/noir-lang/noir/issues/6134)) ([5b1c896](https://github.com/noir-lang/noir/commit/5b1c896c605ed1047fc17a437e0b58792a778e2d))
* Type variables by default should have Any kind ([#6203](https://github.com/noir-lang/noir/issues/6203)) ([268f2a0](https://github.com/noir-lang/noir/commit/268f2a0240c507646c65c932748d1bdf062d00b1))
* Unify macro result type with actual type ([#6086](https://github.com/noir-lang/noir/issues/6086)) ([af52873](https://github.com/noir-lang/noir/commit/af52873dbec9ab980d17d9ba4336181c006a9a53))
* Update databus in flattening ([#6063](https://github.com/noir-lang/noir/issues/6063)) ([e993da1](https://github.com/noir-lang/noir/commit/e993da1b01aa98deed2af7b5cba2da216fb036a0))


### Miscellaneous Chores

* Removing implicit numeric generics ([#5837](https://github.com/noir-lang/noir/issues/5837)) ([eda9043](https://github.com/noir-lang/noir/commit/eda904328b269b5926f8a82ab82e52a485903bbe))

## [0.34.0](https://github.com/noir-lang/noir/compare/v0.33.0...v0.34.0) (2024-09-13)


### ⚠ BREAKING CHANGES

* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)
* Do not encode assertion strings in the programs (https://github.com/AztecProtocol/aztec-packages/pull/8315)
* return arrays instead of slices from `to_be_radix` functions ([#5851](https://github.com/noir-lang/noir/issues/5851))
* Check unused generics are bound ([#5840](https://github.com/noir-lang/noir/issues/5840))

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* (LSP) suggest names that match any part of the current prefix ([#5752](https://github.com/noir-lang/noir/issues/5752)) ([cb0d490](https://github.com/noir-lang/noir/commit/cb0d49017a3b592afc2002e592a61d33bf3ac3a4))
* `Module::add_item` ([#5947](https://github.com/noir-lang/noir/issues/5947)) ([af50a7b](https://github.com/noir-lang/noir/commit/af50a7b3ad511de68c584e65ec4eec8b703bbc14))
* Add `Expr::as_any_integer` and `Expr::as_member_access` ([#5742](https://github.com/noir-lang/noir/issues/5742)) ([6266755](https://github.com/noir-lang/noir/commit/626675567bb0bfff3c7984ed7f75c488e441ef98))
* Add `Expr::as_array`, `Expr::as_repeated_element_array` and same for slice ([#5750](https://github.com/noir-lang/noir/issues/5750)) ([f44e0b3](https://github.com/noir-lang/noir/commit/f44e0b3ebfb30e9323ebf2d537830ea64d59488c))
* Add `Expr::as_assert_eq` ([#5880](https://github.com/noir-lang/noir/issues/5880)) ([88f7858](https://github.com/noir-lang/noir/commit/88f785803ddb1a7d395a899b65e500e46bba1a5d))
* Add `Expr::as_assert` ([#5857](https://github.com/noir-lang/noir/issues/5857)) ([4e4ad26](https://github.com/noir-lang/noir/commit/4e4ad26d56e6a487ca446ea4e1732c6af04e1410))
* Add `Expr::as_binary_op` ([#5734](https://github.com/noir-lang/noir/issues/5734)) ([73a9f51](https://github.com/noir-lang/noir/commit/73a9f51e1fd1ba513ef721e07990abf510e8bf01))
* Add `Expr::as_block` and `Expr::has_semicolon` ([#5784](https://github.com/noir-lang/noir/issues/5784)) ([19ffa20](https://github.com/noir-lang/noir/commit/19ffa2008fc9cbb5972b50d66d14908d5c82ed75))
* Add `Expr::as_bool` ([#5729](https://github.com/noir-lang/noir/issues/5729)) ([ca75cc2](https://github.com/noir-lang/noir/commit/ca75cc2e35530c82cef3b86edf99a232f88b11e8))
* Add `Expr::as_cast` and `UnresolvedType::is_field` ([#5801](https://github.com/noir-lang/noir/issues/5801)) ([c9aa50d](https://github.com/noir-lang/noir/commit/c9aa50dd25887a7e8b903515a0fd290335d1e572))
* Add `Expr::as_let` ([#5964](https://github.com/noir-lang/noir/issues/5964)) ([65da598](https://github.com/noir-lang/noir/commit/65da5983ece16249fa939a493f197d13fbb1f9a4))
* Add `Expr::as_unary` ([#5731](https://github.com/noir-lang/noir/issues/5731)) ([ae33811](https://github.com/noir-lang/noir/commit/ae33811f7ca770b54880d0095c1d5be0ee85c6e4))
* Add `Expr::resolve` and `TypedExpr::as_function_definition` ([#5859](https://github.com/noir-lang/noir/issues/5859)) ([bceee55](https://github.com/noir-lang/noir/commit/bceee55cc3833978d120e194820cfae9132c8006))
* Add `Expr` methods: `as_tuple`, `as_parenthesized`, `as_index`, `as_if` ([#5726](https://github.com/noir-lang/noir/issues/5726)) ([f57a7b2](https://github.com/noir-lang/noir/commit/f57a7b2bd4457cbbfd650c7467d1f96d65ea6c8b))
* Add `Expr` methods: as_comptime, as_unsafe, is_break, is_continue ([#5799](https://github.com/noir-lang/noir/issues/5799)) ([619fa5c](https://github.com/noir-lang/noir/commit/619fa5c0ad115ac910abfc9995a4362271847d59))
* Add `fmtstr::contents` ([#5928](https://github.com/noir-lang/noir/issues/5928)) ([f18e9ca](https://github.com/noir-lang/noir/commit/f18e9ca86c025f736af6e515f812e36fbb622930))
* Add `FunctionDef::body` ([#5825](https://github.com/noir-lang/noir/issues/5825)) ([39b30ba](https://github.com/noir-lang/noir/commit/39b30ba2e9f13d8d99bfb1833e14e294f80773e5))
* Add `FunctionDef::has_named_attribute` ([#5870](https://github.com/noir-lang/noir/issues/5870)) ([a950195](https://github.com/noir-lang/noir/commit/a950195baa9e6ed3880ad1d2f619e442b4c49473))
* Add `FunctionDef::set_return_visibility` ([#5941](https://github.com/noir-lang/noir/issues/5941)) ([8beda6b](https://github.com/noir-lang/noir/commit/8beda6beb10a2e42da788bcc9bf2b375055675c6))
* Add `FunctionDefinition::add_attribute` ([#5944](https://github.com/noir-lang/noir/issues/5944)) ([c7479c4](https://github.com/noir-lang/noir/commit/c7479c4e55f47f7c652f0e202636b9e590d11f5d))
* Add `FunctionDefinition::module` and `StructDefinition::module` ([#5956](https://github.com/noir-lang/noir/issues/5956)) ([f19344c](https://github.com/noir-lang/noir/commit/f19344ca1a6d9ae78cd433864f71705f3381320f))
* Add `FunctionDefinition` methods `is_unconstrained` and `set_unconstrained` ([#5962](https://github.com/noir-lang/noir/issues/5962)) ([b9a072d](https://github.com/noir-lang/noir/commit/b9a072d29c0f4abc4c6c683b9b2a872728d971fa))
* Add `Module::structs` ([#6017](https://github.com/noir-lang/noir/issues/6017)) ([fc5bb02](https://github.com/noir-lang/noir/commit/fc5bb025d7df901050af1d8ad6ebb9283faf641f))
* Add `Quoted::as_expr` and `Expr::as_function_call` ([#5708](https://github.com/noir-lang/noir/issues/5708)) ([3f79607](https://github.com/noir-lang/noir/commit/3f79607002a75880b6e21aadd15dd7e55f15dbfa))
* Add `Quoted::tokens` ([#5942](https://github.com/noir-lang/noir/issues/5942)) ([a297ec6](https://github.com/noir-lang/noir/commit/a297ec643eb3b6c0e8bcf62abdc005414283c7c2))
* Add `std::meta::typ::fresh_type_variable` ([#5948](https://github.com/noir-lang/noir/issues/5948)) ([3dab4dd](https://github.com/noir-lang/noir/commit/3dab4dd771b7d8b9242ce3a9aeff5770f4d85cf6))
* Add `StructDefinition::add_attribute` and `has_named_attribute` ([#5945](https://github.com/noir-lang/noir/issues/5945)) ([344dd5e](https://github.com/noir-lang/noir/commit/344dd5ea7ed551dcc3fd414d1c5f49f44721c28c))
* Add `StructDefinition::add_generic` ([#5961](https://github.com/noir-lang/noir/issues/5961)) ([6004067](https://github.com/noir-lang/noir/commit/6004067e42572c34dd6465e66d36410826e2fd90))
* Add `StructDefinition::name` ([#5960](https://github.com/noir-lang/noir/issues/5960)) ([102ebe3](https://github.com/noir-lang/noir/commit/102ebe33694d65e1024fcba8260ada6f30c49578))
* Add `StructDefinition::set_fields` ([#5931](https://github.com/noir-lang/noir/issues/5931)) ([9d2629d](https://github.com/noir-lang/noir/commit/9d2629dd1bb28a8c2ecb4c33d26119da75d626c2))
* Add `TraitImpl::trait_generic_args` and `TraitImpl::methods` ([#5722](https://github.com/noir-lang/noir/issues/5722)) ([8c7e493](https://github.com/noir-lang/noir/commit/8c7e4937b24e6d782543dd42ac9fc293af550f7c))
* Add `Type::as_string` ([#5871](https://github.com/noir-lang/noir/issues/5871)) ([e29d4b3](https://github.com/noir-lang/noir/commit/e29d4b3646f0527fc01bc4584ee33616db922c72))
* Add `Type::get_trait_impl` ([#5716](https://github.com/noir-lang/noir/issues/5716)) ([eb33d1c](https://github.com/noir-lang/noir/commit/eb33d1cae626244a220e6ceea176be6f5fb1073d))
* Add `Type::implements` ([#5701](https://github.com/noir-lang/noir/issues/5701)) ([2166c94](https://github.com/noir-lang/noir/commit/2166c9441c739ab6a3ee029ed051f1857bd27170))
* Add `TypedExpr::get_type` ([#5992](https://github.com/noir-lang/noir/issues/5992)) ([31f50c4](https://github.com/noir-lang/noir/commit/31f50c442b59eac4de2c5c530278e345bd2f149f))
* Add `UnresolvedType::is_field` and `Expr::as_assign` ([#5804](https://github.com/noir-lang/noir/issues/5804)) ([c45df4e](https://github.com/noir-lang/noir/commit/c45df4e83ab1ff5f6c35c4115aebf317110ee419))
* Add `unsafe` blocks for calling unconstrained code from constrained functions ([#4429](https://github.com/noir-lang/noir/issues/4429)) ([79593b4](https://github.com/noir-lang/noir/commit/79593b4235efc031ed9b95c0b301cef66b4ab88c))
* Add a `panic` method to the stdlib ([#5966](https://github.com/noir-lang/noir/issues/5966)) ([b86c2bc](https://github.com/noir-lang/noir/commit/b86c2bc0ec2712e9c24309a6f5e92afc3ef0a2dc))
* Add array_to_str_lossy ([#5613](https://github.com/noir-lang/noir/issues/5613)) ([af5acf4](https://github.com/noir-lang/noir/commit/af5acf4eb4af38fd346b6365a45d8e7e83899542))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Expr::as_method_call ([#5822](https://github.com/noir-lang/noir/issues/5822)) ([806af24](https://github.com/noir-lang/noir/commit/806af24e44b3abcc50e552fff0883f2497ba152f))
* Add mutating FunctionDefinition functions ([#5685](https://github.com/noir-lang/noir/issues/5685)) ([2882eae](https://github.com/noir-lang/noir/commit/2882eaeb176988bb3d216d091c0e239f5b80f276))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add some `Module` comptime functions ([#5684](https://github.com/noir-lang/noir/issues/5684)) ([eefd69b](https://github.com/noir-lang/noir/commit/eefd69b1d72a9f5cb2e7bbd3e554925a7670a2f3))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Allow inserting new structs and into programs from attributes ([#5927](https://github.com/noir-lang/noir/issues/5927)) ([94e661e](https://github.com/noir-lang/noir/commit/94e661e7520d80496bdc9da39b9736bafacb96dc))
* Arithmetic Generics ([#5950](https://github.com/noir-lang/noir/issues/5950)) ([00a79ce](https://github.com/noir-lang/noir/commit/00a79ce6374bb09616ffb6f431cb6c011d786877))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Better error message for misplaced doc comments ([#5990](https://github.com/noir-lang/noir/issues/5990)) ([28415ef](https://github.com/noir-lang/noir/commit/28415efd2fd8c7b836516b154ab54d65f15fbc23))
* Better println for Quoted ([#5896](https://github.com/noir-lang/noir/issues/5896)) ([6f30e42](https://github.com/noir-lang/noir/commit/6f30e42f8a895c7813e770d6ee9ffbc9977c335b))
* Calculate `FunctionSelector`s and `EventSelector`s during comptime (https://github.com/AztecProtocol/aztec-packages/pull/8354) ([33bd102](https://github.com/noir-lang/noir/commit/33bd102d6021912b56fe880efab65346c3ea9228))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Check argument count and types on attribute function callback ([#5921](https://github.com/noir-lang/noir/issues/5921)) ([91f693d](https://github.com/noir-lang/noir/commit/91f693d81edb1913bf56d2c1038441cec5844646))
* Do not encode assertion strings in the programs (https://github.com/AztecProtocol/aztec-packages/pull/8315) ([4144152](https://github.com/noir-lang/noir/commit/41441527700d7c0fe59769803048a3b285badd77))
* Explicit Associated Types & Constants ([#5739](https://github.com/noir-lang/noir/issues/5739)) ([e050e93](https://github.com/noir-lang/noir/commit/e050e93a963b407dabedf7c236f59c387f787514))
* Extract brillig slice ops to reusable procedures ([#6002](https://github.com/noir-lang/noir/issues/6002)) ([339c17b](https://github.com/noir-lang/noir/commit/339c17bb5253f0d290fa56644a49b2881c9de889))
* Fault-tolerant parsing of `fn` and `impl` ([#5753](https://github.com/noir-lang/noir/issues/5753)) ([d4e2f0a](https://github.com/noir-lang/noir/commit/d4e2f0a30b07a98772fbc321a760641466cc01d1))
* Format trait impl functions ([#6016](https://github.com/noir-lang/noir/issues/6016)) ([da32bd8](https://github.com/noir-lang/noir/commit/da32bd82d749a9c388e970883cc1ea756ce2db6b))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Impl Hash and Eq on more comptime types ([#6022](https://github.com/noir-lang/noir/issues/6022)) ([114903d](https://github.com/noir-lang/noir/commit/114903d6fbcb035b46478db36d696efa99e919d6))
* Implement `str_as_bytes` in the `comptime` interpreter ([#5887](https://github.com/noir-lang/noir/issues/5887)) ([45344bf](https://github.com/noir-lang/noir/commit/45344bfe1148a2f592c2e432744d3fb3d46340cc))
* Implement LSP code action "Implement missing members" ([#6020](https://github.com/noir-lang/noir/issues/6020)) ([9bf2dcb](https://github.com/noir-lang/noir/commit/9bf2dcbf166f9ffd97c369c0de3d95329c850d47))
* Improve "type annotations needed" errors ([#5830](https://github.com/noir-lang/noir/issues/5830)) ([90f9ea0](https://github.com/noir-lang/noir/commit/90f9ea0df7055aa5881a77981c8be7862478c848))
* Let `has_named_attribute` work for built-in attributes ([#6024](https://github.com/noir-lang/noir/issues/6024)) ([a09646b](https://github.com/noir-lang/noir/commit/a09646bde7ae27c1aa423ef56757d2fb8753658a))
* Let `nargo` and LSP work well in the stdlib ([#5969](https://github.com/noir-lang/noir/issues/5969)) ([8e8e97c](https://github.com/noir-lang/noir/commit/8e8e97c68e48245a6c7de9b3a0fe9960a889c47a))
* Liveness analysis for constants (https://github.com/AztecProtocol/aztec-packages/pull/8294) ([71e1556](https://github.com/noir-lang/noir/commit/71e1556717695e1ef80c53d273f7acbdf0d5b4e7))
* LSP auto-import completion ([#5741](https://github.com/noir-lang/noir/issues/5741)) ([cdbb940](https://github.com/noir-lang/noir/commit/cdbb940a883ae32dd84c667ec06b0d155f2d7520))
* LSP autocomplete constructor fields ([#5732](https://github.com/noir-lang/noir/issues/5732)) ([e71c75a](https://github.com/noir-lang/noir/commit/e71c75a0862dda26e5b08318bcec71d5b41ba9e9))
* LSP autocompletion for attributes ([#5963](https://github.com/noir-lang/noir/issues/5963)) ([b7b9e3f](https://github.com/noir-lang/noir/commit/b7b9e3f2212db2b9c3412ddcfd1c40c6200a1740))
* LSP autocompletion for use statement ([#5704](https://github.com/noir-lang/noir/issues/5704)) ([226aeb1](https://github.com/noir-lang/noir/commit/226aeb1400adc6d9028e9ad9f496783606fd9e11))
* LSP code action "Fill struct fields" ([#5885](https://github.com/noir-lang/noir/issues/5885)) ([1e6e4f4](https://github.com/noir-lang/noir/commit/1e6e4f4f53c7d331c054dd84f3fe6064d2e844e3))
* LSP code actions to import or qualify unresolved paths ([#5876](https://github.com/noir-lang/noir/issues/5876)) ([410c1f6](https://github.com/noir-lang/noir/commit/410c1f67ee93634bcfb22b236035d97eee33b0cf))
* LSP completion function detail ([#5993](https://github.com/noir-lang/noir/issues/5993)) ([e84f7d2](https://github.com/noir-lang/noir/commit/e84f7d2e81c1f59e9af015f38c2d477607a9c558))
* LSP completion now works better in the middle of idents ([#5795](https://github.com/noir-lang/noir/issues/5795)) ([1c84038](https://github.com/noir-lang/noir/commit/1c84038e4a1b2515f4f91aca4c833dd3b6c05d91))
* LSP diagnostics for all package files ([#5895](https://github.com/noir-lang/noir/issues/5895)) ([4e616b3](https://github.com/noir-lang/noir/commit/4e616b340d144a615795e37ab87ced1d175188b3))
* LSP diagnostics now have "unnecessary" and "deprecated" tags ([#5878](https://github.com/noir-lang/noir/issues/5878)) ([2f0d4e0](https://github.com/noir-lang/noir/commit/2f0d4e017b701b46b5c675e3b34af15ad6f28823))
* LSP fields, functions and methods completion after "." and "::" ([#5714](https://github.com/noir-lang/noir/issues/5714)) ([13c1fe6](https://github.com/noir-lang/noir/commit/13c1fe686c51b762df71a138b1af474d67da7560))
* LSP hover and go-to-definition for crates ([#5786](https://github.com/noir-lang/noir/issues/5786)) ([86d8840](https://github.com/noir-lang/noir/commit/86d884044ee5bac72af820d623e00e1375271845))
* LSP now suggests self fields and methods ([#5955](https://github.com/noir-lang/noir/issues/5955)) ([f57ce85](https://github.com/noir-lang/noir/commit/f57ce850fdb42a33177638f2f4af1335023c5e62))
* LSP path completion ([#5712](https://github.com/noir-lang/noir/issues/5712)) ([3c6b998](https://github.com/noir-lang/noir/commit/3c6b9982048e168fc86cb834b5e8e72b51d2498d))
* LSP signature help ([#5725](https://github.com/noir-lang/noir/issues/5725)) ([5a3d241](https://github.com/noir-lang/noir/commit/5a3d24192d440c5bfe3749d4bcd8ebbc9cf4902b))
* LSP signature help for assert and assert_eq ([#5862](https://github.com/noir-lang/noir/issues/5862)) ([663e00c](https://github.com/noir-lang/noir/commit/663e00cffcb2cd66ddc2b33c0453afca0e15f703))
* LSP will now suggest private items if they are visible ([#5923](https://github.com/noir-lang/noir/issues/5923)) ([d2caa5b](https://github.com/noir-lang/noir/commit/d2caa5bb86f944d6d09182482bef6e35ca2213d6))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **meta:** Comptime keccak ([#5854](https://github.com/noir-lang/noir/issues/5854)) ([0e8becc](https://github.com/noir-lang/noir/commit/0e8becc7bccee2ae4e4e3ef373df08c3e9ef88c9))
* Module attributes ([#5888](https://github.com/noir-lang/noir/issues/5888)) ([2ca2e5c](https://github.com/noir-lang/noir/commit/2ca2e5cf207a2a1f41ca86d877f0288bcbbfd212))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Only check array bounds in brillig if index is unsafe ([#5938](https://github.com/noir-lang/noir/issues/5938)) ([8b60bbc](https://github.com/noir-lang/noir/commit/8b60bbc8082513e29f6573e5235e0a33fdd1517b))
* **optimization:** Avoid merging identical (by ID) arrays ([#5853](https://github.com/noir-lang/noir/issues/5853)) ([062103e](https://github.com/noir-lang/noir/commit/062103ea039042e8e999b29dbb1fafc3cebd513c))
* **optimization:** Follow past `array_set`s when optimizing `array_get`s ([#5772](https://github.com/noir-lang/noir/issues/5772)) ([090501d](https://github.com/noir-lang/noir/commit/090501dfaf7c569b1aa944856bf68ad663572ae4))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **perf:** Mem2reg function state for value loads to optimize across blocks ([#5757](https://github.com/noir-lang/noir/issues/5757)) ([0b297b3](https://github.com/noir-lang/noir/commit/0b297b3830ac26551bfb39fad01d74cd8ab341c3))
* **perf:** Remove known store values that equal the store address in mem2reg ([#5935](https://github.com/noir-lang/noir/issues/5935)) ([b84009c](https://github.com/noir-lang/noir/commit/b84009ca428a5790acf53a6c027146b706170574))
* **perf:** Remove last store in return block if last load is before that store ([#5910](https://github.com/noir-lang/noir/issues/5910)) ([1737b65](https://github.com/noir-lang/noir/commit/1737b656c861706c38b59bd5ef6cd095687a2898))
* **perf:** Simplify poseidon2 cache zero-pad ([#5869](https://github.com/noir-lang/noir/issues/5869)) ([31e9be6](https://github.com/noir-lang/noir/commit/31e9be6b83b448eb6834645dc124589dc724a7b2))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Remove blocks which consist of only a jump to another block ([#5889](https://github.com/noir-lang/noir/issues/5889)) ([f391af2](https://github.com/noir-lang/noir/commit/f391af2d61f4a38e02cb92c76fa4c2c148af3833))
* Remove unnecessary copying of vector size during reversal ([#5852](https://github.com/noir-lang/noir/issues/5852)) ([5739904](https://github.com/noir-lang/noir/commit/5739904f8d9e6c00d9e140cd4926b4d149412476))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Return arrays instead of slices from `to_be_radix` functions ([#5851](https://github.com/noir-lang/noir/issues/5851)) ([d59c708](https://github.com/noir-lang/noir/commit/d59c7087495f8af0dfb387dc587ecc422888096b))
* Show backtrace on comptime assertion failures ([#5842](https://github.com/noir-lang/noir/issues/5842)) ([cfd68d4](https://github.com/noir-lang/noir/commit/cfd68d4c1bd1a2319698fca99d200a5d86ffa771))
* Show doc comments in LSP ([#5968](https://github.com/noir-lang/noir/issues/5968)) ([45f4ae0](https://github.com/noir-lang/noir/commit/45f4ae09ca5fa5516e13c34c2ae9379077461cc9))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Suggest trait methods in LSP completion ([#5735](https://github.com/noir-lang/noir/issues/5735)) ([e2f7e95](https://github.com/noir-lang/noir/commit/e2f7e950c44883228d5e1230b04c83e479de7ed0))
* Suggest tuple fields in LSP completion ([#5730](https://github.com/noir-lang/noir/issues/5730)) ([64d7d78](https://github.com/noir-lang/noir/commit/64d7d786ad2ddf0942690912cf05ca3b438c43be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8314) ([4144152](https://github.com/noir-lang/noir/commit/41441527700d7c0fe59769803048a3b285badd77))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8333) ([33bd102](https://github.com/noir-lang/noir/commit/33bd102d6021912b56fe880efab65346c3ea9228))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Unquote some value as tokens, not as unquote markers ([#5924](https://github.com/noir-lang/noir/issues/5924)) ([70ebb90](https://github.com/noir-lang/noir/commit/70ebb905da23a0541915a8f6883d6f530934be4e))
* Use visibility ([#5856](https://github.com/noir-lang/noir/issues/5856)) ([e349f30](https://github.com/noir-lang/noir/commit/e349f30b60a473e2068afafb6fae4a4ea50d185b))
* Use Zac's quicksort algorithm in stdlib sorting ([#5940](https://github.com/noir-lang/noir/issues/5940)) ([19f5757](https://github.com/noir-lang/noir/commit/19f5757a64c15a6b5a9478eceedb17c02d2351d7))
* User `super::` in LSP autocompletion if possible ([#5751](https://github.com/noir-lang/noir/issues/5751)) ([5192e53](https://github.com/noir-lang/noir/commit/5192e537708fc9ec51f53bb6a6629c9d682532d5))
* Warn on unused functions ([#5892](https://github.com/noir-lang/noir/issues/5892)) ([af3db4b](https://github.com/noir-lang/noir/commit/af3db4bf2e8f7feba6d06c3095d7cdf17c8dde75))
* Warn on unused imports ([#5847](https://github.com/noir-lang/noir/issues/5847)) ([58f855e](https://github.com/noir-lang/noir/commit/58f855ec2124db39e5b2b08630d514d852d0e7df))


### Bug Fixes

* (LSP) only add cached files relevant to workspace ([#5775](https://github.com/noir-lang/noir/issues/5775)) ([1958a79](https://github.com/noir-lang/noir/commit/1958a7932642e2fa556a903a3186b142a70e3e48))
* **acir_gen:** Nested dynamic array initialization ([#5810](https://github.com/noir-lang/noir/issues/5810)) ([4df53ad](https://github.com/noir-lang/noir/commit/4df53adfd0c5e2da70462b29fbf8d08e32203fc4))
* **acvm:** Clear ACIR call stack after successful circuit execution ([#5783](https://github.com/noir-lang/noir/issues/5783)) ([656a7d6](https://github.com/noir-lang/noir/commit/656a7d6c1e0c3597a61c3606e3155a70032c1599))
* Add locations to most SSA instructions ([#5697](https://github.com/noir-lang/noir/issues/5697)) ([85d5c85](https://github.com/noir-lang/noir/commit/85d5c8532acb21c39f3db466982039d1415d9300))
* Add missing trait impls for integer types to stdlib ([#5738](https://github.com/noir-lang/noir/issues/5738)) ([d3f20c6](https://github.com/noir-lang/noir/commit/d3f20c6f830a84fce9d75ce3fe28e31b391b47ab))
* Allow comptime code to use break without also being `unconstrained` ([#5744](https://github.com/noir-lang/noir/issues/5744)) ([c2a1a87](https://github.com/noir-lang/noir/commit/c2a1a87a6bcfc161ef5f550a17b603b0bccbab8e))
* Always place module attribute generated items inside module ([#5943](https://github.com/noir-lang/noir/issues/5943)) ([89ac6e0](https://github.com/noir-lang/noir/commit/89ac6e087debc37dcc729db0b68062418cd64d2e))
* Bit shifting type checking ([#5824](https://github.com/noir-lang/noir/issues/5824)) ([fb5136e](https://github.com/noir-lang/noir/commit/fb5136edda4b5b8ac6bba998939c94f11a27a59a))
* Check unused generics are bound ([#5840](https://github.com/noir-lang/noir/issues/5840)) ([82eb158](https://github.com/noir-lang/noir/commit/82eb1581251faa9716d762a673fa1b871b3e7be2))
* Collect functions generated by attributes ([#5930](https://github.com/noir-lang/noir/issues/5930)) ([2c22fe5](https://github.com/noir-lang/noir/commit/2c22fe555dc41fffc623026b4b8c57d44b869cd2))
* Correctly print string tokens ([#6021](https://github.com/noir-lang/noir/issues/6021)) ([b8a3a9b](https://github.com/noir-lang/noir/commit/b8a3a9b03f83bba486d2623640f97f1a080f2d73))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Do not use predicate for index in array operation, when the index is safe ([#5779](https://github.com/noir-lang/noir/issues/5779)) ([9d8f2bd](https://github.com/noir-lang/noir/commit/9d8f2bd759837d7f1f78c1b56b8e30de35c80867))
* **docs:** Fix file paths for metaprogramming docs ([#5826](https://github.com/noir-lang/noir/issues/5826)) ([a764c5b](https://github.com/noir-lang/noir/commit/a764c5be9b15e499e0720f28a1a177bfecbef352))
* Error when `quote` is used in runtime code ([#5978](https://github.com/noir-lang/noir/issues/5978)) ([cc30d88](https://github.com/noir-lang/noir/commit/cc30d88d85bb70248e452d9ec549d6dfe6be62ff))
* Error when comptime functions are used in runtime code ([#5976](https://github.com/noir-lang/noir/issues/5976)) ([ec24917](https://github.com/noir-lang/noir/commit/ec24917bfda55746c7509dd28f8d808f97c948b8))
* Error when comptime types are used in runtime code ([#5987](https://github.com/noir-lang/noir/issues/5987)) ([3d39196](https://github.com/noir-lang/noir/commit/3d39196040aa01e64c8a7fe989e2979a5de80023))
* Error when mutating comptime variables in non-comptime code ([#6003](https://github.com/noir-lang/noir/issues/6003)) ([e20c44d](https://github.com/noir-lang/noir/commit/e20c44dcb21edd3ec2bbc015d85754872e86740e))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Fix some mistakes in arithmetic generics docs ([#5999](https://github.com/noir-lang/noir/issues/5999)) ([29550d1](https://github.com/noir-lang/noir/commit/29550d1d7698a1af65b867171ff80e817f3ed2f6))
* Fix using lazily elaborated comptime globals ([#5995](https://github.com/noir-lang/noir/issues/5995)) ([f6f493c](https://github.com/noir-lang/noir/commit/f6f493cb73e24337a7f11507b2b492d98cac2ada))
* **frontend:** Ban type vars bound to a reference from passing the unconstrained boundary  ([#5949](https://github.com/noir-lang/noir/issues/5949)) ([ce34fbd](https://github.com/noir-lang/noir/commit/ce34fbd19702b71426563a589235a2c5a1efb265))
* **frontend:** Continue type check if we are missing an unsafe block ([#5720](https://github.com/noir-lang/noir/issues/5720)) ([86de991](https://github.com/noir-lang/noir/commit/86de991051a34567077076aa09a85b26eeff2ab2))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Help link was outdated ([#6004](https://github.com/noir-lang/noir/issues/6004)) ([d1e52f3](https://github.com/noir-lang/noir/commit/d1e52f3f3824ead1fd617fc21fcbe1051911986d))
* Honor function visibility in LSP completion ([#5809](https://github.com/noir-lang/noir/issues/5809)) ([335de05](https://github.com/noir-lang/noir/commit/335de054dfcda366df50cc215900910ebdc8be63))
* Let `derive(Eq)` work for empty structs ([#5965](https://github.com/noir-lang/noir/issues/5965)) ([ff8e8b5](https://github.com/noir-lang/noir/commit/ff8e8b5fae4db57bd7f819d0e23c68262057b790))
* Let LSP autocompletion work in more contexts ([#5719](https://github.com/noir-lang/noir/issues/5719)) ([03ba6dd](https://github.com/noir-lang/noir/commit/03ba6dd328d56bf71c9e2b501c59eb9a6cdb95db))
* LSP document symbol didn't work for primitive impls ([#5970](https://github.com/noir-lang/noir/issues/5970)) ([e1f81da](https://github.com/noir-lang/noir/commit/e1f81da1d8cfcc9cfe3d1bd2ed6f762580800ad9))
* **mem2reg:** Handle aliases better when setting a known value for a load ([#5959](https://github.com/noir-lang/noir/issues/5959)) ([1b72a17](https://github.com/noir-lang/noir/commit/1b72a17e621465ac1dfaaf8948edcebd4f1b0b15))
* **mem2reg:** Handle aliases in function last store cleanup and additional alias unit test ([#5967](https://github.com/noir-lang/noir/issues/5967)) ([36756e8](https://github.com/noir-lang/noir/commit/36756e8757ad40e2b231747ed754273f50e5dc2f))
* **nargo:** Resolve Brillig assertion payloads ([#5872](https://github.com/noir-lang/noir/issues/5872)) ([f53a28b](https://github.com/noir-lang/noir/commit/f53a28bd3e70e9331e01f1fec4984e747723df74))
* Prevent comptime println from crashing LSP ([#5918](https://github.com/noir-lang/noir/issues/5918)) ([44cf9a2](https://github.com/noir-lang/noir/commit/44cf9a2140bc06b550d4b46966f1637598ac11a7))
* Replace unused ArrayGet/Set with constrain if possibly out of bounds ([#5691](https://github.com/noir-lang/noir/issues/5691)) ([a87d926](https://github.com/noir-lang/noir/commit/a87d92629c49c91d47685dba9a2a6dce4440756d))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* **sha256:** Add extra checks against message size when constructing msg blocks ([#5861](https://github.com/noir-lang/noir/issues/5861)) ([46e266a](https://github.com/noir-lang/noir/commit/46e266a5229dada42ee397beb0d39322451b1458))
* **sha256:** Fix upper bound when building msg block and delay final block compression under certain cases  ([#5838](https://github.com/noir-lang/noir/issues/5838)) ([130b7b6](https://github.com/noir-lang/noir/commit/130b7b6871ad165a75df5fa5760c94a7402521f4))
* **sha256:** Perform compression per block and utilize ROM instead of RAM when setting up the message block ([#5760](https://github.com/noir-lang/noir/issues/5760)) ([c52dc1c](https://github.com/noir-lang/noir/commit/c52dc1c77aedf5a876a858cc5a942c29e868e9e6))
* Suggest trait attributes in LSP ([#5972](https://github.com/noir-lang/noir/issues/5972)) ([d6f60d7](https://github.com/noir-lang/noir/commit/d6f60d70dc41640ad84f7a968927b20818bcaf2a))
* Support debug comptime flag for attributes ([#5929](https://github.com/noir-lang/noir/issues/5929)) ([34f21c0](https://github.com/noir-lang/noir/commit/34f21c0eadfc8a03f5177d72de7958903de8ac98))
* Temporary register leaks in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/8350) ([33bd102](https://github.com/noir-lang/noir/commit/33bd102d6021912b56fe880efab65346c3ea9228))
* Try to move constant terms to one side for arithmetic generics ([#6008](https://github.com/noir-lang/noir/issues/6008)) ([4d8fe28](https://github.com/noir-lang/noir/commit/4d8fe28f6d0930b6e9cfe0d39dd003466b20b8b6))
* Unconstrained fn mismatch is now a warning ([#5764](https://github.com/noir-lang/noir/issues/5764)) ([37af966](https://github.com/noir-lang/noir/commit/37af966024d5eb38eae5092a7976445e4bbe8adb))
* Use element_size() instead of computing it with division ([#5939](https://github.com/noir-lang/noir/issues/5939)) ([6a45007](https://github.com/noir-lang/noir/commit/6a450076be2889c05428ea1285c5c149cfaf4456))
* Use module name as line after which we'll insert auto-import ([#6025](https://github.com/noir-lang/noir/issues/6025)) ([c2e4a9a](https://github.com/noir-lang/noir/commit/c2e4a9a02c0138f6a8878f51291320ba7e57c79c))

## [0.33.0](https://github.com/noir-lang/noir/compare/v0.32.0...v0.33.0) (2024-08-06)


### ⚠ BREAKING CHANGES

* parse block and if statements independently of expressions in statements ([#5634](https://github.com/noir-lang/noir/issues/5634))
* **frontend:** Restrict numeric generic types to unsigned ints up to `u32` ([#5581](https://github.com/noir-lang/noir/issues/5581))

### Features

* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add `FunctionDefinition::parameters`, `FunctionDefinition::return_type` and `impl Eq for Quoted` ([#5681](https://github.com/noir-lang/noir/issues/5681)) ([d52fc05](https://github.com/noir-lang/noir/commit/d52fc056ae5dec4457dd869d50dd9a6e5b0b5cb1))
* Add `std::meta::type_of` and `impl Eq for Type` ([#5669](https://github.com/noir-lang/noir/issues/5669)) ([0503956](https://github.com/noir-lang/noir/commit/05039568e317c16cecc173717e63288e7ca3890c))
* Add `TraitDefinition::as_trait_constraint()` ([#5541](https://github.com/noir-lang/noir/issues/5541)) ([0943223](https://github.com/noir-lang/noir/commit/094322381da67e2b9aef27b5558ba98a47abfccc))
* Add `Type::as_struct` ([#5680](https://github.com/noir-lang/noir/issues/5680)) ([ade69a9](https://github.com/noir-lang/noir/commit/ade69a9e5f1546249e9b43b40e9ff0da87c4632e))
* Add `Type::is_field` and `Type::as_integer` ([#5670](https://github.com/noir-lang/noir/issues/5670)) ([939357a](https://github.com/noir-lang/noir/commit/939357acae87eae833ff10572f7a1ad52b35d94e))
* Add `Type` methods: `as_tuple`, `as_slice`, `as_array`, `as_constant`, `is_bool` ([#5678](https://github.com/noir-lang/noir/issues/5678)) ([604fa0d](https://github.com/noir-lang/noir/commit/604fa0d85c1a0c47c7c0b8402576afb61ceb664b))
* Add a compile-time hash map type ([#5543](https://github.com/noir-lang/noir/issues/5543)) ([c6e5c4b](https://github.com/noir-lang/noir/commit/c6e5c4b304eb4e87ca519151f610e7e03b1a4fba))
* Add a limited form of arithmetic on generics ([#5625](https://github.com/noir-lang/noir/issues/5625)) ([0afb680](https://github.com/noir-lang/noir/commit/0afb6805e3c3cbe0f84f39d9e2e57e038450547d))
* Add parameter to call_data attribute ([#5599](https://github.com/noir-lang/noir/issues/5599)) ([e8bb341](https://github.com/noir-lang/noir/commit/e8bb3417d276d17db85b408b825e61c32fa20744))
* Allow inserting LSP inlay type hints ([#5620](https://github.com/noir-lang/noir/issues/5620)) ([b33495d](https://github.com/noir-lang/noir/commit/b33495d0799f7c296cf6e284ea19abbbe5821793))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Derive `Ord` and `Hash` in the stdlib; add `std::meta::make_impl` helper ([#5683](https://github.com/noir-lang/noir/issues/5683)) ([38397d3](https://github.com/noir-lang/noir/commit/38397d346aac4ec19026ede2776e776fdceb854c))
* Don't eagerly error on cast expressions ([#5635](https://github.com/noir-lang/noir/issues/5635)) ([0ca5d9d](https://github.com/noir-lang/noir/commit/0ca5d9d7b389d24cf48fa62a29cb437b1855438a))
* Implement `poseidon2_permutation` in comptime interpreter ([#5590](https://github.com/noir-lang/noir/issues/5590)) ([89dfbbf](https://github.com/noir-lang/noir/commit/89dfbbfe6efffcce9a44c39ccf7e92036a0e222a))
* Implement `Value::Type` in comptime interpreter ([#5593](https://github.com/noir-lang/noir/issues/5593)) ([4c3bf97](https://github.com/noir-lang/noir/commit/4c3bf97fe7475f1027285cb5ad26b3c578a632b7))
* Implement `zeroed` in the interpreter ([#5540](https://github.com/noir-lang/noir/issues/5540)) ([ff8ca91](https://github.com/noir-lang/noir/commit/ff8ca91efc925bf8bdbe7f2d2feb651981c5c1b9))
* Implement closures in the comptime interpreter ([#5682](https://github.com/noir-lang/noir/issues/5682)) ([9e2a323](https://github.com/noir-lang/noir/commit/9e2a3232c8849f19732472e75840717b8b95a4a9))
* Implement format strings in the comptime interpreter ([#5596](https://github.com/noir-lang/noir/issues/5596)) ([fd7002c](https://github.com/noir-lang/noir/commit/fd7002caaf15c297227ce53047dd3361674a527d))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Let filenames in errors be relative to the current dir if possible ([#5642](https://github.com/noir-lang/noir/issues/5642)) ([f656681](https://github.com/noir-lang/noir/commit/f656681dfccf01bac2811835fc9ab482e54e3da8))
* Let LSP work will with code generated by macros ([#5665](https://github.com/noir-lang/noir/issues/5665)) ([8122624](https://github.com/noir-lang/noir/commit/812262413770d2f20cba04eb0e3176320a3b704a))
* LSP closing brace hints ([#5686](https://github.com/noir-lang/noir/issues/5686)) ([2b18151](https://github.com/noir-lang/noir/commit/2b18151168b4fa0b1a4173b11f047ff2fc338d28))
* LSP hover now includes "Go to" links ([#5677](https://github.com/noir-lang/noir/issues/5677)) ([d466d49](https://github.com/noir-lang/noir/commit/d466d491ea50b495be7d5a45a8c3d85771f9b1c0))
* LSP inlay parameter hints ([#5553](https://github.com/noir-lang/noir/issues/5553)) ([822fe2c](https://github.com/noir-lang/noir/commit/822fe2ce38184243789a97f79ee412b9cef614e2))
* LSP inlay type hints on lambda parameters ([#5639](https://github.com/noir-lang/noir/issues/5639)) ([80128ff](https://github.com/noir-lang/noir/commit/80128ff0a5d54fc777ffef89a7acc27d347181e6))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **noir_js:** Expose UltraHonk and integration tests ([#5656](https://github.com/noir-lang/noir/issues/5656)) ([4552b4f](https://github.com/noir-lang/noir/commit/4552b4f357f023a9d054a34b3dc94d7a659c7d09))
* Remove 'comptime or separate crate' restriction on comptime code ([#5609](https://github.com/noir-lang/noir/issues/5609)) ([1cddf42](https://github.com/noir-lang/noir/commit/1cddf427b7f52b3cb394c8c4c682cfd176d5eb93))
* Resolve arguments to attributes ([#5649](https://github.com/noir-lang/noir/issues/5649)) ([e139002](https://github.com/noir-lang/noir/commit/e1390020c5ffc9c252b47eeffb3219ffa20c4879))
* **ssa:** Simple serialization of unoptimized SSA to file ([#5679](https://github.com/noir-lang/noir/issues/5679)) ([07ea107](https://github.com/noir-lang/noir/commit/07ea1077cad8a332970d9d5f40a520009976a033))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Turbofish in struct pattern ([#5616](https://github.com/noir-lang/noir/issues/5616)) ([b3c408b](https://github.com/noir-lang/noir/commit/b3c408b62424c87f9be5b58c33be7d77e62af98e))
* Turbofish operator in struct constructor ([#5607](https://github.com/noir-lang/noir/issues/5607)) ([106abd7](https://github.com/noir-lang/noir/commit/106abd71299a54dc68eb6ff39a0a8135b3f8eb49))
* Turbofish operator on path segments ([#5603](https://github.com/noir-lang/noir/issues/5603)) ([0bb8372](https://github.com/noir-lang/noir/commit/0bb8372e118036a34709da37c26d11a539a86bb3))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))


### Bug Fixes

* 'cannot eval non-comptime global' error ([#5586](https://github.com/noir-lang/noir/issues/5586)) ([0a987c7](https://github.com/noir-lang/noir/commit/0a987c774e0349e2cdc17ad2aaee634c732b8785))
* `NoMatchingImplFound` in comptime code only ([#5617](https://github.com/noir-lang/noir/issues/5617)) ([28211a3](https://github.com/noir-lang/noir/commit/28211a397b810c661204b45b7da06f7cad345278))
* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Allow calling a trait method with paths that don't consist of exactly two segments ([#5577](https://github.com/noir-lang/noir/issues/5577)) ([88c0a40](https://github.com/noir-lang/noir/commit/88c0a40ea35c7a64e4361e3022286be6fa7da666))
* Allow trailing comma when parsing where clauses ([#5594](https://github.com/noir-lang/noir/issues/5594)) ([75bfe13](https://github.com/noir-lang/noir/commit/75bfe134457788f912da5173c395d7fbe0bb731a))
* Allow using Self for function calls ([#5629](https://github.com/noir-lang/noir/issues/5629)) ([b7e4f42](https://github.com/noir-lang/noir/commit/b7e4f424f3fc461122447c30989a3ce52ddea09d))
* Correct span for prefix operator ([#5624](https://github.com/noir-lang/noir/issues/5624)) ([5824785](https://github.com/noir-lang/noir/commit/58247854fba4309991529317671280bccd5cf21f))
* Correctly track sources for open LSP documents ([#5561](https://github.com/noir-lang/noir/issues/5561)) ([9e61e97](https://github.com/noir-lang/noir/commit/9e61e97a6cc5aecaf673742f4b333879eeb687d0))
* Derive generic types ([#5674](https://github.com/noir-lang/noir/issues/5674)) ([19e58a9](https://github.com/noir-lang/noir/commit/19e58a91a3b1d0534d8b0198347e3a2fb5488599))
* Don't panic when a macro fails to resolve ([#5537](https://github.com/noir-lang/noir/issues/5537)) ([6109ddc](https://github.com/noir-lang/noir/commit/6109ddc4a12a4f7593c87fcc42a059737febd470))
* Elaborate struct & trait annotations in the correct module ([#5643](https://github.com/noir-lang/noir/issues/5643)) ([d0a957b](https://github.com/noir-lang/noir/commit/d0a957ba9bd743ae00959b0680e275a0a3992308))
* Error on duplicate struct field ([#5585](https://github.com/noir-lang/noir/issues/5585)) ([3aed671](https://github.com/noir-lang/noir/commit/3aed671d2fdca661fdef160b3e2468ce10eda028))
* Error on incorrect generic count for impl and type alias ([#5623](https://github.com/noir-lang/noir/issues/5623)) ([1f5d000](https://github.com/noir-lang/noir/commit/1f5d0007430cd5cf057ce61ebc87304bb8cb557c))
* Error on trait impl generics count mismatch ([#5582](https://github.com/noir-lang/noir/issues/5582)) ([da3d607](https://github.com/noir-lang/noir/commit/da3d607fb30143f7fd4077765119f98e664f31f7))
* Error on unbound generics in structs ([#5619](https://github.com/noir-lang/noir/issues/5619)) ([efef6b4](https://github.com/noir-lang/noir/commit/efef6b4c9f2ff4bce7e83bed004eb05332ac349f))
* Filter comptime globals ([#5538](https://github.com/noir-lang/noir/issues/5538)) ([2adc6ac](https://github.com/noir-lang/noir/commit/2adc6ac372b41ab7f9e803311d3d733b1a5ca4fb))
* Fix `uhashmap` test name ([#5563](https://github.com/noir-lang/noir/issues/5563)) ([d5de83f](https://github.com/noir-lang/noir/commit/d5de83f7fefe014c5e5d9c3d82ab6aff6fd3217c))
* Fix occurs check ([#5535](https://github.com/noir-lang/noir/issues/5535)) ([51dd529](https://github.com/noir-lang/noir/commit/51dd529872da96ad7f7770d6bc5f7c23f5415b5a))
* Fix where clause issue in items generated from attributes ([#5673](https://github.com/noir-lang/noir/issues/5673)) ([9a8cfc9](https://github.com/noir-lang/noir/commit/9a8cfc9cfbf1861ca2b6563030b315bfcfbab130))
* **frontend:** Disallow signed numeric generics ([#5572](https://github.com/noir-lang/noir/issues/5572)) ([2b4853e](https://github.com/noir-lang/noir/commit/2b4853e71859f225acc123160e87c522212b16b5))
* **frontend:** Error for when impl is stricter than trait ([#5343](https://github.com/noir-lang/noir/issues/5343)) ([ece033f](https://github.com/noir-lang/noir/commit/ece033fcbf90ffbea992b4519d40076bf573b7af))
* **frontend:** Restrict numeric generic types to unsigned ints up to `u32` ([#5581](https://github.com/noir-lang/noir/issues/5581)) ([b85e764](https://github.com/noir-lang/noir/commit/b85e764c2156ebb68acb7fba68e63856f9d1235b))
* Let a trait impl that relies on another trait work ([#5646](https://github.com/noir-lang/noir/issues/5646)) ([e00c370](https://github.com/noir-lang/noir/commit/e00c3705794657ea8f8faa16bc2325511567e185))
* Let std::unsafe::zeroed() work for slices ([#5592](https://github.com/noir-lang/noir/issues/5592)) ([7daee20](https://github.com/noir-lang/noir/commit/7daee20a3c3628044e2e76b0a52abd5285a4432a))
* Let trait calls work in globals ([#5602](https://github.com/noir-lang/noir/issues/5602)) ([c02a6f6](https://github.com/noir-lang/noir/commit/c02a6f64b1920ec5a1a05e20df34227cc95d7b0a))
* Let unary traits work at comptime ([#5507](https://github.com/noir-lang/noir/issues/5507)) ([aa62d8a](https://github.com/noir-lang/noir/commit/aa62d8adce7ca2a4e021c8ec18b6056a7600fe95))
* Lookup trait constraints methods in composite types ([#5595](https://github.com/noir-lang/noir/issues/5595)) ([cec6390](https://github.com/noir-lang/noir/commit/cec63902c52710ba7df433eda310296f7b7652d2))
* Parse block and if statements independently of expressions in statements ([#5634](https://github.com/noir-lang/noir/issues/5634)) ([9341113](https://github.com/noir-lang/noir/commit/9341113840294d6d895d5ed9713ba551cf8a1db9))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Run macros within comptime contexts ([#5576](https://github.com/noir-lang/noir/issues/5576)) ([df44919](https://github.com/noir-lang/noir/commit/df449191a4edf669e1d3412789422177c54a9304))
* Speed up LSP ([#5650](https://github.com/noir-lang/noir/issues/5650)) ([e5f1b36](https://github.com/noir-lang/noir/commit/e5f1b368c8894b3e37209b22c48fde8c82851cba))
* **ssa:** More robust array deduplication check ([#5547](https://github.com/noir-lang/noir/issues/5547)) ([dd89b90](https://github.com/noir-lang/noir/commit/dd89b900dfabbea889f6f181aa562e73a87215b7))
* Switch verify proof to arrays ([#5664](https://github.com/noir-lang/noir/issues/5664)) ([c1ed9fb](https://github.com/noir-lang/noir/commit/c1ed9fb3f6e6778b519685fe218178b52c12076f))
* Type_of for pointer types ([#5536](https://github.com/noir-lang/noir/issues/5536)) ([edb3810](https://github.com/noir-lang/noir/commit/edb3810c6863e3c7802fcaece82eb6de21ebd71f))
* Workaround from_slice with nested slices ([#5648](https://github.com/noir-lang/noir/issues/5648)) ([6310a55](https://github.com/noir-lang/noir/commit/6310a55163ba3ce84e50a6df68d8963cd5222bd9))

## [0.32.0](https://github.com/noir-lang/noir/compare/v0.31.0...v0.32.0) (2024-07-18)


### ⚠ BREAKING CHANGES

* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)
* error on too large integer value ([#5371](https://github.com/noir-lang/noir/issues/5371))
* rename struct-specific TypeDefinition -> StructDefinition ([#5356](https://github.com/noir-lang/noir/issues/5356))
* **frontend:** Explicit numeric generics and type kinds ([#5155](https://github.com/noir-lang/noir/issues/5155))

### Features

* `mod.nr` entrypoint ([#5039](https://github.com/noir-lang/noir/issues/5039)) ([076fe0a](https://github.com/noir-lang/noir/commit/076fe0a11869f6975d214c5b9a5ed1e8f7cdbded))
* `static_assert` builtin ([#5342](https://github.com/noir-lang/noir/issues/5342)) ([ef44270](https://github.com/noir-lang/noir/commit/ef4427051eebf323462cbb1fed205e8b555712a1))
* Add `map`, `fold`, `reduce`, `any`, and `all` for slices ([#5331](https://github.com/noir-lang/noir/issues/5331)) ([03e25b4](https://github.com/noir-lang/noir/commit/03e25b4577349859c85203fadafc3c63aa4e4dd0))
* Add CLI argument for debugging comptime blocks ([#5192](https://github.com/noir-lang/noir/issues/5192)) ([0b74a18](https://github.com/noir-lang/noir/commit/0b74a18537b84a0f774d54518fd938f8c11e1baf))
* Add comptime support for `modulus_*` compiler builtins ([#5530](https://github.com/noir-lang/noir/issues/5530)) ([5bbce79](https://github.com/noir-lang/noir/commit/5bbce7977f72b07336bc8ef09f6acff687f1644a))
* Add debug codelens action ([#5474](https://github.com/noir-lang/noir/issues/5474)) ([6bcdac4](https://github.com/noir-lang/noir/commit/6bcdac428a48083c9b0d85d42b4d8635a182fda1))
* Add fuzzer for Noir programs ([#5251](https://github.com/noir-lang/noir/issues/5251)) ([e100017](https://github.com/noir-lang/noir/commit/e1000176a31140b2abd79c47653cbc4bb1a6808a))
* Add gate profiler for noir circuits (https://github.com/AztecProtocol/aztec-packages/pull/7004) ([083070e](https://github.com/noir-lang/noir/commit/083070e83e916b68799358b119a9f843223f2686))
* Add more slice methods to the stdlib ([#5424](https://github.com/noir-lang/noir/issues/5424)) ([4020e77](https://github.com/noir-lang/noir/commit/4020e77145b99861b8bd6027a6823ccf2c39271f))
* Add opcodes flamegraph and refactor gates flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7282) ([32029f9](https://github.com/noir-lang/noir/commit/32029f91f6aae4d2f6b08b4ea40481f5837e50bc))
* Add support for fieldable in events (https://github.com/AztecProtocol/aztec-packages/pull/7310) ([3f71169](https://github.com/noir-lang/noir/commit/3f71169ef4ef8ea8a3dcf355bf360195bfa6772c))
* Add support for usage of `super` in import paths ([#5502](https://github.com/noir-lang/noir/issues/5502)) ([256509e](https://github.com/noir-lang/noir/commit/256509e5083895b6115b110aedd5a97bd9e74fc0))
* Add support for wildcard types ([#5275](https://github.com/noir-lang/noir/issues/5275)) ([7445efb](https://github.com/noir-lang/noir/commit/7445efb05165bf7df2f9dfe325abbc42f839364c))
* Add TraitConstraint type ([#5499](https://github.com/noir-lang/noir/issues/5499)) ([30cb65a](https://github.com/noir-lang/noir/commit/30cb65a12668d192f8da940c32961210a05a962f))
* Add unquote function ([#5497](https://github.com/noir-lang/noir/issues/5497)) ([2947aba](https://github.com/noir-lang/noir/commit/2947ababcbcc7cbe5d99f6a8ed0dc6ad756ebeb8))
* Allow arguments to attribute functions ([#5494](https://github.com/noir-lang/noir/issues/5494)) ([a33cafc](https://github.com/noir-lang/noir/commit/a33cafcb7e175ad8b3b80b8c9419a32e009ec702))
* Allow comptime attributes on traits & functions ([#5496](https://github.com/noir-lang/noir/issues/5496)) ([b59a29e](https://github.com/noir-lang/noir/commit/b59a29e5b246121a4d81e4894a4b10f5df4dd5cf))
* Apply `no_predicates` in stdlib ([#5454](https://github.com/noir-lang/noir/issues/5454)) ([24d26c0](https://github.com/noir-lang/noir/commit/24d26c05705fabca81b19d789203ebb6fc22ff32))
* Build releases for `aarch64-unknown-linux-gnu` target ([#5289](https://github.com/noir-lang/noir/issues/5289)) ([f35614a](https://github.com/noir-lang/noir/commit/f35614a43cf8c5cfb244d9f6ffc9d63282a63e6d))
* Build simple dictionary from inspecting ACIR program ([#5264](https://github.com/noir-lang/noir/issues/5264)) ([508e677](https://github.com/noir-lang/noir/commit/508e677cf2c66ac3427932a18f1661f5f4dc4202))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Detect subgraphs that are completely independent from inputs or outputs ([#5402](https://github.com/noir-lang/noir/issues/5402)) ([7ea83a9](https://github.com/noir-lang/noir/commit/7ea83a9de4d3096d27e79faf5d8081b9e9108c4a))
* Disable nargo color output if stderr is tty ([#5346](https://github.com/noir-lang/noir/issues/5346)) ([554dd6b](https://github.com/noir-lang/noir/commit/554dd6b01b1d3417448d8ecc95165fd0c3ca36e9))
* Error on too large integer value ([#5371](https://github.com/noir-lang/noir/issues/5371)) ([0c4fffa](https://github.com/noir-lang/noir/commit/0c4fffa86f6605e8f16f973ad42c6927a03fc3cc))
* **frontend:** Explicit numeric generics and type kinds ([#5155](https://github.com/noir-lang/noir/issues/5155)) ([d4e03d0](https://github.com/noir-lang/noir/commit/d4e03d07bb00d1ba0f8f43bd0dd9e967a10a28b9))
* **frontend:** Where clause on impl ([#5320](https://github.com/noir-lang/noir/issues/5320)) ([cf938bc](https://github.com/noir-lang/noir/commit/cf938bc06b7015dae94847f146dc7fd38055f064))
* Handle ACIR calls in the debugger ([#5051](https://github.com/noir-lang/noir/issues/5051)) ([0541568](https://github.com/noir-lang/noir/commit/0541568b4c209927a70778b895e8f1e50d9b6543))
* Implement comptime support for `array_len` builtin ([#5272](https://github.com/noir-lang/noir/issues/5272)) ([c91186a](https://github.com/noir-lang/noir/commit/c91186a5c0d9e84767f160e6acd63672b23e8f52))
* Implement comptime support for `as_slice` builtin ([#5276](https://github.com/noir-lang/noir/issues/5276)) ([9db65d8](https://github.com/noir-lang/noir/commit/9db65d8706ac8b67921f2a73163ab8bee3dfb4e8))
* Implement trait dispatch in the comptime interpreter ([#5376](https://github.com/noir-lang/noir/issues/5376)) ([8aa5b2e](https://github.com/noir-lang/noir/commit/8aa5b2e4cc69ca6ac1077e8e08c28e9cb30ffb51))
* Insert trait impls into the program from type annotations ([#5327](https://github.com/noir-lang/noir/issues/5327)) ([efdd818](https://github.com/noir-lang/noir/commit/efdd818a1fc52f31bda4e4519a4ba42887cec87a))
* Let `should_fail_with` check that the failure reason contains the expected message ([#5319](https://github.com/noir-lang/noir/issues/5319)) ([cb9db55](https://github.com/noir-lang/noir/commit/cb9db55dcf87a45356af362f6f90681dd0e00212))
* Let LSP always work in a Noir workspace if there's any ([#5461](https://github.com/noir-lang/noir/issues/5461)) ([e0d7833](https://github.com/noir-lang/noir/commit/e0d78334e4b7c7cdd2e4778c3f13dd12ddbef59c))
* Lsp "find all references" ([#5395](https://github.com/noir-lang/noir/issues/5395)) ([ce1994c](https://github.com/noir-lang/noir/commit/ce1994ca87cb47ec22aa95e566a4e18f0c931ea1))
* Lsp "go to definition" for modules ([#5406](https://github.com/noir-lang/noir/issues/5406)) ([3e7f1f2](https://github.com/noir-lang/noir/commit/3e7f1f28e5836b164bebdc3bad20d8d91dccd211))
* LSP document symbol ([#5532](https://github.com/noir-lang/noir/issues/5532)) ([1fabcde](https://github.com/noir-lang/noir/commit/1fabcde195f3965c6b8701eb4e1fed49ec1bde4b))
* LSP hover ([#5491](https://github.com/noir-lang/noir/issues/5491)) ([010c835](https://github.com/noir-lang/noir/commit/010c835e4ebfdf49ea4e9326abafcdeb587153b6))
* LSP inlay hints for let and global ([#5510](https://github.com/noir-lang/noir/issues/5510)) ([43f5b8d](https://github.com/noir-lang/noir/commit/43f5b8d8eba5011b163e30a09ad743f893aa841a))
* Lsp rename struct ([#5380](https://github.com/noir-lang/noir/issues/5380)) ([ee8b0cd](https://github.com/noir-lang/noir/commit/ee8b0cdbc919fbf924c5d42067c0f18db8def2bf))
* Lsp rename/find-all-references for globals ([#5415](https://github.com/noir-lang/noir/issues/5415)) ([fa9b444](https://github.com/noir-lang/noir/commit/fa9b4446f96155fc08d8087444fc856e86e7ab62))
* Lsp rename/find-all-references for local variables ([#5439](https://github.com/noir-lang/noir/issues/5439)) ([bb6913a](https://github.com/noir-lang/noir/commit/bb6913ac53620fabd73e24ca1a2b1369225903ec))
* Lsp rename/find-all-references for struct members ([#5443](https://github.com/noir-lang/noir/issues/5443)) ([a6d213d](https://github.com/noir-lang/noir/commit/a6d213d41aa5a8e31a1d6210f2ea98a501b8f67d))
* Lsp rename/find-all-references for traits ([#5409](https://github.com/noir-lang/noir/issues/5409)) ([bf3a75a](https://github.com/noir-lang/noir/commit/bf3a75a3f9c6926baaa1408767dd929de2f8a8f9))
* Lsp rename/find-all-references for type aliases ([#5414](https://github.com/noir-lang/noir/issues/5414)) ([24c621f](https://github.com/noir-lang/noir/commit/24c621fa96783373ab81da66cb6076e130c4a3a5))
* **lsp:** Allow function rename ([#4294](https://github.com/noir-lang/noir/issues/4294)) ([3d86dc6](https://github.com/noir-lang/noir/commit/3d86dc6118d083c686b1061a52eb4f113e9a9f7c))
* Make macros operate on token streams instead of AST nodes ([#5301](https://github.com/noir-lang/noir/issues/5301)) ([7689d59](https://github.com/noir-lang/noir/commit/7689d59aa12003994cea6a3ff4bf87484e41aa6b))
* **nargo:** Default expression width field in `Nargo.toml` ([#5505](https://github.com/noir-lang/noir/issues/5505)) ([dea6b32](https://github.com/noir-lang/noir/commit/dea6b323fe8db636f5991cfc206ea9222addca30))
* **optimization:** Deduplicate more instructions ([#5457](https://github.com/noir-lang/noir/issues/5457)) ([c47242a](https://github.com/noir-lang/noir/commit/c47242ab624f4a1d564b3b62bc84a1b4bb5bd549))
* Prefix operator overload trait dispatch ([#5423](https://github.com/noir-lang/noir/issues/5423)) ([a3bb09e](https://github.com/noir-lang/noir/commit/a3bb09ebe2df473d4a34a34fbfc3966ffbc630cb))
* Remove duplicated array reads at constant indices ([#5445](https://github.com/noir-lang/noir/issues/5445)) ([82a67a0](https://github.com/noir-lang/noir/commit/82a67a0e9554afeadb1839e6511794b41960f241))
* Remove redundant `EnableSideEffects` instructions ([#5440](https://github.com/noir-lang/noir/issues/5440)) ([e153ecb](https://github.com/noir-lang/noir/commit/e153ecbe068f5974d5836aedebb8a41c5620d5f7))
* Rename struct-specific TypeDefinition -&gt; StructDefinition ([#5356](https://github.com/noir-lang/noir/issues/5356)) ([7ffccf7](https://github.com/noir-lang/noir/commit/7ffccf7f060aee30b08ef7fda75d8695f047abd8))
* Run `comptime` code from annotations on a type definition ([#5256](https://github.com/noir-lang/noir/issues/5256)) ([6cbe6a0](https://github.com/noir-lang/noir/commit/6cbe6a0c830b2992666e0f9bdbc8f66ec41eed84))
* Skip reading values immediately after it being written into an array ([#5449](https://github.com/noir-lang/noir/issues/5449)) ([141ecdd](https://github.com/noir-lang/noir/commit/141ecddf79b27244a52097577395c7b41cd4d331))
* **stdlib:** Update stdlib to use explicit numeric generics ([#5306](https://github.com/noir-lang/noir/issues/5306)) ([8456185](https://github.com/noir-lang/noir/commit/8456185078c90cfcb8e63caf147ea6cdbbd786af))
* Sync from aztec-packages ([#5347](https://github.com/noir-lang/noir/issues/5347)) ([47b621f](https://github.com/noir-lang/noir/commit/47b621fcb8a971b353ce5bda3a506da5504ae9a3))
* Sync from aztec-packages ([#5377](https://github.com/noir-lang/noir/issues/5377)) ([7b77bbf](https://github.com/noir-lang/noir/commit/7b77bbfc19c51829814149e623257a3424d8e8c2))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7257) ([32029f9](https://github.com/noir-lang/noir/commit/32029f91f6aae4d2f6b08b4ea40481f5837e50bc))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7288) ([322f863](https://github.com/noir-lang/noir/commit/322f86392a899fa6e1765cb30b72768211605a9f))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7308) ([322f863](https://github.com/noir-lang/noir/commit/322f86392a899fa6e1765cb30b72768211605a9f))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7332) ([3f71169](https://github.com/noir-lang/noir/commit/3f71169ef4ef8ea8a3dcf355bf360195bfa6772c))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7352) ([98e72ac](https://github.com/noir-lang/noir/commit/98e72acd72e9a01376cf69d20c539ba9dbe0942b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Unquote multiple items from annotations ([#5441](https://github.com/noir-lang/noir/issues/5441)) ([be8eac6](https://github.com/noir-lang/noir/commit/be8eac6ff44dac442df9b09de1fd2269a6371d56))
* Use runtime loops for brillig array initialization ([#5243](https://github.com/noir-lang/noir/issues/5243)) ([0bd22bb](https://github.com/noir-lang/noir/commit/0bd22bb460ff0bf134ce3acf79e018c1e360d31c))


### Bug Fixes

* Account for the expected kind when resolving turbofish generics ([#5448](https://github.com/noir-lang/noir/issues/5448)) ([82c335d](https://github.com/noir-lang/noir/commit/82c335d3e36365695eccc1c4af63e58dd0633328))
* Add more thorough check for whether a type is valid when passing it from constrained code to unconstrained code ([#5009](https://github.com/noir-lang/noir/issues/5009)) ([318314d](https://github.com/noir-lang/noir/commit/318314d6dd35674328f534ebd882d4b0e66eab24))
* Address compiler warnings coming from stdlib ([#5351](https://github.com/noir-lang/noir/issues/5351)) ([758a905](https://github.com/noir-lang/noir/commit/758a905fc740971c995151bfb5f997bcc080397c))
* Allow importing notes from other contracts and inject them in the macros (https://github.com/AztecProtocol/aztec-packages/pull/7349) ([98e72ac](https://github.com/noir-lang/noir/commit/98e72acd72e9a01376cf69d20c539ba9dbe0942b))
* Avoid duplicating constant arrays ([#5287](https://github.com/noir-lang/noir/issues/5287)) ([3ef3645](https://github.com/noir-lang/noir/commit/3ef36458fef36b2a2f6cf99b35a43339f3721b27))
* Avoid panic in type system ([#5332](https://github.com/noir-lang/noir/issues/5332)) ([52d48ff](https://github.com/noir-lang/noir/commit/52d48ff1cf1415fa87fbaf76249b2e0d042de8bd))
* Avoid unnecessarily splitting expressions with multiplication terms with a shared term ([#5291](https://github.com/noir-lang/noir/issues/5291)) ([19884f1](https://github.com/noir-lang/noir/commit/19884f161dfc7d7ce75dd2c404b8ef39cdad2240))
* Change panic to error in interpreter ([#5446](https://github.com/noir-lang/noir/issues/5446)) ([d44f882](https://github.com/noir-lang/noir/commit/d44f882be094bf492b1742370fd3896b0c371f59))
* Complete call stacks with no_predicates ([#5418](https://github.com/noir-lang/noir/issues/5418)) ([df73fe2](https://github.com/noir-lang/noir/commit/df73fe2f345422516bfa01462c0c76d3b924b772))
* Correct range for overlfowing/underflowing integer assignment ([#5416](https://github.com/noir-lang/noir/issues/5416)) ([30c50f5](https://github.com/noir-lang/noir/commit/30c50f52a6d58163e39006b73f4eb5003afc239b))
* Correctly detect signed/unsigned integer overflows/underflows ([#5375](https://github.com/noir-lang/noir/issues/5375)) ([0603bd3](https://github.com/noir-lang/noir/commit/0603bd39bff1183725e9aeeaba678c421c7b1daf))
* **docs:** Fix broken docs link to gihtub ([#5398](https://github.com/noir-lang/noir/issues/5398)) ([70ebf60](https://github.com/noir-lang/noir/commit/70ebf607e566a95ff7eb2c7a0eee7c36465ba5b4))
* Don't benchmark the "prove" command as it doesn't exist anymore ([#5323](https://github.com/noir-lang/noir/issues/5323)) ([3bb3b03](https://github.com/noir-lang/noir/commit/3bb3b03aedab9c7abfeb3d3141e04b07b7aeeffb))
* Don't lazily elaborate functions ([#5282](https://github.com/noir-lang/noir/issues/5282)) ([0ea608f](https://github.com/noir-lang/noir/commit/0ea608f10bdeb26df7dfc17b1a0bad5db1967be8))
* Don't panic when using undefined variables in the interpreter ([#5381](https://github.com/noir-lang/noir/issues/5381)) ([94d209a](https://github.com/noir-lang/noir/commit/94d209acb70064d5f8a5d427bade18d3cd975be0))
* Don't type error when calling certain trait impls in the interpreter ([#5471](https://github.com/noir-lang/noir/issues/5471)) ([299703c](https://github.com/noir-lang/noir/commit/299703cf4b87a84257f48f059eb58135ad36265d))
* Error on empty function bodies ([#5519](https://github.com/noir-lang/noir/issues/5519)) ([6a7f593](https://github.com/noir-lang/noir/commit/6a7f593a04ee1caefd6a19a5cba1c0dbeee22ee1))
* Error when a local function is called in a comptime context ([#5334](https://github.com/noir-lang/noir/issues/5334)) ([7cd4a4d](https://github.com/noir-lang/noir/commit/7cd4a4d1cde4446c8ace7439ce9f8d42ded70869))
* Fix incorrect return type being applied to stdlib functions `modulus_be_bytes()`, `modulus_be_bits()`, etc. ([#5278](https://github.com/noir-lang/noir/issues/5278)) ([91a9b72](https://github.com/noir-lang/noir/commit/91a9b725cdb75c08cde888f49e7b8d11257e5de6))
* Fix issue with unresolved results ([#5453](https://github.com/noir-lang/noir/issues/5453)) ([c4154cb](https://github.com/noir-lang/noir/commit/c4154cbb0e8e56d351d012eb284c34424821e25a))
* Fix tokenization of unquoted types in macros ([#5326](https://github.com/noir-lang/noir/issues/5326)) ([6673c8b](https://github.com/noir-lang/noir/commit/6673c8b7068a3cd5d5914e1b0ecb9457a7e26bab))
* Fix usage of `#[abi(tag)]` attribute with elaborator ([#5298](https://github.com/noir-lang/noir/issues/5298)) ([64dd48a](https://github.com/noir-lang/noir/commit/64dd48a19060ccce8758851ea7bcec1f287f1156))
* Go to definition from `use` statement ([#5390](https://github.com/noir-lang/noir/issues/5390)) ([53bae3b](https://github.com/noir-lang/noir/commit/53bae3b99b2aec0b7d5c65d4f9f60e2eafdd2b1f))
* Go to definition from aliased use ([#5396](https://github.com/noir-lang/noir/issues/5396)) ([90b135c](https://github.com/noir-lang/noir/commit/90b135c44bdf91603f2e2cdf0ab6f168087bab36))
* Handle struct with nested arrays in oracle return values ([#5244](https://github.com/noir-lang/noir/issues/5244)) ([a30814f](https://github.com/noir-lang/noir/commit/a30814f1f767bf874cd7e2969f5061c68f16b9a7))
* ICE when using a comptime let variable in runtime code ([#5391](https://github.com/noir-lang/noir/issues/5391)) ([9fb7e4d](https://github.com/noir-lang/noir/commit/9fb7e4d306041edc5158e2dffd71a19ccc578ac2))
* Ignore calls to `Intrinsic::AsWitness` during brillig codegen ([#5350](https://github.com/noir-lang/noir/issues/5350)) ([9c11fd2](https://github.com/noir-lang/noir/commit/9c11fd264451a3d2b8617ee5e47e6db3fcb148d8))
* Implement generic functions in the interpreter ([#5330](https://github.com/noir-lang/noir/issues/5330)) ([d8b9870](https://github.com/noir-lang/noir/commit/d8b9870a991b724ec337b58380b50464ba274d8a))
* Included argshash computation in public call_interfaces and cleanup (https://github.com/AztecProtocol/aztec-packages/pull/7354) ([98e72ac](https://github.com/noir-lang/noir/commit/98e72acd72e9a01376cf69d20c539ba9dbe0942b))
* Lsp find struct reference in return locations and paths ([#5404](https://github.com/noir-lang/noir/issues/5404)) ([e1bcb73](https://github.com/noir-lang/noir/commit/e1bcb73f8c2e2c6786faeb18b8ce070a2400635d))
* Lsp hover wasn't always working ([#5515](https://github.com/noir-lang/noir/issues/5515)) ([951e821](https://github.com/noir-lang/noir/commit/951e821a585fe7e0697291cadd4d3c3aa49fd8e4))
* Lsp struct rename/reference difference ([#5411](https://github.com/noir-lang/noir/issues/5411)) ([580c16d](https://github.com/noir-lang/noir/commit/580c16dd61b044c7ebfb31958822c23ea9b20ed2))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Mutability in the comptime interpreter ([#5517](https://github.com/noir-lang/noir/issues/5517)) ([8cab4ac](https://github.com/noir-lang/noir/commit/8cab4ac0c0275fae691731b6d774e51b633f9478))
* **nargo_fmt:** Account for spaces before the generic list of a function ([#5303](https://github.com/noir-lang/noir/issues/5303)) ([ec728dd](https://github.com/noir-lang/noir/commit/ec728dd909fce33ab712116f61d672b1ee552fc4))
* Never panic in LSP inlay hints ([#5534](https://github.com/noir-lang/noir/issues/5534)) ([6b11445](https://github.com/noir-lang/noir/commit/6b11445d9913e2953a96d09f86826aa652a233c4))
* Prevent `no_predicates` from removing predicates in calling function ([#5452](https://github.com/noir-lang/noir/issues/5452)) ([66244b6](https://github.com/noir-lang/noir/commit/66244b6e5b505f692c7e9a41bdc061c77fd1284d))
* Remove compile-time error for invalid indices ([#5466](https://github.com/noir-lang/noir/issues/5466)) ([323e0c9](https://github.com/noir-lang/noir/commit/323e0c9d31cdec7d6bef76a418d1b663d9640143))
* Remove panics in the interpreter when a builtin fails to type check ([#5382](https://github.com/noir-lang/noir/issues/5382)) ([c8161c8](https://github.com/noir-lang/noir/commit/c8161c81a3c6599a3b0380f4c80c730a41a75f22))
* Replace expects in interpreter with errors ([#5383](https://github.com/noir-lang/noir/issues/5383)) ([ac738b2](https://github.com/noir-lang/noir/commit/ac738b21bc19181b021f909a8e60752dff5ac713))
* Replace panic in monomorphization with an error ([#5305](https://github.com/noir-lang/noir/issues/5305)) ([49e1b0c](https://github.com/noir-lang/noir/commit/49e1b0c0d45565f3e87469b77f2fef0c283f6ea1))
* Replace std::HashMap with FxHashMap to fix frontend indeterminism ([#5385](https://github.com/noir-lang/noir/issues/5385)) ([9501495](https://github.com/noir-lang/noir/commit/95014950a9685ee8fdae69457cfe45d6c509172a))
* Revert PR [#5449](https://github.com/noir-lang/noir/issues/5449) ([#5548](https://github.com/noir-lang/noir/issues/5548)) ([a213c15](https://github.com/noir-lang/noir/commit/a213c15275892581e5d8f7235baf08a6cb137da4))
* Run macro processors in the elaborator ([#5472](https://github.com/noir-lang/noir/issues/5472)) ([89642c2](https://github.com/noir-lang/noir/commit/89642c220791b2b91bd350960ed6a822103ccca7))
* Runtime brillig bigint id assignment ([#5369](https://github.com/noir-lang/noir/issues/5369)) ([a8928dd](https://github.com/noir-lang/noir/commit/a8928ddcffcae15babf7aa5aff0e462e4549552e))
* Skip emission of brillig calls which will never be executed ([#5314](https://github.com/noir-lang/noir/issues/5314)) ([b859ef9](https://github.com/noir-lang/noir/commit/b859ef90af9944a83f197c26408a55988b143e0e))
* Truncate flamegraph text to the right (https://github.com/AztecProtocol/aztec-packages/pull/7333) ([3f71169](https://github.com/noir-lang/noir/commit/3f71169ef4ef8ea8a3dcf355bf360195bfa6772c))
* Update `in_contract` flag before handling function metadata in elaborator ([#5292](https://github.com/noir-lang/noir/issues/5292)) ([4c4ea2d](https://github.com/noir-lang/noir/commit/4c4ea2df0163d4989c922c6a1377e04c2cd0540c))
* Use proper serialization in `AbiValue` ([#5270](https://github.com/noir-lang/noir/issues/5270)) ([d08b7b9](https://github.com/noir-lang/noir/commit/d08b7b93a981f2e01a3d9754b194c5565ad3a7c2))

## [0.31.0](https://github.com/noir-lang/noir/compare/v0.30.0...v0.31.0) (2024-06-17)


### ⚠ BREAKING CHANGES

* remove `dep::` prefix ([#4946](https://github.com/noir-lang/noir/issues/4946))
* remove `distinct` keyword ([#5219](https://github.com/noir-lang/noir/issues/5219))
* remove `param_witnesses` and `return_witnesses` from ABI ([#5154](https://github.com/noir-lang/noir/issues/5154))
* add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205))
* restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180))
* separate proving from `noir_js` ([#5072](https://github.com/noir-lang/noir/issues/5072))
* switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283)
* specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239)
* **stdlib:** eddsa function using turbofish ([#5050](https://github.com/noir-lang/noir/issues/5050))

### Features

* `pxe.addNullifiedNote(...)` (https://github.com/AztecProtocol/aztec-packages/pull/6948) ([7de19f5](https://github.com/noir-lang/noir/commit/7de19f5856591203271836f07154abae13f5102b))
* Activate return_data in ACIR opcodes ([#5080](https://github.com/noir-lang/noir/issues/5080)) ([c9fda3c](https://github.com/noir-lang/noir/commit/c9fda3c7fd4575bfe7d457e8d4230e071f0129a0))
* Add `as_witness` builtin function in order to constrain a witness to be equal to a variable  ([#4641](https://github.com/noir-lang/noir/issues/4641)) ([faf5bd8](https://github.com/noir-lang/noir/commit/faf5bd8ed80fb89b4bb6a2536b9bfa9649579da7))
* Add `set` and `set_unchecked` methods to `Vec` and `BoundedVec` ([#5241](https://github.com/noir-lang/noir/issues/5241)) ([1849389](https://github.com/noir-lang/noir/commit/1849389362e22e8236177f84b735dadf840cd637))
* Add BoundedVec::map ([#5250](https://github.com/noir-lang/noir/issues/5250)) ([da1549c](https://github.com/noir-lang/noir/commit/da1549cfb296261b273a3a64908382e7b71512ad))
* Add intrinsic to get if running inside an unconstrained context ([#5098](https://github.com/noir-lang/noir/issues/5098)) ([281ebf2](https://github.com/noir-lang/noir/commit/281ebf26e4cd16daf361938de505697f8d5fbd5e))
* Add native rust implementation of schnorr signature verification ([#5053](https://github.com/noir-lang/noir/issues/5053)) ([fab1c35](https://github.com/noir-lang/noir/commit/fab1c3567d731ea7902635a7a020a8d14f94fd27))
* Add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205)) ([14adafc](https://github.com/noir-lang/noir/commit/14adafc965fa9c833e096ec037e086aae67703ad))
* Consider block parameters in variable liveness ([#5097](https://github.com/noir-lang/noir/issues/5097)) ([e4eb5f5](https://github.com/noir-lang/noir/commit/e4eb5f539f377fd3c2e1a874707ffce62a5bc10a))
* **experimental:** Implement macro calls & splicing into `Expr` values ([#5203](https://github.com/noir-lang/noir/issues/5203)) ([d9b4712](https://github.com/noir-lang/noir/commit/d9b4712bf1a62548dd7ed17b181882ae537d70dd))
* Implement println in the comptime interpreter ([#5197](https://github.com/noir-lang/noir/issues/5197)) ([7f08343](https://github.com/noir-lang/noir/commit/7f08343dfcafddfcec1b238746a69273ae4f4e2b))
* Implement turbofish operator ([#3542](https://github.com/noir-lang/noir/issues/3542)) ([226724e](https://github.com/noir-lang/noir/commit/226724e3b54c2e0d9ba005661c76b40a87d9295a))
* Make ACVM generic across fields ([#5114](https://github.com/noir-lang/noir/issues/5114)) ([70f374c](https://github.com/noir-lang/noir/commit/70f374c06642962d8f2b95b80f8c938fcf7761d7))
* Move abi demonomorphizer to noir_codegen and use noir_codegen in protocol types (https://github.com/AztecProtocol/aztec-packages/pull/6302) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Move to_radix to a blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6294) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` (https://github.com/AztecProtocol/aztec-packages/pull/6131) ([ff67e14](https://github.com/noir-lang/noir/commit/ff67e145d086bf6fdf58fb5e57927033e52e03d3))
* Place return value witnesses directly after function arguments ([#5142](https://github.com/noir-lang/noir/issues/5142)) ([1252b5f](https://github.com/noir-lang/noir/commit/1252b5fcc7ed56bb55e95745b83be6e556805397))
* Private Kernel Recursion (https://github.com/AztecProtocol/aztec-packages/pull/6278) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Proper padding in ts AES and constrained AES in body and header computations (https://github.com/AztecProtocol/aztec-packages/pull/6269) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Remove `dep::` prefix ([#4946](https://github.com/noir-lang/noir/issues/4946)) ([d6d0ae2](https://github.com/noir-lang/noir/commit/d6d0ae26d2fef083dc240539b834d934c84b0326))
* Remove conditional compilation of `bn254_blackbox_solver` ([#5058](https://github.com/noir-lang/noir/issues/5058)) ([9420d7c](https://github.com/noir-lang/noir/commit/9420d7c2ba6bbbf5ecb9a066837c505310955b6c))
* Remove external blackbox solver from acir simulator (https://github.com/AztecProtocol/aztec-packages/pull/6586) ([a40a9a5](https://github.com/noir-lang/noir/commit/a40a9a55571deed386688fb84260bdf2794d4d38))
* Replace stdlib poseidon implementation with optimized version ([#5122](https://github.com/noir-lang/noir/issues/5122)) ([11e98f3](https://github.com/noir-lang/noir/commit/11e98f348d1d43a9b28d83ec3308027b7afc0da6))
* Restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180)) ([bdb2bc6](https://github.com/noir-lang/noir/commit/bdb2bc608ea8fd52d46545a38b68dd2558b28110))
* Separate proving from `noir_js` ([#5072](https://github.com/noir-lang/noir/issues/5072)) ([c93c738](https://github.com/noir-lang/noir/commit/c93c7380c705fcec5c77bfc436c2f5ea085edd77))
* Separate runtimes of SSA functions before inlining ([#5121](https://github.com/noir-lang/noir/issues/5121)) ([69eca9b](https://github.com/noir-lang/noir/commit/69eca9b8671fa54192bef814dd584fdb5387a5f7))
* Specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Standardize pedersen functions to return `EmbeddedCurvePoint` ([#5190](https://github.com/noir-lang/noir/issues/5190)) ([3b85b36](https://github.com/noir-lang/noir/commit/3b85b3637f81f3894a7faa07fd299f9d64747214))
* **stdlib:** Eddsa function using turbofish ([#5050](https://github.com/noir-lang/noir/issues/5050)) ([7936262](https://github.com/noir-lang/noir/commit/79362629ed8cf42b6601e9a551ed8f9fe03e0112))
* Support casting in globals ([#5164](https://github.com/noir-lang/noir/issues/5164)) ([6d3e732](https://github.com/noir-lang/noir/commit/6d3e732e06033b53506656acdd3d7759bd27f106))
* Switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6280) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6332) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6573) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6986) ([7de19f5](https://github.com/noir-lang/noir/commit/7de19f5856591203271836f07154abae13f5102b))
* ToRadix BB + avm transpiler support (https://github.com/AztecProtocol/aztec-packages/pull/6330) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))


### Bug Fixes

* Add support for nested arrays returned by oracles ([#5132](https://github.com/noir-lang/noir/issues/5132)) ([f846879](https://github.com/noir-lang/noir/commit/f846879dd038328bd0a1d39a72b448ef52a1002b))
* Apply self type from generic trait constraint before instantiating identifiers ([#5087](https://github.com/noir-lang/noir/issues/5087)) ([2b4755c](https://github.com/noir-lang/noir/commit/2b4755c2b57460d5eb839ee835f8c9acd5773a7c))
* Auto dereference trait methods in the elaborator ([#5124](https://github.com/noir-lang/noir/issues/5124)) ([56c1a85](https://github.com/noir-lang/noir/commit/56c1a85056ed338644595f1aa58cc94563786b9e))
* Check for public args in aztec functions (https://github.com/AztecProtocol/aztec-packages/pull/6355) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Disable `if` optimization ([#5240](https://github.com/noir-lang/noir/issues/5240)) ([a2816db](https://github.com/noir-lang/noir/commit/a2816dbf7f9d31967fc95205a43fdfdf181029b0))
* **elaborator:** Fix duplicate methods error ([#5225](https://github.com/noir-lang/noir/issues/5225)) ([87a1d8e](https://github.com/noir-lang/noir/commit/87a1d8ebaadb5f0f1ed637b96816f971f946af87))
* **elaborator:** Fix regression introduced by lazy-global changes ([#5223](https://github.com/noir-lang/noir/issues/5223)) ([fde432a](https://github.com/noir-lang/noir/commit/fde432aacc436b6c57f0d937d7c86836bac0b465))
* **elaborator:** Invert unconstrained check ([#5176](https://github.com/noir-lang/noir/issues/5176)) ([967c0fa](https://github.com/noir-lang/noir/commit/967c0fa76da9384afe918a8b23eef60f12f29292))
* **elaborator:** Lazily elaborate globals ([#5191](https://github.com/noir-lang/noir/issues/5191)) ([9c99a97](https://github.com/noir-lang/noir/commit/9c99a97ca8f42bee23cf97ebd724fdc51e647c60))
* Error for allocate instructions in acir-gen ([#5200](https://github.com/noir-lang/noir/issues/5200)) ([58c7532](https://github.com/noir-lang/noir/commit/58c7532da8dd86ee02b20d7e7809f5437f667845))
* **experimental elaborator:** Avoid calling `add_generics` twice on trait methods ([#5108](https://github.com/noir-lang/noir/issues/5108)) ([7d8c0a3](https://github.com/noir-lang/noir/commit/7d8c0a3a1ae143b574b2fa62cae7c0a493005c70))
* **experimental elaborator:** Clear generics after elaborating type aliases ([#5136](https://github.com/noir-lang/noir/issues/5136)) ([b0a7d0b](https://github.com/noir-lang/noir/commit/b0a7d0b12328d3ed9faed87b78792b77786018e0))
* **experimental elaborator:** Fix `impl Trait` when `--use-elaborator` is selected ([#5138](https://github.com/noir-lang/noir/issues/5138)) ([7ea5962](https://github.com/noir-lang/noir/commit/7ea5962e77b7183374a4e14da3a237ccd63f00a0))
* **experimental elaborator:** Fix definition kind of globals and tuple patterns with `--use-elaborator` flag ([#5139](https://github.com/noir-lang/noir/issues/5139)) ([a140dec](https://github.com/noir-lang/noir/commit/a140dec4580459c5856d44337de3ea08aa7fb44a))
* **experimental elaborator:** Fix duplicate `resolve_type` on self type and don't leak a trait impl's generics ([#5102](https://github.com/noir-lang/noir/issues/5102)) ([db561e2](https://github.com/noir-lang/noir/commit/db561e229cfcb35f23205cbb7e41fcf5ece68ee5))
* **experimental elaborator:** Fix frontend tests when `--use-elaborator` flag is specified ([#5145](https://github.com/noir-lang/noir/issues/5145)) ([d6122eb](https://github.com/noir-lang/noir/commit/d6122eb9e88aa2b1bb6c990e452fa9678ae49704))
* **experimental elaborator:** Fix global values used in the elaborator ([#5135](https://github.com/noir-lang/noir/issues/5135)) ([e73cdbb](https://github.com/noir-lang/noir/commit/e73cdbb93b0714331fef754f862d89c08c28a9e5))
* **experimental elaborator:** Fix globals which use function calls ([#5172](https://github.com/noir-lang/noir/issues/5172)) ([ab0b1a8](https://github.com/noir-lang/noir/commit/ab0b1a85cc91f8ed748ee393ece54f5c3b43d7ef))
* **experimental elaborator:** Fix panic in the elaborator ([#5082](https://github.com/noir-lang/noir/issues/5082)) ([ffcb410](https://github.com/noir-lang/noir/commit/ffcb410978a362c73783fbfe5bbdc9691499609e))
* **experimental elaborator:** Only call `add_generics` once ([#5091](https://github.com/noir-lang/noir/issues/5091)) ([f5d2946](https://github.com/noir-lang/noir/commit/f5d294645e82fc85d8dc28ee2a846ba11af85ce5))
* Fix panic in `get_global_let_statement` ([#5177](https://github.com/noir-lang/noir/issues/5177)) ([b769b01](https://github.com/noir-lang/noir/commit/b769b01fd06a6a2c66c72f9aa4e1d346b0fca123))
* **frontend:** Call trait method with mut self from generic definition ([#5041](https://github.com/noir-lang/noir/issues/5041)) ([89846cf](https://github.com/noir-lang/noir/commit/89846cfbc4961c5258d91b5973f027be80885a20))
* **frontend:** Correctly monomorphize turbofish functions ([#5049](https://github.com/noir-lang/noir/issues/5049)) ([fd772e7](https://github.com/noir-lang/noir/commit/fd772e7a764004373f5a41a54eb6847f4decda77))
* **frontend:** Resolve object types from method calls a single time ([#5131](https://github.com/noir-lang/noir/issues/5131)) ([3afe023](https://github.com/noir-lang/noir/commit/3afe023543e301aafaf2b79f0ccd6d7936dd53a9))
* Temporarily revert to_radix blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6304) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Use plain integer addresses for opcodes in DAP disassembly view ([#4941](https://github.com/noir-lang/noir/issues/4941)) ([d43ba1b](https://github.com/noir-lang/noir/commit/d43ba1bddbf6ebd56a7bee0e1db38d155fec95d5))
* Use predicate for curve operations ([#5076](https://github.com/noir-lang/noir/issues/5076)) ([145b909](https://github.com/noir-lang/noir/commit/145b90945486907cb6db75d3f3f93a58d19b2a32))
* Wrapping in signed division ([#5134](https://github.com/noir-lang/noir/issues/5134)) ([29baeb4](https://github.com/noir-lang/noir/commit/29baeb41e15918935c437e0a2759c6b936f125a4))


### Miscellaneous Chores

* Remove `distinct` keyword ([#5219](https://github.com/noir-lang/noir/issues/5219)) ([1d62c59](https://github.com/noir-lang/noir/commit/1d62c59a8f02f7d277c5bf9ed637348a3b2f399c))
* Remove `param_witnesses` and `return_witnesses` from ABI ([#5154](https://github.com/noir-lang/noir/issues/5154)) ([21562ae](https://github.com/noir-lang/noir/commit/21562aeea162d246573967115e7c519715f6d3d8))

## [0.30.0](https://github.com/noir-lang/noir/compare/v0.29.0...v0.30.0) (2024-05-20)


### ⚠ BREAKING CHANGES

* remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995)
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016)

### Features

* `multi_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6097) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* `variable_base_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6039) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Add `Not` trait to stdlib ([#4999](https://github.com/noir-lang/noir/issues/4999)) ([95d4d13](https://github.com/noir-lang/noir/commit/95d4d133d1eb5e0eb44cd928d8183d890e970a13))
* Add `std::ops::Neg` trait to stdlib ([07930d4](https://github.com/noir-lang/noir/commit/07930d4373a393146210efae69e6ec40171f047b))
* Add native rust implementations of pedersen functions ([#4871](https://github.com/noir-lang/noir/issues/4871)) ([fb039f7](https://github.com/noir-lang/noir/commit/fb039f74df23aea39bc0593a5d538d82b4efadf0))
* Add support for u16/i16 ([#4985](https://github.com/noir-lang/noir/issues/4985)) ([e43661d](https://github.com/noir-lang/noir/commit/e43661d16c1cd07e2af2392b88d29c689889ff9a))
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Do not return databus returndata, keep it private. ([#5023](https://github.com/noir-lang/noir/issues/5023)) ([a5b7df1](https://github.com/noir-lang/noir/commit/a5b7df12faf9d71ff24f8c5cde5e78da44558caf))
* Dynamic assertion payloads v2 (https://github.com/AztecProtocol/aztec-packages/pull/5949) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Implement `From` array trait for `BoundedVec` ([#4927](https://github.com/noir-lang/noir/issues/4927)) ([bf491dc](https://github.com/noir-lang/noir/commit/bf491dce9595d0e37057ae6d6721eb7a83cec0e2))
* Implement `ops` traits on `u16`/`i16` ([#4996](https://github.com/noir-lang/noir/issues/4996)) ([8b65663](https://github.com/noir-lang/noir/commit/8b65663f9e836c11a87e458bd7c6a52920448d5c))
* Implement `std::ops::Sub` on `EmbeddedCurvePoint` ([07930d4](https://github.com/noir-lang/noir/commit/07930d4373a393146210efae69e6ec40171f047b))
* Increase default expression width to 4 ([#4995](https://github.com/noir-lang/noir/issues/4995)) ([f01d309](https://github.com/noir-lang/noir/commit/f01d3090759a5ff0f1f83c5616d22890c6bd76be))
* Parsing non-string assertion payloads in noir js (https://github.com/AztecProtocol/aztec-packages/pull/6079) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove query to backend to get expression width ([#4975](https://github.com/noir-lang/noir/issues/4975)) ([e5f356b](https://github.com/noir-lang/noir/commit/e5f356b063fe4facbd14320b7eafed664d0bb027))
* Set aztec private functions to be recursive (https://github.com/AztecProtocol/aztec-packages/pull/6192) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))


### Bug Fixes

* Compute the correct slice length when coercing from a literal array of complex types ([#4986](https://github.com/noir-lang/noir/issues/4986)) ([f3f1150](https://github.com/noir-lang/noir/commit/f3f11507983009771656811f9570bdbe6849c7ef))
* Defer overflow checks for unsigned integers to acir-gen ([#4832](https://github.com/noir-lang/noir/issues/4832)) ([b577761](https://github.com/noir-lang/noir/commit/b5777613c51f26fb4f580b9168c4190b1f4bd8f7))
* Fix no predicates for brillig with intermediate functions ([#5015](https://github.com/noir-lang/noir/issues/5015)) ([9c6de4b](https://github.com/noir-lang/noir/commit/9c6de4b25d318c6d211361dd62a112a9d2432c56))
* Fixed several vulnerabilities in U128, added some tests ([#5024](https://github.com/noir-lang/noir/issues/5024)) ([e5ab24d](https://github.com/noir-lang/noir/commit/e5ab24d6a4154d11b3c8ae08f4431b7b93c76f23))
* Ignore no_predicates in brillig functions ([#5012](https://github.com/noir-lang/noir/issues/5012)) ([b541e79](https://github.com/noir-lang/noir/commit/b541e793e20fa3c991e0328ec2ff7926bdcdfd45))
* Set index and value to 0 for array_get with predicate ([#4971](https://github.com/noir-lang/noir/issues/4971)) ([c49d3a9](https://github.com/noir-lang/noir/commit/c49d3a9ded819b828cffdfc031e86614da21e329))


### Miscellaneous Chores

* Remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))

## [0.29.0](https://github.com/noir-lang/noir/compare/v0.28.0...v0.29.0) (2024-05-03)


### ⚠ BREAKING CHANGES

* use `distinct` return value witnesses by default ([#4951](https://github.com/noir-lang/noir/issues/4951))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))

### Features

* Add `#[inline(tag)]` attribute and codegen ([#4913](https://github.com/noir-lang/noir/issues/4913)) ([1ec9cdc](https://github.com/noir-lang/noir/commit/1ec9cdc7013e867db3672d27e3a6104e4b7e7eef))
* Add de-sugaring for `impl Trait` in function parameters ([#4919](https://github.com/noir-lang/noir/issues/4919)) ([8aad2e4](https://github.com/noir-lang/noir/commit/8aad2e45acbe08afc3902db95a83324f822c35eb))
* Add variable size sha256 ([#4920](https://github.com/noir-lang/noir/issues/4920)) ([dbfca58](https://github.com/noir-lang/noir/commit/dbfca58a817ee1f1512e3e02138119f363c3d12b))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Complex outputs from acir call ([#4952](https://github.com/noir-lang/noir/issues/4952)) ([2e085b9](https://github.com/noir-lang/noir/commit/2e085b935b143c1305b70cd7ae86907b61a45fc0))
* **experimental:** `comptime` globals ([#4918](https://github.com/noir-lang/noir/issues/4918)) ([8a3c7f1](https://github.com/noir-lang/noir/commit/8a3c7f1c11666ed5140a63a5aa296ef417c97bfa))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Handle `no_predicates` attribute ([#4942](https://github.com/noir-lang/noir/issues/4942)) ([0ce04d3](https://github.com/noir-lang/noir/commit/0ce04d3ea8734b76d96f5dd0fb2a6cdd4081969e))
* Handle empty response foreign calls without an external resolver ([#4959](https://github.com/noir-lang/noir/issues/4959)) ([0154bde](https://github.com/noir-lang/noir/commit/0154bdef9f6dfe45497d77ecbf3904dcc138b8d7))
* Optimize array sets in if conditions (alternate version) ([#4716](https://github.com/noir-lang/noir/issues/4716)) ([a87c655](https://github.com/noir-lang/noir/commit/a87c655c6c8c077c71e3372cc9181b7870348a3d))
* Use `distinct` return value witnesses by default ([#4951](https://github.com/noir-lang/noir/issues/4951)) ([5f1b584](https://github.com/noir-lang/noir/commit/5f1b58470779e977293323d10ab9a8f0857ea29e))


### Bug Fixes

* Ban self-referential structs ([#4883](https://github.com/noir-lang/noir/issues/4883)) ([800f670](https://github.com/noir-lang/noir/commit/800f670b63a5a2ae08f09a86dae767089f7f67af))
* Discard ref counts during unrolling ([#4923](https://github.com/noir-lang/noir/issues/4923)) ([91062db](https://github.com/noir-lang/noir/commit/91062db84a749bf191eae9ce487a2315cc74bfb2))
* Ensure where clauses propagated to trait default definitions ([#4894](https://github.com/noir-lang/noir/issues/4894)) ([aaac0f6](https://github.com/noir-lang/noir/commit/aaac0f6bffbe11eb090145354f1b82919bb93cb7))
* Move remove_if_else pass after second inlining  ([#4976](https://github.com/noir-lang/noir/issues/4976)) ([96fb3e9](https://github.com/noir-lang/noir/commit/96fb3e94b3a2f7b586d17ea9445f44267f5d9c6d))
* Nested array equality ([#4903](https://github.com/noir-lang/noir/issues/4903)) ([0cf2e2a](https://github.com/noir-lang/noir/commit/0cf2e2a1b8d247bed03ba5b7b1be5cd30f0d51b2))
* Require for all foldable functions to use distinct return  ([#4949](https://github.com/noir-lang/noir/issues/4949)) ([d4c6806](https://github.com/noir-lang/noir/commit/d4c68066ab35ce1c52510cf0c038fb627a0677c3))
* Use annotated type when checking declaration ([#4966](https://github.com/noir-lang/noir/issues/4966)) ([f7fa696](https://github.com/noir-lang/noir/commit/f7fa69661006e1e10ddeecee1cdf8f024d6bc3e9))

## [0.28.0](https://github.com/noir-lang/noir/compare/v0.27.0...v0.28.0) (2024-04-24)


### ⚠ BREAKING CHANGES

* Add `as_array` and remove `_slice` variants of hash functions ([#4675](https://github.com/noir-lang/noir/issues/4675))
* reserve keyword `super` ([#4836](https://github.com/noir-lang/noir/issues/4836))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)

### Features

* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Add `min` and `max` functions to the stdlib ([#4839](https://github.com/noir-lang/noir/issues/4839)) ([6cfb328](https://github.com/noir-lang/noir/commit/6cfb328d0d162eaa20ad1a118d085e03a52d049d))
* Add `NARGO_FOREIGN_CALL_TIMEOUT` environment variable ([#4780](https://github.com/noir-lang/noir/issues/4780)) ([791f1c8](https://github.com/noir-lang/noir/commit/791f1c8522d49972dad4eb940f9cad437e28b25b))
* Add comptime Interpreter ([#4821](https://github.com/noir-lang/noir/issues/4821)) ([5992436](https://github.com/noir-lang/noir/commit/599243633281e6827f0f4f095fb12d313e0125fa))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Allow numeric generics to non inlined ACIR functions ([#4834](https://github.com/noir-lang/noir/issues/4834)) ([9cc03a4](https://github.com/noir-lang/noir/commit/9cc03a4d6f714a1b2d31c6982eb8e791ba5c869c))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **experimental:** Add `comptime` keyword ([#4840](https://github.com/noir-lang/noir/issues/4840)) ([4dfd7f0](https://github.com/noir-lang/noir/commit/4dfd7f03bc1b9cf57f5829c435a560bed53b7f46))
* Get last mock oracles params ([#4789](https://github.com/noir-lang/noir/issues/4789)) ([1d96937](https://github.com/noir-lang/noir/commit/1d96937a8e94a91c0c17c97102498d067fca76c3))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Implement `Eq` trait on `BoundedVec` ([#4830](https://github.com/noir-lang/noir/issues/4830)) ([6cefe16](https://github.com/noir-lang/noir/commit/6cefe16deb643951c0cc552d08e22272900ed456))
* Lalrpop lexer prototype ([#4656](https://github.com/noir-lang/noir/issues/4656)) ([25ad018](https://github.com/noir-lang/noir/commit/25ad018a55b61dd861e899f050c48200f0a00430))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* Narrow ABI encoding errors down to target problem argument/field ([#4798](https://github.com/noir-lang/noir/issues/4798)) ([e412e6e](https://github.com/noir-lang/noir/commit/e412e6e30910472b9d5f9000370ce5138ad39ce7))
* Proving the rollup circuits (https://github.com/AztecProtocol/aztec-packages/pull/5599) ([5b352d6](https://github.com/noir-lang/noir/commit/5b352d6266c40522f5626f79d2f36a409b482aaa))
* Reserve keyword `super` ([#4836](https://github.com/noir-lang/noir/issues/4836)) ([d5028a6](https://github.com/noir-lang/noir/commit/d5028a613e5a65ad1286dd20ce0fb0313f19f6ee))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Simplify `BoundedVec::eq` ([#4838](https://github.com/noir-lang/noir/issues/4838)) ([3d33a33](https://github.com/noir-lang/noir/commit/3d33a33e74c3e7d0fc511059b07f0ef9ddd9b667))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Split `backend_barretenburg` into prover and verifier classes ([#4769](https://github.com/noir-lang/noir/issues/4769)) ([ce1e662](https://github.com/noir-lang/noir/commit/ce1e6624ece3c91f06b0273af9ba88e703c1b589))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5725) ([5b352d6](https://github.com/noir-lang/noir/commit/5b352d6266c40522f5626f79d2f36a409b482aaa))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Unroll loops iteratively ([#4779](https://github.com/noir-lang/noir/issues/4779)) ([f831b0b](https://github.com/noir-lang/noir/commit/f831b0bdbf99cab1bcd24d494c4546a36309465e))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* ArrayGet and Set are not pure ([#4783](https://github.com/noir-lang/noir/issues/4783)) ([90ee479](https://github.com/noir-lang/noir/commit/90ee4792c8e7115e55a3b1dadd1e43066ad8ac66))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **experimental:** Skip over comptime functions in scan pass ([#4893](https://github.com/noir-lang/noir/issues/4893)) ([f267d42](https://github.com/noir-lang/noir/commit/f267d4205b46317eacb1c247c9dca0e7698d1259))
* Fix curve parameters for bigints ([#4900](https://github.com/noir-lang/noir/issues/4900)) ([5985e42](https://github.com/noir-lang/noir/commit/5985e4285de9e29f7c986103a49fdaec59228887))
* Fix panic when returning a zeroed unit value ([#4797](https://github.com/noir-lang/noir/issues/4797)) ([2ea9292](https://github.com/noir-lang/noir/commit/2ea92926956658ea99d8fb97734831eba00d3a4b))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Primary_message typo in errors.rs (https://github.com/AztecProtocol/aztec-packages/pull/5646) ([5b352d6](https://github.com/noir-lang/noir/commit/5b352d6266c40522f5626f79d2f36a409b482aaa))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Reset the noir-gates-diff report on master ([#4878](https://github.com/noir-lang/noir/issues/4878)) ([50bc325](https://github.com/noir-lang/noir/commit/50bc32587a837c930ed14175c98ace1530c54bef))
* Update noir-gates-diff commit to use master reference report ([#4891](https://github.com/noir-lang/noir/issues/4891)) ([4a3ffb7](https://github.com/noir-lang/noir/commit/4a3ffb7b4c5cdd5fcadb19e7f251b1ee27b0c02b))


### Miscellaneous Chores

* Add `as_array` and remove `_slice` variants of hash functions ([#4675](https://github.com/noir-lang/noir/issues/4675)) ([8e39706](https://github.com/noir-lang/noir/commit/8e39706cbb51f27b42fbe851aaa6a67070d07c74))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))

## [0.27.0](https://github.com/noir-lang/noir/compare/v0.26.0...v0.27.0) (2024-04-10)


### ⚠ BREAKING CHANGES

* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)

### Features

* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add `remove_enable_side_effects` SSA pass ([#4224](https://github.com/noir-lang/noir/issues/4224)) ([94952db](https://github.com/noir-lang/noir/commit/94952db604b70a1ec18115b291de3c52565a641e))
* Allow slices to brillig entry points ([#4713](https://github.com/noir-lang/noir/issues/4713)) ([62423d5](https://github.com/noir-lang/noir/commit/62423d552beca749b6f86b1330555aab18db58d0))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* **docs:** Documenting noir codegen ([#4454](https://github.com/noir-lang/noir/issues/4454)) ([24f6d85](https://github.com/noir-lang/noir/commit/24f6d85f2467a109399d21729f8bb0f97c5ba6db))
* Improve nargo check cli with --override flag and feedback for existing files ([#4575](https://github.com/noir-lang/noir/issues/4575)) ([5e7fbd4](https://github.com/noir-lang/noir/commit/5e7fbd4e706b1691ba2dd960469cfa3b31dfb753))
* Improve optimisations on range constraints ([#4690](https://github.com/noir-lang/noir/issues/4690)) ([96b8110](https://github.com/noir-lang/noir/commit/96b811079b0e7c0345210cfc705c00345b0b3334))
* Improve SSA type-awareness in EQ and MUL instructions ([#4691](https://github.com/noir-lang/noir/issues/4691)) ([669f1a0](https://github.com/noir-lang/noir/commit/669f1a0fa47ad9093888a8ce8e525cb02bcf19b5))
* **nargo:** Multiple circuits info for binary programs ([#4719](https://github.com/noir-lang/noir/issues/4719)) ([50d2735](https://github.com/noir-lang/noir/commit/50d2735825454a8638a308156d4ea23b3c4420d8))


### Bug Fixes

* "Types in a binary operation should match, but found T and T" ([#4648](https://github.com/noir-lang/noir/issues/4648)) ([30c9f31](https://github.com/noir-lang/noir/commit/30c9f3151d447de8c7467ccbee82e32b8c46a396))
* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Correct ICE panic messages in brillig `convert_black_box_call` ([#4761](https://github.com/noir-lang/noir/issues/4761)) ([f3eee6c](https://github.com/noir-lang/noir/commit/f3eee6c00a9b1ea939c5757d91faac693e909301))
* Error when a type variable is unbound during monomorphization instead of defaulting to Field ([#4674](https://github.com/noir-lang/noir/issues/4674)) ([03cdba4](https://github.com/noir-lang/noir/commit/03cdba45ac073fd6fdd91549736f36f1abaef15a))
* Field comparisons ([#4704](https://github.com/noir-lang/noir/issues/4704)) ([079cb2a](https://github.com/noir-lang/noir/commit/079cb2a99d2d50b50688bfb56fa014acde3e3d71))
* Impl search no longer selects an impl if multiple are applicable ([#4662](https://github.com/noir-lang/noir/issues/4662)) ([0150600](https://github.com/noir-lang/noir/commit/0150600922ee8b3e67c9b592338e8832f446685b))
* Last use analysis & make it an SSA pass ([#4686](https://github.com/noir-lang/noir/issues/4686)) ([0d3d5fd](https://github.com/noir-lang/noir/commit/0d3d5fda9659a563ba9c2014b7c1af9e1d332ab0))
* Slice coercions ([#4640](https://github.com/noir-lang/noir/issues/4640)) ([c0bae17](https://github.com/noir-lang/noir/commit/c0bae17e70f55ebf4b1639e0dfb075d8c5c97892))
* **ssa:** Accurate constant type for slice dummy data in flattening ([#4661](https://github.com/noir-lang/noir/issues/4661)) ([b87654e](https://github.com/noir-lang/noir/commit/b87654e2b4761dfacc916dac70d43c1b572ec636))
* **ssa:** Do not use get_value_max_num_bits when we want pure type information ([#4700](https://github.com/noir-lang/noir/issues/4700)) ([b55a580](https://github.com/noir-lang/noir/commit/b55a580388abc95bab6c6ef8c50eae3c5497eb3f))
* **ssa:** Fix slice intrinsic handling in the capacity tracker  ([#4643](https://github.com/noir-lang/noir/issues/4643)) ([1b50ce1](https://github.com/noir-lang/noir/commit/1b50ce155cf95193937729c2a23f34b0ade42ea0))
* Unknown slice lengths coming from as_slice ([#4725](https://github.com/noir-lang/noir/issues/4725)) ([f21129e](https://github.com/noir-lang/noir/commit/f21129ef05efb76c5df6ee15a134f1ea535d8e90))
* Update commit for noir-gates-diff ([#4773](https://github.com/noir-lang/noir/issues/4773)) ([a9766c5](https://github.com/noir-lang/noir/commit/a9766c5e9650160bcafc693f2617e441ed47721a))

## [0.26.0](https://github.com/noir-lang/noir/compare/v0.25.0...v0.26.0) (2024-03-25)


### ⚠ BREAKING CHANGES

* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* separating out array and slice types in the AST ([#4504](https://github.com/noir-lang/noir/issues/4504))
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Remove open keyword from Noir (https://github.com/AztecProtocol/aztec-packages/pull/4967)

### Features

* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add `break` and `continue` in unconstrained code ([#4569](https://github.com/noir-lang/noir/issues/4569)) ([f2f827d](https://github.com/noir-lang/noir/commit/f2f827d51e6fe99fa3d17f125b22743da25e25be))
* Add `nargo compile --watch` command ([#4464](https://github.com/noir-lang/noir/issues/4464)) ([44e60b6](https://github.com/noir-lang/noir/commit/44e60b67469de88f20842c4eead64d736f7bd4a0))
* Add as_slice builtin function, add execution test ([#4523](https://github.com/noir-lang/noir/issues/4523)) ([6a9ea35](https://github.com/noir-lang/noir/commit/6a9ea35c4f1578058179aa08eedf44eb18bad4a1))
* Add checks for bit size consistency on brillig gen ([#4542](https://github.com/noir-lang/noir/issues/4542)) ([f3243b7](https://github.com/noir-lang/noir/commit/f3243b763c0b15ae90beb8e35630df27f3d314c0))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add experimental `quote` expression to parser ([#4595](https://github.com/noir-lang/noir/issues/4595)) ([4c3a30b](https://github.com/noir-lang/noir/commit/4c3a30b4991a329d3c52e1dfa59d854d7e6910db))
* Add more impls on Option ([#4549](https://github.com/noir-lang/noir/issues/4549)) ([4cf700b](https://github.com/noir-lang/noir/commit/4cf700bcfe157ebc82cdf7321a16959b7a4add57))
* Add specific error for attempting `string[x] = ".."` ([#4611](https://github.com/noir-lang/noir/issues/4611)) ([ff95fd9](https://github.com/noir-lang/noir/commit/ff95fd93451b2053360a16b7d3204ca251199296))
* Allow usage of noir `#[test]` syntax in stdlib ([#4553](https://github.com/noir-lang/noir/issues/4553)) ([a8b7cdb](https://github.com/noir-lang/noir/commit/a8b7cdb8a3698bc8923b6fa8714deebb8bf3923f))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Check initialization arguments in constructors (https://github.com/AztecProtocol/aztec-packages/pull/5144) ([d4213a0](https://github.com/noir-lang/noir/commit/d4213a03c9f77ee8e7663fc965a825258d90a368))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Integrated native ACVM (https://github.com/AztecProtocol/aztec-packages/pull/4903) ([a6016b4](https://github.com/noir-lang/noir/commit/a6016b46abf6da6de4566cf6d35a675d805dd9b5))
* Make brillig-gen more AVM-friendly (https://github.com/AztecProtocol/aztec-packages/pull/5091) ([a6016b4](https://github.com/noir-lang/noir/commit/a6016b46abf6da6de4566cf6d35a675d805dd9b5))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Optimize sha2 implementation ([#4441](https://github.com/noir-lang/noir/issues/4441)) ([80373d6](https://github.com/noir-lang/noir/commit/80373d612c023e3e165b49b6d1729486b0ba3b4b))
* RC optimization pass ([#4560](https://github.com/noir-lang/noir/issues/4560)) ([dfa5126](https://github.com/noir-lang/noir/commit/dfa5126f2c65843c34701cacddf2cbcfb0d7ff11))
* Remove curly braces with fmt  ([#4529](https://github.com/noir-lang/noir/issues/4529)) ([fe9a437](https://github.com/noir-lang/noir/commit/fe9a437b6d7ddc3f78665df1a576236555880c51))
* Separating out array and slice types in the AST ([#4504](https://github.com/noir-lang/noir/issues/4504)) ([9a241f9](https://github.com/noir-lang/noir/commit/9a241f9622b342cd9d56bf8481219cfc374c0510))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Visible aliases for nargo commands ([#4453](https://github.com/noir-lang/noir/issues/4453)) ([773cf19](https://github.com/noir-lang/noir/commit/773cf190ee21381d826ba80391a5d7d5efae9174))


### Bug Fixes

* **acir_gen:** More granular element sizes array check ([#4528](https://github.com/noir-lang/noir/issues/4528)) ([f93d16e](https://github.com/noir-lang/noir/commit/f93d16e3e89c5df358c982deae4f3c2d4c82b77f))
* Added error messages for passing oracles and references from unconstrained to constrained functions ([#4570](https://github.com/noir-lang/noir/issues/4570)) ([265bd8b](https://github.com/noir-lang/noir/commit/265bd8b284e5acd572a3812a94a99fc102227ff2))
* Allow non-integer globals to reference struct methods ([#4490](https://github.com/noir-lang/noir/issues/4490)) ([00d6494](https://github.com/noir-lang/noir/commit/00d6494ae70b10e1872d96fb4e57ecb0b5f01787))
* Dynamic assert messages in brillig ([#4531](https://github.com/noir-lang/noir/issues/4531)) ([e24d3fc](https://github.com/noir-lang/noir/commit/e24d3fc5a084610d9511e3c5421275cb9c84a548))
* Evaluate operators in globals in types ([#4537](https://github.com/noir-lang/noir/issues/4537)) ([c8aa16b](https://github.com/noir-lang/noir/commit/c8aa16bc7e78456cce1736fac82496996a8761f4))
* Make `nargo` the default binary for cargo run ([#4554](https://github.com/noir-lang/noir/issues/4554)) ([de4986e](https://github.com/noir-lang/noir/commit/de4986eb74b28b2e1065fa6b413d02457ddf61b0))
* Signed integer comparisons in brillig ([#4579](https://github.com/noir-lang/noir/issues/4579)) ([938d5e8](https://github.com/noir-lang/noir/commit/938d5e85eda00a05de5014e64d3dc9fc7c24936d))
* **ssa:** Use accurate type during SSA AsSlice simplficiation ([#4610](https://github.com/noir-lang/noir/issues/4610)) ([0473497](https://github.com/noir-lang/noir/commit/04734976e92475b1ab94257e30bc3438c7358681))
* Substitute generics when checking the field count of a type ([#4547](https://github.com/noir-lang/noir/issues/4547)) ([eeeebac](https://github.com/noir-lang/noir/commit/eeeebacd10698e847f773e26dac8a4a5eb8e84ed))


### Miscellaneous Chores

* Remove open keyword from Noir (https://github.com/AztecProtocol/aztec-packages/pull/4967) ([a6016b4](https://github.com/noir-lang/noir/commit/a6016b46abf6da6de4566cf6d35a675d805dd9b5))

## [0.25.0](https://github.com/noir-lang/noir/compare/v0.24.0...v0.25.0) (2024-03-11)


### ⚠ BREAKING CHANGES

* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* reserve `unchecked` keyword ([#4432](https://github.com/noir-lang/noir/issues/4432))
* Remove empty value from bounded vec ([#4431](https://github.com/noir-lang/noir/issues/4431))
* Ban Fields in for loop indices and bitwise ops ([#4376](https://github.com/noir-lang/noir/issues/4376))
* Bump msrv to 1.73.0 ([#4406](https://github.com/noir-lang/noir/issues/4406))
* **ci:** Bump MSRV to 1.72.1 and enforce that ACVM can be published using updated lockfile ([#4385](https://github.com/noir-lang/noir/issues/4385))
* Restrict bit sizes ([#4235](https://github.com/noir-lang/noir/issues/4235))
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)

### Features

* Add eddsa_poseidon_to_pub function to stdlib with test + docs ([#4473](https://github.com/noir-lang/noir/issues/4473)) ([00d2c32](https://github.com/noir-lang/noir/commit/00d2c32e58176cc5de3574c8435a54d415c4a5fa))
* Add HashMap to the stdlib ([#4242](https://github.com/noir-lang/noir/issues/4242)) ([650ffc5](https://github.com/noir-lang/noir/commit/650ffc5053cdca4b6ad2e027fa1f4fd90ef64871))
* Add option to set max memory for bb.js ([#4227](https://github.com/noir-lang/noir/issues/4227)) ([8a6b131](https://github.com/noir-lang/noir/commit/8a6b131402892a570bc2de6f5869de73b0bd979e))
* Add overflow and underflow checks for unsigned integers in brillig ([#4445](https://github.com/noir-lang/noir/issues/4445)) ([21fc4b8](https://github.com/noir-lang/noir/commit/21fc4b85763dccae6dce0a46a318718c3c913471))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow type aliases to reference other aliases ([#4353](https://github.com/noir-lang/noir/issues/4353)) ([c44ef14](https://github.com/noir-lang/noir/commit/c44ef14847a436733206b6dd9590a7ab214ecd97))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* **ci:** Use wasm-opt when compiling wasm packages ([#4334](https://github.com/noir-lang/noir/issues/4334)) ([e382921](https://github.com/noir-lang/noir/commit/e3829213d8411f84e117a14b43816967925095e0))
* DAP Preflight and debugger compilation options ([#4185](https://github.com/noir-lang/noir/issues/4185)) ([e0ad0b2](https://github.com/noir-lang/noir/commit/e0ad0b2b31f6d46be75d23aec6a82850a9c4bd75))
* Expose separate functions to compile programs vs contracts in `noir_wasm` ([#4413](https://github.com/noir-lang/noir/issues/4413)) ([7cd5fdb](https://github.com/noir-lang/noir/commit/7cd5fdb3d2a53475b7c8681231d517cab30f9f9b))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Restrict bit sizes ([#4235](https://github.com/noir-lang/noir/issues/4235)) ([1048f81](https://github.com/noir-lang/noir/commit/1048f815abb1f27e9c84ab5b9568a3673c12a50a))
* Run tests in parallel in `nargo test`  ([#4484](https://github.com/noir-lang/noir/issues/4484)) ([761734e](https://github.com/noir-lang/noir/commit/761734e6cb3ff5911aa85d0cee96ad26092b4905))
* Skip redundant range checks in brillig ([#4460](https://github.com/noir-lang/noir/issues/4460)) ([cb4c1c5](https://github.com/noir-lang/noir/commit/cb4c1c5264b95d01f69d99f916ced71ad9cdc9d1))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Track stack frames and their variables in the debugger ([#4188](https://github.com/noir-lang/noir/issues/4188)) ([ae1a9d9](https://github.com/noir-lang/noir/commit/ae1a9d923998177516919bbba6ff4b0584fa1e9f))
* TypeVariableKind for just Integers ([#4118](https://github.com/noir-lang/noir/issues/4118)) ([c956be8](https://github.com/noir-lang/noir/commit/c956be870fb47403a6da6585fce6bea2d40ee268))
* Update error message when trying to load workspace as dependency ([#4393](https://github.com/noir-lang/noir/issues/4393)) ([d2585e7](https://github.com/noir-lang/noir/commit/d2585e738a63208fca3c9e26242e896d7f1df1e4))


### Bug Fixes

* **acir:** Array dynamic flatten ([#4351](https://github.com/noir-lang/noir/issues/4351)) ([b2aaeab](https://github.com/noir-lang/noir/commit/b2aaeab319a0c66c431a7db6852f743eccde8e98))
* **acir:** Use types on dynamic arrays ([#4364](https://github.com/noir-lang/noir/issues/4364)) ([ba2c541](https://github.com/noir-lang/noir/commit/ba2c541ec45de92bba98de34771b73cbb7865c93))
* Add `follow_bindings` to follow `Type::Alias` links ([#4521](https://github.com/noir-lang/noir/issues/4521)) ([b94adb9](https://github.com/noir-lang/noir/commit/b94adb92657e2b4a51dc7216a88e080aed1cf8b0))
* Add handling to `noir_wasm` for projects without dependencies ([#4344](https://github.com/noir-lang/noir/issues/4344)) ([4982251](https://github.com/noir-lang/noir/commit/49822511710a7f1c42b8ed343e80456f8e6db2d9))
* Allow type aliases in main ([#4505](https://github.com/noir-lang/noir/issues/4505)) ([8a5359c](https://github.com/noir-lang/noir/commit/8a5359c012579e54c2766de1074482a36ecada32))
* Ban Fields in for loop indices and bitwise ops ([#4376](https://github.com/noir-lang/noir/issues/4376)) ([601fd9a](https://github.com/noir-lang/noir/commit/601fd9afc502236af1db0c4492698ba2298c7501))
* Brillig range check with consistent bit size ([#4357](https://github.com/noir-lang/noir/issues/4357)) ([ea47d4a](https://github.com/noir-lang/noir/commit/ea47d4a67c6a18e4a7d3a49079d9eb24a1026a25))
* Build noir_codegen when publishing ([#4448](https://github.com/noir-lang/noir/issues/4448)) ([cb1ceee](https://github.com/noir-lang/noir/commit/cb1ceee58b11b0ce6f8845361af3418d13c506bd))
* Consistent bit size for truncate ([#4370](https://github.com/noir-lang/noir/issues/4370)) ([dcd7a1e](https://github.com/noir-lang/noir/commit/dcd7a1e561a68504b9038ffbb3c80f5c981f9f0c))
* Correct formatting for databus visibility types ([#4423](https://github.com/noir-lang/noir/issues/4423)) ([cd796de](https://github.com/noir-lang/noir/commit/cd796dea4937dd1a261f154e5f2e599bbc649165))
* Correct invalid brillig codegen for `EmbeddedCurvePoint.add` ([#4382](https://github.com/noir-lang/noir/issues/4382)) ([5051ec4](https://github.com/noir-lang/noir/commit/5051ec4d434a9e5cf405c68357faaf213e68de9e))
* **docs:** Update install versions ([#4396](https://github.com/noir-lang/noir/issues/4396)) ([b283637](https://github.com/noir-lang/noir/commit/b283637e092038eb296c468168aec2d41e1c2734))
* **docs:** Update noirjs_app for 0.23 ([#4378](https://github.com/noir-lang/noir/issues/4378)) ([f77f702](https://github.com/noir-lang/noir/commit/f77f702e0cfb81dcce4dd97e274b831e887ba5d2))
* Enforce matching types of binary ops in SSA ([#4391](https://github.com/noir-lang/noir/issues/4391)) ([70866ae](https://github.com/noir-lang/noir/commit/70866aea976d59dbcbd4af34067fdd8f46555673))
* Fix brillig slowdown when assigning arrays in loops ([#4472](https://github.com/noir-lang/noir/issues/4472)) ([2a53545](https://github.com/noir-lang/noir/commit/2a53545f4238c9b8535e6bc5b0720fa15f44f946))
* **flake:** Stop flake.nix removing ignored-tests.txt ([#4455](https://github.com/noir-lang/noir/issues/4455)) ([ebaf05a](https://github.com/noir-lang/noir/commit/ebaf05ab10834dd10e04c7ea5130f96c6cdf98ed))
* Force src impl for == on slices ([#4507](https://github.com/noir-lang/noir/issues/4507)) ([1691274](https://github.com/noir-lang/noir/commit/169127444e8b16a8aad4acfe29ba812894fd897c))
* Handling of gh deps in noir_wasm ([#4499](https://github.com/noir-lang/noir/issues/4499)) ([1d65370](https://github.com/noir-lang/noir/commit/1d653704715bf9999eb6a40ed7500e752e2c73b7))
* Iterative flattening pass ([#4492](https://github.com/noir-lang/noir/issues/4492)) ([33c1ef7](https://github.com/noir-lang/noir/commit/33c1ef70e7859fdee7babfb5d38191f53e73a0df))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Only add `.nr` files to file manager ([#4380](https://github.com/noir-lang/noir/issues/4380)) ([8536c7c](https://github.com/noir-lang/noir/commit/8536c7c8ea8fc6b740b2ae6d1aef3bc7e1907b8c))
* Remove panic when generic array length is not resolvable ([#4408](https://github.com/noir-lang/noir/issues/4408)) ([00ab3db](https://github.com/noir-lang/noir/commit/00ab3db86b06111d144516e862902b8604284611))
* Remove print from monomorphization pass ([#4417](https://github.com/noir-lang/noir/issues/4417)) ([27c66b3](https://github.com/noir-lang/noir/commit/27c66b3d0741e68ed591ae8a16b47b30bc87175f))
* **ssa:** Handle mergers of slices returned from calls ([#4496](https://github.com/noir-lang/noir/issues/4496)) ([f988d02](https://github.com/noir-lang/noir/commit/f988d020e43cdf36a38613f2052d4518de39193a))
* Use correct type for numeric generics ([#4386](https://github.com/noir-lang/noir/issues/4386)) ([0a1d109](https://github.com/noir-lang/noir/commit/0a1d109f478c997da5c43876fd12464af638bb15))
* Variables from trait constraints being permanently bound over when used within a trait impl ([#4450](https://github.com/noir-lang/noir/issues/4450)) ([ac60ef5](https://github.com/noir-lang/noir/commit/ac60ef5e12fcfb907fbdcff709d7cbad05f2b939))


### Miscellaneous Chores

* Bump msrv to 1.73.0 ([#4406](https://github.com/noir-lang/noir/issues/4406)) ([b5e5c30](https://github.com/noir-lang/noir/commit/b5e5c30f4db52c79ef556e80660f39db369b1911))
* **ci:** Bump MSRV to 1.72.1 and enforce that ACVM can be published using updated lockfile ([#4385](https://github.com/noir-lang/noir/issues/4385)) ([2fc95d2](https://github.com/noir-lang/noir/commit/2fc95d2d82b3220267ce7d5815e7073e00ef1360))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove empty value from bounded vec ([#4431](https://github.com/noir-lang/noir/issues/4431)) ([b9384fb](https://github.com/noir-lang/noir/commit/b9384fb23abf4ab15e880fb7e03c21509a9fa8a6))
* Reserve `unchecked` keyword ([#4432](https://github.com/noir-lang/noir/issues/4432)) ([9544813](https://github.com/noir-lang/noir/commit/9544813fabbd18a87dd88456e6a5b781bd0cf008))

## [0.24.0](https://github.com/noir-lang/noir/compare/v0.23.0...v0.24.0) (2024-02-12)


### ⚠ BREAKING CHANGES

* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))

### Features

* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add brillig array index check ([#4127](https://github.com/noir-lang/noir/issues/4127)) ([c29f85f](https://github.com/noir-lang/noir/commit/c29f85fb5b1795e47282e4dbfbc1ceed2feb420c))
* Add definitions for From and Into traits to Noir prelude ([#4169](https://github.com/noir-lang/noir/issues/4169)) ([4421ce4](https://github.com/noir-lang/noir/commit/4421ce4f8f91c7fcac34fbdb76e204df93a46df8))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add option to print monomorphized program ([#4119](https://github.com/noir-lang/noir/issues/4119)) ([80f7e29](https://github.com/noir-lang/noir/commit/80f7e29340ceb88781dc80a13325468ace3b0cf3))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Add warnings for usage of restricted bit sizes ([#4234](https://github.com/noir-lang/noir/issues/4234)) ([0ffc38b](https://github.com/noir-lang/noir/commit/0ffc38bc8e91291c21cad3682ef77250e3c1e237))
* Allow bitshifts to be represented in SSA for brillig ([#4301](https://github.com/noir-lang/noir/issues/4301)) ([d86ff1a](https://github.com/noir-lang/noir/commit/d86ff1a16eed0a3f2994176c9399dafaf5bde108))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow globals to refer to any expression ([#4293](https://github.com/noir-lang/noir/issues/4293)) ([479330e](https://github.com/noir-lang/noir/commit/479330e9e767e0c06908a63c975341d9f83b5e7a))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Deallocate stack items at the instruction level ([#4339](https://github.com/noir-lang/noir/issues/4339)) ([8f024a8](https://github.com/noir-lang/noir/commit/8f024a86d615da5e10bb198e1a5fca6d565ef547))
* Disable constraint bubbling pass ([#4131](https://github.com/noir-lang/noir/issues/4131)) ([9ba2de6](https://github.com/noir-lang/noir/commit/9ba2de6143cd678b8656a84fab890e836257a13d))
* Disable unused variable checks on low-level and oracle functions ([#4179](https://github.com/noir-lang/noir/issues/4179)) ([8f70e57](https://github.com/noir-lang/noir/commit/8f70e57ded3b8a46388eedf1c0ec83772f88733e))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Improve Error Handling for Cargo in Bootstrap Script ([#4211](https://github.com/noir-lang/noir/issues/4211)) ([3a90849](https://github.com/noir-lang/noir/commit/3a908491d649be503df24038fc1eab875d77c8f1))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **lsp:** Goto type reference for Struct ([#4091](https://github.com/noir-lang/noir/issues/4091)) ([d56cac2](https://github.com/noir-lang/noir/commit/d56cac2af7dc1cce0795f8e9701bb17cc3e67e14))
* Move bounded_vec into the noir stdlib ([#4197](https://github.com/noir-lang/noir/issues/4197)) ([c50621f](https://github.com/noir-lang/noir/commit/c50621f1acddfb9138d6a036fd78c7a6c08dd084))
* Multiply first to allow more ACIR gen optimizations ([#4201](https://github.com/noir-lang/noir/issues/4201)) ([882639d](https://github.com/noir-lang/noir/commit/882639de109f0ecccf2a8522e2181a301145e19f))
* Option expect method ([#4219](https://github.com/noir-lang/noir/issues/4219)) ([8e042f2](https://github.com/noir-lang/noir/commit/8e042f2cbcc8a698aa45241aedbb0131b4acdc46))
* Perform constraints on uncasted values if they are the same type ([#4303](https://github.com/noir-lang/noir/issues/4303)) ([816fa85](https://github.com/noir-lang/noir/commit/816fa85d6fcb081d79d1a255b1503324ce53f71d))
* Remove predicate from `sort` intrinsic function ([#4228](https://github.com/noir-lang/noir/issues/4228)) ([d646243](https://github.com/noir-lang/noir/commit/d646243b2e8deff64d4d7dfe379d9caeba81c3b5))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Replace bitwise ANDs used for truncation with `Instruction::Truncate` ([#4327](https://github.com/noir-lang/noir/issues/4327)) ([eb67ff6](https://github.com/noir-lang/noir/commit/eb67ff6ca8b15eb824ac7ee01ed8387bf50ce57b))
* Replace modulo operations with truncations where possible ([#4329](https://github.com/noir-lang/noir/issues/4329)) ([70f2435](https://github.com/noir-lang/noir/commit/70f2435685d1d2a8fdd28160187d4312ec78d294))
* Separate compilation and expression narrowing in `nargo` interface ([#4100](https://github.com/noir-lang/noir/issues/4100)) ([62a4e37](https://github.com/noir-lang/noir/commit/62a4e37ef2274af2839011c3bab7bfdbf9f164fa))
* Simplify all unsigned constant NOT instructions ([#4230](https://github.com/noir-lang/noir/issues/4230)) ([fab4a6e](https://github.com/noir-lang/noir/commit/fab4a6e6ff025c83ec43313e36b8a236d030313a))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Use constraint information to perform constant folding ([#4060](https://github.com/noir-lang/noir/issues/4060)) ([9a4bf16](https://github.com/noir-lang/noir/commit/9a4bf16033c8d39c351eb532a4b015256cd22186))


### Bug Fixes

* Accurate tracking of slice capacities across blocks ([#4240](https://github.com/noir-lang/noir/issues/4240)) ([7420dbb](https://github.com/noir-lang/noir/commit/7420dbb7471bf243665d0bb3014886095c10c16f))
* Allow function calls in global definitions ([#4320](https://github.com/noir-lang/noir/issues/4320)) ([0dc205c](https://github.com/noir-lang/noir/commit/0dc205cdf28fcd858bdff3e9dd5d21c7498f451c))
* Allow performing bitwise NOT on unsigned integers ([#4229](https://github.com/noir-lang/noir/issues/4229)) ([b3ddf10](https://github.com/noir-lang/noir/commit/b3ddf10a2cbb80e88821baf7d76c478c3b98b3ea))
* Apply generic arguments from trait constraints before instantiating identifiers ([#4121](https://github.com/noir-lang/noir/issues/4121)) ([eb6fc0f](https://github.com/noir-lang/noir/commit/eb6fc0f3658bf126ed38d7aec7ee3f44ee0533b5))
* Apply range constraints to return values from unconstrained functions ([#4217](https://github.com/noir-lang/noir/issues/4217)) ([3af2a89](https://github.com/noir-lang/noir/commit/3af2a89826f7d9b6dcd1782b8b38417c64065293))
* Apply trait constraints from method calls ([#4152](https://github.com/noir-lang/noir/issues/4152)) ([68c5486](https://github.com/noir-lang/noir/commit/68c5486fda5a32eef74dd5b83b51024c1b3ab40c))
* Better errors for missing `fn` keyword ([#4154](https://github.com/noir-lang/noir/issues/4154)) ([057c208](https://github.com/noir-lang/noir/commit/057c2083a61bdad7dfcdc8c3f39769b41ae6926e))
* Check for tests in all packages before failing due to an unsatisfied test filter ([#4114](https://github.com/noir-lang/noir/issues/4114)) ([1107373](https://github.com/noir-lang/noir/commit/1107373bbbb9a8ca088dd6ac43131392cb2f33e1))
* Clean error when attemping to return a slice from Brillig to ACIR ([#4280](https://github.com/noir-lang/noir/issues/4280)) ([bcad4ec](https://github.com/noir-lang/noir/commit/bcad4ec5cc3e3f606e5bf673c7e367f1b63b20a2))
* Correct result when assigning shared arrays in unconstrained code ([#4210](https://github.com/noir-lang/noir/issues/4210)) ([bdd8a96](https://github.com/noir-lang/noir/commit/bdd8a96fb8364edcab4db06804e4949bacf18bf4))
* **docs:** Codegen docs before cutting a new version ([#4183](https://github.com/noir-lang/noir/issues/4183)) ([2914310](https://github.com/noir-lang/noir/commit/29143104fa907b446d534ac204069572cdc6f2f9))
* Ensure that destination register is allocated when moving between registers in brillig gen ([#4316](https://github.com/noir-lang/noir/issues/4316)) ([ca0a56e](https://github.com/noir-lang/noir/commit/ca0a56ee6bd07af8a3af5317d487ac94847115fc))
* Ensure that unconstrained entrypoint functions don't generate constraints ([#4292](https://github.com/noir-lang/noir/issues/4292)) ([fae4ead](https://github.com/noir-lang/noir/commit/fae4eadfedbf42ae73610c3475158072a183b329))
* From field with constant values ([#4226](https://github.com/noir-lang/noir/issues/4226)) ([593916b](https://github.com/noir-lang/noir/commit/593916bb61fb926730a34519b1429a8d035e10b6))
* **lsp:** Crash when file not in workspace ([#4146](https://github.com/noir-lang/noir/issues/4146)) ([cf7130f](https://github.com/noir-lang/noir/commit/cf7130f2e19e2d241e003c5527de9bf9d74cea40))
* **lsp:** Replace panics with errors ([#4209](https://github.com/noir-lang/noir/issues/4209)) ([26e9618](https://github.com/noir-lang/noir/commit/26e961860709e9c0ab3d1eb561fd39b5bd95a0fb))
* Maintain correct type when simplifying `x ^ x` ([#4082](https://github.com/noir-lang/noir/issues/4082)) ([9d83c2b](https://github.com/noir-lang/noir/commit/9d83c2b7d49490027bfa2974c1e2c5a85cc00aff))
* Message formatting for assert statement ([#4323](https://github.com/noir-lang/noir/issues/4323)) ([3972ead](https://github.com/noir-lang/noir/commit/3972ead2593cd1d3f61c3311e948ec27bd9b1491))
* Prevent debugger crashing on circuits with no opcodes ([#4283](https://github.com/noir-lang/noir/issues/4283)) ([2e32845](https://github.com/noir-lang/noir/commit/2e328454054a7c90b8b762b7c9ff0823eb0997c5))
* Prevent declarations of blackbox functions outside of the stdlib ([#4177](https://github.com/noir-lang/noir/issues/4177)) ([9fb6b09](https://github.com/noir-lang/noir/commit/9fb6b092c504d29d7f190952387de66c7e6e570c))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Respect order in bubble up for redundant asserts ([#4109](https://github.com/noir-lang/noir/issues/4109)) ([189aa48](https://github.com/noir-lang/noir/commit/189aa48c6c32fb6621b0e38a1f2d5d76d26ff0f2))
* Revert "correct result when assigning shared arrays" and added regression test ([#4333](https://github.com/noir-lang/noir/issues/4333)) ([05e78b3](https://github.com/noir-lang/noir/commit/05e78b39e9465b37138bba1c9b374a74404925aa))
* Save the data bus to the current function before generating others ([#4047](https://github.com/noir-lang/noir/issues/4047)) ([0a5bd4f](https://github.com/noir-lang/noir/commit/0a5bd4faa880dfcadf74372d8caeb458b2b55132))
* Simplify constant assert messages into `ConstrainError::Static` ([#4287](https://github.com/noir-lang/noir/issues/4287)) ([fd15052](https://github.com/noir-lang/noir/commit/fd150521a480c04ff64f84e3c1a2faf1e8394516))
* Ssa typing for array & slice indexes ([#4278](https://github.com/noir-lang/noir/issues/4278)) ([4074bab](https://github.com/noir-lang/noir/commit/4074babef6e25c0f723f2bc9b1b2c89302f8e0b9))
* Ssa typing for assign_lvalue_index ([#4289](https://github.com/noir-lang/noir/issues/4289)) ([37f149c](https://github.com/noir-lang/noir/commit/37f149c68e195cf29f81bef4616739cda65f8da7))
* SSA typing for right shifts ([#4302](https://github.com/noir-lang/noir/issues/4302)) ([41ee1aa](https://github.com/noir-lang/noir/commit/41ee1aa645e00b5e4926be24a1d8130bb1efad28))
* Ssa typing of make_offset ([#4277](https://github.com/noir-lang/noir/issues/4277)) ([e4378ee](https://github.com/noir-lang/noir/commit/e4378eed877f20ef4de7d5eaac4209c282f2860a))
* Track graphs of item dependencies to find dependency cycles ([#4266](https://github.com/noir-lang/noir/issues/4266)) ([61eabf1](https://github.com/noir-lang/noir/commit/61eabf1aa4f3eeba4695dcd988cdd3828ec269a5))
* Type check ACIR mutable reference passed to brillig ([#4281](https://github.com/noir-lang/noir/issues/4281)) ([7e139de](https://github.com/noir-lang/noir/commit/7e139de3499478cf573d2a7ad480f434cb898d9f))
* Update array method type signatures in the docs ([#4178](https://github.com/noir-lang/noir/issues/4178)) ([7c0a955](https://github.com/noir-lang/noir/commit/7c0a955486e14628356bb269402f4287c5600df4))
* Zero out input to `to_radix` calls if inactive ([#4116](https://github.com/noir-lang/noir/issues/4116)) ([3f5bad3](https://github.com/noir-lang/noir/commit/3f5bad3e60b8e2e72155e09f3951a73c3087a9c0))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.23.0](https://github.com/noir-lang/noir/compare/v0.22.0...v0.23.0) (2024-01-22)


### ⚠ BREAKING CHANGES

* Ban nested slices ([#4018](https://github.com/noir-lang/noir/issues/4018))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* remove circuit methods from noir_wasm ([#3869](https://github.com/noir-lang/noir/issues/3869))

### Features

* Add `assert_max_bit_size` method to `Field` ([#4016](https://github.com/noir-lang/noir/issues/4016)) ([bc9a44f](https://github.com/noir-lang/noir/commit/bc9a44f285e0569825a307b06ee8acd93461c87e))
* Add `noir-compiler` checks to `aztec_macros` ([#4031](https://github.com/noir-lang/noir/issues/4031)) ([420a5c7](https://github.com/noir-lang/noir/commit/420a5c74a14dcfeede04337a42282093a7b5e63e))
* Add a `--force` flag to force a full recompile ([#4054](https://github.com/noir-lang/noir/issues/4054)) ([27a8e68](https://github.com/noir-lang/noir/commit/27a8e6864643d81d96e84990e2e26cd16596a695))
* Add dependency resolver for `noir_wasm` and implement `FileManager` for consistency with native interface ([#3891](https://github.com/noir-lang/noir/issues/3891)) ([c29c7d7](https://github.com/noir-lang/noir/commit/c29c7d7c9615b9f45c696b1bdc1c497d55469dfa))
* Add foreign call support to `noir_codegen` functions ([#3933](https://github.com/noir-lang/noir/issues/3933)) ([e5e52a8](https://github.com/noir-lang/noir/commit/e5e52a81b31d7735b680e97a9bef89a010a99763))
* Add MVP `nargo export` command ([#3870](https://github.com/noir-lang/noir/issues/3870)) ([fbb51ed](https://github.com/noir-lang/noir/commit/fbb51ed33e9e4d9105d8946cdfc4ea387c85258e))
* Add support for codegenning multiple functions which use the same structs in their interface ([#3868](https://github.com/noir-lang/noir/issues/3868)) ([1dcfcc5](https://github.com/noir-lang/noir/commit/1dcfcc5265f618685a783504b1d4be213e4cda2d))
* Added efficient field comparisons for bn254 ([#4042](https://github.com/noir-lang/noir/issues/4042)) ([1f9cad0](https://github.com/noir-lang/noir/commit/1f9cad00c57ea257f57419d2446a46938beb19f9))
* Assert maximum bit size when creating a U128 from an integer ([#4024](https://github.com/noir-lang/noir/issues/4024)) ([8f9c7e4](https://github.com/noir-lang/noir/commit/8f9c7e4de9f2ae5b39714d8e0d26b2befcd11c4a))
* Avoid unnecessary range checks by inspecting instructions for casts ([#4039](https://github.com/noir-lang/noir/issues/4039)) ([378c18e](https://github.com/noir-lang/noir/commit/378c18eb42d75852b97f849d05c9e3f650601339))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Bubble up `Instruction::Constrain`s to be applied as early as possible. ([#4065](https://github.com/noir-lang/noir/issues/4065)) ([66f5cdd](https://github.com/noir-lang/noir/commit/66f5cddc133ba0311028eba96c0ff6ec2ecaee59))
* Cached LSP parsing ([#4083](https://github.com/noir-lang/noir/issues/4083)) ([b4f724e](https://github.com/noir-lang/noir/commit/b4f724e848b291a733e417c394ac3fc7649c08c5))
* Comparison for signed integers ([#3873](https://github.com/noir-lang/noir/issues/3873)) ([bcbd49b](https://github.com/noir-lang/noir/commit/bcbd49b8b44749e149f83c1240094fa2f0a19087))
* Decompose `Instruction::Cast` to have an explicit truncation instruction ([#3946](https://github.com/noir-lang/noir/issues/3946)) ([35f18ef](https://github.com/noir-lang/noir/commit/35f18ef4d7c8041e3cf622a5643748d0793c2aa6))
* Decompose `Instruction::Constrain` into multiple more basic constraints ([#3892](https://github.com/noir-lang/noir/issues/3892)) ([51cf9d3](https://github.com/noir-lang/noir/commit/51cf9d37c8b9fbb14bb54b178d93129a7563e131))
* Docker testing flow ([#3895](https://github.com/noir-lang/noir/issues/3895)) ([179c90d](https://github.com/noir-lang/noir/commit/179c90dc3263c85de105c57925d9c5894427e8e1))
* Extract parsing to its own pass and do it in parallel ([#4063](https://github.com/noir-lang/noir/issues/4063)) ([569cbbc](https://github.com/noir-lang/noir/commit/569cbbc231a242c32821cba56f3649f3228a1cc7))
* Implement `Eq` trait on curve points ([#3944](https://github.com/noir-lang/noir/issues/3944)) ([abf751a](https://github.com/noir-lang/noir/commit/abf751ab7f57f87520be16b2bc6168efdf95a430))
* Implement DAP protocol in Nargo ([#3627](https://github.com/noir-lang/noir/issues/3627)) ([13834d4](https://github.com/noir-lang/noir/commit/13834d43bd876909cb50494a41b42297f7e6375b))
* Implement generic traits ([#4000](https://github.com/noir-lang/noir/issues/4000)) ([916fd15](https://github.com/noir-lang/noir/commit/916fd158aa361ac80d32767f575ad896c3462b15))
* Implement Operator Overloading ([#3931](https://github.com/noir-lang/noir/issues/3931)) ([4b16090](https://github.com/noir-lang/noir/commit/4b16090beecd0fcdd41c9e7b8f615c4625c26a5b))
* **lsp:** Cache definitions for goto requests ([#3930](https://github.com/noir-lang/noir/issues/3930)) ([4a2140f](https://github.com/noir-lang/noir/commit/4a2140f1f36bbe3afbc006f8db74820308ae27d5))
* **lsp:** Goto global ([#4043](https://github.com/noir-lang/noir/issues/4043)) ([15237b3](https://github.com/noir-lang/noir/commit/15237b34dbce5ea54973a178449e67cca8ac4f9d))
* **lsp:** Goto struct member inside Impl method ([#3918](https://github.com/noir-lang/noir/issues/3918)) ([99c2c5a](https://github.com/noir-lang/noir/commit/99c2c5a2c2c0da6bad783b60d9e3de8d9a1f4ee4))
* **lsp:** Goto trait from trait impl ([#3956](https://github.com/noir-lang/noir/issues/3956)) ([eb566e2](https://github.com/noir-lang/noir/commit/eb566e2125e847a3e3efbd2bc15a88a1c454a7df))
* **lsp:** Goto trait method declaration ([#3991](https://github.com/noir-lang/noir/issues/3991)) ([eb79166](https://github.com/noir-lang/noir/commit/eb79166f7d2b7aa45c9c6c0aa37db1c0a5dfa00f))
* **lsp:** Goto type alias  ([#4061](https://github.com/noir-lang/noir/issues/4061)) ([dc83385](https://github.com/noir-lang/noir/commit/dc83385e9fe5766cd8218265be38c54243cae76e))
* **lsp:** Goto type definition ([#4029](https://github.com/noir-lang/noir/issues/4029)) ([8bb4ddf](https://github.com/noir-lang/noir/commit/8bb4ddfdd81d491ff713a056a7eae522f329d173))
* **lsp:** Re-add code lens feature with improved performance ([#3829](https://github.com/noir-lang/noir/issues/3829)) ([8f5cd6c](https://github.com/noir-lang/noir/commit/8f5cd6c0b641b3970bf626e8910b2a4c7cc8c310))
* Optimize array ops for arrays of structs ([#4027](https://github.com/noir-lang/noir/issues/4027)) ([c9ec0d8](https://github.com/noir-lang/noir/commit/c9ec0d811ddc8653201ed765b51585a7c1b946fb))
* Optimize logic gate ACIR-gen ([#3897](https://github.com/noir-lang/noir/issues/3897)) ([926460a](https://github.com/noir-lang/noir/commit/926460a0c70e21e2f4720148cf424e44ab9b0678))
* Prefer `AcirContext`-native methods for performing logic operations ([#3898](https://github.com/noir-lang/noir/issues/3898)) ([0ec39b8](https://github.com/noir-lang/noir/commit/0ec39b8396084ed1e7f20609c8ad8a5844a86674))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove truncation from brillig casts ([#3997](https://github.com/noir-lang/noir/issues/3997)) ([857ff97](https://github.com/noir-lang/noir/commit/857ff97b196174a0999f0fe7e387bfca5c3b7cd3))
* Remove truncations which can be seen to be noops using type information ([#3953](https://github.com/noir-lang/noir/issues/3953)) ([cc3c2c2](https://github.com/noir-lang/noir/commit/cc3c2c22644f0b5d8369bad2362ea6e9112a0713))
* Remove unnecessary predicate from `Lt` instruction ([#3922](https://github.com/noir-lang/noir/issues/3922)) ([a63433f](https://github.com/noir-lang/noir/commit/a63433fb8747722ec3cf2c6eb85d34e5b04bc15c))
* Simplify chains of casts to be all in terms of the original `ValueId` ([#3984](https://github.com/noir-lang/noir/issues/3984)) ([2384d3e](https://github.com/noir-lang/noir/commit/2384d3e97af24a8718fbf57f6b276a5ce1de06fe))
* Simplify multiplications by `0` or `1` in ACIR gen ([#3924](https://github.com/noir-lang/noir/issues/3924)) ([e58844d](https://github.com/noir-lang/noir/commit/e58844daf9f040626a3a7595f8c4f831e48a4037))
* Support for u128 ([#3913](https://github.com/noir-lang/noir/issues/3913)) ([b4911dc](https://github.com/noir-lang/noir/commit/b4911dcf676f0925ac631ba6f60fc9c4945b2fee))
* Support printing more types ([#4071](https://github.com/noir-lang/noir/issues/4071)) ([f5c4632](https://github.com/noir-lang/noir/commit/f5c4632e174beba508e1e31d0e2ae3f6d028ae2c))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Use singleton `WasmBlackBoxFunctionSolver` in `noir_js` ([#3966](https://github.com/noir-lang/noir/issues/3966)) ([10b28de](https://github.com/noir-lang/noir/commit/10b28def4d74822b7af2c19a1cc693788272b00b))


### Bug Fixes

* Acir gen doesn't panic on unsupported BB function ([#3866](https://github.com/noir-lang/noir/issues/3866)) ([34fd978](https://github.com/noir-lang/noir/commit/34fd978d206789a9e9f5167bfd690a34386834d0))
* Allow abi encoding arrays of structs from JS ([#3867](https://github.com/noir-lang/noir/issues/3867)) ([9b713f8](https://github.com/noir-lang/noir/commit/9b713f8cf599df262a12ec1098136c50b2b46766))
* Allow abi encoding tuples from JS ([#3894](https://github.com/noir-lang/noir/issues/3894)) ([f7fa181](https://github.com/noir-lang/noir/commit/f7fa1811ad2591020c914976f26e2f11a91cd177))
* Allow ast when macro errors ([#4005](https://github.com/noir-lang/noir/issues/4005)) ([efccec3](https://github.com/noir-lang/noir/commit/efccec3c24eb093fba99b1c29f01a78aae5776d0))
* Allow lsp to run inside of a docker container ([#3876](https://github.com/noir-lang/noir/issues/3876)) ([2529977](https://github.com/noir-lang/noir/commit/2529977acd684219f57ef086415557cc07af043b))
* Bit-shifts for signed integers ([#3890](https://github.com/noir-lang/noir/issues/3890)) ([6ddd98a](https://github.com/noir-lang/noir/commit/6ddd98ab7d3fefde491cf12b785f76bf0585609e))
* Checks for cyclic dependencies ([#3699](https://github.com/noir-lang/noir/issues/3699)) ([642011a](https://github.com/noir-lang/noir/commit/642011ab6ebbe8f012eda1da1abbf8660500723d))
* **debugger:** Crash when stepping through locations spanning multiple lines ([#3920](https://github.com/noir-lang/noir/issues/3920)) ([223e860](https://github.com/noir-lang/noir/commit/223e860975c2698bd5043340b937de74552ec15b))
* Don't fail if no tests and the user didn't provide a pattern ([#3864](https://github.com/noir-lang/noir/issues/3864)) ([decbd0f](https://github.com/noir-lang/noir/commit/decbd0f0c019844cd2b235e7804d2f6ba7b23897))
* Fix advisory issue in cargo-deny ([#4077](https://github.com/noir-lang/noir/issues/4077)) ([19baea0](https://github.com/noir-lang/noir/commit/19baea0d18e2d26bd04b649f79dd8e681488d1dc))
* Fixing dark mode background on the CTA button ([#3882](https://github.com/noir-lang/noir/issues/3882)) ([57eae42](https://github.com/noir-lang/noir/commit/57eae42080d6a928e8010c6bc77489964a5777ef))
* Fixup exports from `noir_wasm` ([#4022](https://github.com/noir-lang/noir/issues/4022)) ([358cdd2](https://github.com/noir-lang/noir/commit/358cdd2725444091b3322c47754e3cbd9b1d3614))
* Handle multiple imports in the same file ([#3903](https://github.com/noir-lang/noir/issues/3903)) ([219423e](https://github.com/noir-lang/noir/commit/219423eb87fa12bd8cca2a6fd2ce4c06e308783c))
* Hoist constraints on inputs to top of program ([#4076](https://github.com/noir-lang/noir/issues/4076)) ([447aa34](https://github.com/noir-lang/noir/commit/447aa343555cbd5a7cd735876e08f43271ecdd40))
* Implement missing codegen for `BlackBoxFunc::EcdsaSecp256r1` in brillig ([#3943](https://github.com/noir-lang/noir/issues/3943)) ([2c5eceb](https://github.com/noir-lang/noir/commit/2c5eceb04ab6bc38e954492642121c7fe3da866f))
* Improve `nargo test` output ([#3973](https://github.com/noir-lang/noir/issues/3973)) ([3ab5ff4](https://github.com/noir-lang/noir/commit/3ab5ff431145a1f747b698caed15caebaa145f04))
* Make `constant_to_radix` emit a slice instead of an array ([#4049](https://github.com/noir-lang/noir/issues/4049)) ([5cdb1d0](https://github.com/noir-lang/noir/commit/5cdb1d0dabe2e38a1610f718747cc2fb4263339d))
* Operator overloading & static trait method references resolving to generic impls ([#3967](https://github.com/noir-lang/noir/issues/3967)) ([f1de8fa](https://github.com/noir-lang/noir/commit/f1de8fa3247bcee624bcd7a0f89fe7c7cd8430f1))
* Preserve brillig entrypoint functions without arguments ([#3951](https://github.com/noir-lang/noir/issues/3951)) ([1111465](https://github.com/noir-lang/noir/commit/1111465551557ed9e97e4b43d6eccc4b5896a39f))
* Prevent `Instruction::Constrain`s for non-primitive types ([#3916](https://github.com/noir-lang/noir/issues/3916)) ([467948f](https://github.com/noir-lang/noir/commit/467948f9ee9ae65b4e2badaa1d15835fced3e835))
* Remove panic for adding an invalid crate name in wasm compiler ([#3977](https://github.com/noir-lang/noir/issues/3977)) ([7a1baa5](https://github.com/noir-lang/noir/commit/7a1baa56faa2deb385ef1b6c9da9073dafd5a376))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))
* Search all levels of struct nesting before codegenning primitive types ([#3970](https://github.com/noir-lang/noir/issues/3970)) ([13ae014](https://github.com/noir-lang/noir/commit/13ae014ddcbd9eddb401c563b95053f7a1a89f1c))
* Update generics docs to mention we have traits now ([#3980](https://github.com/noir-lang/noir/issues/3980)) ([c2acdf1](https://github.com/noir-lang/noir/commit/c2acdf1793a67abc9a074457e057a44da3b82c39))


### Miscellaneous Chores

* Ban nested slices ([#4018](https://github.com/noir-lang/noir/issues/4018)) ([f8a1fb7](https://github.com/noir-lang/noir/commit/f8a1fb7eed1ae4a9779eb16b142a64094aa603c6))
* Remove circuit methods from noir_wasm ([#3869](https://github.com/noir-lang/noir/issues/3869)) ([12d884e](https://github.com/noir-lang/noir/commit/12d884e2b74efab7257626d8878ea1a7455ecf85))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))

## [0.22.0](https://github.com/noir-lang/noir/compare/v0.21.0...v0.22.0) (2023-12-18)


### ⚠ BREAKING CHANGES

* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove backend field from artifacts ([#3819](https://github.com/noir-lang/noir/issues/3819))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Add context-centric based API for noir_wasm ([#3798](https://github.com/noir-lang/noir/issues/3798)) ([19155d0](https://github.com/noir-lang/noir/commit/19155d02a1248c85e94f14a2a0bb383a4edeb16f))


### Miscellaneous Chores

* Remove backend field from artifacts ([#3819](https://github.com/noir-lang/noir/issues/3819)) ([fa1cf5f](https://github.com/noir-lang/noir/commit/fa1cf5f03aa21b001c31ebb9ce405e3c2859bb57))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))

## [0.21.0](https://github.com/noir-lang/noir/compare/v0.20.0...v0.21.0) (2023-12-15)


### ⚠ BREAKING CHANGES

* remove unused `source-resolver` package ([#3791](https://github.com/noir-lang/noir/issues/3791))
* Make file manager read-only to the compiler ([#3760](https://github.com/noir-lang/noir/issues/3760))

### Features

* Add `prelude.nr` ([#3693](https://github.com/noir-lang/noir/issues/3693)) ([5f0f81f](https://github.com/noir-lang/noir/commit/5f0f81f7f49b021880e0bff648aa6c6d0fede46c))
* Add some traits to the stdlib ([#3796](https://github.com/noir-lang/noir/issues/3796)) ([8e11352](https://github.com/noir-lang/noir/commit/8e113526a2d78d27ed4e489f16d5604a2aaa18ea))
* Add support for writing tracing debug info to file ([#3790](https://github.com/noir-lang/noir/issues/3790)) ([98a5004](https://github.com/noir-lang/noir/commit/98a500436a68652a367ccbf77e32f8544aff73bc))
* Allow passing custom foreign call handlers when creating proofs in NoirJS ([#3764](https://github.com/noir-lang/noir/issues/3764)) ([6076e08](https://github.com/noir-lang/noir/commit/6076e08a0814bb6f3836af3c65a7b40c066b9494))
* Allow underscores in integer literals ([#3746](https://github.com/noir-lang/noir/issues/3746)) ([2c06a64](https://github.com/noir-lang/noir/commit/2c06a64e502bac6839375c5636d39a172a609a5f))
* Avoid overflow checks on boolean multiplication ([#3745](https://github.com/noir-lang/noir/issues/3745)) ([9b5b686](https://github.com/noir-lang/noir/commit/9b5b6861c3aa0e154e17598ac9994d3970f0e752))
* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Dockerfile to test cargo and JS packages ([#3684](https://github.com/noir-lang/noir/issues/3684)) ([513d619](https://github.com/noir-lang/noir/commit/513d6196a0766082a3c88a4050498bae2cfa7e13))
* Docs landing page with a playground ([#3667](https://github.com/noir-lang/noir/issues/3667)) ([9a95fbe](https://github.com/noir-lang/noir/commit/9a95fbeefb2ecd5a898006530a1e054cd345bfe8))
* Enhance test information output ([#3696](https://github.com/noir-lang/noir/issues/3696)) ([468fbbc](https://github.com/noir-lang/noir/commit/468fbbca43e33b23bc662bf1d36dcb79830a291c))
* Implement print without newline ([#3650](https://github.com/noir-lang/noir/issues/3650)) ([9827dfe](https://github.com/noir-lang/noir/commit/9827dfe51118ba55da6da51ab8bf45cffd2ca756))
* **lsp:** Add goto definition for locals ([#3705](https://github.com/noir-lang/noir/issues/3705)) ([9dd465c](https://github.com/noir-lang/noir/commit/9dd465c23e286481fa9a35632d133901f86d5883))
* **lsp:** Add goto definition for structs ([#3718](https://github.com/noir-lang/noir/issues/3718)) ([a576c5b](https://github.com/noir-lang/noir/commit/a576c5bba6ab92eb4798715a43475808ac954fba))
* Optimize out unnecessary truncation instructions ([#3717](https://github.com/noir-lang/noir/issues/3717)) ([c9c72ae](https://github.com/noir-lang/noir/commit/c9c72ae7b80aa9504a082dd083b19d4b80d954c5))
* Remove experimental feature warning for traits ([#3783](https://github.com/noir-lang/noir/issues/3783)) ([cb52242](https://github.com/noir-lang/noir/commit/cb522429592477c2b0544f3b3026a1a946b0e5b1))
* Reorganizing docs to fit diataxis framework ([#3711](https://github.com/noir-lang/noir/issues/3711)) ([54a1ed5](https://github.com/noir-lang/noir/commit/54a1ed58c991eefa7ac9304b894c7046c294487b))
* Simplify explicit equality assertions to assert equality directly ([#3708](https://github.com/noir-lang/noir/issues/3708)) ([2fc46e2](https://github.com/noir-lang/noir/commit/2fc46e2269bba8d9ad6ae5fcea10e64dce9b3745))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))


### Bug Fixes

* `try_unify` no longer binds types on failure ([#3697](https://github.com/noir-lang/noir/issues/3697)) ([f03e581](https://github.com/noir-lang/noir/commit/f03e5812439bdf9d1aedc69debdc50ba5dba2049))
* Add missing assertion to test ([#3765](https://github.com/noir-lang/noir/issues/3765)) ([bcbe116](https://github.com/noir-lang/noir/commit/bcbe11613b7205476a49ad0d588b868b4fc43ba1))
* Add negative integer literals ([#3690](https://github.com/noir-lang/noir/issues/3690)) ([8b3a68f](https://github.com/noir-lang/noir/commit/8b3a68f5286c09e1f612dbcfff3fe41023ab7109))
* Allow trait method references from the trait name ([#3774](https://github.com/noir-lang/noir/issues/3774)) ([cfa34d4](https://github.com/noir-lang/noir/commit/cfa34d4d913dbd35f8329430e0d58830e069d6ff))
* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))
* **docs:** Trigger `update-docs` workflow when the `release-please` PR gets merged and not on every merge to master ([#3677](https://github.com/noir-lang/noir/issues/3677)) ([9a3d1d2](https://github.com/noir-lang/noir/commit/9a3d1d2cf647cd583344f8da122fed1acbca9397))
* Initialize strings as u8 array ([#3682](https://github.com/noir-lang/noir/issues/3682)) ([8da40b7](https://github.com/noir-lang/noir/commit/8da40b75a36ebac51d5377311db3c55fa339dcac))
* **lsp:** Package resolution on save ([#3794](https://github.com/noir-lang/noir/issues/3794)) ([14f2fff](https://github.com/noir-lang/noir/commit/14f2fffeb3de5f653c11694ee3c5e5d62aaa34ec))
* Parse negative integer literals ([#3698](https://github.com/noir-lang/noir/issues/3698)) ([463ab06](https://github.com/noir-lang/noir/commit/463ab060075db1915127c3f6cef11bfed9d40109))
* Pub is required on return for entry points ([#3616](https://github.com/noir-lang/noir/issues/3616)) ([7f1d796](https://github.com/noir-lang/noir/commit/7f1d7968368734e02b152e2e907dc7af9e1604c8))
* Remove `noirc_driver/aztec` feature flag in docker ([#3784](https://github.com/noir-lang/noir/issues/3784)) ([a48d562](https://github.com/noir-lang/noir/commit/a48d562b59aa2009a9c9b65dd71e11cdd8d06cf0))
* Remove include-keys option ([#3692](https://github.com/noir-lang/noir/issues/3692)) ([95d7ce2](https://github.com/noir-lang/noir/commit/95d7ce21016e3603bf279efb970536ad32d89a3a))
* Revert change to modify version in workspace file for acvm dependencies ([#3673](https://github.com/noir-lang/noir/issues/3673)) ([0696f75](https://github.com/noir-lang/noir/commit/0696f755364293bcc7ebc7a0def0dcafede2e543))
* Sequence update-lockfile workflow so it gets modified after the ACVM version in the root has been changed ([#3676](https://github.com/noir-lang/noir/issues/3676)) ([c00cd85](https://github.com/noir-lang/noir/commit/c00cd8537836f8e4d8559b01d16dfdd1b5cad519))
* **ssa:** Handle array arguments to side effectual constrain statements ([#3740](https://github.com/noir-lang/noir/issues/3740)) ([028d65e](https://github.com/noir-lang/noir/commit/028d65ea71f9c11e69784d06e0f9768668455f83))
* Stop cloning Traits! ([#3736](https://github.com/noir-lang/noir/issues/3736)) ([fcff412](https://github.com/noir-lang/noir/commit/fcff412bb39a04a5c88506ae5a5ee2fbdefd93ef))
* Stop issuing unused variable warnings for variables in trait definitions ([#3797](https://github.com/noir-lang/noir/issues/3797)) ([0bb44c3](https://github.com/noir-lang/noir/commit/0bb44c3bbc63d385d77d93da6abd07214bcfd700))
* Unsigned integers cannot be negated ([#3688](https://github.com/noir-lang/noir/issues/3688)) ([f904ae1](https://github.com/noir-lang/noir/commit/f904ae1065af74652b2111ea17b72f994de37472))


### Miscellaneous Chores

* Make file manager read-only to the compiler ([#3760](https://github.com/noir-lang/noir/issues/3760)) ([e3dcc21](https://github.com/noir-lang/noir/commit/e3dcc21cb2c0fef7f28f50b018747c4f09609b11))
* Remove unused `source-resolver` package ([#3791](https://github.com/noir-lang/noir/issues/3791)) ([57d2505](https://github.com/noir-lang/noir/commit/57d2505d53e2233becd1e2a7de882c4acb518eff))

## [0.20.0](https://github.com/noir-lang/noir/compare/v0.19.5...v0.20.0) (2023-12-01)


### ⚠ BREAKING CHANGES

* avoid integer overflows ([#2713](https://github.com/noir-lang/noir/issues/2713))
* return Pedersen structure in stdlib ([#3190](https://github.com/noir-lang/noir/issues/3190))
* noir-wasm outputs debug symbols ([#3317](https://github.com/noir-lang/noir/issues/3317))
* move mimc to hash submodule ([#3361](https://github.com/noir-lang/noir/issues/3361))
* bump MSRV to 1.71.1 ([#3353](https://github.com/noir-lang/noir/issues/3353))
* Add semver checks for the compiler version in Nargo.toml ([#3336](https://github.com/noir-lang/noir/issues/3336))
* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* change stdlib function `pedersen` to `pedersen_commitment` ([#3341](https://github.com/noir-lang/noir/issues/3341))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* Make for loops a statement ([#2975](https://github.com/noir-lang/noir/issues/2975))
* **traits:** trait functions with a default implementation must not be followed by a semicolon ([#2987](https://github.com/noir-lang/noir/issues/2987))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* **wasm:** update wasm artifacts to match cli artifacts ([#2973](https://github.com/noir-lang/noir/issues/2973))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))
* update to `bb` version 0.7.3 ([#2729](https://github.com/noir-lang/noir/issues/2729))
* **noir_js:** Rename inner and outer proof methods ([#2845](https://github.com/noir-lang/noir/issues/2845))
* `generateWitness` now returns a serialized witness file ([#2842](https://github.com/noir-lang/noir/issues/2842))
* Issue an error when a module is declared twice & fix module search path ([#2801](https://github.com/noir-lang/noir/issues/2801))
* Default integers to u64 ([#2764](https://github.com/noir-lang/noir/issues/2764))

### Features

* `compute_note_hash_and_nullifier` check ([#3216](https://github.com/noir-lang/noir/issues/3216)) ([4963c6c](https://github.com/noir-lang/noir/commit/4963c6c024bba710dd8907147f631f5c26094f0a))
* **abi:** Throw errors rather than returning string from `noirc_abi_wasm` ([#2817](https://github.com/noir-lang/noir/issues/2817)) ([df7b42c](https://github.com/noir-lang/noir/commit/df7b42cd253d1b908a42c367b116813f9999d93b))
* **abi:** Tuples as inputs/outputs to main ([#2899](https://github.com/noir-lang/noir/issues/2899)) ([d8bd78f](https://github.com/noir-lang/noir/commit/d8bd78f60c447bb8488a844d779e8aaf4150afe7))
* **acir:** Enable dynamic indices on non-homogenous arrays ([#2703](https://github.com/noir-lang/noir/issues/2703)) ([622d2e4](https://github.com/noir-lang/noir/commit/622d2e436992c23e6d0885b591bd1072ca57b307))
* **acir:** Handle dynamic array operations for nested slices ([#3187](https://github.com/noir-lang/noir/issues/3187)) ([e026319](https://github.com/noir-lang/noir/commit/e026319fc25763d30781b90e6a4454ddb5d3bc7b))
* **acir:** Set dynamic array values ([#3054](https://github.com/noir-lang/noir/issues/3054)) ([e871866](https://github.com/noir-lang/noir/commit/e871866d2203f0f0f49f3b273d99d385b950b65f))
* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add --check option to nargo fmt for dry-run formatting verification ([#3530](https://github.com/noir-lang/noir/issues/3530)) ([4469707](https://github.com/noir-lang/noir/commit/4469707d97085fab0f7ade8d015dc827c56156ee))
* Add `destroy` method to `Noir` ([#3105](https://github.com/noir-lang/noir/issues/3105)) ([7e40274](https://github.com/noir-lang/noir/commit/7e402744a7d64ffcac6db026cec1631230204f0f))
* Add `execute` method to `Noir` class ([#3081](https://github.com/noir-lang/noir/issues/3081)) ([17bdd7e](https://github.com/noir-lang/noir/commit/17bdd7e3909f0ddd195e5cb7095cd0d30758ed43))
* Add `FieldElement::from&lt;usize&gt;` implementation ([#3647](https://github.com/noir-lang/noir/issues/3647)) ([8b7c5aa](https://github.com/noir-lang/noir/commit/8b7c5aa5311f4e6811438f67bd552b641b13fc9a))
* Add `noir_codegen` package ([#3392](https://github.com/noir-lang/noir/issues/3392)) ([6c4cd4d](https://github.com/noir-lang/noir/commit/6c4cd4d37e4af38dccf899bcbd3950d1e236b35d))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add an options object to `BarretenbergBackend` constructor ([#3105](https://github.com/noir-lang/noir/issues/3105)) ([7e40274](https://github.com/noir-lang/noir/commit/7e402744a7d64ffcac6db026cec1631230204f0f))
* Add aztec selectors for event structs ([#2983](https://github.com/noir-lang/noir/issues/2983)) ([982380e](https://github.com/noir-lang/noir/commit/982380e54bb4d696688522c540f1234734ae2e80))
* Add bb interface implementation ([#2902](https://github.com/noir-lang/noir/issues/2902)) ([fe92dc0](https://github.com/noir-lang/noir/commit/fe92dc0df57b2cbc0e7b8cd1f3a91cba6b0f3049))
* Add check for overlapping generic traits ([#3307](https://github.com/noir-lang/noir/issues/3307)) ([8cf81b6](https://github.com/noir-lang/noir/commit/8cf81b659bed9522aede29c1ebb4a4ed2bfa1205))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add crate for pub modifier ([#3271](https://github.com/noir-lang/noir/issues/3271)) ([e7a1a1a](https://github.com/noir-lang/noir/commit/e7a1a1a4b42b6b72c16f2204e33af80dbabba6b5))
* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Add experimental REPL-based debugger ([#2995](https://github.com/noir-lang/noir/issues/2995)) ([281c696](https://github.com/noir-lang/noir/commit/281c696da61c64b42b9525b8756ffc195f70d775))
* Add exports of JS black box solvers to noirJS ([#3295](https://github.com/noir-lang/noir/issues/3295)) ([8369871](https://github.com/noir-lang/noir/commit/836987150f82354d3dfc01cfaad69f70240ca80c))
* Add generic count check for trait methods ([#3382](https://github.com/noir-lang/noir/issues/3382)) ([a9f9717](https://github.com/noir-lang/noir/commit/a9f9717ba69c0dd8e4fc7045fe6aea2077b84c95))
* Add JS types for ABI and input maps ([#3023](https://github.com/noir-lang/noir/issues/3023)) ([599e7a1](https://github.com/noir-lang/noir/commit/599e7a1d6bae5d93273e9ef1265024eac909660d))
* Add LSP command to profile opcodes in vscode ([#3496](https://github.com/noir-lang/noir/issues/3496)) ([6fbf77a](https://github.com/noir-lang/noir/commit/6fbf77ae2b87a55db92344f5066a82ccaf6c2086))
* Add lsp formatting ([#3433](https://github.com/noir-lang/noir/issues/3433)) ([286c876](https://github.com/noir-lang/noir/commit/286c87694fda185f25b05cec5504142643bc207f))
* Add noir types package ([#2893](https://github.com/noir-lang/noir/issues/2893)) ([e8fc868](https://github.com/noir-lang/noir/commit/e8fc8687e6dd89295fd023201443f1197963a243))
* Add package version to Nargo.toml metadata ([#3427](https://github.com/noir-lang/noir/issues/3427)) ([9e1717c](https://github.com/noir-lang/noir/commit/9e1717c2d96a0b9e394e5cb2fb9e1d09b5259ca0))
* Add profile info print out ([#3425](https://github.com/noir-lang/noir/issues/3425)) ([a8b5fa8](https://github.com/noir-lang/noir/commit/a8b5fa8e30dc27e64666381b7451569f350967d1))
* Add semver checks for the compiler version in Nargo.toml ([#3336](https://github.com/noir-lang/noir/issues/3336)) ([0e530cf](https://github.com/noir-lang/noir/commit/0e530cfe86f87a532be30a02f4353d010e47e458))
* Add special case for boolean AND in acir-gen ([#3615](https://github.com/noir-lang/noir/issues/3615)) ([824039b](https://github.com/noir-lang/noir/commit/824039bcc0a3275f333ea94aecc701d129f99fe5))
* Add support for tuple values in `noir_codegen` ([#3592](https://github.com/noir-lang/noir/issues/3592)) ([346d75f](https://github.com/noir-lang/noir/commit/346d75f9dd9261996d4d7bb80eb7e4118e8f8ce2))
* Allow a trait to be implemented multiple times for the same struct ([#3292](https://github.com/noir-lang/noir/issues/3292)) ([51831df](https://github.com/noir-lang/noir/commit/51831df68bc20460c6d05d55469002db06113925))
* Allow providing custom foreign call executors to `execute_circuit` ([#3506](https://github.com/noir-lang/noir/issues/3506)) ([d27db33](https://github.com/noir-lang/noir/commit/d27db332f8c320ffd9b5520bebbd83ae09e31de7))
* Allow traits to have generic functions ([#3365](https://github.com/noir-lang/noir/issues/3365)) ([0f9af65](https://github.com/noir-lang/noir/commit/0f9af652efc6b7628784a397a9df674eaa30de61))
* Avoid integer overflows ([#2713](https://github.com/noir-lang/noir/issues/2713)) ([7d7d632](https://github.com/noir-lang/noir/commit/7d7d63291d712137f97e6d44a774acdf2bd20512))
* Aztec-packages ([#3599](https://github.com/noir-lang/noir/issues/3599)) ([2cd6dc3](https://github.com/noir-lang/noir/commit/2cd6dc39e3a956aa5dff721d47aaf1921f98fded))
* Aztec-packages ([#3626](https://github.com/noir-lang/noir/issues/3626)) ([e0a96ea](https://github.com/noir-lang/noir/commit/e0a96ea70b17c8c898dd72ac929f3969a4cec1d3))
* Cache debug artifacts  ([#3133](https://github.com/noir-lang/noir/issues/3133)) ([c5a6229](https://github.com/noir-lang/noir/commit/c5a622983e4049d82589f185be5e96c63ed6066d))
* Check where clauses when searching for trait impls ([#3407](https://github.com/noir-lang/noir/issues/3407)) ([84c6604](https://github.com/noir-lang/noir/commit/84c6604397f262c09ba4bac157fead38b8280313))
* Codegen typed interfaces for functions in `noir_codegen` ([#3533](https://github.com/noir-lang/noir/issues/3533)) ([290c463](https://github.com/noir-lang/noir/commit/290c463622a93a34293f73b5bf2aea7ade30a11c))
* Compile without a backend ([#3437](https://github.com/noir-lang/noir/issues/3437)) ([d69cf5d](https://github.com/noir-lang/noir/commit/d69cf5debcc430bb019b6cc95774aac084776dda))
* Complex slice inputs for dynamic slice builtins ([#3617](https://github.com/noir-lang/noir/issues/3617)) ([8b23b34](https://github.com/noir-lang/noir/commit/8b23b349ae15afa48f6cbe8962586bbe79e79890))
* Contract events in artifacts ([#2873](https://github.com/noir-lang/noir/issues/2873)) ([4765c82](https://github.com/noir-lang/noir/commit/4765c8288c583a61a81ff97eea1ef49df13eeca0))
* Copy on write optimization for brillig ([#3522](https://github.com/noir-lang/noir/issues/3522)) ([da29c02](https://github.com/noir-lang/noir/commit/da29c02327acb2f46f5a7a25c7404dfa44c82616))
* Data bus ([#3508](https://github.com/noir-lang/noir/issues/3508)) ([6b0bdbc](https://github.com/noir-lang/noir/commit/6b0bdbced1bfe2c92e90dd7b70ca8f9e5ccb7c0d))
* **debugger:** Highlight current src code loc ([#3174](https://github.com/noir-lang/noir/issues/3174)) ([6b87582](https://github.com/noir-lang/noir/commit/6b87582dfe872ad6c248cf9995d76b0ef1580625))
* **debugger:** Print limited source code context ([#3217](https://github.com/noir-lang/noir/issues/3217)) ([dcda1c7](https://github.com/noir-lang/noir/commit/dcda1c7aed69ae8f55cd3f680e3cc1ece9de7541))
* Default integers to u64 ([#2764](https://github.com/noir-lang/noir/issues/2764)) ([01cb041](https://github.com/noir-lang/noir/commit/01cb041a92ef6043dd5a160e0a56a63400801980))
* Dynamic indexing of non-homogenous slices ([#2883](https://github.com/noir-lang/noir/issues/2883)) ([72c3661](https://github.com/noir-lang/noir/commit/72c3661c86712b99236eafaac99f76f13d42b9d9))
* Enable the `fmt` command in the help menu ([#3328](https://github.com/noir-lang/noir/issues/3328)) ([63d414c](https://github.com/noir-lang/noir/commit/63d414c06a399525601e3db11dc48b180e93c2d8))
* Expand trait impl overlap check to cover generic types ([#3320](https://github.com/noir-lang/noir/issues/3320)) ([a01549b](https://github.com/noir-lang/noir/commit/a01549b78a2c5de3e64ce5d3e5c4bb73a8c8b4fb))
* Export `CompiledCircuit` from codegened TS ([#3589](https://github.com/noir-lang/noir/issues/3589)) ([e06c675](https://github.com/noir-lang/noir/commit/e06c67500da11518caffe0e98bdb9cd7f5f89049))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Format infix expressions ([#3001](https://github.com/noir-lang/noir/issues/3001)) ([7926ada](https://github.com/noir-lang/noir/commit/7926ada88ed08ac9d874604834533d900fbb16b0))
* **formatter:** Add formatter support for array literals ([#3061](https://github.com/noir-lang/noir/issues/3061)) ([a535321](https://github.com/noir-lang/noir/commit/a5353217a1f49b83daf11d5fa55e0bcccebf0271))
* Handle constant index operations on simple slices ([#3464](https://github.com/noir-lang/noir/issues/3464)) ([7ae12f8](https://github.com/noir-lang/noir/commit/7ae12f8c5243d31b2f410c246ed6b9e2fcea5d4c))
* Handle warnings in evaluator ([#3205](https://github.com/noir-lang/noir/issues/3205)) ([5cfd156](https://github.com/noir-lang/noir/commit/5cfd156ca2035038a226bdd81a51636d3de3c34e))
* Implement `bound_constraint_with_offset` in terms of `AcirVar`s ([#3233](https://github.com/noir-lang/noir/issues/3233)) ([8d89cb5](https://github.com/noir-lang/noir/commit/8d89cb59fe710859a96eaed4f988952bd727fb7d))
* Implement automatic dereferencing for index expressions ([#3082](https://github.com/noir-lang/noir/issues/3082)) ([8221bfd](https://github.com/noir-lang/noir/commit/8221bfd2ffde7d1dbf71a72d95257acf76ecca74))
* Implement automatic dereferencing for indexing lvalues ([#3083](https://github.com/noir-lang/noir/issues/3083)) ([6e2b70a](https://github.com/noir-lang/noir/commit/6e2b70ae90b686158957ea29ef1b2a5f0ed38e5f))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Implement impl specialization ([#3087](https://github.com/noir-lang/noir/issues/3087)) ([44716fa](https://github.com/noir-lang/noir/commit/44716fae0bae0f78ceee76f7231af49c4abeace1))
* Implement integer printing ([#3577](https://github.com/noir-lang/noir/issues/3577)) ([6601408](https://github.com/noir-lang/noir/commit/6601408e378b2afe4fdfd8e04482c39311ccc7e9))
* Implement raw string literals ([#3556](https://github.com/noir-lang/noir/issues/3556)) ([87a302f](https://github.com/noir-lang/noir/commit/87a302fbc26d5fd42694953935d95a7ccb3d50a0))
* Implement string escape sequences ([#2803](https://github.com/noir-lang/noir/issues/2803)) ([f7529b8](https://github.com/noir-lang/noir/commit/f7529b80f0958fd47a525f25a123f16438bbb892))
* Implement where clauses on impls ([#3324](https://github.com/noir-lang/noir/issues/3324)) ([4c3d1de](https://github.com/noir-lang/noir/commit/4c3d1dea27726133335538443df3dbbd2c8f2d58))
* **lsp:** Add "info" codelens ([#2982](https://github.com/noir-lang/noir/issues/2982)) ([80770d9](https://github.com/noir-lang/noir/commit/80770d9fae7c42e69a62cf01babfc69449600ac5))
* **lsp:** Add goto definition for functions ([#3656](https://github.com/noir-lang/noir/issues/3656)) ([7bb7356](https://github.com/noir-lang/noir/commit/7bb735662c86e6533ac58f448ad748e34f02edb7))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Make generic impls callable ([#3297](https://github.com/noir-lang/noir/issues/3297)) ([8d9b738](https://github.com/noir-lang/noir/commit/8d9b738ea958320a16946ebe6fcfb6bbf2dda42e))
* Manage breakpoints and allow restarting a debugging session ([#3325](https://github.com/noir-lang/noir/issues/3325)) ([f502108](https://github.com/noir-lang/noir/commit/f502108a59e2141f4898b0a25e84d7ed7d2bff58))
* Nargo test runtime callstacks and assert messages without string matching ([#2953](https://github.com/noir-lang/noir/issues/2953)) ([1b6a4e6](https://github.com/noir-lang/noir/commit/1b6a4e6021929c23a1bca5dff02c004422cc71f8))
* **noir_js:** Allow providing foreign call handlers in noirJS ([#3294](https://github.com/noir-lang/noir/issues/3294)) ([c76b0f8](https://github.com/noir-lang/noir/commit/c76b0f81b89ae02c39c13c3bd400b5d13f083759))
* Noir-wasm outputs debug symbols ([#3317](https://github.com/noir-lang/noir/issues/3317)) ([f9933fa](https://github.com/noir-lang/noir/commit/f9933fa50c42aade8ddc13d29deef7645ddcf586))
* Noir-wasm takes dependency graph ([#3213](https://github.com/noir-lang/noir/issues/3213)) ([a2c8ebd](https://github.com/noir-lang/noir/commit/a2c8ebd4a800d7ef042ac9cbe5ee6a837c715634))
* Old docs issues ([#3195](https://github.com/noir-lang/noir/issues/3195)) ([26746c5](https://github.com/noir-lang/noir/commit/26746c59e12a60f3869a5b885b05926c94f01215))
* Optimize euclidean division acir-gen ([#3121](https://github.com/noir-lang/noir/issues/3121)) ([2c175c0](https://github.com/noir-lang/noir/commit/2c175c0d886eea390ef97ada1c2a5b0e1bef15e8))
* Oracle mocker for nargo test ([#2928](https://github.com/noir-lang/noir/issues/2928)) ([0dd1e77](https://github.com/noir-lang/noir/commit/0dd1e77c0e625805e15fa56b4738c93ebae19b6d))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Perform compile-time euclidean division on constants ([#3231](https://github.com/noir-lang/noir/issues/3231)) ([3866d7e](https://github.com/noir-lang/noir/commit/3866d7ea32dcdabbdb5ba85478d97fd51a6850f3))
* Prevent unnecessary witness creation in euclidean division ([#2980](https://github.com/noir-lang/noir/issues/2980)) ([c6f660e](https://github.com/noir-lang/noir/commit/c6f660e86d40a106930483f1d6161814e3c0de10))
* Properly track equivalence of witnesses generated for black box functions ([#3428](https://github.com/noir-lang/noir/issues/3428)) ([20b70c2](https://github.com/noir-lang/noir/commit/20b70c29b4bd323d67ef56ab1933341a7747d3cb))
* Provide formatting subcommand  ([#2640](https://github.com/noir-lang/noir/issues/2640)) ([a38b15f](https://github.com/noir-lang/noir/commit/a38b15f5d8e69faff125d363f2fd1f2f90ae6830))
* Publish aztec build of noir_wasm ([#3049](https://github.com/noir-lang/noir/issues/3049)) ([3b51f4d](https://github.com/noir-lang/noir/commit/3b51f4df7e808233f6987baec93f4b5de7e5b304))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Remove redundant predicate from brillig quotients ([#2784](https://github.com/noir-lang/noir/issues/2784)) ([a8f18c5](https://github.com/noir-lang/noir/commit/a8f18c55b35f47c6fa3ebfebcd827aeb55e5c850))
* Remove type arrays for flat slices ([#3466](https://github.com/noir-lang/noir/issues/3466)) ([8225b2b](https://github.com/noir-lang/noir/commit/8225b2b379ddf145f9418f8517478704f9aac350))
* Remove unnecessary truncation of boolean multiplication ([#3122](https://github.com/noir-lang/noir/issues/3122)) ([39dbcf1](https://github.com/noir-lang/noir/commit/39dbcf1ab80d2bb472d08db4de15d4e0c1f2eb52))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Return compilation errors from noir_wasm ([#3091](https://github.com/noir-lang/noir/issues/3091)) ([55f63c9](https://github.com/noir-lang/noir/commit/55f63c935cec62fbba63eed421812a4372c1aa4d))
* Return Pedersen structure in stdlib ([#3190](https://github.com/noir-lang/noir/issues/3190)) ([be30d59](https://github.com/noir-lang/noir/commit/be30d59e61a9dff6ab94ffb97365c1282c331643))
* Reuse witnesses more when interacting with memory ([#3658](https://github.com/noir-lang/noir/issues/3658)) ([5a4a73d](https://github.com/noir-lang/noir/commit/5a4a73d24ddd7d2bb46c85714397ee6e031c97a6))
* Reuse witnesses which have been assigned constant values during ACIR gen ([#3137](https://github.com/noir-lang/noir/issues/3137)) ([9eb43e2](https://github.com/noir-lang/noir/commit/9eb43e2a4665397295e74a593f73d19fa9fa5d27))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Send and receive unflattened public inputs to backend ([#3543](https://github.com/noir-lang/noir/issues/3543)) ([a7bdc67](https://github.com/noir-lang/noir/commit/a7bdc67ef3ec2037bffc4f1f472907cad786c319))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* **ssa:** Multiple slice mergers ([#2753](https://github.com/noir-lang/noir/issues/2753)) ([8f76fe5](https://github.com/noir-lang/noir/commit/8f76fe5819e95ed111587090e15add48a2b4e859))
* **stdlib:** Optimize constraint counts in sha256/sha512 ([#3253](https://github.com/noir-lang/noir/issues/3253)) ([d3be552](https://github.com/noir-lang/noir/commit/d3be552149ab375b24b509603fcd7237b374ca5a))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **traits:** Add impl Trait as function return type [#2397](https://github.com/noir-lang/noir/issues/2397) ([#3176](https://github.com/noir-lang/noir/issues/3176)) ([4cb2024](https://github.com/noir-lang/noir/commit/4cb20244abba0abc49be0376611979a786563565))
* **traits:** Add trait impl for buildin types ([#2964](https://github.com/noir-lang/noir/issues/2964)) ([2c87b27](https://github.com/noir-lang/noir/commit/2c87b273dfdf033dd8c79b78f006a0e9813559d7))
* **traits:** Added checks for duplicated trait associated items (types, consts, functions) ([#2927](https://github.com/noir-lang/noir/issues/2927)) ([d49492c](https://github.com/noir-lang/noir/commit/d49492cd80d04ee6acc01247b06b088deefcd0c6))
* **traits:** Allow multiple traits to share the same associated function name and to be implemented for the same type ([#3126](https://github.com/noir-lang/noir/issues/3126)) ([004f8dd](https://github.com/noir-lang/noir/commit/004f8dd733cb23da4ed57b160f6b86d53bc0b5f1))
* **traits:** Implement trait bounds typechecker + monomorphizer passes ([#2717](https://github.com/noir-lang/noir/issues/2717)) ([5ca99b1](https://github.com/noir-lang/noir/commit/5ca99b128e9991b5272c00292208d85415e70edf))
* **traits:** Improve support for traits static method resolution ([#2958](https://github.com/noir-lang/noir/issues/2958)) ([0d0d8f7](https://github.com/noir-lang/noir/commit/0d0d8f7d2b401eb6b534dbb175dfd4b26d2a5f7d))
* **traits:** Multi module support for traits ([#2844](https://github.com/noir-lang/noir/issues/2844)) ([4deb07f](https://github.com/noir-lang/noir/commit/4deb07f80ce110187b66a46dd5624af3b8df3dbd))
* Use ranges instead of a vector for input witness ([#3314](https://github.com/noir-lang/noir/issues/3314)) ([b12b7ec](https://github.com/noir-lang/noir/commit/b12b7ecb995988c731be5f1f2f67fda952f1a228))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))
* **wasm:** Update wasm artifacts to match cli artifacts ([#2973](https://github.com/noir-lang/noir/issues/2973)) ([ce16c0b](https://github.com/noir-lang/noir/commit/ce16c0b14565cfe1bc2c9f09ae71643d2657440b))


### Bug Fixes

* "Missing trait impl" error in trait dispatch ([#3440](https://github.com/noir-lang/noir/issues/3440)) ([52daaec](https://github.com/noir-lang/noir/commit/52daaec504101fe3c0caa30441c17f30a34af475))
* `compute_note_hash_and_nullifier` compiler check ([#3351](https://github.com/noir-lang/noir/issues/3351)) ([4e2d35f](https://github.com/noir-lang/noir/commit/4e2d35f256bea2fee3a6cbd7af0c0c15a37c0a2e))
* **3275:** Activate brillig modulo test with negative integers ([#3318](https://github.com/noir-lang/noir/issues/3318)) ([31c493c](https://github.com/noir-lang/noir/commit/31c493ce2082a571d147f707837beb4f3ed2ca64))
* **3300:** Cache warnings into debug artefacts ([#3313](https://github.com/noir-lang/noir/issues/3313)) ([cb5a15b](https://github.com/noir-lang/noir/commit/cb5a15b9dbcfdaac5d656a122c73dca23855307d))
* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Add `pub` modifier to grumpkin functions ([#3036](https://github.com/noir-lang/noir/issues/3036)) ([f8990d7](https://github.com/noir-lang/noir/commit/f8990d75b948ce0a6968db659370f7ece7f5db08))
* Add compiler error message for invalid input types ([#3220](https://github.com/noir-lang/noir/issues/3220)) ([989e80d](https://github.com/noir-lang/noir/commit/989e80d4ea62e68cfab69a1cd16d481cbccc6c02))
* Add size checks to integer literals ([#3236](https://github.com/noir-lang/noir/issues/3236)) ([7f8fe8c](https://github.com/noir-lang/noir/commit/7f8fe8c88eb2d26ae3a93e2f74430fadc74b4836))
* Adding proving key initialization ([#3322](https://github.com/noir-lang/noir/issues/3322)) ([3383740](https://github.com/noir-lang/noir/commit/3383740f9a0004f2ee77c9686f81baed6cd1917c))
* Allow `where` clause on all functions and improve error message ([#3465](https://github.com/noir-lang/noir/issues/3465)) ([1647e33](https://github.com/noir-lang/noir/commit/1647e33564bf56ab8721a365f5fc6bcb38901412))
* Allow constructors in parentheses in `if` conditions and `for` ranges ([#3219](https://github.com/noir-lang/noir/issues/3219)) ([ad192d1](https://github.com/noir-lang/noir/commit/ad192d1b7492f6ecd5fc98bb88201d6c442dc052))
* Allow two `TypeVariable::Constant(N)` to unify even if their constants are not equal ([#3225](https://github.com/noir-lang/noir/issues/3225)) ([cc4ca4b](https://github.com/noir-lang/noir/commit/cc4ca4bb5f4fed5f531a2040501fcc6ed53a9ab4))
* Apply predicate to over/underflow checks ([#3494](https://github.com/noir-lang/noir/issues/3494)) ([fc3edf7](https://github.com/noir-lang/noir/commit/fc3edf7aa5da9074614fa900bbcb57e512e3d56b))
* **aztec_nr:** Serialise arrays of structs ([#3401](https://github.com/noir-lang/noir/issues/3401)) ([e979a58](https://github.com/noir-lang/noir/commit/e979a587e755d4b715ebacba715d778938026ac0))
* Change non-constant argument errors from `to_be_radix`  from ICE to proper error ([#3048](https://github.com/noir-lang/noir/issues/3048)) ([19ce286](https://github.com/noir-lang/noir/commit/19ce28638fe3ea42ab4984cb99e3898cd17fa8d9))
* Check for overflow with hexadecimal inputs ([#3004](https://github.com/noir-lang/noir/issues/3004)) ([db1e736](https://github.com/noir-lang/noir/commit/db1e736240c0b74f6f59504db5a50de1c749d395))
* Compiler version error message ([#3558](https://github.com/noir-lang/noir/issues/3558)) ([026a358](https://github.com/noir-lang/noir/commit/026a3587b01ddc8f444ff588a7b3f3fd1a0bb386))
* Complete debug metadata ([#3228](https://github.com/noir-lang/noir/issues/3228)) ([2f6509d](https://github.com/noir-lang/noir/commit/2f6509d2acdee5014d65efaca9e6a9e0df3ca160))
* Conditionally run the "Create or Update PR" step in acir artifacts rebuild workflow ([#2849](https://github.com/noir-lang/noir/issues/2849)) ([63da875](https://github.com/noir-lang/noir/commit/63da875a85a2ad4ad3038443ba52eb28ea44ad10))
* Corrected the formatting of error message parameters in index out of bounds error ([#3630](https://github.com/noir-lang/noir/issues/3630)) ([3bba386](https://github.com/noir-lang/noir/commit/3bba3862dc8703410681300be894bfd1ebca7336))
* **debugger:** Step through foreign calls and breakpoints inside Brillig blocks ([#3511](https://github.com/noir-lang/noir/issues/3511)) ([5d77d7a](https://github.com/noir-lang/noir/commit/5d77d7ac82a4df6995ca151b2c8070044cb1fe9d))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Disable modulo for fields ([#3009](https://github.com/noir-lang/noir/issues/3009)) ([7e68976](https://github.com/noir-lang/noir/commit/7e689768f4af1188e01a1a300a0d2fa152cea504))
* Disallow returning constant values ([#2978](https://github.com/noir-lang/noir/issues/2978)) ([79c2e88](https://github.com/noir-lang/noir/commit/79c2e88ebefe71ebc0fe457347570df31b24ac36))
* Do not perform dead instruction elimination on mod,div unless rhs is constant ([#3141](https://github.com/noir-lang/noir/issues/3141)) ([af3d771](https://github.com/noir-lang/noir/commit/af3d77182054845303fa59de92d783453079a048))
* Do not simply divisions ([#3664](https://github.com/noir-lang/noir/issues/3664)) ([e5b981b](https://github.com/noir-lang/noir/commit/e5b981b08c2b345f00426acafe47b76d5262254d))
* Docker builds ([#3620](https://github.com/noir-lang/noir/issues/3620)) ([f3eac52](https://github.com/noir-lang/noir/commit/f3eac5282860c1954ea2cee6a21633df5b1865fd))
* **docs:** Update `editUrl` path for docusaurus ([#3184](https://github.com/noir-lang/noir/issues/3184)) ([4646a93](https://github.com/noir-lang/noir/commit/4646a93f5e95604b5710353764b2c4295efaef6b))
* Download expected `bb` version if installed backend has version mismatch ([#3150](https://github.com/noir-lang/noir/issues/3150)) ([3f03435](https://github.com/noir-lang/noir/commit/3f03435552fe75b5c7a49bfc8d63d06573381220))
* Error message for assigning the wrong type is backwards [#2804](https://github.com/noir-lang/noir/issues/2804)  ([#2805](https://github.com/noir-lang/noir/issues/2805)) ([b2d62bf](https://github.com/noir-lang/noir/commit/b2d62bff3b7958b3ed62c285a7ebd45045ac2e05))
* Finer bit size in bound constrain ([#2869](https://github.com/noir-lang/noir/issues/2869)) ([68385e2](https://github.com/noir-lang/noir/commit/68385e294a1501b19b28f3f5510e973283ed0821))
* Fix aztec library after nargo fmt ([#3014](https://github.com/noir-lang/noir/issues/3014)) ([f43083c](https://github.com/noir-lang/noir/commit/f43083c744ff13aefa4d294a090c9445a9b70aac))
* Fix crash when using undeclared traits ([#3509](https://github.com/noir-lang/noir/issues/3509)) ([8bb095a](https://github.com/noir-lang/noir/commit/8bb095af77d3b4043855841f1ae5799d75ed94f0))
* Fix lexer error formatting ([#3274](https://github.com/noir-lang/noir/issues/3274)) ([74bd517](https://github.com/noir-lang/noir/commit/74bd517fe7839465ff086ffe622462bed5159006))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Fix panic in some cases when calling a private function ([#2799](https://github.com/noir-lang/noir/issues/2799)) ([078d5df](https://github.com/noir-lang/noir/commit/078d5df691d4ea48e83c9530cd40b64917eba0a7))
* Fix panic when using repeated arrays which define variables ([#3221](https://github.com/noir-lang/noir/issues/3221)) ([c4faf3a](https://github.com/noir-lang/noir/commit/c4faf3a0a40eea1ee02e11dfe08b48c6b4438bbf))
* Fix should_fail_with ([#2940](https://github.com/noir-lang/noir/issues/2940)) ([4f07b84](https://github.com/noir-lang/noir/commit/4f07b84458dba97530d8179a3b9b19101b472616))
* Fix subtract with underflow in flattening pass ([#2796](https://github.com/noir-lang/noir/issues/2796)) ([f2ed505](https://github.com/noir-lang/noir/commit/f2ed5054b0b0335dd3ecb17369b0d2e6eafb1171))
* Fixing versioning workflow ([#3296](https://github.com/noir-lang/noir/issues/3296)) ([3d5e43a](https://github.com/noir-lang/noir/commit/3d5e43a4b8cd9d2bb67d44a2eff93374c3603e42))
* Flatten public inputs according to their index in numerial rather than ascii order ([#3605](https://github.com/noir-lang/noir/issues/3605)) ([a1f6343](https://github.com/noir-lang/noir/commit/a1f6343b7df1b166b1be4db09527694a3df2738a))
* Follow dependencies when looking for a struct ([#3405](https://github.com/noir-lang/noir/issues/3405)) ([561b1b8](https://github.com/noir-lang/noir/commit/561b1b8f0b22d8b1800cb3552942a442a27c2a2c))
* Force recompilation when `output_debug` flag is set. ([#2898](https://github.com/noir-lang/noir/issues/2898)) ([9854416](https://github.com/noir-lang/noir/commit/9854416f5ac03c9da6538edc6a0a540ccccb4b61))
* **frontend:** Error on unsupported integer annotation ([#2778](https://github.com/noir-lang/noir/issues/2778)) ([90c3d8b](https://github.com/noir-lang/noir/commit/90c3d8baa3b7ae10bc99f6a767121f556ff75967))
* Impl methods are no longer placed in contracts ([#3255](https://github.com/noir-lang/noir/issues/3255)) ([b673b07](https://github.com/noir-lang/noir/commit/b673b071663d9756d6346954fce7d4ec6e1577dd))
* Improve error message when multiplying unit values ([#2950](https://github.com/noir-lang/noir/issues/2950)) ([57b7c55](https://github.com/noir-lang/noir/commit/57b7c55e7005876dc2e070c64e1b8115ca8a4242))
* Include .nr and .sol files in builds ([#3039](https://github.com/noir-lang/noir/issues/3039)) ([ae8d0e9](https://github.com/noir-lang/noir/commit/ae8d0e9013f26b52e8f0bdc9f84866ffec50872d))
* Issue an error when a module is declared twice & fix module search path ([#2801](https://github.com/noir-lang/noir/issues/2801)) ([7f76910](https://github.com/noir-lang/noir/commit/7f76910ebbd20e3d7a1db7541f2b7f43cd9b546d))
* Lack of cjs package version ([#2848](https://github.com/noir-lang/noir/issues/2848)) ([adc2d59](https://github.com/noir-lang/noir/commit/adc2d597536b52c690dceb14ea5f8e30a493452c))
* Make for loops a statement ([#2975](https://github.com/noir-lang/noir/issues/2975)) ([0e266eb](https://github.com/noir-lang/noir/commit/0e266ebc7328866b0b10554e37c9d9012a7b501c))
* Match rust behaviour for left-shift overflow ([#3518](https://github.com/noir-lang/noir/issues/3518)) ([2d7ceb1](https://github.com/noir-lang/noir/commit/2d7ceb17edda1d9e70901cfd13f45cdc0df0d28d))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Move mimc to hash submodule ([#3361](https://github.com/noir-lang/noir/issues/3361)) ([3ec29f1](https://github.com/noir-lang/noir/commit/3ec29f17464703716978daacfa9f00c4f5013551))
* Overflow checks for constant folding ([#3420](https://github.com/noir-lang/noir/issues/3420)) ([b7a6383](https://github.com/noir-lang/noir/commit/b7a6383cf9dc3bc4a71b9644352340c1e9339c81))
* Parse parenthesized lvalues ([#3058](https://github.com/noir-lang/noir/issues/3058)) ([50ca58c](https://github.com/noir-lang/noir/commit/50ca58c7b133f8b21091dfd304379429284b0d60))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Prevent mutating immutable bindings to mutable types ([#3075](https://github.com/noir-lang/noir/issues/3075)) ([d5ee20e](https://github.com/noir-lang/noir/commit/d5ee20ea43ccf1130f7d34231562f13e98ea636b))
* **println:** Enable printing of arrays/strings &gt;2 in fmt strings  ([#2947](https://github.com/noir-lang/noir/issues/2947)) ([309fa70](https://github.com/noir-lang/noir/commit/309fa70823535c5340f986a17f4ddddcb8723bb8))
* Recompile artefacts from a different noir version ([#3248](https://github.com/noir-lang/noir/issues/3248)) ([7347b27](https://github.com/noir-lang/noir/commit/7347b2742a5ad38d3d252e657810d061bab83e24))
* Remove cast for field comparisons in brillig ([#2874](https://github.com/noir-lang/noir/issues/2874)) ([1fc1fdb](https://github.com/noir-lang/noir/commit/1fc1fdb4e15d2ce625ea79d458c5346fab418e49))
* Remove duplication of code to load stdlib files ([#2868](https://github.com/noir-lang/noir/issues/2868)) ([b694aab](https://github.com/noir-lang/noir/commit/b694aab87c4665a3a89715c9d4096eeb3efb9944))
* Remove quotes from println output ([#3574](https://github.com/noir-lang/noir/issues/3574)) ([127b6aa](https://github.com/noir-lang/noir/commit/127b6aa1ec8893275fdfa7795db7c52c4fc1d4dd))
* Remove sha2_block test ([#3360](https://github.com/noir-lang/noir/issues/3360)) ([a48c03b](https://github.com/noir-lang/noir/commit/a48c03bec786d1fb85eef46eeddeccf29e81fe76))
* Restrict fill_internal_slices pass to acir functions ([#3634](https://github.com/noir-lang/noir/issues/3634)) ([0cad9aa](https://github.com/noir-lang/noir/commit/0cad9aa9c19091b3679bdc6e7fe044194c5db7e0))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))
* Show println output before an error occurs in `nargo execute` ([#3211](https://github.com/noir-lang/noir/issues/3211)) ([2f0b80d](https://github.com/noir-lang/noir/commit/2f0b80dda8401ce8962c857dbcd9548e7fdde4aa))
* Silence unused variable warnings in stdlib ([#2795](https://github.com/noir-lang/noir/issues/2795)) ([5747bfe](https://github.com/noir-lang/noir/commit/5747bfed256f9179321ec0bd1e02f5f82723a4c7))
* Somewhat reduce mem2reg memory usage ([#3572](https://github.com/noir-lang/noir/issues/3572)) ([9b9ed89](https://github.com/noir-lang/noir/commit/9b9ed890e68b6c7f0671b05919bdc86f593c5df5))
* Split conditional_regression tests ([#2774](https://github.com/noir-lang/noir/issues/2774)) ([8ed8832](https://github.com/noir-lang/noir/commit/8ed8832c7b475cd28ae697a09f1ad07c539736db))
* **ssa:** Do not replace previously constrained values ([#2647](https://github.com/noir-lang/noir/issues/2647)) ([d528844](https://github.com/noir-lang/noir/commit/d5288449a10d162a0340818a6beab54dd985a11a))
* **traits:** Trait functions with a default implementation must not be followed by a semicolon ([#2987](https://github.com/noir-lang/noir/issues/2987)) ([a3593c0](https://github.com/noir-lang/noir/commit/a3593c042163d89bd012b7f901f3b18446209e82))
* Transform hir before type checks  ([#2994](https://github.com/noir-lang/noir/issues/2994)) ([a29b568](https://github.com/noir-lang/noir/commit/a29b568295e40e19dd354bbe47e31f922e08d8c9))
* Update link to recursion example ([#3224](https://github.com/noir-lang/noir/issues/3224)) ([10eae15](https://github.com/noir-lang/noir/commit/10eae15c6992442876e184c7d2bd36a34f639ea1))
* Use 128 bits for constant bit shift ([#3586](https://github.com/noir-lang/noir/issues/3586)) ([2ca9b05](https://github.com/noir-lang/noir/commit/2ca9b059317f0513ea21153ebdb468c4f6633de5))
* Use pedersen_hash for merkle tree ([#3357](https://github.com/noir-lang/noir/issues/3357)) ([6b74d31](https://github.com/noir-lang/noir/commit/6b74d316fec3b379dd7b51064f1acb1a0e6a15cc))
* Verify impls arising from function calls exist ([#3472](https://github.com/noir-lang/noir/issues/3472)) ([d7f919d](https://github.com/noir-lang/noir/commit/d7f919dcc001080ed24616ebbc37426ef7ac7638))


### Miscellaneous Chores

* `generateWitness` now returns a serialized witness file ([#2842](https://github.com/noir-lang/noir/issues/2842)) ([57d3f37](https://github.com/noir-lang/noir/commit/57d3f376d9ceadb75caf37a2bfc0e9394f76bfe6))
* Bump MSRV to 1.71.1 ([#3353](https://github.com/noir-lang/noir/issues/3353)) ([78f2127](https://github.com/noir-lang/noir/commit/78f2127dd12e36e831e63fd670d9f9d870818af7))
* Change stdlib function `pedersen` to `pedersen_commitment` ([#3341](https://github.com/noir-lang/noir/issues/3341)) ([964b777](https://github.com/noir-lang/noir/commit/964b7771506bdf8408d8917ab32bf51db8ce09d2))
* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))
* **noir_js:** Rename inner and outer proof methods ([#2845](https://github.com/noir-lang/noir/issues/2845)) ([71dbbb8](https://github.com/noir-lang/noir/commit/71dbbb863a6f262da4804c17965ace627bf3a278))
* Update to `bb` version 0.7.3 ([#2729](https://github.com/noir-lang/noir/issues/2729)) ([fce68d1](https://github.com/noir-lang/noir/commit/fce68d1404ae66bd7a71417d791dd70545bf24f2))

## [0.19.5](https://github.com/noir-lang/noir/compare/v0.19.4...v0.19.5) (2023-12-01)


### Features

* Add `FieldElement::from&lt;usize&gt;` implementation ([#3647](https://github.com/noir-lang/noir/issues/3647)) ([8b7c5aa](https://github.com/noir-lang/noir/commit/8b7c5aa5311f4e6811438f67bd552b641b13fc9a))
* Add package version to Nargo.toml metadata ([#3427](https://github.com/noir-lang/noir/issues/3427)) ([9e1717c](https://github.com/noir-lang/noir/commit/9e1717c2d96a0b9e394e5cb2fb9e1d09b5259ca0))
* Add special case for boolean AND in acir-gen ([#3615](https://github.com/noir-lang/noir/issues/3615)) ([824039b](https://github.com/noir-lang/noir/commit/824039bcc0a3275f333ea94aecc701d129f99fe5))
* Aztec-packages ([#3599](https://github.com/noir-lang/noir/issues/3599)) ([2cd6dc3](https://github.com/noir-lang/noir/commit/2cd6dc39e3a956aa5dff721d47aaf1921f98fded))
* Aztec-packages ([#3626](https://github.com/noir-lang/noir/issues/3626)) ([e0a96ea](https://github.com/noir-lang/noir/commit/e0a96ea70b17c8c898dd72ac929f3969a4cec1d3))
* Complex slice inputs for dynamic slice builtins ([#3617](https://github.com/noir-lang/noir/issues/3617)) ([8b23b34](https://github.com/noir-lang/noir/commit/8b23b349ae15afa48f6cbe8962586bbe79e79890))
* Copy on write optimization for brillig ([#3522](https://github.com/noir-lang/noir/issues/3522)) ([da29c02](https://github.com/noir-lang/noir/commit/da29c02327acb2f46f5a7a25c7404dfa44c82616))
* Data bus ([#3508](https://github.com/noir-lang/noir/issues/3508)) ([6b0bdbc](https://github.com/noir-lang/noir/commit/6b0bdbced1bfe2c92e90dd7b70ca8f9e5ccb7c0d))
* Implement integer printing ([#3577](https://github.com/noir-lang/noir/issues/3577)) ([6601408](https://github.com/noir-lang/noir/commit/6601408e378b2afe4fdfd8e04482c39311ccc7e9))
* Implement raw string literals ([#3556](https://github.com/noir-lang/noir/issues/3556)) ([87a302f](https://github.com/noir-lang/noir/commit/87a302fbc26d5fd42694953935d95a7ccb3d50a0))
* **lsp:** Add goto definition for functions ([#3656](https://github.com/noir-lang/noir/issues/3656)) ([7bb7356](https://github.com/noir-lang/noir/commit/7bb735662c86e6533ac58f448ad748e34f02edb7))
* Reuse witnesses more when interacting with memory ([#3658](https://github.com/noir-lang/noir/issues/3658)) ([5a4a73d](https://github.com/noir-lang/noir/commit/5a4a73d24ddd7d2bb46c85714397ee6e031c97a6))


### Bug Fixes

* Corrected the formatting of error message parameters in index out of bounds error ([#3630](https://github.com/noir-lang/noir/issues/3630)) ([3bba386](https://github.com/noir-lang/noir/commit/3bba3862dc8703410681300be894bfd1ebca7336))
* Do not simply divisions ([#3664](https://github.com/noir-lang/noir/issues/3664)) ([e5b981b](https://github.com/noir-lang/noir/commit/e5b981b08c2b345f00426acafe47b76d5262254d))
* Docker builds ([#3620](https://github.com/noir-lang/noir/issues/3620)) ([f3eac52](https://github.com/noir-lang/noir/commit/f3eac5282860c1954ea2cee6a21633df5b1865fd))
* Flatten public inputs according to their index in numerial rather than ascii order ([#3605](https://github.com/noir-lang/noir/issues/3605)) ([a1f6343](https://github.com/noir-lang/noir/commit/a1f6343b7df1b166b1be4db09527694a3df2738a))
* Restrict fill_internal_slices pass to acir functions ([#3634](https://github.com/noir-lang/noir/issues/3634)) ([0cad9aa](https://github.com/noir-lang/noir/commit/0cad9aa9c19091b3679bdc6e7fe044194c5db7e0))

## [0.19.4](https://github.com/noir-lang/noir/compare/v0.19.3...v0.19.4) (2023-11-28)


### Features

* Add --check option to nargo fmt for dry-run formatting verification ([#3530](https://github.com/noir-lang/noir/issues/3530)) ([4469707](https://github.com/noir-lang/noir/commit/4469707d97085fab0f7ade8d015dc827c56156ee))
* Add support for tuple values in `noir_codegen` ([#3592](https://github.com/noir-lang/noir/issues/3592)) ([346d75f](https://github.com/noir-lang/noir/commit/346d75f9dd9261996d4d7bb80eb7e4118e8f8ce2))
* Codegen typed interfaces for functions in `noir_codegen` ([#3533](https://github.com/noir-lang/noir/issues/3533)) ([290c463](https://github.com/noir-lang/noir/commit/290c463622a93a34293f73b5bf2aea7ade30a11c))
* Export `CompiledCircuit` from codegened TS ([#3589](https://github.com/noir-lang/noir/issues/3589)) ([e06c675](https://github.com/noir-lang/noir/commit/e06c67500da11518caffe0e98bdb9cd7f5f89049))
* Remove type arrays for flat slices ([#3466](https://github.com/noir-lang/noir/issues/3466)) ([8225b2b](https://github.com/noir-lang/noir/commit/8225b2b379ddf145f9418f8517478704f9aac350))
* Send and receive unflattened public inputs to backend ([#3543](https://github.com/noir-lang/noir/issues/3543)) ([a7bdc67](https://github.com/noir-lang/noir/commit/a7bdc67ef3ec2037bffc4f1f472907cad786c319))


### Bug Fixes

* Compiler version error message ([#3558](https://github.com/noir-lang/noir/issues/3558)) ([026a358](https://github.com/noir-lang/noir/commit/026a3587b01ddc8f444ff588a7b3f3fd1a0bb386))
* Remove quotes from println output ([#3574](https://github.com/noir-lang/noir/issues/3574)) ([127b6aa](https://github.com/noir-lang/noir/commit/127b6aa1ec8893275fdfa7795db7c52c4fc1d4dd))
* Somewhat reduce mem2reg memory usage ([#3572](https://github.com/noir-lang/noir/issues/3572)) ([9b9ed89](https://github.com/noir-lang/noir/commit/9b9ed890e68b6c7f0671b05919bdc86f593c5df5))
* Use 128 bits for constant bit shift ([#3586](https://github.com/noir-lang/noir/issues/3586)) ([2ca9b05](https://github.com/noir-lang/noir/commit/2ca9b059317f0513ea21153ebdb468c4f6633de5))

## [0.19.3](https://github.com/noir-lang/noir/compare/v0.19.2...v0.19.3) (2023-11-22)


### Features

* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Add LSP command to profile opcodes in vscode ([#3496](https://github.com/noir-lang/noir/issues/3496)) ([6fbf77a](https://github.com/noir-lang/noir/commit/6fbf77ae2b87a55db92344f5066a82ccaf6c2086))
* Add lsp formatting ([#3433](https://github.com/noir-lang/noir/issues/3433)) ([286c876](https://github.com/noir-lang/noir/commit/286c87694fda185f25b05cec5504142643bc207f))
* Allow providing custom foreign call executors to `execute_circuit` ([#3506](https://github.com/noir-lang/noir/issues/3506)) ([d27db33](https://github.com/noir-lang/noir/commit/d27db332f8c320ffd9b5520bebbd83ae09e31de7))
* Compile without a backend ([#3437](https://github.com/noir-lang/noir/issues/3437)) ([d69cf5d](https://github.com/noir-lang/noir/commit/d69cf5debcc430bb019b6cc95774aac084776dda))
* Enable the `fmt` command in the help menu ([#3328](https://github.com/noir-lang/noir/issues/3328)) ([63d414c](https://github.com/noir-lang/noir/commit/63d414c06a399525601e3db11dc48b180e93c2d8))
* Handle constant index operations on simple slices ([#3464](https://github.com/noir-lang/noir/issues/3464)) ([7ae12f8](https://github.com/noir-lang/noir/commit/7ae12f8c5243d31b2f410c246ed6b9e2fcea5d4c))


### Bug Fixes

* "Missing trait impl" error in trait dispatch ([#3440](https://github.com/noir-lang/noir/issues/3440)) ([52daaec](https://github.com/noir-lang/noir/commit/52daaec504101fe3c0caa30441c17f30a34af475))
* Adding proving key initialization ([#3322](https://github.com/noir-lang/noir/issues/3322)) ([3383740](https://github.com/noir-lang/noir/commit/3383740f9a0004f2ee77c9686f81baed6cd1917c))
* Allow `where` clause on all functions and improve error message ([#3465](https://github.com/noir-lang/noir/issues/3465)) ([1647e33](https://github.com/noir-lang/noir/commit/1647e33564bf56ab8721a365f5fc6bcb38901412))
* Apply predicate to over/underflow checks ([#3494](https://github.com/noir-lang/noir/issues/3494)) ([fc3edf7](https://github.com/noir-lang/noir/commit/fc3edf7aa5da9074614fa900bbcb57e512e3d56b))
* **debugger:** Step through foreign calls and breakpoints inside Brillig blocks ([#3511](https://github.com/noir-lang/noir/issues/3511)) ([5d77d7a](https://github.com/noir-lang/noir/commit/5d77d7ac82a4df6995ca151b2c8070044cb1fe9d))
* Fix crash when using undeclared traits ([#3509](https://github.com/noir-lang/noir/issues/3509)) ([8bb095a](https://github.com/noir-lang/noir/commit/8bb095af77d3b4043855841f1ae5799d75ed94f0))
* Match rust behaviour for left-shift overflow ([#3518](https://github.com/noir-lang/noir/issues/3518)) ([2d7ceb1](https://github.com/noir-lang/noir/commit/2d7ceb17edda1d9e70901cfd13f45cdc0df0d28d))
* Verify impls arising from function calls exist ([#3472](https://github.com/noir-lang/noir/issues/3472)) ([d7f919d](https://github.com/noir-lang/noir/commit/d7f919dcc001080ed24616ebbc37426ef7ac7638))

## [0.19.2](https://github.com/noir-lang/noir/compare/v0.19.1...v0.19.2) (2023-11-07)


### Features

* Add profile info print out ([#3425](https://github.com/noir-lang/noir/issues/3425)) ([a8b5fa8](https://github.com/noir-lang/noir/commit/a8b5fa8e30dc27e64666381b7451569f350967d1))

## [0.19.1](https://github.com/noir-lang/noir/compare/v0.19.0...v0.19.1) (2023-11-07)


### Features

* **acir:** Handle dynamic array operations for nested slices ([#3187](https://github.com/noir-lang/noir/issues/3187)) ([e026319](https://github.com/noir-lang/noir/commit/e026319fc25763d30781b90e6a4454ddb5d3bc7b))
* Properly track equivalence of witnesses generated for black box functions ([#3428](https://github.com/noir-lang/noir/issues/3428)) ([20b70c2](https://github.com/noir-lang/noir/commit/20b70c29b4bd323d67ef56ab1933341a7747d3cb))
* Use ranges instead of a vector for input witness ([#3314](https://github.com/noir-lang/noir/issues/3314)) ([b12b7ec](https://github.com/noir-lang/noir/commit/b12b7ecb995988c731be5f1f2f67fda952f1a228))


### Bug Fixes

* Follow dependencies when looking for a struct ([#3405](https://github.com/noir-lang/noir/issues/3405)) ([561b1b8](https://github.com/noir-lang/noir/commit/561b1b8f0b22d8b1800cb3552942a442a27c2a2c))
* Overflow checks for constant folding ([#3420](https://github.com/noir-lang/noir/issues/3420)) ([b7a6383](https://github.com/noir-lang/noir/commit/b7a6383cf9dc3bc4a71b9644352340c1e9339c81))

## [0.19.0](https://github.com/noir-lang/noir/compare/v0.18.0...v0.19.0) (2023-11-02)


### ⚠ BREAKING CHANGES

* avoid integer overflows ([#2713](https://github.com/noir-lang/noir/issues/2713))
* return Pedersen structure in stdlib ([#3190](https://github.com/noir-lang/noir/issues/3190))
* noir-wasm outputs debug symbols ([#3317](https://github.com/noir-lang/noir/issues/3317))
* move mimc to hash submodule ([#3361](https://github.com/noir-lang/noir/issues/3361))
* bump MSRV to 1.71.1 ([#3353](https://github.com/noir-lang/noir/issues/3353))
* Add semver checks for the compiler version in Nargo.toml ([#3336](https://github.com/noir-lang/noir/issues/3336))
* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* change stdlib function `pedersen` to `pedersen_commitment` ([#3341](https://github.com/noir-lang/noir/issues/3341))

### Features

* `compute_note_hash_and_nullifier` check ([#3216](https://github.com/noir-lang/noir/issues/3216)) ([4963c6c](https://github.com/noir-lang/noir/commit/4963c6c024bba710dd8907147f631f5c26094f0a))
* Add `noir_codegen` package ([#3392](https://github.com/noir-lang/noir/issues/3392)) ([6c4cd4d](https://github.com/noir-lang/noir/commit/6c4cd4d37e4af38dccf899bcbd3950d1e236b35d))
* Add check for overlapping generic traits ([#3307](https://github.com/noir-lang/noir/issues/3307)) ([8cf81b6](https://github.com/noir-lang/noir/commit/8cf81b659bed9522aede29c1ebb4a4ed2bfa1205))
* Add exports of JS black box solvers to noirJS ([#3295](https://github.com/noir-lang/noir/issues/3295)) ([8369871](https://github.com/noir-lang/noir/commit/836987150f82354d3dfc01cfaad69f70240ca80c))
* Add generic count check for trait methods ([#3382](https://github.com/noir-lang/noir/issues/3382)) ([a9f9717](https://github.com/noir-lang/noir/commit/a9f9717ba69c0dd8e4fc7045fe6aea2077b84c95))
* Add semver checks for the compiler version in Nargo.toml ([#3336](https://github.com/noir-lang/noir/issues/3336)) ([0e530cf](https://github.com/noir-lang/noir/commit/0e530cfe86f87a532be30a02f4353d010e47e458))
* Allow a trait to be implemented multiple times for the same struct ([#3292](https://github.com/noir-lang/noir/issues/3292)) ([51831df](https://github.com/noir-lang/noir/commit/51831df68bc20460c6d05d55469002db06113925))
* Allow traits to have generic functions ([#3365](https://github.com/noir-lang/noir/issues/3365)) ([0f9af65](https://github.com/noir-lang/noir/commit/0f9af652efc6b7628784a397a9df674eaa30de61))
* Avoid integer overflows ([#2713](https://github.com/noir-lang/noir/issues/2713)) ([7d7d632](https://github.com/noir-lang/noir/commit/7d7d63291d712137f97e6d44a774acdf2bd20512))
* Check where clauses when searching for trait impls ([#3407](https://github.com/noir-lang/noir/issues/3407)) ([84c6604](https://github.com/noir-lang/noir/commit/84c6604397f262c09ba4bac157fead38b8280313))
* Expand trait impl overlap check to cover generic types ([#3320](https://github.com/noir-lang/noir/issues/3320)) ([a01549b](https://github.com/noir-lang/noir/commit/a01549b78a2c5de3e64ce5d3e5c4bb73a8c8b4fb))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Handle warnings in evaluator ([#3205](https://github.com/noir-lang/noir/issues/3205)) ([5cfd156](https://github.com/noir-lang/noir/commit/5cfd156ca2035038a226bdd81a51636d3de3c34e))
* Implement where clauses on impls ([#3324](https://github.com/noir-lang/noir/issues/3324)) ([4c3d1de](https://github.com/noir-lang/noir/commit/4c3d1dea27726133335538443df3dbbd2c8f2d58))
* Make generic impls callable ([#3297](https://github.com/noir-lang/noir/issues/3297)) ([8d9b738](https://github.com/noir-lang/noir/commit/8d9b738ea958320a16946ebe6fcfb6bbf2dda42e))
* Manage breakpoints and allow restarting a debugging session ([#3325](https://github.com/noir-lang/noir/issues/3325)) ([f502108](https://github.com/noir-lang/noir/commit/f502108a59e2141f4898b0a25e84d7ed7d2bff58))
* **noir_js:** Allow providing foreign call handlers in noirJS ([#3294](https://github.com/noir-lang/noir/issues/3294)) ([c76b0f8](https://github.com/noir-lang/noir/commit/c76b0f81b89ae02c39c13c3bd400b5d13f083759))
* Noir-wasm outputs debug symbols ([#3317](https://github.com/noir-lang/noir/issues/3317)) ([f9933fa](https://github.com/noir-lang/noir/commit/f9933fa50c42aade8ddc13d29deef7645ddcf586))
* Perform compile-time euclidean division on constants ([#3231](https://github.com/noir-lang/noir/issues/3231)) ([3866d7e](https://github.com/noir-lang/noir/commit/3866d7ea32dcdabbdb5ba85478d97fd51a6850f3))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Return Pedersen structure in stdlib ([#3190](https://github.com/noir-lang/noir/issues/3190)) ([be30d59](https://github.com/noir-lang/noir/commit/be30d59e61a9dff6ab94ffb97365c1282c331643))


### Bug Fixes

* `compute_note_hash_and_nullifier` compiler check ([#3351](https://github.com/noir-lang/noir/issues/3351)) ([4e2d35f](https://github.com/noir-lang/noir/commit/4e2d35f256bea2fee3a6cbd7af0c0c15a37c0a2e))
* **3275:** Activate brillig modulo test with negative integers ([#3318](https://github.com/noir-lang/noir/issues/3318)) ([31c493c](https://github.com/noir-lang/noir/commit/31c493ce2082a571d147f707837beb4f3ed2ca64))
* **3300:** Cache warnings into debug artefacts ([#3313](https://github.com/noir-lang/noir/issues/3313)) ([cb5a15b](https://github.com/noir-lang/noir/commit/cb5a15b9dbcfdaac5d656a122c73dca23855307d))
* **aztec_nr:** Serialise arrays of structs ([#3401](https://github.com/noir-lang/noir/issues/3401)) ([e979a58](https://github.com/noir-lang/noir/commit/e979a587e755d4b715ebacba715d778938026ac0))
* Fixing versioning workflow ([#3296](https://github.com/noir-lang/noir/issues/3296)) ([3d5e43a](https://github.com/noir-lang/noir/commit/3d5e43a4b8cd9d2bb67d44a2eff93374c3603e42))
* Move mimc to hash submodule ([#3361](https://github.com/noir-lang/noir/issues/3361)) ([3ec29f1](https://github.com/noir-lang/noir/commit/3ec29f17464703716978daacfa9f00c4f5013551))
* Remove sha2_block test ([#3360](https://github.com/noir-lang/noir/issues/3360)) ([a48c03b](https://github.com/noir-lang/noir/commit/a48c03bec786d1fb85eef46eeddeccf29e81fe76))
* Use pedersen_hash for merkle tree ([#3357](https://github.com/noir-lang/noir/issues/3357)) ([6b74d31](https://github.com/noir-lang/noir/commit/6b74d316fec3b379dd7b51064f1acb1a0e6a15cc))


### Miscellaneous Chores

* Bump MSRV to 1.71.1 ([#3353](https://github.com/noir-lang/noir/issues/3353)) ([78f2127](https://github.com/noir-lang/noir/commit/78f2127dd12e36e831e63fd670d9f9d870818af7))
* Change stdlib function `pedersen` to `pedersen_commitment` ([#3341](https://github.com/noir-lang/noir/issues/3341)) ([964b777](https://github.com/noir-lang/noir/commit/964b7771506bdf8408d8917ab32bf51db8ce09d2))
* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.18.0](https://github.com/noir-lang/noir/compare/v0.17.0...v0.18.0) (2023-10-25)


### ⚠ BREAKING CHANGES

* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))

### Features

* Add crate for pub modifier ([#3271](https://github.com/noir-lang/noir/issues/3271)) ([e7a1a1a](https://github.com/noir-lang/noir/commit/e7a1a1a4b42b6b72c16f2204e33af80dbabba6b5))
* Cache debug artifacts  ([#3133](https://github.com/noir-lang/noir/issues/3133)) ([c5a6229](https://github.com/noir-lang/noir/commit/c5a622983e4049d82589f185be5e96c63ed6066d))
* **debugger:** Print limited source code context ([#3217](https://github.com/noir-lang/noir/issues/3217)) ([dcda1c7](https://github.com/noir-lang/noir/commit/dcda1c7aed69ae8f55cd3f680e3cc1ece9de7541))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Implement `bound_constraint_with_offset` in terms of `AcirVar`s ([#3233](https://github.com/noir-lang/noir/issues/3233)) ([8d89cb5](https://github.com/noir-lang/noir/commit/8d89cb59fe710859a96eaed4f988952bd727fb7d))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Noir-wasm takes dependency graph ([#3213](https://github.com/noir-lang/noir/issues/3213)) ([a2c8ebd](https://github.com/noir-lang/noir/commit/a2c8ebd4a800d7ef042ac9cbe5ee6a837c715634))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* **stdlib:** Optimize constraint counts in sha256/sha512 ([#3253](https://github.com/noir-lang/noir/issues/3253)) ([d3be552](https://github.com/noir-lang/noir/commit/d3be552149ab375b24b509603fcd7237b374ca5a))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))


### Bug Fixes

* Add size checks to integer literals ([#3236](https://github.com/noir-lang/noir/issues/3236)) ([7f8fe8c](https://github.com/noir-lang/noir/commit/7f8fe8c88eb2d26ae3a93e2f74430fadc74b4836))
* Fix lexer error formatting ([#3274](https://github.com/noir-lang/noir/issues/3274)) ([74bd517](https://github.com/noir-lang/noir/commit/74bd517fe7839465ff086ffe622462bed5159006))
* Impl methods are no longer placed in contracts ([#3255](https://github.com/noir-lang/noir/issues/3255)) ([b673b07](https://github.com/noir-lang/noir/commit/b673b071663d9756d6346954fce7d4ec6e1577dd))
* Recompile artefacts from a different noir version ([#3248](https://github.com/noir-lang/noir/issues/3248)) ([7347b27](https://github.com/noir-lang/noir/commit/7347b2742a5ad38d3d252e657810d061bab83e24))
* Show println output before an error occurs in `nargo execute` ([#3211](https://github.com/noir-lang/noir/issues/3211)) ([2f0b80d](https://github.com/noir-lang/noir/commit/2f0b80dda8401ce8962c857dbcd9548e7fdde4aa))

## [0.17.0](https://github.com/noir-lang/noir/compare/v0.16.0...v0.17.0) (2023-10-20)


### ⚠ BREAKING CHANGES

* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* Make for loops a statement ([#2975](https://github.com/noir-lang/noir/issues/2975))
* **traits:** trait functions with a default implementation must not be followed by a semicolon ([#2987](https://github.com/noir-lang/noir/issues/2987))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* **wasm:** update wasm artifacts to match cli artifacts ([#2973](https://github.com/noir-lang/noir/issues/2973))

### Features

* **acir:** Set dynamic array values ([#3054](https://github.com/noir-lang/noir/issues/3054)) ([e871866](https://github.com/noir-lang/noir/commit/e871866d2203f0f0f49f3b273d99d385b950b65f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add `destroy` method to `Noir` ([#3105](https://github.com/noir-lang/noir/issues/3105)) ([7e40274](https://github.com/noir-lang/noir/commit/7e402744a7d64ffcac6db026cec1631230204f0f))
* Add `execute` method to `Noir` class ([#3081](https://github.com/noir-lang/noir/issues/3081)) ([17bdd7e](https://github.com/noir-lang/noir/commit/17bdd7e3909f0ddd195e5cb7095cd0d30758ed43))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add an options object to `BarretenbergBackend` constructor ([#3105](https://github.com/noir-lang/noir/issues/3105)) ([7e40274](https://github.com/noir-lang/noir/commit/7e402744a7d64ffcac6db026cec1631230204f0f))
* Add aztec selectors for event structs ([#2983](https://github.com/noir-lang/noir/issues/2983)) ([982380e](https://github.com/noir-lang/noir/commit/982380e54bb4d696688522c540f1234734ae2e80))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add experimental REPL-based debugger ([#2995](https://github.com/noir-lang/noir/issues/2995)) ([281c696](https://github.com/noir-lang/noir/commit/281c696da61c64b42b9525b8756ffc195f70d775))
* Add JS types for ABI and input maps ([#3023](https://github.com/noir-lang/noir/issues/3023)) ([599e7a1](https://github.com/noir-lang/noir/commit/599e7a1d6bae5d93273e9ef1265024eac909660d))
* **debugger:** Highlight current src code loc ([#3174](https://github.com/noir-lang/noir/issues/3174)) ([6b87582](https://github.com/noir-lang/noir/commit/6b87582dfe872ad6c248cf9995d76b0ef1580625))
* Format infix expressions ([#3001](https://github.com/noir-lang/noir/issues/3001)) ([7926ada](https://github.com/noir-lang/noir/commit/7926ada88ed08ac9d874604834533d900fbb16b0))
* **formatter:** Add formatter support for array literals ([#3061](https://github.com/noir-lang/noir/issues/3061)) ([a535321](https://github.com/noir-lang/noir/commit/a5353217a1f49b83daf11d5fa55e0bcccebf0271))
* Implement automatic dereferencing for index expressions ([#3082](https://github.com/noir-lang/noir/issues/3082)) ([8221bfd](https://github.com/noir-lang/noir/commit/8221bfd2ffde7d1dbf71a72d95257acf76ecca74))
* Implement automatic dereferencing for indexing lvalues ([#3083](https://github.com/noir-lang/noir/issues/3083)) ([6e2b70a](https://github.com/noir-lang/noir/commit/6e2b70ae90b686158957ea29ef1b2a5f0ed38e5f))
* Implement impl specialization ([#3087](https://github.com/noir-lang/noir/issues/3087)) ([44716fa](https://github.com/noir-lang/noir/commit/44716fae0bae0f78ceee76f7231af49c4abeace1))
* **lsp:** Add "info" codelens ([#2982](https://github.com/noir-lang/noir/issues/2982)) ([80770d9](https://github.com/noir-lang/noir/commit/80770d9fae7c42e69a62cf01babfc69449600ac5))
* Nargo test runtime callstacks and assert messages without string matching ([#2953](https://github.com/noir-lang/noir/issues/2953)) ([1b6a4e6](https://github.com/noir-lang/noir/commit/1b6a4e6021929c23a1bca5dff02c004422cc71f8))
* Old docs issues ([#3195](https://github.com/noir-lang/noir/issues/3195)) ([26746c5](https://github.com/noir-lang/noir/commit/26746c59e12a60f3869a5b885b05926c94f01215))
* Optimize euclidean division acir-gen ([#3121](https://github.com/noir-lang/noir/issues/3121)) ([2c175c0](https://github.com/noir-lang/noir/commit/2c175c0d886eea390ef97ada1c2a5b0e1bef15e8))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Prevent unnecessary witness creation in euclidean division ([#2980](https://github.com/noir-lang/noir/issues/2980)) ([c6f660e](https://github.com/noir-lang/noir/commit/c6f660e86d40a106930483f1d6161814e3c0de10))
* Provide formatting subcommand  ([#2640](https://github.com/noir-lang/noir/issues/2640)) ([a38b15f](https://github.com/noir-lang/noir/commit/a38b15f5d8e69faff125d363f2fd1f2f90ae6830))
* Publish aztec build of noir_wasm ([#3049](https://github.com/noir-lang/noir/issues/3049)) ([3b51f4d](https://github.com/noir-lang/noir/commit/3b51f4df7e808233f6987baec93f4b5de7e5b304))
* Remove unnecessary truncation of boolean multiplication ([#3122](https://github.com/noir-lang/noir/issues/3122)) ([39dbcf1](https://github.com/noir-lang/noir/commit/39dbcf1ab80d2bb472d08db4de15d4e0c1f2eb52))
* Return compilation errors from noir_wasm ([#3091](https://github.com/noir-lang/noir/issues/3091)) ([55f63c9](https://github.com/noir-lang/noir/commit/55f63c935cec62fbba63eed421812a4372c1aa4d))
* Reuse witnesses which have been assigned constant values during ACIR gen ([#3137](https://github.com/noir-lang/noir/issues/3137)) ([9eb43e2](https://github.com/noir-lang/noir/commit/9eb43e2a4665397295e74a593f73d19fa9fa5d27))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* **traits:** Add impl Trait as function return type [#2397](https://github.com/noir-lang/noir/issues/2397) ([#3176](https://github.com/noir-lang/noir/issues/3176)) ([4cb2024](https://github.com/noir-lang/noir/commit/4cb20244abba0abc49be0376611979a786563565))
* **traits:** Add trait impl for buildin types ([#2964](https://github.com/noir-lang/noir/issues/2964)) ([2c87b27](https://github.com/noir-lang/noir/commit/2c87b273dfdf033dd8c79b78f006a0e9813559d7))
* **traits:** Added checks for duplicated trait associated items (types, consts, functions) ([#2927](https://github.com/noir-lang/noir/issues/2927)) ([d49492c](https://github.com/noir-lang/noir/commit/d49492cd80d04ee6acc01247b06b088deefcd0c6))
* **traits:** Allow multiple traits to share the same associated function name and to be implemented for the same type ([#3126](https://github.com/noir-lang/noir/issues/3126)) ([004f8dd](https://github.com/noir-lang/noir/commit/004f8dd733cb23da4ed57b160f6b86d53bc0b5f1))
* **traits:** Improve support for traits static method resolution ([#2958](https://github.com/noir-lang/noir/issues/2958)) ([0d0d8f7](https://github.com/noir-lang/noir/commit/0d0d8f7d2b401eb6b534dbb175dfd4b26d2a5f7d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))
* **wasm:** Update wasm artifacts to match cli artifacts ([#2973](https://github.com/noir-lang/noir/issues/2973)) ([ce16c0b](https://github.com/noir-lang/noir/commit/ce16c0b14565cfe1bc2c9f09ae71643d2657440b))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* Add `pub` modifier to grumpkin functions ([#3036](https://github.com/noir-lang/noir/issues/3036)) ([f8990d7](https://github.com/noir-lang/noir/commit/f8990d75b948ce0a6968db659370f7ece7f5db08))
* Add compiler error message for invalid input types ([#3220](https://github.com/noir-lang/noir/issues/3220)) ([989e80d](https://github.com/noir-lang/noir/commit/989e80d4ea62e68cfab69a1cd16d481cbccc6c02))
* Allow constructors in parentheses in `if` conditions and `for` ranges ([#3219](https://github.com/noir-lang/noir/issues/3219)) ([ad192d1](https://github.com/noir-lang/noir/commit/ad192d1b7492f6ecd5fc98bb88201d6c442dc052))
* Allow two `TypeVariable::Constant(N)` to unify even if their constants are not equal ([#3225](https://github.com/noir-lang/noir/issues/3225)) ([cc4ca4b](https://github.com/noir-lang/noir/commit/cc4ca4bb5f4fed5f531a2040501fcc6ed53a9ab4))
* Change non-constant argument errors from `to_be_radix`  from ICE to proper error ([#3048](https://github.com/noir-lang/noir/issues/3048)) ([19ce286](https://github.com/noir-lang/noir/commit/19ce28638fe3ea42ab4984cb99e3898cd17fa8d9))
* Check for overflow with hexadecimal inputs ([#3004](https://github.com/noir-lang/noir/issues/3004)) ([db1e736](https://github.com/noir-lang/noir/commit/db1e736240c0b74f6f59504db5a50de1c749d395))
* Complete debug metadata ([#3228](https://github.com/noir-lang/noir/issues/3228)) ([2f6509d](https://github.com/noir-lang/noir/commit/2f6509d2acdee5014d65efaca9e6a9e0df3ca160))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Disable modulo for fields ([#3009](https://github.com/noir-lang/noir/issues/3009)) ([7e68976](https://github.com/noir-lang/noir/commit/7e689768f4af1188e01a1a300a0d2fa152cea504))
* Disallow returning constant values ([#2978](https://github.com/noir-lang/noir/issues/2978)) ([79c2e88](https://github.com/noir-lang/noir/commit/79c2e88ebefe71ebc0fe457347570df31b24ac36))
* Do not perform dead instruction elimination on mod,div unless rhs is constant ([#3141](https://github.com/noir-lang/noir/issues/3141)) ([af3d771](https://github.com/noir-lang/noir/commit/af3d77182054845303fa59de92d783453079a048))
* **docs:** Update `editUrl` path for docusaurus ([#3184](https://github.com/noir-lang/noir/issues/3184)) ([4646a93](https://github.com/noir-lang/noir/commit/4646a93f5e95604b5710353764b2c4295efaef6b))
* Download expected `bb` version if installed backend has version mismatch ([#3150](https://github.com/noir-lang/noir/issues/3150)) ([3f03435](https://github.com/noir-lang/noir/commit/3f03435552fe75b5c7a49bfc8d63d06573381220))
* Fix aztec library after nargo fmt ([#3014](https://github.com/noir-lang/noir/issues/3014)) ([f43083c](https://github.com/noir-lang/noir/commit/f43083c744ff13aefa4d294a090c9445a9b70aac))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Fix panic when using repeated arrays which define variables ([#3221](https://github.com/noir-lang/noir/issues/3221)) ([c4faf3a](https://github.com/noir-lang/noir/commit/c4faf3a0a40eea1ee02e11dfe08b48c6b4438bbf))
* Include .nr and .sol files in builds ([#3039](https://github.com/noir-lang/noir/issues/3039)) ([ae8d0e9](https://github.com/noir-lang/noir/commit/ae8d0e9013f26b52e8f0bdc9f84866ffec50872d))
* Make for loops a statement ([#2975](https://github.com/noir-lang/noir/issues/2975)) ([0e266eb](https://github.com/noir-lang/noir/commit/0e266ebc7328866b0b10554e37c9d9012a7b501c))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Parse parenthesized lvalues ([#3058](https://github.com/noir-lang/noir/issues/3058)) ([50ca58c](https://github.com/noir-lang/noir/commit/50ca58c7b133f8b21091dfd304379429284b0d60))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Prevent mutating immutable bindings to mutable types ([#3075](https://github.com/noir-lang/noir/issues/3075)) ([d5ee20e](https://github.com/noir-lang/noir/commit/d5ee20ea43ccf1130f7d34231562f13e98ea636b))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))
* **traits:** Trait functions with a default implementation must not be followed by a semicolon ([#2987](https://github.com/noir-lang/noir/issues/2987)) ([a3593c0](https://github.com/noir-lang/noir/commit/a3593c042163d89bd012b7f901f3b18446209e82))
* Transform hir before type checks  ([#2994](https://github.com/noir-lang/noir/issues/2994)) ([a29b568](https://github.com/noir-lang/noir/commit/a29b568295e40e19dd354bbe47e31f922e08d8c9))
* Update link to recursion example ([#3224](https://github.com/noir-lang/noir/issues/3224)) ([10eae15](https://github.com/noir-lang/noir/commit/10eae15c6992442876e184c7d2bd36a34f639ea1))

## [0.16.0](https://github.com/noir-lang/noir/compare/v0.15.0...v0.16.0) (2023-10-03)


### ⚠ BREAKING CHANGES

* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **abi:** Tuples as inputs/outputs to main ([#2899](https://github.com/noir-lang/noir/issues/2899)) ([d8bd78f](https://github.com/noir-lang/noir/commit/d8bd78f60c447bb8488a844d779e8aaf4150afe7))
* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* Add bb interface implementation ([#2902](https://github.com/noir-lang/noir/issues/2902)) ([fe92dc0](https://github.com/noir-lang/noir/commit/fe92dc0df57b2cbc0e7b8cd1f3a91cba6b0f3049))
* Add noir types package ([#2893](https://github.com/noir-lang/noir/issues/2893)) ([e8fc868](https://github.com/noir-lang/noir/commit/e8fc8687e6dd89295fd023201443f1197963a243))
* Dynamic indexing of non-homogenous slices ([#2883](https://github.com/noir-lang/noir/issues/2883)) ([72c3661](https://github.com/noir-lang/noir/commit/72c3661c86712b99236eafaac99f76f13d42b9d9))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Oracle mocker for nargo test ([#2928](https://github.com/noir-lang/noir/issues/2928)) ([0dd1e77](https://github.com/noir-lang/noir/commit/0dd1e77c0e625805e15fa56b4738c93ebae19b6d))
* **ssa:** Multiple slice mergers ([#2753](https://github.com/noir-lang/noir/issues/2753)) ([8f76fe5](https://github.com/noir-lang/noir/commit/8f76fe5819e95ed111587090e15add48a2b4e859))
* **traits:** Multi module support for traits ([#2844](https://github.com/noir-lang/noir/issues/2844)) ([4deb07f](https://github.com/noir-lang/noir/commit/4deb07f80ce110187b66a46dd5624af3b8df3dbd))


### Bug Fixes

* Fix should_fail_with ([#2940](https://github.com/noir-lang/noir/issues/2940)) ([4f07b84](https://github.com/noir-lang/noir/commit/4f07b84458dba97530d8179a3b9b19101b472616))
* Force recompilation when `output_debug` flag is set. ([#2898](https://github.com/noir-lang/noir/issues/2898)) ([9854416](https://github.com/noir-lang/noir/commit/9854416f5ac03c9da6538edc6a0a540ccccb4b61))
* Improve error message when multiplying unit values ([#2950](https://github.com/noir-lang/noir/issues/2950)) ([57b7c55](https://github.com/noir-lang/noir/commit/57b7c55e7005876dc2e070c64e1b8115ca8a4242))
* **println:** Enable printing of arrays/strings &gt;2 in fmt strings  ([#2947](https://github.com/noir-lang/noir/issues/2947)) ([309fa70](https://github.com/noir-lang/noir/commit/309fa70823535c5340f986a17f4ddddcb8723bb8))

## [0.15.0](https://github.com/noir-lang/noir/compare/v0.14.1...v0.15.0) (2023-09-28)


### ⚠ BREAKING CHANGES

* update to `bb` version 0.7.3 ([#2729](https://github.com/noir-lang/noir/issues/2729))

### Features

* Contract events in artifacts ([#2873](https://github.com/noir-lang/noir/issues/2873)) ([4765c82](https://github.com/noir-lang/noir/commit/4765c8288c583a61a81ff97eea1ef49df13eeca0))


### Bug Fixes

* Finer bit size in bound constrain ([#2869](https://github.com/noir-lang/noir/issues/2869)) ([68385e2](https://github.com/noir-lang/noir/commit/68385e294a1501b19b28f3f5510e973283ed0821))


### Miscellaneous Chores

* Update to `bb` version 0.7.3 ([#2729](https://github.com/noir-lang/noir/issues/2729)) ([fce68d1](https://github.com/noir-lang/noir/commit/fce68d1404ae66bd7a71417d791dd70545bf24f2))

## [0.14.1](https://github.com/noir-lang/noir/compare/v0.14.0...v0.14.1) (2023-09-27)


### Bug Fixes

* Remove cast for field comparisons in brillig ([#2874](https://github.com/noir-lang/noir/issues/2874)) ([1fc1fdb](https://github.com/noir-lang/noir/commit/1fc1fdb4e15d2ce625ea79d458c5346fab418e49))
* Remove duplication of code to load stdlib files ([#2868](https://github.com/noir-lang/noir/issues/2868)) ([b694aab](https://github.com/noir-lang/noir/commit/b694aab87c4665a3a89715c9d4096eeb3efb9944))

## [0.14.0](https://github.com/noir-lang/noir/compare/v0.13.0...v0.14.0) (2023-09-26)


### ⚠ BREAKING CHANGES

* **noir_js:** Rename inner and outer proof methods ([#2845](https://github.com/noir-lang/noir/issues/2845))
* `generateWitness` now returns a serialized witness file ([#2842](https://github.com/noir-lang/noir/issues/2842))
* Issue an error when a module is declared twice & fix module search path ([#2801](https://github.com/noir-lang/noir/issues/2801))
* Default integers to u64 ([#2764](https://github.com/noir-lang/noir/issues/2764))

### Features

* **abi:** Throw errors rather than returning string from `noirc_abi_wasm` ([#2817](https://github.com/noir-lang/noir/issues/2817)) ([df7b42c](https://github.com/noir-lang/noir/commit/df7b42cd253d1b908a42c367b116813f9999d93b))
* **acir:** Enable dynamic indices on non-homogenous arrays ([#2703](https://github.com/noir-lang/noir/issues/2703)) ([622d2e4](https://github.com/noir-lang/noir/commit/622d2e436992c23e6d0885b591bd1072ca57b307))
* Default integers to u64 ([#2764](https://github.com/noir-lang/noir/issues/2764)) ([01cb041](https://github.com/noir-lang/noir/commit/01cb041a92ef6043dd5a160e0a56a63400801980))
* Implement string escape sequences ([#2803](https://github.com/noir-lang/noir/issues/2803)) ([f7529b8](https://github.com/noir-lang/noir/commit/f7529b80f0958fd47a525f25a123f16438bbb892))
* Remove redundant predicate from brillig quotients ([#2784](https://github.com/noir-lang/noir/issues/2784)) ([a8f18c5](https://github.com/noir-lang/noir/commit/a8f18c55b35f47c6fa3ebfebcd827aeb55e5c850))
* **traits:** Implement trait bounds typechecker + monomorphizer passes ([#2717](https://github.com/noir-lang/noir/issues/2717)) ([5ca99b1](https://github.com/noir-lang/noir/commit/5ca99b128e9991b5272c00292208d85415e70edf))


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Conditionally run the "Create or Update PR" step in acir artifacts rebuild workflow ([#2849](https://github.com/noir-lang/noir/issues/2849)) ([63da875](https://github.com/noir-lang/noir/commit/63da875a85a2ad4ad3038443ba52eb28ea44ad10))
* Error message for assigning the wrong type is backwards [#2804](https://github.com/noir-lang/noir/issues/2804)  ([#2805](https://github.com/noir-lang/noir/issues/2805)) ([b2d62bf](https://github.com/noir-lang/noir/commit/b2d62bff3b7958b3ed62c285a7ebd45045ac2e05))
* Fix panic in some cases when calling a private function ([#2799](https://github.com/noir-lang/noir/issues/2799)) ([078d5df](https://github.com/noir-lang/noir/commit/078d5df691d4ea48e83c9530cd40b64917eba0a7))
* Fix subtract with underflow in flattening pass ([#2796](https://github.com/noir-lang/noir/issues/2796)) ([f2ed505](https://github.com/noir-lang/noir/commit/f2ed5054b0b0335dd3ecb17369b0d2e6eafb1171))
* **frontend:** Error on unsupported integer annotation ([#2778](https://github.com/noir-lang/noir/issues/2778)) ([90c3d8b](https://github.com/noir-lang/noir/commit/90c3d8baa3b7ae10bc99f6a767121f556ff75967))
* Issue an error when a module is declared twice & fix module search path ([#2801](https://github.com/noir-lang/noir/issues/2801)) ([7f76910](https://github.com/noir-lang/noir/commit/7f76910ebbd20e3d7a1db7541f2b7f43cd9b546d))
* Lack of cjs package version ([#2848](https://github.com/noir-lang/noir/issues/2848)) ([adc2d59](https://github.com/noir-lang/noir/commit/adc2d597536b52c690dceb14ea5f8e30a493452c))
* Silence unused variable warnings in stdlib ([#2795](https://github.com/noir-lang/noir/issues/2795)) ([5747bfe](https://github.com/noir-lang/noir/commit/5747bfed256f9179321ec0bd1e02f5f82723a4c7))
* Split conditional_regression tests ([#2774](https://github.com/noir-lang/noir/issues/2774)) ([8ed8832](https://github.com/noir-lang/noir/commit/8ed8832c7b475cd28ae697a09f1ad07c539736db))
* **ssa:** Do not replace previously constrained values ([#2647](https://github.com/noir-lang/noir/issues/2647)) ([d528844](https://github.com/noir-lang/noir/commit/d5288449a10d162a0340818a6beab54dd985a11a))


### Miscellaneous Chores

* `generateWitness` now returns a serialized witness file ([#2842](https://github.com/noir-lang/noir/issues/2842)) ([57d3f37](https://github.com/noir-lang/noir/commit/57d3f376d9ceadb75caf37a2bfc0e9394f76bfe6))
* **noir_js:** Rename inner and outer proof methods ([#2845](https://github.com/noir-lang/noir/issues/2845)) ([71dbbb8](https://github.com/noir-lang/noir/commit/71dbbb863a6f262da4804c17965ace627bf3a278))

## [0.13.0](https://github.com/noir-lang/noir/compare/v0.12.0...v0.13.0) (2023-09-21)


### ⚠ BREAKING CHANGES

* constrain is now a hard error ([#2758](https://github.com/noir-lang/noir/issues/2758))

### Features

* Add `pub` modifier ([#2754](https://github.com/noir-lang/noir/issues/2754)) ([dda964e](https://github.com/noir-lang/noir/commit/dda964e82e170a59c328908117677c16f691be7b))
* Add support for attributes on structs ([#2733](https://github.com/noir-lang/noir/issues/2733)) ([7b3df8e](https://github.com/noir-lang/noir/commit/7b3df8e8be11fe4288ed865951ef88566160f4af))
* Add wrapping functions in stdlib and use them in relevant test cases ([#2725](https://github.com/noir-lang/noir/issues/2725)) ([49ab121](https://github.com/noir-lang/noir/commit/49ab121ef21819e028d407999a689b92c67d8df7))
* **aztec-noir:** Abstract storage ([#2750](https://github.com/noir-lang/noir/issues/2750)) ([5481344](https://github.com/noir-lang/noir/commit/5481344feaa0403e1f6a499ff1e8e4dbbd0297aa))
* Constrain is now a hard error ([#2758](https://github.com/noir-lang/noir/issues/2758)) ([388a2b1](https://github.com/noir-lang/noir/commit/388a2b1659b2a07bde1bc376fc4669f855780858))
* Refine Noir.js API ([#2732](https://github.com/noir-lang/noir/issues/2732)) ([e79f1ed](https://github.com/noir-lang/noir/commit/e79f1ed357bf7002f14001689fb4b33e0346e679))
* Short-circuit compilation and read build artifacts from file if program is unchanged ([#2743](https://github.com/noir-lang/noir/issues/2743)) ([87fea4b](https://github.com/noir-lang/noir/commit/87fea4b447596bdd11ab461f847e03d4f1cc45f2))
* Signed arithmetic ([#2748](https://github.com/noir-lang/noir/issues/2748)) ([a84216d](https://github.com/noir-lang/noir/commit/a84216dd23513b008739ae0a749e48d0dd262a28))
* **traits:** Implement trait bounds def collector + resolver passes ([#2716](https://github.com/noir-lang/noir/issues/2716)) ([e3d18bb](https://github.com/noir-lang/noir/commit/e3d18bb9889d84fa78eecf3783bac446eac5adef))
* **traits:** Type checking for Trait impl method signatures  ([#2652](https://github.com/noir-lang/noir/issues/2652)) ([8617008](https://github.com/noir-lang/noir/commit/8617008d572c22fd9c830c233bfc0088fe0bafe4))
* Variable liveness analysis for brillig ([#2715](https://github.com/noir-lang/noir/issues/2715)) ([ddb05ab](https://github.com/noir-lang/noir/commit/ddb05ab8d30ea2b60c06f3cd7d36d5bf1b21b3ef))


### Bug Fixes

* Add error message for a contract package with no contracts ([#2762](https://github.com/noir-lang/noir/issues/2762)) ([9701a0c](https://github.com/noir-lang/noir/commit/9701a0cc2cde3b3e8fa55c3f8d09343f8861f2f8))
* Check for literal overflows in expressions ([#2742](https://github.com/noir-lang/noir/issues/2742)) ([4009f30](https://github.com/noir-lang/noir/commit/4009f30e18b17b5e7ef5af324bb9eaea5ed3780a))
* Keep the correct type for bitshift ([#2739](https://github.com/noir-lang/noir/issues/2739)) ([04fc2ea](https://github.com/noir-lang/noir/commit/04fc2ea5bc2490cdd2cb4ec90e34986fa91f43d4))
* Make `Vec::get` accept immutable `Vec`s ([#2776](https://github.com/noir-lang/noir/issues/2776)) ([f168a54](https://github.com/noir-lang/noir/commit/f168a5407b303d2e13d5975e9dc18ec13ff68c5f))
* Nightly js test ([#2740](https://github.com/noir-lang/noir/issues/2740)) ([36dcd48](https://github.com/noir-lang/noir/commit/36dcd4883313faabefe201be3645dcad79dc7970))

## [0.12.0](https://github.com/noir-lang/noir/compare/v0.11.1...v0.12.0) (2023-09-15)


### ⚠ BREAKING CHANGES

* Change `noir-lang/noir-source-resolver` to `noir-lang/source-resolver` ([#2718](https://github.com/noir-lang/noir/issues/2718))
* use american spelling of "serialize" in stdlib ([#2675](https://github.com/noir-lang/noir/issues/2675))
* Restrict packages to contain at most a single contract ([#2668](https://github.com/noir-lang/noir/issues/2668))
* use two limbs for scalar mul ([#2602](https://github.com/noir-lang/noir/issues/2602))

### Features

* Add initial version of noir.js ([#2681](https://github.com/noir-lang/noir/issues/2681)) ([e1687c9](https://github.com/noir-lang/noir/commit/e1687c9443aaa58030c38942b6aa22001e6c3e57))
* Allow methods defined in a contract to be non-entry points ([#2687](https://github.com/noir-lang/noir/issues/2687)) ([2103b2f](https://github.com/noir-lang/noir/commit/2103b2ffb640fe457b24be09b6d63fe6ee1c6ac1))
* Compile circuits and query circuit sizes in parallel for `nargo info` ([#2665](https://github.com/noir-lang/noir/issues/2665)) ([f173c05](https://github.com/noir-lang/noir/commit/f173c05cbff96dfc48a22cc2f1f76396b968d5a0))
* Compile workspace packages in parallel ([#2612](https://github.com/noir-lang/noir/issues/2612)) ([16e5e4d](https://github.com/noir-lang/noir/commit/16e5e4ddb33209a84e29dc4bea5813baba8bd5f3))
* Handle `should_fail_with` case ([#2541](https://github.com/noir-lang/noir/issues/2541)) ([291d002](https://github.com/noir-lang/noir/commit/291d0025b7d7db0a1b11fb05b72d45e8f36405da))
* **lsp:** Add nargo capabilities for test metadata ([#2532](https://github.com/noir-lang/noir/issues/2532)) ([b4ee23e](https://github.com/noir-lang/noir/commit/b4ee23e763a65323879eeda51be3a0c302b3ede6))
* **nargo:** Allow installing custom backends from the CLI ([#2632](https://github.com/noir-lang/noir/issues/2632)) ([c0c462c](https://github.com/noir-lang/noir/commit/c0c462c4b1e686816e300c504c4dee163af10805))
* **parser:** Allow multiple attributes ([#2537](https://github.com/noir-lang/noir/issues/2537)) ([7cdff2e](https://github.com/noir-lang/noir/commit/7cdff2ecbdb42c5f8ef33da6efde325ac971bbdb))
* **traits:** Add default and override of methods ([#2585](https://github.com/noir-lang/noir/issues/2585)) ([98c3ba9](https://github.com/noir-lang/noir/commit/98c3ba90907f55533f895760621f3334e75be8ff))


### Bug Fixes

* Avoid overflows in integer division ([#2180](https://github.com/noir-lang/noir/issues/2180)) ([6665210](https://github.com/noir-lang/noir/commit/66652102adee3f3318ab7a538c6f9684420f00eb))
* **aztec_noir:** Support bools as input types ([#2674](https://github.com/noir-lang/noir/issues/2674)) ([9e7a0f0](https://github.com/noir-lang/noir/commit/9e7a0f08795a4c86ab4b50f88898eabcb5462d7e))
* Failing js tests ([#2722](https://github.com/noir-lang/noir/issues/2722)) ([398b6d7](https://github.com/noir-lang/noir/commit/398b6d73a16424a1467b0d48756b4eeb8f84e408))
* Fix `update_acir` deleting all debug information ([#2643](https://github.com/noir-lang/noir/issues/2643)) ([a8a5395](https://github.com/noir-lang/noir/commit/a8a5395f357ef26890af526f417418c49b032d17))
* Fix compilation using `aztec` feature flag ([#2663](https://github.com/noir-lang/noir/issues/2663)) ([7f6fe46](https://github.com/noir-lang/noir/commit/7f6fe46f8bc00f24ff8d14b3a517e27b50db4ee5))
* Implement auto-dereferencing when calling methods ([#2581](https://github.com/noir-lang/noir/issues/2581)) ([3c731b1](https://github.com/noir-lang/noir/commit/3c731b11b31b8556eeebc4fe59b68609aa96c463))
* Initialize arrays returned by brillig ([#2048](https://github.com/noir-lang/noir/issues/2048)) ([788dfb4](https://github.com/noir-lang/noir/commit/788dfb45e025786b13035d4c3d6ccf1e1614ef2f))
* Remove duplicate file extension in stack trace ([#2655](https://github.com/noir-lang/noir/issues/2655)) ([1114871](https://github.com/noir-lang/noir/commit/1114871d538767c053d71c67577890dd29f0b490))
* **ssa:** Slice mergers with multiple ifs ([#2597](https://github.com/noir-lang/noir/issues/2597)) ([6110638](https://github.com/noir-lang/noir/commit/6110638ec743616b9a3f38650838dda631a25efd))
* Support for conditional stores ([#2553](https://github.com/noir-lang/noir/issues/2553)) ([6e6d952](https://github.com/noir-lang/noir/commit/6e6d952c052a893e897eaa42b36d3a15426a4f78))
* Use high limb in scalar multiplication ([#2619](https://github.com/noir-lang/noir/issues/2619)) ([9014b8a](https://github.com/noir-lang/noir/commit/9014b8a7cd43112e2129b6a7c5e76708e5ad37b0))
* Use two limbs for scalar mul ([#2602](https://github.com/noir-lang/noir/issues/2602)) ([d0884ca](https://github.com/noir-lang/noir/commit/d0884cae61926c2f76e27b87212b8c4bd239cbb0))
* **wasm:** Apply transformation map to circuit debug information in `noir_wasm` ([#2635](https://github.com/noir-lang/noir/issues/2635)) ([9da822f](https://github.com/noir-lang/noir/commit/9da822f59923a9953894c43afd1ddbeffa871dbf))
* **wasm:** Avoid requesting stdlib paths from the source-resolver ([#2650](https://github.com/noir-lang/noir/issues/2650)) ([aebab34](https://github.com/noir-lang/noir/commit/aebab34520b31502bb8bf0c028aa2ea8bb33142b))
* **wasm:** Remove stacker from dependencies ([#2637](https://github.com/noir-lang/noir/issues/2637)) ([36691ab](https://github.com/noir-lang/noir/commit/36691aba1be6c26216b9da518543e4a1665da56f))


### Miscellaneous Chores

* Change `noir-lang/noir-source-resolver` to `noir-lang/source-resolver` ([#2718](https://github.com/noir-lang/noir/issues/2718)) ([31e489e](https://github.com/noir-lang/noir/commit/31e489e85582de702d5798c633de9b7c4008169c))
* Restrict packages to contain at most a single contract ([#2668](https://github.com/noir-lang/noir/issues/2668)) ([dc3358b](https://github.com/noir-lang/noir/commit/dc3358b7e12ba25bedf3aa47a82b2e994a41e8c0))
* Use american spelling of "serialize" in stdlib ([#2675](https://github.com/noir-lang/noir/issues/2675)) ([56c96d0](https://github.com/noir-lang/noir/commit/56c96d06b6c18cbb59320d1d0745536ddcf2d4dd))

## [0.11.1](https://github.com/noir-lang/noir/compare/v0.11.0...v0.11.1) (2023-09-07)


### Features

* Enable dynamic indices on slices ([#2446](https://github.com/noir-lang/noir/issues/2446)) ([c5c4052](https://github.com/noir-lang/noir/commit/c5c40529d8c000ba61f3372b336e57947673646a))


### Bug Fixes

* Disable loop unrolling in brillig ([#2590](https://github.com/noir-lang/noir/issues/2590)) ([464f878](https://github.com/noir-lang/noir/commit/464f87834ada04320ea396cb4bdbab3317e036db))

## [0.11.0](https://github.com/noir-lang/noir/compare/v0.10.5...v0.11.0) (2023-09-07)


### ⚠ BREAKING CHANGES

* **stdlib:** Rename `fixed_base_scalar_mul` to be more descriptive ([#2488](https://github.com/noir-lang/noir/issues/2488))
* ACVM 0.24 ([#2504](https://github.com/noir-lang/noir/issues/2504))
* Update to `acvm-backend-barretenberg` v0.12.0 ([#2377](https://github.com/noir-lang/noir/issues/2377))
* **abi:** Replace struct name with fully qualified struct path ([#2374](https://github.com/noir-lang/noir/issues/2374))
* Remove keys from preprocessed artifacts ([#2283](https://github.com/noir-lang/noir/issues/2283))

### Features

* Add `nargo backend ls` and `nargo backend use` command to switch between backends ([#2552](https://github.com/noir-lang/noir/issues/2552)) ([7471147](https://github.com/noir-lang/noir/commit/7471147e4239410557f2f98d6e5102d8090dd09c))
* Add `noirc_abi_wasm` crate for ABI encoding in JS ([#1945](https://github.com/noir-lang/noir/issues/1945)) ([669e0da](https://github.com/noir-lang/noir/commit/669e0dab56f7368e805aaf651eb4052f476029e4))
* Add support for brillig call stacks in runtime errors ([#2549](https://github.com/noir-lang/noir/issues/2549)) ([a077391](https://github.com/noir-lang/noir/commit/a07739112ca8928d2211dd09adf89692d8b429d0))
* Apply optimizations to unconstrained code ([#2348](https://github.com/noir-lang/noir/issues/2348)) ([8e0f6c4](https://github.com/noir-lang/noir/commit/8e0f6c4e1004d50b6392941ccf72a78f3a5870da))
* **aztec_noir:** Abstract kernel return types ([#2521](https://github.com/noir-lang/noir/issues/2521)) ([2668ac2](https://github.com/noir-lang/noir/commit/2668ac2a8380ac362de34e7b8f1c231608d3606a))
* **nargo:** Add commands to install and uninstall custom backends. ([#2575](https://github.com/noir-lang/noir/issues/2575)) ([28a413c](https://github.com/noir-lang/noir/commit/28a413c5b6a92cbfdb94eca5787e7369ef03f4a3))
* **nargo:** Add hidden option to produce JSON output from `nargo info` ([#2542](https://github.com/noir-lang/noir/issues/2542)) ([14d31a5](https://github.com/noir-lang/noir/commit/14d31a543e0dd53476d35a0f32b048323f277f7c))
* Pull `Language` and `Opcode` support from backend ([#2563](https://github.com/noir-lang/noir/issues/2563)) ([2d0a5e4](https://github.com/noir-lang/noir/commit/2d0a5e447b02b11426ad80b64fba817dfce38e44))
* **ssa:** Replace values which have previously been constrained with simplified value ([#2483](https://github.com/noir-lang/noir/issues/2483)) ([9be750a](https://github.com/noir-lang/noir/commit/9be750a713485ff84b111128db62b56fc0d0c5a5))
* **stdlib:** Grumpkin scalar multiplication API ([#2586](https://github.com/noir-lang/noir/issues/2586)) ([dc34bc4](https://github.com/noir-lang/noir/commit/dc34bc46a7ee1ac7f1bcfbcdcbaccd4680a4ca31))
* Support for optional assertion messages ([#2491](https://github.com/noir-lang/noir/issues/2491)) ([5f78772](https://github.com/noir-lang/noir/commit/5f78772fefdc84b67f28fe8b671a56e280313f38))


### Bug Fixes

* Allow usage of decimal string encoding for fields larger than a `i128` ([#2547](https://github.com/noir-lang/noir/issues/2547)) ([d73f30e](https://github.com/noir-lang/noir/commit/d73f30e9ce53acd0866281f331bd2ee8ff6112bd))
* **aztec_noir:** Fix compilation of `aztec_library.rs` ([#2567](https://github.com/noir-lang/noir/issues/2567)) ([a8d0328](https://github.com/noir-lang/noir/commit/a8d03285e0c54fae525b3019dd7cc4807c6437c8))
* **aztec_noir:** Generalise loop to not always inject a hasher instance ([#2529](https://github.com/noir-lang/noir/issues/2529)) ([9fe4cfd](https://github.com/noir-lang/noir/commit/9fe4cfd05b46d1d8867bc2583a11da32480366fc))
* Black box func slice handling ([#2562](https://github.com/noir-lang/noir/issues/2562)) ([c67cd7d](https://github.com/noir-lang/noir/commit/c67cd7df9b5b47a554cc35a50f5bb80d1a4a12f0))
* Initialize structs during def collection, not name resolution ([#2528](https://github.com/noir-lang/noir/issues/2528)) ([f170529](https://github.com/noir-lang/noir/commit/f170529bfcd9044bc685ed0f49af27c2e527964b))
* Make def collector ordering more deterministic ([#2515](https://github.com/noir-lang/noir/issues/2515)) ([d49e0af](https://github.com/noir-lang/noir/commit/d49e0affa00fd29e7e5033ef464dbdd217980c8e))
* Modulo with divisor of zero should fail constraints ([#2578](https://github.com/noir-lang/noir/issues/2578)) ([fe6e2e6](https://github.com/noir-lang/noir/commit/fe6e2e6775a9b1b9fbcab96947fa6047eb80371e))


### Miscellaneous Chores

* **abi:** Replace struct name with fully qualified struct path ([#2374](https://github.com/noir-lang/noir/issues/2374)) ([0920dd0](https://github.com/noir-lang/noir/commit/0920dd03d67c50da36bfb87db2e50f6a4aa155bd))
* ACVM 0.24 ([#2504](https://github.com/noir-lang/noir/issues/2504)) ([f06fbdb](https://github.com/noir-lang/noir/commit/f06fbdb37d77b4e17d4f8eec103a93848b013963))
* Remove keys from preprocessed artifacts ([#2283](https://github.com/noir-lang/noir/issues/2283)) ([4554287](https://github.com/noir-lang/noir/commit/45542870c85ff59487ad14c25f3e1d6692623644))
* **stdlib:** Rename `fixed_base_scalar_mul` to be more descriptive ([#2488](https://github.com/noir-lang/noir/issues/2488)) ([6efc007](https://github.com/noir-lang/noir/commit/6efc007d3f53cf0ab52491e73c7bb9e2520938e0))
* Update to `acvm-backend-barretenberg` v0.12.0 ([#2377](https://github.com/noir-lang/noir/issues/2377)) ([1467275](https://github.com/noir-lang/noir/commit/1467275666a01fe1dfdaf54527440df06303eb93))

## [0.10.5](https://github.com/noir-lang/noir/compare/v0.10.4...v0.10.5) (2023-08-30)


### Features

* Basic implementation of traits ([#2368](https://github.com/noir-lang/noir/issues/2368)) ([df9f09e](https://github.com/noir-lang/noir/commit/df9f09eda62b7d09ed8ade8cad907453ea91d3e2))


### Bug Fixes

* Implement constant folding during the mem2reg pass ([#2464](https://github.com/noir-lang/noir/issues/2464)) ([5361ebd](https://github.com/noir-lang/noir/commit/5361ebd8a66648678702258bd07c9d221c748c8c))
* **ssa:** Handle right shift with constants ([#2481](https://github.com/noir-lang/noir/issues/2481)) ([13a8c87](https://github.com/noir-lang/noir/commit/13a8c878422f03c33c924ff9cb56d5fd08195357))

## [0.10.4](https://github.com/noir-lang/noir/compare/v0.10.3...v0.10.4) (2023-08-29)


### Features

* Add `assert_eq` keyword ([#2137](https://github.com/noir-lang/noir/issues/2137)) ([b467a2d](https://github.com/noir-lang/noir/commit/b467a2d72659d28195ea2015a6fba2738eae1f16))
* Add `test(should_fail)` attribute for tests that are meant to fail ([#2418](https://github.com/noir-lang/noir/issues/2418)) ([74af99d](https://github.com/noir-lang/noir/commit/74af99d7230abf453e00ef4a48a79e4f0ed17a10))
* Add syntax for specifying function type environments ([#2357](https://github.com/noir-lang/noir/issues/2357)) ([495a479](https://github.com/noir-lang/noir/commit/495a4796ff224f70fcd7408a7818d9f9e627b827))
* Add trait definition representation in DefCollector and HIR ([#2338](https://github.com/noir-lang/noir/issues/2338)) ([406a595](https://github.com/noir-lang/noir/commit/406a59564ec31c43e72229d2f97663e5223785d7))
* **attributes:** Enable custom attributes ([#2395](https://github.com/noir-lang/noir/issues/2395)) ([179611b](https://github.com/noir-lang/noir/commit/179611b646ce59a26cea6a4f3a61fc84f3ae9be3))
* **brillig:** Added locations for brillig artifacts ([#2415](https://github.com/noir-lang/noir/issues/2415)) ([3771e52](https://github.com/noir-lang/noir/commit/3771e521110da845a14058b97c5e5037daf599b0))
* Create equivalence relationships for intermediate witnesses from multiplication ([#2414](https://github.com/noir-lang/noir/issues/2414)) ([cc2a2d8](https://github.com/noir-lang/noir/commit/cc2a2d83bf6cf12406a690ca4b2f43032270ef5d))
* **frontend:** Aztec syntactic sugar (feature flagged) ([#2403](https://github.com/noir-lang/noir/issues/2403)) ([a894a6e](https://github.com/noir-lang/noir/commit/a894a6eda49d8ba565a83be75489e710cc968895))
* **nargo:** Support optional directory in git dependencies ([#2436](https://github.com/noir-lang/noir/issues/2436)) ([84fdc55](https://github.com/noir-lang/noir/commit/84fdc55a635ea6198e877621f0ca97be558bda77))
* Perform more checks for compile-time arithmetic ([#2380](https://github.com/noir-lang/noir/issues/2380)) ([1be2b1e](https://github.com/noir-lang/noir/commit/1be2b1ea702991df6ea80a8d9fbe2fb08154a3d9))
* Report compilation warnings before errors ([#2398](https://github.com/noir-lang/noir/issues/2398)) ([a1d1267](https://github.com/noir-lang/noir/commit/a1d12675a8bc75651d9634776c9d6c7cbf81ff7c))
* **ssa:** Merge slices in if statements with witness conditions ([#2347](https://github.com/noir-lang/noir/issues/2347)) ([76f7e43](https://github.com/noir-lang/noir/commit/76f7e43bde28ae60b1def6cfdede2b6e76031cc1))
* **ssa:** Reuse existing results for duplicated instructions with no side-effects ([#2460](https://github.com/noir-lang/noir/issues/2460)) ([93726c4](https://github.com/noir-lang/noir/commit/93726c4b4938512db6e36de47dc6ad77487c1acb))
* Standard library functions can now be called with closure args  ([#2471](https://github.com/noir-lang/noir/issues/2471)) ([feb8d0e](https://github.com/noir-lang/noir/commit/feb8d0e1840d2f297de53e0aaa3587ab6d7c55d6))
* Syntax for environment types now works with generics ([#2383](https://github.com/noir-lang/noir/issues/2383)) ([4609c1a](https://github.com/noir-lang/noir/commit/4609c1addc8d1a63ab8d47212c0328927483d4d0))
* Update to `acvm` 0.22.0 ([#2363](https://github.com/noir-lang/noir/issues/2363)) ([e050fab](https://github.com/noir-lang/noir/commit/e050fab89935cde96a972c2300145063687ebf5a))
* Use equivalence information from equality assertions to simplify circuit ([#2378](https://github.com/noir-lang/noir/issues/2378)) ([ec5b021](https://github.com/noir-lang/noir/commit/ec5b0216ee3889c5e926d0d1ddcb74ef983269f6))


### Bug Fixes

* **acir_gen:** Pass accurate contents to slice inputs for bb func calls ([#2435](https://github.com/noir-lang/noir/issues/2435)) ([054642b](https://github.com/noir-lang/noir/commit/054642b0daa325476bb085f5a03b55fc63a8e5fc))
* **acir:** Attach locations to MemoryOps in ACIR ([#2389](https://github.com/noir-lang/noir/issues/2389)) ([d7d7f22](https://github.com/noir-lang/noir/commit/d7d7f2273685606e8023ec90e93c48fdcb60202e))
* Closure lvalue capture bugfix ([#2457](https://github.com/noir-lang/noir/issues/2457)) ([632006a](https://github.com/noir-lang/noir/commit/632006abd2400cca9a5a7ba21380ab5e33988a6b))
* Correct off-by-one errors in lexer spans ([#2393](https://github.com/noir-lang/noir/issues/2393)) ([bbda9b0](https://github.com/noir-lang/noir/commit/bbda9b04be6c4f1ca3510f32d1abd8c2373aea54))
* Divide by zero should fail to satisfy constraints for `Field` and ints ([#2475](https://github.com/noir-lang/noir/issues/2475)) ([1b85816](https://github.com/noir-lang/noir/commit/1b85816cb1f7539917ed9212c411613f29168add))
* Implement handling of array aliasing in the mem2reg optimization pass ([#2463](https://github.com/noir-lang/noir/issues/2463)) ([7123fa9](https://github.com/noir-lang/noir/commit/7123fa9a4a55f5ea0ebdc502e8ff5eeb1a031709))
* Implement new mem2reg pass ([#2420](https://github.com/noir-lang/noir/issues/2420)) ([7714cd0](https://github.com/noir-lang/noir/commit/7714cd01858d816d67b5b1319022ef849977d0da))
* **lsp:** Remove duplicated creation of lenses ([#2433](https://github.com/noir-lang/noir/issues/2433)) ([41b568d](https://github.com/noir-lang/noir/commit/41b568d1950f45049a322e316fd9acfa52a43208))
* **parser:** Fixes for the parsing of 'where' clauses ([#2430](https://github.com/noir-lang/noir/issues/2430)) ([fa31015](https://github.com/noir-lang/noir/commit/fa31015e76e5f747a218acb4dad8af3c3b7a78ef))
* Remove duplicate `T` in `expected T, found T` error on tuple assignment ([#2360](https://github.com/noir-lang/noir/issues/2360)) ([c964ee8](https://github.com/noir-lang/noir/commit/c964ee8b54d8496b4de738395b4519d4cb36fb43))
* Run `wasm` nodejs tests with no fails ([#2387](https://github.com/noir-lang/noir/issues/2387)) ([67b6710](https://github.com/noir-lang/noir/commit/67b67100bf46d3f101538bd3552ed63e5fbf654c))
* Show types in error message in same order as in source code ([#2353](https://github.com/noir-lang/noir/issues/2353)) ([feebee4](https://github.com/noir-lang/noir/commit/feebee4cf567fa9cfd16db141851efb9a467a9cd))
* **ssa:** Codegen missing check for unary minus ([#2413](https://github.com/noir-lang/noir/issues/2413)) ([1435a86](https://github.com/noir-lang/noir/commit/1435a86b0ae315abf7553e140dd091d0161ed7b5))
* **ssa:** Do not optimize for allocates in constant folding ([#2466](https://github.com/noir-lang/noir/issues/2466)) ([9e272f3](https://github.com/noir-lang/noir/commit/9e272f39403afd61ff6a8fbe7655ac1698d9f845))
* **ssa:** Remove padding from ToRadix call with constant inputs ([#2479](https://github.com/noir-lang/noir/issues/2479)) ([37bb781](https://github.com/noir-lang/noir/commit/37bb78192521fe5a2b1ae6b053772cf0fe472102))

## [0.10.3](https://github.com/noir-lang/noir/compare/v0.10.2...v0.10.3) (2023-08-16)


### Features

* Allow calling higher-order functions with closures ([#2335](https://github.com/noir-lang/noir/issues/2335)) ([75fd3e0](https://github.com/noir-lang/noir/commit/75fd3e0e27f16fb0aa5f8b01cefe78e04f867726))
* **lsp:** Add `Compile` code lens for `main` function and contracts ([#2309](https://github.com/noir-lang/noir/issues/2309)) ([5fe69c6](https://github.com/noir-lang/noir/commit/5fe69c6eeef0b7ed2e4df9c3a80627f54c75a355))
* **lsp:** Add `Execute` code lens for `main` functions ([#2330](https://github.com/noir-lang/noir/issues/2330)) ([5aa59e0](https://github.com/noir-lang/noir/commit/5aa59e0f3c4b3e6e14330d1f0e45ec912f562892))


### Bug Fixes

* Display warning if last expression of block is unused ([#2314](https://github.com/noir-lang/noir/issues/2314)) ([8110136](https://github.com/noir-lang/noir/commit/81101362ccba787a44c6d48c0378696cb16f0acb))

## [0.10.2](https://github.com/noir-lang/noir/compare/v0.10.1...v0.10.2) (2023-08-16)


### Bug Fixes

* Prevent dead instruction elimination of brillig functions which may contain side-effects ([#2340](https://github.com/noir-lang/noir/issues/2340)) ([ba8ffd8](https://github.com/noir-lang/noir/commit/ba8ffd84c19b3516334c126bc2f25c725985baa3))

## [0.10.1](https://github.com/noir-lang/noir/compare/v0.10.0...v0.10.1) (2023-08-15)


### Features

* Add full call stacks to runtime errors ([#2310](https://github.com/noir-lang/noir/issues/2310)) ([9004181](https://github.com/noir-lang/noir/commit/900418192216dc2657d6ffe48f85ac82411fb054))
* Improved error message for unexpected return type ([#2302](https://github.com/noir-lang/noir/issues/2302)) ([d7e1e65](https://github.com/noir-lang/noir/commit/d7e1e658fe09443ae37f34e3fc6a8cb7765cf1d9))
* **ssa:** Perform dead instruction elimination on intrinsic functions ([#2276](https://github.com/noir-lang/noir/issues/2276)) ([3fe3f8c](https://github.com/noir-lang/noir/commit/3fe3f8ca11d646a054f36e6939211a22f79f10f1))
* **ssa:** Switch mem2reg pass to be per function rather than per block ([#2243](https://github.com/noir-lang/noir/issues/2243)) ([0d548b9](https://github.com/noir-lang/noir/commit/0d548b9b27710de231759c34e1a198c9991d33ef))
* **stdlib:** Implement `str` `as_bytes` and `into_bytes` function ([#2298](https://github.com/noir-lang/noir/issues/2298)) ([92549d4](https://github.com/noir-lang/noir/commit/92549d432470244ff7e8581fbe02c744c88942d9))

## [0.10.0](https://github.com/noir-lang/noir/compare/v0.9.0...v0.10.0) (2023-08-15)


### ⚠ BREAKING CHANGES

* **nargo:** Remove `-p` short flag from the `--program-dir` flag ([#2300](https://github.com/noir-lang/noir/issues/2300))
* **nargo:** Replace `--contracts` flag with `contract` package type ([#2204](https://github.com/noir-lang/noir/issues/2204))
* **nargo:** remove `flat_witness` feature flag ([#2208](https://github.com/noir-lang/noir/issues/2208))
* **nargo:** Require package `type` be specified in Nargo.toml ([#2134](https://github.com/noir-lang/noir/issues/2134))
* Allow specifying new package name with `--name` flag ([#2144](https://github.com/noir-lang/noir/issues/2144))
* **nargo:** Remove unused flags on LSP command ([#2170](https://github.com/noir-lang/noir/issues/2170))
* Support workspaces and package selection on every nargo command ([#1992](https://github.com/noir-lang/noir/issues/1992))
* **nargo:** Require package names in Nargo.toml files ([#2056](https://github.com/noir-lang/noir/issues/2056))
* Update to ACVM 0.21.0 ([#2051](https://github.com/noir-lang/noir/issues/2051))
* Drop support for the legacy SSA ([#2049](https://github.com/noir-lang/noir/issues/2049))
* **nargo:** Rename nargo gates to nargo info ([#2038](https://github.com/noir-lang/noir/issues/2038))
* **nargo:** Default to new SSA code for compilation

### Features

* **acir_gen:** RecursiveAggregation opcode and updates to black box func call generation ([#2097](https://github.com/noir-lang/noir/issues/2097)) ([5cb8166](https://github.com/noir-lang/noir/commit/5cb816664e03992a766ba9dcb2650e9596fbb291))
* Add `assert_constant` ([#2242](https://github.com/noir-lang/noir/issues/2242)) ([a72daa4](https://github.com/noir-lang/noir/commit/a72daa4764310078ab0c70720f16c85b2c0375bb))
* Add `deprecated` attribute ([#2041](https://github.com/noir-lang/noir/issues/2041)) ([9e2cf6f](https://github.com/noir-lang/noir/commit/9e2cf6f25f775d927b67c12aba1698c5635242e3))
* Add `Option&lt;T&gt;` to noir stdlib ([#1781](https://github.com/noir-lang/noir/issues/1781)) ([920a900](https://github.com/noir-lang/noir/commit/920a900818b31285c9bf2f5dd5b84c2799610a7c))
* Add basic benchmarking ([#2213](https://github.com/noir-lang/noir/issues/2213)) ([c8fe617](https://github.com/noir-lang/noir/commit/c8fe6175fa69abdfa29d7a9e1c5c653af5f15f1d))
* Add slice append ([#2241](https://github.com/noir-lang/noir/issues/2241)) ([90c5d18](https://github.com/noir-lang/noir/commit/90c5d182b578b6d512e4b5dc2c07810e6815f31e))
* Add support for bitshifts by distances known at runtime ([#2072](https://github.com/noir-lang/noir/issues/2072)) ([b0fbc53](https://github.com/noir-lang/noir/commit/b0fbc536dc432ba8d3ab6c12462758b11c2c21c4))
* Add support for slices of structs and nested slices in brillig ([#2084](https://github.com/noir-lang/noir/issues/2084)) ([620517f](https://github.com/noir-lang/noir/commit/620517f6527a7b5173dccc16e200ce349f489998))
* allow returning nested arrays from brillig ([#2047](https://github.com/noir-lang/noir/issues/2047)) ([4378bb8](https://github.com/noir-lang/noir/commit/4378bb85bf2900e7ab13856cadc764fd4a80bff5))
* Allow specifying new package name with `--name` flag ([#2144](https://github.com/noir-lang/noir/issues/2144)) ([e932599](https://github.com/noir-lang/noir/commit/e932599b1187fbf426b73c364626d1b17348a55e))
* Drop support for the legacy SSA ([#2049](https://github.com/noir-lang/noir/issues/2049)) ([3f33e44](https://github.com/noir-lang/noir/commit/3f33e447fbd6f1b94ff9935e21905c68c1dc9c83))
* Execute brillig opcodes with constant inputs at compile-time ([#2190](https://github.com/noir-lang/noir/issues/2190)) ([79af8e6](https://github.com/noir-lang/noir/commit/79af8e6fd359723716395913b23057beddcbdb83))
* Format strings for prints  ([#1952](https://github.com/noir-lang/noir/issues/1952)) ([3c82721](https://github.com/noir-lang/noir/commit/3c827217900d19a710ee8a49d782ed3d43a6336c))
* Implement traits - parser support [#2094](https://github.com/noir-lang/noir/issues/2094) ([#2230](https://github.com/noir-lang/noir/issues/2230)) ([589f173](https://github.com/noir-lang/noir/commit/589f173a85dabb68ada248e5c44fc0e13c6eb0f3))
* Implement type aliases ([#2112](https://github.com/noir-lang/noir/issues/2112)) ([ce94cb4](https://github.com/noir-lang/noir/commit/ce94cb4f9f9fccf504de9d0b12b8760fc8fab75c))
* Include struct names in ABIs ([#2266](https://github.com/noir-lang/noir/issues/2266)) ([9824ca5](https://github.com/noir-lang/noir/commit/9824ca567c6c151c0cc5469be402ffba2cbaa9cc))
* Issue warning for signed integers ([#2185](https://github.com/noir-lang/noir/issues/2185)) ([1be1bcc](https://github.com/noir-lang/noir/commit/1be1bcc5b40374c76f0b8e4d717e9292e15457f5))
* Make arrays and slices polymorphic over each other ([#2070](https://github.com/noir-lang/noir/issues/2070)) ([ef91286](https://github.com/noir-lang/noir/commit/ef91286b920fb3e17c7368839a93ccad2441edc8))
* **nargo:** Add `--exact` flag to `nargo test` ([#2272](https://github.com/noir-lang/noir/issues/2272)) ([1ad9199](https://github.com/noir-lang/noir/commit/1ad9199bcfbc5dd52166038a25ddfc7b03d90981))
* **nargo:** Add `--workspace` flag to run commands in every package ([#2313](https://github.com/noir-lang/noir/issues/2313)) ([d6deb0c](https://github.com/noir-lang/noir/commit/d6deb0c96bf8a12e21881bf10bbd139bc6531796))
* **nargo:** Add support for contracts in `nargo check` ([#2267](https://github.com/noir-lang/noir/issues/2267)) ([3d1b252](https://github.com/noir-lang/noir/commit/3d1b2522c8d9a0acebff102f9f877c44178c5db5))
* **nargo:** Default to new SSA code for compilation ([ce37718](https://github.com/noir-lang/noir/commit/ce377186a1d6afa025bd88d7436f61319c30cc33))
* **nargo:** Replace `--contracts` flag with `contract` package type ([#2204](https://github.com/noir-lang/noir/issues/2204)) ([968e12c](https://github.com/noir-lang/noir/commit/968e12c896f3fc910f36063b5afa542756ea95db))
* **nargo:** Require package `type` be specified in Nargo.toml ([#2134](https://github.com/noir-lang/noir/issues/2134)) ([1c991d0](https://github.com/noir-lang/noir/commit/1c991d0f0eac9270eb218b9ad672e36e8af74bc9))
* **nargo:** Support custom entry points specified in TOML ([#2158](https://github.com/noir-lang/noir/issues/2158)) ([effb02a](https://github.com/noir-lang/noir/commit/effb02afc78f379d023719a0d869f42e7109b05f))
* Only create new witnesses for distinctiveness when duplicates exist ([#2191](https://github.com/noir-lang/noir/issues/2191)) ([14cbdbc](https://github.com/noir-lang/noir/commit/14cbdbc1055ce7efe5d31bb02707c9a601ee7745))
* open functions are unconstrained ([be44c7b](https://github.com/noir-lang/noir/commit/be44c7be172b93ebaf74719b870fc9cc3bc24105))
* Optimize `x &lt; 0` for unsigned `x` to false ([#2206](https://github.com/noir-lang/noir/issues/2206)) ([25bc969](https://github.com/noir-lang/noir/commit/25bc9698efee601f5d8d4531a1bece8e5dc293ab))
* Optimize away constant calls to black box functions ([#1981](https://github.com/noir-lang/noir/issues/1981)) ([47b372c](https://github.com/noir-lang/noir/commit/47b372c1762ed1184bf2ed9b90d7dc3e2c161880))
* Optimize equality checks between a boolean and constant ([#2201](https://github.com/noir-lang/noir/issues/2201)) ([478c026](https://github.com/noir-lang/noir/commit/478c0266cc267b942f7ff10d32fffdeb6affa140))
* Optionally output a debug artifact on compile ([#2260](https://github.com/noir-lang/noir/issues/2260)) ([edded24](https://github.com/noir-lang/noir/commit/edded24d2256a074e8e390285e123e39f926551d))
* Perform input validation on user's package names ([#2293](https://github.com/noir-lang/noir/issues/2293)) ([87174ac](https://github.com/noir-lang/noir/commit/87174ac4927c4e237a2d0dbd6290da309e9f70c0))
* Perform sorting of constant arrays at compile time ([#2195](https://github.com/noir-lang/noir/issues/2195)) ([c46d7a0](https://github.com/noir-lang/noir/commit/c46d7a01ca49bb47548df6f3b2aa25d35aa43360))
* Remove `comptime` and warn upon usage ([#2178](https://github.com/noir-lang/noir/issues/2178)) ([98d0de3](https://github.com/noir-lang/noir/commit/98d0de3814eb228f38c2985be99095e1db564065))
* Remove an unnecessary witness in `mul_with_witness` ([#2078](https://github.com/noir-lang/noir/issues/2078)) ([9f3198e](https://github.com/noir-lang/noir/commit/9f3198efc77c308028f761175da4fe3659f70579))
* replace boolean `AND`s with multiplication ([#1954](https://github.com/noir-lang/noir/issues/1954)) ([435ab35](https://github.com/noir-lang/noir/commit/435ab3520d06b6b4f898d41a5ad403c5ddbd7771))
* **ssa:** Add additional BinaryOp simplifications ([#2124](https://github.com/noir-lang/noir/issues/2124)) ([50b2816](https://github.com/noir-lang/noir/commit/50b2816099a021e4b8cb44a9017fb849abf014e6))
* Support `contract` package type in `nargo info` command ([#2249](https://github.com/noir-lang/noir/issues/2249)) ([d309cc0](https://github.com/noir-lang/noir/commit/d309cc0086df4c2a5697269ef9618cf026d323ff))
* Support workspaces and package selection on every nargo command ([#1992](https://github.com/noir-lang/noir/issues/1992)) ([940b189](https://github.com/noir-lang/noir/commit/940b189d4fd47dad8cc9f2650162da9e99c5024c))
* Update to ACVM 0.21.0 ([#2051](https://github.com/noir-lang/noir/issues/2051)) ([ad118eb](https://github.com/noir-lang/noir/commit/ad118eb8165ef83402e25b3001dfe27cf3a358b1))


### Bug Fixes

* Add foreign impl error ([#2216](https://github.com/noir-lang/noir/issues/2216)) ([a53f5ed](https://github.com/noir-lang/noir/commit/a53f5ed86ad9a372ecad8a0367f7af3a843aae56))
* Avoid non-determinism in defunctionalization ([#2069](https://github.com/noir-lang/noir/issues/2069)) ([898a9fa](https://github.com/noir-lang/noir/commit/898a9fa3328b24334e5fac1a8ae8d43570652599))
* avoid non-determinism in defunctionalize ([898a9fa](https://github.com/noir-lang/noir/commit/898a9fa3328b24334e5fac1a8ae8d43570652599))
* avoid potential panic in `two_complement` ([#2081](https://github.com/noir-lang/noir/issues/2081)) ([63c4da0](https://github.com/noir-lang/noir/commit/63c4da0586e2575d6d14a3e537ccb64863a13f78))
* Fix 3 parser test cases in parsing ([#2284](https://github.com/noir-lang/noir/issues/2284)) ([094aef1](https://github.com/noir-lang/noir/commit/094aef191a3eafeccba714823e43d8e73ede8f50))
* fix an ICE happening when we call a closure result from if/else ([#2146](https://github.com/noir-lang/noir/issues/2146)) ([928b3ad](https://github.com/noir-lang/noir/commit/928b3ad5d93943960cc6f480b28bce25f29b3271))
* Fix an ICE when reassigning a mutable lambda variable to one with a different environment type ([#2172](https://github.com/noir-lang/noir/issues/2172)) ([a56db3e](https://github.com/noir-lang/noir/commit/a56db3ec9b20de587735e2f002be5c355c6b6b83))
* Fix assignment when both `mut` and `&mut` are used ([#2264](https://github.com/noir-lang/noir/issues/2264)) ([b07a7ff](https://github.com/noir-lang/noir/commit/b07a7ff90445afa1f173934367ffaecd0878777c))
* Fix methods not mutating fields ([#2087](https://github.com/noir-lang/noir/issues/2087)) ([6acc242](https://github.com/noir-lang/noir/commit/6acc242bae48aee7e1de013ceadb6587dc900296))
* flattening pass no longer overwrites previously mapped condition values ([#2117](https://github.com/noir-lang/noir/issues/2117)) ([f7742ab](https://github.com/noir-lang/noir/commit/f7742ab026092f129bd4ec4f122bcd3249100529))
* **globals:** Accurately filter literals for resolving globals ([#2126](https://github.com/noir-lang/noir/issues/2126)) ([1c21d0c](https://github.com/noir-lang/noir/commit/1c21d0caf1e3b3a92266b4b8238f3e6e6c394d05))
* Implement `.len()` in Acir-Gen ([#2077](https://github.com/noir-lang/noir/issues/2077)) ([ab61e3a](https://github.com/noir-lang/noir/commit/ab61e3ab70aa0f7a037e0ad4a430975f50266097))
* Implement slices of structs ([#2150](https://github.com/noir-lang/noir/issues/2150)) ([6abcb79](https://github.com/noir-lang/noir/commit/6abcb792e510454896d032cea5017bd43ef8cfc3))
* Initialize numeric generics' type to a polymorphic integer when used in an expression ([#2179](https://github.com/noir-lang/noir/issues/2179)) ([c74b228](https://github.com/noir-lang/noir/commit/c74b22850ef0a530d0a3327c2bb3a8a05bd43bbb))
* **lsp:** Ensure lsp does not crawl past the root specified ([#2322](https://github.com/noir-lang/noir/issues/2322)) ([d69e372](https://github.com/noir-lang/noir/commit/d69e3728a22a31a7d170bf383ac9e65cc4cf61cc))
* **lsp:** Improve dependency resolution in context of `Nargo.toml` ([#2226](https://github.com/noir-lang/noir/issues/2226)) ([8846bf2](https://github.com/noir-lang/noir/commit/8846bf23364b6fdcb4e79171dffedddad5df91b6))
* **lsp:** Pass `--program-dir` to test command from codelens ([#2292](https://github.com/noir-lang/noir/issues/2292)) ([92e1802](https://github.com/noir-lang/noir/commit/92e1802979e5713ec4287d8932e4675c95439861))
* Mutating a variable no longer mutates its copy ([#2057](https://github.com/noir-lang/noir/issues/2057)) ([e85e485](https://github.com/noir-lang/noir/commit/e85e4850546552b7240466031e770c2667280444))
* **nargo:** Allow `--program-dir` flag anywhere in a command ([#2290](https://github.com/noir-lang/noir/issues/2290)) ([7834fce](https://github.com/noir-lang/noir/commit/7834fcee0bda8f72d97a65964605fd82742ea75f))
* **nargo:** Indicate which TOML file is missing package name ([#2177](https://github.com/noir-lang/noir/issues/2177)) ([9529157](https://github.com/noir-lang/noir/commit/9529157bd759d1ce1f632b732d76a58417ddfb51))
* **nargo:** Make dependencies section optional in TOML ([#2161](https://github.com/noir-lang/noir/issues/2161)) ([099f4d4](https://github.com/noir-lang/noir/commit/099f4d421e86c471343693d29e77beb1fb912a33))
* **nargo:** Remove `-p` short flag from the `--program-dir` flag ([#2300](https://github.com/noir-lang/noir/issues/2300)) ([cc2af74](https://github.com/noir-lang/noir/commit/cc2af74e586bbbba0c45aa0b7c9f9a9e6480f851))
* Open contract functions are unconstrained ([#2052](https://github.com/noir-lang/noir/issues/2052)) ([be44c7b](https://github.com/noir-lang/noir/commit/be44c7be172b93ebaf74719b870fc9cc3bc24105))
* optimize contracts built by `nargo info` ([b30b3f4](https://github.com/noir-lang/noir/commit/b30b3f438e8ed6953f2fec9c610619ac4fb17553))
* Optimize contracts built by `nargo info` ([#2259](https://github.com/noir-lang/noir/issues/2259)) ([b30b3f4](https://github.com/noir-lang/noir/commit/b30b3f438e8ed6953f2fec9c610619ac4fb17553))
* Overflowing assignment will result in an error ([#2321](https://github.com/noir-lang/noir/issues/2321)) ([bc645fc](https://github.com/noir-lang/noir/commit/bc645fcebb42858984ee0e7df560e40b56438512))
* Prevent panic when passing relative paths to `--program-dir` ([#2324](https://github.com/noir-lang/noir/issues/2324)) ([9eb45da](https://github.com/noir-lang/noir/commit/9eb45dafc7bef8e1714235e95d9e703c2b8c3c3b))
* properly capture lvalues in closure environments ([#2120](https://github.com/noir-lang/noir/issues/2120)) ([#2257](https://github.com/noir-lang/noir/issues/2257)) ([ed5273c](https://github.com/noir-lang/noir/commit/ed5273c827c5556f1b92e5ed8b628a0be77870be))
* remove duplicated `name` option in `nargo new` ([#2183](https://github.com/noir-lang/noir/issues/2183)) ([68f5887](https://github.com/noir-lang/noir/commit/68f5887f9083e8194a9252d09ee0af363ffffa03))
* Remove last vestige of array of structs to struct of arrays conversion ([#2217](https://github.com/noir-lang/noir/issues/2217)) ([34be264](https://github.com/noir-lang/noir/commit/34be264c0c112e9d0139654eabe4840f35535c1e))
* Rename `Option::value` to `Option::_value` ([#2127](https://github.com/noir-lang/noir/issues/2127)) ([8a1ace7](https://github.com/noir-lang/noir/commit/8a1ace792c4550ab1ce8c6044794abdb39d02872))
* Require package names to be non-empty ([#2293](https://github.com/noir-lang/noir/issues/2293)) ([87174ac](https://github.com/noir-lang/noir/commit/87174ac4927c4e237a2d0dbd6290da309e9f70c0))
* Set location before cast instructions in SSA ([#2202](https://github.com/noir-lang/noir/issues/2202)) ([a72cc96](https://github.com/noir-lang/noir/commit/a72cc96e7535f3b85db005f2b09014488933b4df))
* simplification of overflowing integer operations ([#2153](https://github.com/noir-lang/noir/issues/2153)) ([4a5d2de](https://github.com/noir-lang/noir/commit/4a5d2de23af112b9cb794a2e86caf313f860f8d3))
* **stdlib:** correct `tecurve::contains` formula ([#1821](https://github.com/noir-lang/noir/issues/1821)) ([6a10ecf](https://github.com/noir-lang/noir/commit/6a10ecf829a5c228b1e1e8a3e9ded886e53cad48))


### Miscellaneous Chores

* **nargo:** remove `flat_witness` feature flag ([#2208](https://github.com/noir-lang/noir/issues/2208)) ([32d52d3](https://github.com/noir-lang/noir/commit/32d52d36052b954b777e918d2cd67d056dd04232))
* **nargo:** Remove unused flags on LSP command ([#2170](https://github.com/noir-lang/noir/issues/2170)) ([ccba78e](https://github.com/noir-lang/noir/commit/ccba78e330463ea9eee00f745e0b489379059bd9))
* **nargo:** Rename nargo gates to nargo info ([#2038](https://github.com/noir-lang/noir/issues/2038)) ([5907e96](https://github.com/noir-lang/noir/commit/5907e96b8dded6eb3a68d5b9e167b055f65bf783))
* **nargo:** Require package names in Nargo.toml files ([#2056](https://github.com/noir-lang/noir/issues/2056)) ([bb28223](https://github.com/noir-lang/noir/commit/bb282232aec7b0b9dae08a062b586e4564036123))

## [0.9.0](https://github.com/noir-lang/noir/compare/v0.8.0...v0.9.0) (2023-07-25)


### ⚠ BREAKING CHANGES

* ACIR bytecode encoding with Base64 ([#1935](https://github.com/noir-lang/noir/issues/1935))
* Update to ACVM 0.18.1 and implement missing brillig blackboxes ([#1914](https://github.com/noir-lang/noir/issues/1914))

### Features

* ACIR bytecode encoding with Base64 ([#1935](https://github.com/noir-lang/noir/issues/1935)) ([347cfc4](https://github.com/noir-lang/noir/commit/347cfc4ce2ed463b457fce9a2530cad4b06516e0))
* Add `nargo build` as alias for `nargo compile` ([#1940](https://github.com/noir-lang/noir/issues/1940)) ([13618d4](https://github.com/noir-lang/noir/commit/13618d4bcc89079155a9fcadc3cbe2c07d2aa972))
* add `nargo init` command ([#1859](https://github.com/noir-lang/noir/issues/1859)) ([2d87c87](https://github.com/noir-lang/noir/commit/2d87c873a286b21741736ad61fbef546b6d42b21))
* Add ability to create a proof for a workspace member using `nargo prove -p {crate_name}` ([#1930](https://github.com/noir-lang/noir/issues/1930)) ([266126f](https://github.com/noir-lang/noir/commit/266126f89935ffe9abcecac709b7b06af36a5c95))
* Add Acir debug information ([#1864](https://github.com/noir-lang/noir/issues/1864)) ([5ff8b53](https://github.com/noir-lang/noir/commit/5ff8b53bbb4720241768acfcb76e9866214d43c2))
* Add multi-line comments ([#1936](https://github.com/noir-lang/noir/issues/1936)) ([cfb1765](https://github.com/noir-lang/noir/commit/cfb176562736207e5844ac16f0f941b4ee4e12d0))
* Add support for nested arrays on brillig gen ([#2029](https://github.com/noir-lang/noir/issues/2029)) ([8adc57c](https://github.com/noir-lang/noir/commit/8adc57c77ad0012d329684781d9cbee882d0b100))
* Add to_radix and to_bits support to brillig gen ([#2012](https://github.com/noir-lang/noir/issues/2012)) ([3eef41c](https://github.com/noir-lang/noir/commit/3eef41c752fabd1d0c989084f12cd82f81a6fa4c))
* Add unit literals and unit types to parser ([#1960](https://github.com/noir-lang/noir/issues/1960)) ([ea80de5](https://github.com/noir-lang/noir/commit/ea80de57a57a92533b3fb545a0920bca2d74e109))
* Adding internal keyword ([#1873](https://github.com/noir-lang/noir/issues/1873)) ([7a85493](https://github.com/noir-lang/noir/commit/7a854937ca5a300ae05f335612d2ff72ce88b4b1))
* Allow arrays of arbitrary types in the program ABI ([#1651](https://github.com/noir-lang/noir/issues/1651)) ([811ede1](https://github.com/noir-lang/noir/commit/811ede19f2160d809904deffc09a51799448d8d6))
* Allow shadowing by default ([#2000](https://github.com/noir-lang/noir/issues/2000)) ([88a4f74](https://github.com/noir-lang/noir/commit/88a4f74a36704137d7de94e3791c2e6bea9319b5))
* avoid unnecessary witness assignments in euclidean division / bound constraint  ([#1989](https://github.com/noir-lang/noir/issues/1989)) ([c23257d](https://github.com/noir-lang/noir/commit/c23257d4bdd8d93b9219fd767de6d806e237ccea))
* **brillig_gen:** Return slices from foreign calls ([#1909](https://github.com/noir-lang/noir/issues/1909)) ([6fa3144](https://github.com/noir-lang/noir/commit/6fa3144b30ef908a350273fbfd950d5a247104b2))
* compile to brillig reachable acir fns ([#1919](https://github.com/noir-lang/noir/issues/1919)) ([2b4237d](https://github.com/noir-lang/noir/commit/2b4237d7ffc2a0246cdaa1b7d85cc1ef7d7b3eb1))
* dynamic arrays for experimental-ssa ([#1969](https://github.com/noir-lang/noir/issues/1969)) ([08d199a](https://github.com/noir-lang/noir/commit/08d199aa4daa2038ca01f5ad23376fec27950f9a))
* Implement parsing of traits ([#1886](https://github.com/noir-lang/noir/issues/1886)) ([3ba1e72](https://github.com/noir-lang/noir/commit/3ba1e72408b5f15cc623a4b2ca9f5c2e4b9652ae))
* Implement references in brillig ([#1901](https://github.com/noir-lang/noir/issues/1901)) ([3a078fb](https://github.com/noir-lang/noir/commit/3a078fb9c5c5c256a767c8bd7f1312d07c8db93c))
* initial implementation of slices in brillig ([#1932](https://github.com/noir-lang/noir/issues/1932)) ([ea47936](https://github.com/noir-lang/noir/commit/ea47936cfea201aa634432c972b71e6b89cdb513))
* Refactor Logging to use Brillig foreign calls ([#1917](https://github.com/noir-lang/noir/issues/1917)) ([c15f9aa](https://github.com/noir-lang/noir/commit/c15f9aa8a7d21ec44e9b63e90cc83290ac96cd9c))
* **stdlib:** Add multiple slice builtins ([#1888](https://github.com/noir-lang/noir/issues/1888)) ([008a16b](https://github.com/noir-lang/noir/commit/008a16b799f494115f028e523f9daa54fd8f476f))
* **stdlib:** Add secp256r1 builtin function   ([#1858](https://github.com/noir-lang/noir/issues/1858)) ([f3800c5](https://github.com/noir-lang/noir/commit/f3800c52c81a27d3b52cfe23f45e764234b1c268))
* **stdlib:** Vec type ([#1905](https://github.com/noir-lang/noir/issues/1905)) ([3734e25](https://github.com/noir-lang/noir/commit/3734e2554661567a77e7a18d91134b2d521a5c06))
* Update to ACVM 0.18.1 and implement missing brillig blackboxes ([#1914](https://github.com/noir-lang/noir/issues/1914)) ([2bc7d25](https://github.com/noir-lang/noir/commit/2bc7d25271ca8c375a54d00116f507857b4b79ae))


### Bug Fixes

* `9_conditional` end to end test ([#1951](https://github.com/noir-lang/noir/issues/1951)) ([2f6741f](https://github.com/noir-lang/noir/commit/2f6741f4f3eaa892bd970ffbb19703546c4254c4))
* `regression` end to end test ([#1965](https://github.com/noir-lang/noir/issues/1965)) ([59f92e3](https://github.com/noir-lang/noir/commit/59f92e303a7d3279af779946e216860082567de3))
* Account for missing indices in flattened witness map ([#1907](https://github.com/noir-lang/noir/issues/1907)) ([3972410](https://github.com/noir-lang/noir/commit/39724108a428015cfade4c4ef032af8941bc9a93))
* Add missing `follow_bindings` when checking if a type can be casted ([#2022](https://github.com/noir-lang/noir/issues/2022)) ([537c2bd](https://github.com/noir-lang/noir/commit/537c2bd7844dea85c9d7136b09a5d2ccd33c3108))
* Add Result to acir gen ([#1927](https://github.com/noir-lang/noir/issues/1927)) ([1f8fd51](https://github.com/noir-lang/noir/commit/1f8fd51fb28b62e05f4b0c0829d446e43e8b85cc))
* Assignment to arrays of structs ([#1998](https://github.com/noir-lang/noir/issues/1998)) ([2c3d976](https://github.com/noir-lang/noir/commit/2c3d976ded4d98529a76b7f24284209f58bc04b9))
* **brillig_gen:** Pass correct size of complex types input for brillig foreign calls ([#1922](https://github.com/noir-lang/noir/issues/1922)) ([04c89d2](https://github.com/noir-lang/noir/commit/04c89d2581f3f73073bb0ab83d37a853c638959f))
* Create `FileManager` with a root and normalize filenames against it ([#1881](https://github.com/noir-lang/noir/issues/1881)) ([50c1648](https://github.com/noir-lang/noir/commit/50c16489173f847dc466e2f82738a5e441445407))
* Differentiate stdlib `CrateId` from others ([#1895](https://github.com/noir-lang/noir/issues/1895)) ([211e251](https://github.com/noir-lang/noir/commit/211e2512861566f21c5bfec4b47eb6964211f4c0))
* Don't panic when checking if an undeclared variable is mutable ([#1987](https://github.com/noir-lang/noir/issues/1987)) ([0449518](https://github.com/noir-lang/noir/commit/0449518a430d1148b4edccb819af072cf029a83d))
* emit `Opcode`s in correct order from `GeneratedAcir::radix_le_decompose` ([#1903](https://github.com/noir-lang/noir/issues/1903)) ([e5fe839](https://github.com/noir-lang/noir/commit/e5fe839876210a208f68fd4672b4b1e86d3c0073))
* emit opcode in correct order from `GeneratedAcir::radix_le_decompose` ([e5fe839](https://github.com/noir-lang/noir/commit/e5fe839876210a208f68fd4672b4b1e86d3c0073))
* emit opcodes for sorting variables in order of execution ([c43efab](https://github.com/noir-lang/noir/commit/c43efab06065c32fa83f8b09afca9605ba82da45))
* Emit opcodes for sorting variables in order of execution ([#1941](https://github.com/noir-lang/noir/issues/1941)) ([c43efab](https://github.com/noir-lang/noir/commit/c43efab06065c32fa83f8b09afca9605ba82da45))
* Fix auto-deref operations assigning the wrong result type ([#1904](https://github.com/noir-lang/noir/issues/1904)) ([827f78c](https://github.com/noir-lang/noir/commit/827f78c8d3cd478b7917deb2fcd3c854540116cb))
* **lsp:** Avoid storing Context until recompiles are possible ([#1891](https://github.com/noir-lang/noir/issues/1891)) ([fb5f20b](https://github.com/noir-lang/noir/commit/fb5f20b1b4d911de565faebfc9baa498cd5e2128))
* method resolution when calling an `&mut` method with an `&mut` object type ([#1947](https://github.com/noir-lang/noir/issues/1947)) ([73c2e94](https://github.com/noir-lang/noir/commit/73c2e9416c2c77cc384b9cfe76b594cd1764c586))
* Only flatten main ([#1984](https://github.com/noir-lang/noir/issues/1984)) ([ac865b1](https://github.com/noir-lang/noir/commit/ac865b1b83952015d89cc8fde4702148c5eac3c6))
* Parse an if followed by a tuple as a block ([#1924](https://github.com/noir-lang/noir/issues/1924)) ([8df4f05](https://github.com/noir-lang/noir/commit/8df4f05d3ae467c74c409287ad6202c5778b073d))
* Perform `occurs` check before binding function types ([#2027](https://github.com/noir-lang/noir/issues/2027)) ([1544786](https://github.com/noir-lang/noir/commit/154478698db4192d56050e57091991ffd25da36b))
* Prevent `if` and `for` from parsing constructor expressions ([#1916](https://github.com/noir-lang/noir/issues/1916)) ([6d3029a](https://github.com/noir-lang/noir/commit/6d3029a10fdcb4f839f624e2011f32b3774dbeea))
* Rebuild tests plus script to include secp256r1 change ([#1908](https://github.com/noir-lang/noir/issues/1908)) ([ca68666](https://github.com/noir-lang/noir/commit/ca68666d959fb63dbddd449691f43305460c1a9d))
* Switch from HashMap to BTreeMap in merge_stores ([#2035](https://github.com/noir-lang/noir/issues/2035)) ([4d179e3](https://github.com/noir-lang/noir/commit/4d179e3862a63d3d924215e75e31199369c6f3e8))
* update int division optimization ([#1928](https://github.com/noir-lang/noir/issues/1928)) ([fb872c6](https://github.com/noir-lang/noir/commit/fb872c624cb04a66b99f121b4e4a478998d96271))
* Various fixes for defunctionalization & brillig gen ([#1973](https://github.com/noir-lang/noir/issues/1973)) ([f99f4bf](https://github.com/noir-lang/noir/commit/f99f4bf94053918f1baee7d985bca92d64977a3e))
* workaround for LSP dependency resolution ([#1865](https://github.com/noir-lang/noir/issues/1865)) ([a8ac338](https://github.com/noir-lang/noir/commit/a8ac338758e4afd1cd459803658f011e04666177))

## [0.8.0](https://github.com/noir-lang/noir/compare/v0.7.1...v0.8.0) (2023-07-07)


### ⚠ BREAKING CHANGES

* **ssa_refactor:** Add Slices ([#1728](https://github.com/noir-lang/noir/issues/1728))
* **nargo:** Make proving and verification keys optional ([#1880](https://github.com/noir-lang/noir/issues/1880))
* update to ACVM 0.16.0 ([#1863](https://github.com/noir-lang/noir/issues/1863))

### Features

* add signed division ([#1831](https://github.com/noir-lang/noir/issues/1831)) ([d0894ad](https://github.com/noir-lang/noir/commit/d0894ada1d292f2910ebb38858b9439066f012d8))
* allow main to be a brillig function ([#1861](https://github.com/noir-lang/noir/issues/1861)) ([1330a2a](https://github.com/noir-lang/noir/commit/1330a2aabeb227146d2ea7d2850d1e8fd05beffe))
* **brillig:** implemented blackbox functions ([#1788](https://github.com/noir-lang/noir/issues/1788)) ([f9f38de](https://github.com/noir-lang/noir/commit/f9f38ded4f0491ad56402a0820cbd355913e6361))
* **brillig:** wrap brillig fns to be top level ([1330a2a](https://github.com/noir-lang/noir/commit/1330a2aabeb227146d2ea7d2850d1e8fd05beffe))
* defunctionalization pass for ssa refactor ([#1870](https://github.com/noir-lang/noir/issues/1870)) ([1d5d84d](https://github.com/noir-lang/noir/commit/1d5d84dd6db650aa9c136d3e9746a6544cf13945))
* **driver:** Remove `Driver` struct and refactor functions to take `Context` ([#1867](https://github.com/noir-lang/noir/issues/1867)) ([8895853](https://github.com/noir-lang/noir/commit/8895853a688b8e3a9d6ffb727dc1435f5687a4b3))
* **driver:** Remove Driver struct and refactor functions to take context ([8895853](https://github.com/noir-lang/noir/commit/8895853a688b8e3a9d6ffb727dc1435f5687a4b3))
* **lsp:** Add a codelens that runs test when clicked ([#1835](https://github.com/noir-lang/noir/issues/1835)) ([5d64f8a](https://github.com/noir-lang/noir/commit/5d64f8a175ea087ed980e20041dc525eb799ba95))
* make array indexes polymophic integers ([#1877](https://github.com/noir-lang/noir/issues/1877)) ([0fc93fa](https://github.com/noir-lang/noir/commit/0fc93fa4d9b2224bef5c5a27e362e88d8996164f))
* make use of type information when serialising inputs ([#1655](https://github.com/noir-lang/noir/issues/1655)) ([310368d](https://github.com/noir-lang/noir/commit/310368d30db3b312117f988c48fca1e22fbb4c03))
* recursion working in brillig ([#1854](https://github.com/noir-lang/noir/issues/1854)) ([e55b5a8](https://github.com/noir-lang/noir/commit/e55b5a8804648511b176f8002209152b3cc8aaaa))
* **ssa refactor:** Implement first-class references ([#1849](https://github.com/noir-lang/noir/issues/1849)) ([e5773e4](https://github.com/noir-lang/noir/commit/e5773e47c212c7c8fa1a7d7456893b508cdb400c))
* **ssa_refactor:** Add Slices ([#1728](https://github.com/noir-lang/noir/issues/1728)) ([4bee979](https://github.com/noir-lang/noir/commit/4bee9794a84f386cbab8f85b9eebe76c8fe90bd0))
* update to ACVM 0.16.0 ([#1863](https://github.com/noir-lang/noir/issues/1863)) ([9c89def](https://github.com/noir-lang/noir/commit/9c89def172a36327e4b75aa510b34f8cca0f998a))


### Bug Fixes

* **crates:** do not process relative dependencies twice ([#1856](https://github.com/noir-lang/noir/issues/1856)) ([b2e71bb](https://github.com/noir-lang/noir/commit/b2e71bb64ecff7d951eb00e7fcea8e316dca9bd5))
* **lsp:** Ensure stdlib is always added before the `check_crate` phase ([#1840](https://github.com/noir-lang/noir/issues/1840)) ([cb607f5](https://github.com/noir-lang/noir/commit/cb607f5787f76856a3b9907151c3de44045bc9c7))
* **lsp:** Ensure that stdlib is always added to the driver during the check_crate phase ([cb607f5](https://github.com/noir-lang/noir/commit/cb607f5787f76856a3b9907151c3de44045bc9c7))
* Prevent comparisons from being used on `Field`s ([#1860](https://github.com/noir-lang/noir/issues/1860)) ([c8858fd](https://github.com/noir-lang/noir/commit/c8858fdaccfd205a69dd918fd262902db92516f3))
* **ssa refactor:** Add missed call to resolve ([#1817](https://github.com/noir-lang/noir/issues/1817)) ([fa9be1d](https://github.com/noir-lang/noir/commit/fa9be1d255cb10fafcf81c92bd02488f366eaf23))
* **ssa refactor:** Fix recursive call to `create_value_from_type` ([#1815](https://github.com/noir-lang/noir/issues/1815)) ([890a63b](https://github.com/noir-lang/noir/commit/890a63be4839520d1fb13ec62e21e36086ae8003))
* **ssa refactor:** Prevent stores in 'then' branch from affecting the 'else' branch ([#1827](https://github.com/noir-lang/noir/issues/1827)) ([e068fd4](https://github.com/noir-lang/noir/commit/e068fd416c0cc6db671be770c30643fe9e2b59fe))


### Miscellaneous Chores

* **nargo:** Make proving and verification keys optional ([#1880](https://github.com/noir-lang/noir/issues/1880)) ([be36c1e](https://github.com/noir-lang/noir/commit/be36c1e816e685f4882538eb3dec4b8e81f61bc2))

## [0.7.1](https://github.com/noir-lang/noir/compare/v0.7.0...v0.7.1) (2023-06-23)


### Features

* **brillig:** foreign calls with dynamic-size objects ([#1705](https://github.com/noir-lang/noir/issues/1705)) ([fe7bb99](https://github.com/noir-lang/noir/commit/fe7bb99045abdd3052614f3a25a5ad7be3bd62a3))
* update acvm to 0.15.1 ([#1764](https://github.com/noir-lang/noir/issues/1764)) ([b52f25d](https://github.com/noir-lang/noir/commit/b52f25da9ddca31fd24a9c0077821a7b31a605c7))


### Bug Fixes

* **lsp:** Ensure LSP can compile on Windows ([#1794](https://github.com/noir-lang/noir/issues/1794)) ([2992915](https://github.com/noir-lang/noir/commit/2992915da7582b0aae2198579c7f928953f3befc))
* Methods called after being passed through a generic type were not being detected ([#1785](https://github.com/noir-lang/noir/issues/1785)) ([e560cd2](https://github.com/noir-lang/noir/commit/e560cd2f56f78486d5add12bc6fce16b6b1d36f6))
* **nargo:** Update acvm-backend-barretenberg to allow wasm backend compilation ([#1771](https://github.com/noir-lang/noir/issues/1771)) ([97da745](https://github.com/noir-lang/noir/commit/97da74572b9eceac3cc819b7ebb39cd6ff632768))
* **old ssa:** fix to_be_bits ([#1765](https://github.com/noir-lang/noir/issues/1765)) ([2541fbd](https://github.com/noir-lang/noir/commit/2541fbd8c62be80caf0e1cea19cd36c5e0d1e62b))
* **ssa refactor:** ACIR gen NOT integer ([#1749](https://github.com/noir-lang/noir/issues/1749)) ([af749a0](https://github.com/noir-lang/noir/commit/af749a0941cbba567c857da964a4fa57b4626004))
* **ssa refactor:** allow simplified call inserts & fix const radix arg handling ([#1774](https://github.com/noir-lang/noir/issues/1774)) ([46facce](https://github.com/noir-lang/noir/commit/46faccefc6e60846143485d5c8320dbb4e7a937c))
* **ssa refactor:** Fix flattening pass inserting loads before stores occur ([#1783](https://github.com/noir-lang/noir/issues/1783)) ([4293b15](https://github.com/noir-lang/noir/commit/4293b15639b58eb27703acffdc034b8219391018))
* **ssa refactor:** Fix panic in acir-gen from multiplying values of different types ([#1769](https://github.com/noir-lang/noir/issues/1769)) ([1f9a132](https://github.com/noir-lang/noir/commit/1f9a132acec8a442df5c9b36976f9ee1688ecc8a))
* **ssa refactor:** function inlining orphans calls ([#1747](https://github.com/noir-lang/noir/issues/1747)) ([f30a90f](https://github.com/noir-lang/noir/commit/f30a90f4eb6c2512eab7ec7f43c9dd287e6080b2))
* **ssa refactor:** Ignore array out of bounds errors when enable_side_effects is false ([#1797](https://github.com/noir-lang/noir/issues/1797)) ([7b7682a](https://github.com/noir-lang/noir/commit/7b7682a575d01a6d798e52ad2f28dde22e60b549))
* **ssa refactor:** Implement merging of array values during flattening pass ([#1767](https://github.com/noir-lang/noir/issues/1767)) ([8f24751](https://github.com/noir-lang/noir/commit/8f24751ec4f49aa46a02d3b45f4dad1323e933d1))
* **ssa refactor:** recursion_level decrement ([#1745](https://github.com/noir-lang/noir/issues/1745)) ([e449b92](https://github.com/noir-lang/noir/commit/e449b924e0baf2f6e34b36d182da3979cd1276ee))
* **ssa refactor:** recursive branch analysis ([#1759](https://github.com/noir-lang/noir/issues/1759)) ([635b574](https://github.com/noir-lang/noir/commit/635b574b14cead36c9e45b5807921885deaa4b61))
* **ssa refactor:** Reset condition value during flattening pass ([#1811](https://github.com/noir-lang/noir/issues/1811)) ([2e330e0](https://github.com/noir-lang/noir/commit/2e330e091c1a4daab25dfb7d9bc829cbc7063ddd))
* **ssa refactor:** Speedup acir-gen ([#1793](https://github.com/noir-lang/noir/issues/1793)) ([1e75f0e](https://github.com/noir-lang/noir/commit/1e75f0e0fea48fa240abf18ab2f5c8dafb458f80))
* **ssa refactor:** Speedup find-branch-ends ([#1786](https://github.com/noir-lang/noir/issues/1786)) ([861e42c](https://github.com/noir-lang/noir/commit/861e42c3ded473522332032cb7124a82dcc2c80c))
* Update array type when processing staged memory ([#1751](https://github.com/noir-lang/noir/issues/1751)) ([27eb748](https://github.com/noir-lang/noir/commit/27eb74885d5e3bddc4a8ef07f5c2f958dee20839))

## [0.7.0](https://github.com/noir-lang/noir/compare/root-v0.6.0...root-v0.7.0) (2023-06-19)


### ⚠ BREAKING CHANGES

* Update to acvm 0.14.0 ([#1594](https://github.com/noir-lang/noir/issues/1594))
* update to ACVM 0.13.0 ([#1393](https://github.com/noir-lang/noir/issues/1393))
* **stdlib:** remove unnecessary merkle functions from stdlib ([#1424](https://github.com/noir-lang/noir/issues/1424))
* **stdlib:** return update verification functions to return `bool`
* **stdlib:** update stdlib functions to return `bool` where appropriate ([#1409](https://github.com/noir-lang/noir/issues/1409))
* Change serialization of struct field order to match the user defined order ([#1166](https://github.com/noir-lang/noir/issues/1166))
* Update to ACVM 0.12.0 ([#1339](https://github.com/noir-lang/noir/issues/1339))
* remove concept of noir fallbacks for foreign functions ([#1371](https://github.com/noir-lang/noir/issues/1371))
* **nargo:** retire print-acir in favour of flag ([#1328](https://github.com/noir-lang/noir/issues/1328))

### Features

* Allow warnings by default ([#1383](https://github.com/noir-lang/noir/issues/1383)) ([e7a0d5c](https://github.com/noir-lang/noir/commit/e7a0d5c7b3b86587861401533d4e6784d0353404))
* **ci:** update noir to build wasm with a nix flake file ([#1208](https://github.com/noir-lang/noir/issues/1208)) ([2209369](https://github.com/noir-lang/noir/commit/22093699a1a9c0c654c57fcce683fb42808db3e4))
* **lsp:** Publish diagnostics on file save ([#1676](https://github.com/noir-lang/noir/issues/1676)) ([c53bfc8](https://github.com/noir-lang/noir/commit/c53bfc8c2207b64ac7e4a8d732dc4bc431b1990e))
* Make for-loop range be a polymorphic integer instead of just Field in unconstrained functions ([#1583](https://github.com/noir-lang/noir/issues/1583)) ([77fba56](https://github.com/noir-lang/noir/commit/77fba5677f9d1466d9d08c2eddc57149f9010db4))
* multiple item imports in use statement ([#1466](https://github.com/noir-lang/noir/issues/1466)) ([1dcd2ee](https://github.com/noir-lang/noir/commit/1dcd2ee9dd42c7867d9abcd528b763dd0a05bdd3))
* **nargo:** Add `lsp` command to start server that reports no capabilities ([#1560](https://github.com/noir-lang/noir/issues/1560)) ([e28529d](https://github.com/noir-lang/noir/commit/e28529d49f82300292e2b4d564f42a1c6bcaab59))
* **nargo:** Allow user-specified file for prover inputs instead of `Prover.toml` ([#1531](https://github.com/noir-lang/noir/issues/1531)) ([91cbec6](https://github.com/noir-lang/noir/commit/91cbec6cff1dabf6fd73a0eeff84006c2aa14080))
* **nargo:** retire print-acir in favour of flag ([#1328](https://github.com/noir-lang/noir/issues/1328)) ([dffa3c5](https://github.com/noir-lang/noir/commit/dffa3c50337ec0f71a62377d985ebdc8eefe490e))
* pass in closure to `Driver` to signal backend opcode support ([#1349](https://github.com/noir-lang/noir/issues/1349)) ([1e958c2](https://github.com/noir-lang/noir/commit/1e958c2aef89328e5354457c2a1e8697486e2978))
* remove concept of noir fallbacks for foreign functions ([#1371](https://github.com/noir-lang/noir/issues/1371)) ([dbec6f2](https://github.com/noir-lang/noir/commit/dbec6f284e17c7d656d8ffcf9534bd370eee9756))
* **ssa refactor:** mem2reg opt pass ([#1363](https://github.com/noir-lang/noir/issues/1363)) ([5d1efd5](https://github.com/noir-lang/noir/commit/5d1efd51dc3cc762ae8b75032bc71705845f30ff))
* **stdlib:** EdDSA sig verification ([#1313](https://github.com/noir-lang/noir/issues/1313)) ([04a15e0](https://github.com/noir-lang/noir/commit/04a15e00331077410a74c91934e7eb64aa165d9e))
* **stdlib:** return update verification functions to return `bool` ([2b2be1e](https://github.com/noir-lang/noir/commit/2b2be1e7fbfbfcb00cfd15587cbc9df083b91055))
* **stdlib:** update stdlib functions to return `bool` where appropriate ([#1409](https://github.com/noir-lang/noir/issues/1409)) ([2b2be1e](https://github.com/noir-lang/noir/commit/2b2be1e7fbfbfcb00cfd15587cbc9df083b91055))
* Update to acvm 0.14.0 ([#1594](https://github.com/noir-lang/noir/issues/1594)) ([f2d6b7b](https://github.com/noir-lang/noir/commit/f2d6b7bd8c909cbe85c8b5ff760ac2a4607ab56e))
* update to ACVM 0.15.0 ([#1616](https://github.com/noir-lang/noir/issues/1616)) ([3109239](https://github.com/noir-lang/noir/commit/3109239f2c0a7ad4767a3cd1bcc4436a367a8860))
* use RAM/ROM opcode when supported by the backend ([#1282](https://github.com/noir-lang/noir/issues/1282)) ([242f07b](https://github.com/noir-lang/noir/commit/242f07b513c0f7141c0c661e6c7913db04eeccef))


### Bug Fixes

* Change serialization of struct field order to match the user defined order ([#1166](https://github.com/noir-lang/noir/issues/1166)) ([809aa3a](https://github.com/noir-lang/noir/commit/809aa3a071ab3eb5143747f5ee8e03597afe7719))
* Fix modulo operator for comptime values ([#1361](https://github.com/noir-lang/noir/issues/1361)) ([ba15d6d](https://github.com/noir-lang/noir/commit/ba15d6d654739cc710e147dc08d94dcfe9dedb2a))
* Fix nargo not showing compiler errors or warnings ([#1694](https://github.com/noir-lang/noir/issues/1694)) ([4233068](https://github.com/noir-lang/noir/commit/4233068e790e6b2544b61571183fdfe8dbaa7c57))
* **frontend:** Avoid panic if dependency cannot be resolved ([#1719](https://github.com/noir-lang/noir/issues/1719)) ([f35b346](https://github.com/noir-lang/noir/commit/f35b3468ee0fe928b472a47a13b2dd0dcf37bb46))
* **nargo:** prevent -p arg clash ([#1605](https://github.com/noir-lang/noir/issues/1605)) ([4867f4e](https://github.com/noir-lang/noir/commit/4867f4ec9d00160640a7665cf64c65bd6982cf77))
* **noirc_driver:** Move error printing into nargo ([#1598](https://github.com/noir-lang/noir/issues/1598)) ([561cd63](https://github.com/noir-lang/noir/commit/561cd63debc24d96fa95d3eced72d8b2f8122f49))
* **ssa refactor:** Add missing calls to resolve in Instruction::simplify ([#1678](https://github.com/noir-lang/noir/issues/1678)) ([07b07d0](https://github.com/noir-lang/noir/commit/07b07d06cf8d73a85f2bde64d10cfbc677d9d3b1))
* **ssa refactor:** BigUint for radix ([#1715](https://github.com/noir-lang/noir/issues/1715)) ([00cf462](https://github.com/noir-lang/noir/commit/00cf462bbe277d7658ca9c6824165b7f3c26514e))
* **ssa refactor:** Change the result of simplifying Eq and Lt to bool ([#1672](https://github.com/noir-lang/noir/issues/1672)) ([1d48929](https://github.com/noir-lang/noir/commit/1d48929ecf20b1d2f9ab07ecf233c2565679ecec))
* **ssa refactor:** Do not remove enable_side_effects instructions in die pass ([#1673](https://github.com/noir-lang/noir/issues/1673)) ([cbee4c0](https://github.com/noir-lang/noir/commit/cbee4c0ad3606b2607fb4fdc88d1caa90a7c3462))
* **ssa refactor:** euclidean division for unsigned ([#1721](https://github.com/noir-lang/noir/issues/1721)) ([a1596bc](https://github.com/noir-lang/noir/commit/a1596bca1794af822a7804c22789ea6598f11edb))
* **ssa refactor:** filter unreachable blocks from cfg ([#1523](https://github.com/noir-lang/noir/issues/1523)) ([202c345](https://github.com/noir-lang/noir/commit/202c34548515bbc542c28a1225882590cfa086eb))
* **ssa refactor:** fix array element propagation through constant folding and DIE ([#1674](https://github.com/noir-lang/noir/issues/1674)) ([301e244](https://github.com/noir-lang/noir/commit/301e24476975a74d21270df5957c7b27f08706aa))
* **ssa refactor:** Fix array elements not being mapped to new values ([#1717](https://github.com/noir-lang/noir/issues/1717)) ([4ebcbeb](https://github.com/noir-lang/noir/commit/4ebcbeba166fb19f277c1a3508e618f989fa75b0)), closes [#1688](https://github.com/noir-lang/noir/issues/1688)
* **ssa refactor:** fix bad constant type caching ([#1593](https://github.com/noir-lang/noir/issues/1593)) ([37c0be6](https://github.com/noir-lang/noir/commit/37c0be65f0a06e6535169193547ed9b9bceb1ff9))
* **ssa refactor:** Fix constant folding looping forever ([#1611](https://github.com/noir-lang/noir/issues/1611)) ([afe58cc](https://github.com/noir-lang/noir/commit/afe58ccc8b80aecadb8c5ab8564d5e87f5d1094c))
* **ssa refactor:** Fix failed_to_inline_a_function being set for intrinsics ([#1675](https://github.com/noir-lang/noir/issues/1675)) ([377ac5c](https://github.com/noir-lang/noir/commit/377ac5c2d2faf38078f682f8428f0de165a7ca68))
* **ssa refactor:** Fix flatten_cfg for ifs with no else ([#1671](https://github.com/noir-lang/noir/issues/1671)) ([7ce8cce](https://github.com/noir-lang/noir/commit/7ce8cce6da5f668ac00dbbdeefda5b3b0f61815f))
* **ssa refactor:** Fix mem2reg pass not always removing unused stores ([#1677](https://github.com/noir-lang/noir/issues/1677)) ([8310544](https://github.com/noir-lang/noir/commit/8310544382d11fcf46a87b6e51f1a7d5f4cbbefc))
* **ssa refactor:** Fix ssa-gen of nested ifs ([#1406](https://github.com/noir-lang/noir/issues/1406)) ([5fd976e](https://github.com/noir-lang/noir/commit/5fd976e03e8034e521840621035c99ea840b13ba))
* **ssa refactor:** Fix stack overflow during loop unrolling ([#1666](https://github.com/noir-lang/noir/issues/1666)) ([c7a7216](https://github.com/noir-lang/noir/commit/c7a7216c9b01bf89aa4493330a71e825378f631e))
* **ssa refactor:** Implement array equality in SSA-gen ([#1704](https://github.com/noir-lang/noir/issues/1704)) ([0d31d83](https://github.com/noir-lang/noir/commit/0d31d831e29016c892bfb21ccc71159591b02519))
* **ssa refactor:** more comprehensive instruction simplification ([#1735](https://github.com/noir-lang/noir/issues/1735)) ([97d6747](https://github.com/noir-lang/noir/commit/97d674728e6c6174d97b096077e13940c20c2eee))
* **ssa refactor:** pad radix result ([#1730](https://github.com/noir-lang/noir/issues/1730)) ([8e9b612](https://github.com/noir-lang/noir/commit/8e9b6122532079ecf71aafe448265797828b69cf))
* **ssa refactor:** resolve replaced value ids for printing ([#1535](https://github.com/noir-lang/noir/issues/1535)) ([08ca847](https://github.com/noir-lang/noir/commit/08ca847d764fdd0eff357d199d0a9d9eac44e5de))
* **ssa refactor:** safe to query cfg for single block programs ([#1401](https://github.com/noir-lang/noir/issues/1401)) ([e2a23b3](https://github.com/noir-lang/noir/commit/e2a23b3d933824f09d8a8f0e2535531e6dcf76cf))
* **ssa refactor:** schnorr signature handling ([#1727](https://github.com/noir-lang/noir/issues/1727)) ([98ecf93](https://github.com/noir-lang/noir/commit/98ecf9315431afa67e7239fed6b3f4da9cced294))
* **ssa refactor:** Translate strings as arrays of characters ([#1669](https://github.com/noir-lang/noir/issues/1669)) ([2ba2ef6](https://github.com/noir-lang/noir/commit/2ba2ef632cd6ffdd9f162e87108b19833973450f))
* **ssa refactor:** truncate when simplifying constant casts ([#1714](https://github.com/noir-lang/noir/issues/1714)) ([a2108d7](https://github.com/noir-lang/noir/commit/a2108d7931bbd65a32aa56c2f5c36900cf706fd9))
* **ssa:** conditionalise array indexes under IF statements ([#1395](https://github.com/noir-lang/noir/issues/1395)) ([ddca3b4](https://github.com/noir-lang/noir/commit/ddca3b4fd1902275f7094251bba88c3eba4d3854))
* **stdlib:** Workaround for Field comparison error in EdDSA signature verification ([#1372](https://github.com/noir-lang/noir/issues/1372)) ([e790c9f](https://github.com/noir-lang/noir/commit/e790c9f5da784f7617a0b578623b470af7e01116))


### Miscellaneous Chores

* **stdlib:** remove unnecessary merkle functions from stdlib ([#1424](https://github.com/noir-lang/noir/issues/1424)) ([50fcb3c](https://github.com/noir-lang/noir/commit/50fcb3cded8cf37403a2dc3839bf99b7df4261b5))
* Update to ACVM 0.12.0 ([#1339](https://github.com/noir-lang/noir/issues/1339)) ([b938c7e](https://github.com/noir-lang/noir/commit/b938c7eeaa5ee493b28cad5451e7d5b7921ad934))
* update to ACVM 0.13.0 ([#1393](https://github.com/noir-lang/noir/issues/1393)) ([22dee75](https://github.com/noir-lang/noir/commit/22dee75464d3d02af17109d9065d37342fbbcddb))

## [0.6.0](https://github.com/noir-lang/noir/compare/v0.5.1...v0.6.0) (2023-05-16)


### ⚠ BREAKING CHANGES

* Update to acvm 0.11.0 ([#1322](https://github.com/noir-lang/noir/issues/1322))
* **parser:** deprecate `constrain` keyword for `assert` ([#1286](https://github.com/noir-lang/noir/issues/1286))

### Features

* Enable `to_radix` for any field element ([#1343](https://github.com/noir-lang/noir/issues/1343)) ([c3bdec2](https://github.com/noir-lang/noir/commit/c3bdec294234e92a73f39720ec7202fbb17ddc79))
* Enable dynamic arrays ([#1271](https://github.com/noir-lang/noir/issues/1271)) ([9f43450](https://github.com/noir-lang/noir/commit/9f434507fa431a9dbf4130374b866a5de6176d76))
* Issue an error when attempting to use a `return` expression ([#1330](https://github.com/noir-lang/noir/issues/1330)) ([a6de557](https://github.com/noir-lang/noir/commit/a6de557e83eb6318d091e40553bb3e2b3823fdc5))
* **nargo:** Remove usage of `CompiledProgram` in CLI code and use separate ABI/bytecode ([#1269](https://github.com/noir-lang/noir/issues/1269)) ([f144391](https://github.com/noir-lang/noir/commit/f144391b4295b127f3f422e862a087a90dac1dbf))
* **ssa refactor:** experimental-ssa compiler flag ([#1289](https://github.com/noir-lang/noir/issues/1289)) ([afa6749](https://github.com/noir-lang/noir/commit/afa67494c564b68b667535f2d8ef234fbc4bec12))
* **ssa refactor:** Implement dominator tree ([#1278](https://github.com/noir-lang/noir/issues/1278)) ([144ebf5](https://github.com/noir-lang/noir/commit/144ebf51522fb19847be28de5595247051fcd92e))
* **ssa:** add block opcode ([#1291](https://github.com/noir-lang/noir/issues/1291)) ([951ad71](https://github.com/noir-lang/noir/commit/951ad71e0f8bc9a6e95ae21197854396ed7f6e78))
* **stdlib:** add keccak256 foreign function ([#1249](https://github.com/noir-lang/noir/issues/1249)) ([260d87d](https://github.com/noir-lang/noir/commit/260d87d1ef86069a1fcf0f9b4969589273e381d1))


### Bug Fixes

* Fix issue with parsing nested generics ([#1319](https://github.com/noir-lang/noir/issues/1319)) ([36f5b8e](https://github.com/noir-lang/noir/commit/36f5b8e88fe8048ece1a54755789d56c8803b3ab))
* Fix parser error preventing assignments to tuple fields ([#1318](https://github.com/noir-lang/noir/issues/1318)) ([460568e](https://github.com/noir-lang/noir/commit/460568e50a810f90db6559195492547095ab8c32))
* Fix struct or tuple field assignment failing with generics ([#1317](https://github.com/noir-lang/noir/issues/1317)) ([d872890](https://github.com/noir-lang/noir/commit/d872890e408ada056e9aab84a7774dcaa2049324)), closes [#1315](https://github.com/noir-lang/noir/issues/1315)
* **stdlib:** support use of `to_bits` and `to_radix` for values &gt;128 bits ([#1312](https://github.com/noir-lang/noir/issues/1312)) ([12f3e7e](https://github.com/noir-lang/noir/commit/12f3e7e5917fdcb6b8648032772a7541eaef4751))


### Miscellaneous Chores

* **parser:** deprecate `constrain` keyword for `assert` ([#1286](https://github.com/noir-lang/noir/issues/1286)) ([9740f54](https://github.com/noir-lang/noir/commit/9740f54c28f30ea9367897fa986d8aea1aba79f2))
* Update to acvm 0.11.0 ([#1322](https://github.com/noir-lang/noir/issues/1322)) ([da47368](https://github.com/noir-lang/noir/commit/da473685524fc6e5e17f9c3eb95116378ac41fb8))

## [0.5.1](https://github.com/noir-lang/noir/compare/v0.5.0...v0.5.1) (2023-05-01)


### Bug Fixes

* Add Poseidon examples into integration tests ([#1257](https://github.com/noir-lang/noir/issues/1257)) ([2a5aa52](https://github.com/noir-lang/noir/commit/2a5aa52435294ddeda5b4506c3117cbd164ca2ff))
* fix `linear_eval is no 0` serialisation issue ([#1226](https://github.com/noir-lang/noir/issues/1226)) ([41d96ae](https://github.com/noir-lang/noir/commit/41d96ae9bbb9ce7010451cae5dc1f66d5e57d45b))

## [0.5.0](https://github.com/noir-lang/noir/compare/v0.4.1...v0.5.0) (2023-04-28)


### ⚠ BREAKING CHANGES

* Switch to aztec_backend that uses upstream BB & UltraPlonk ([#1114](https://github.com/noir-lang/noir/issues/1114))

### Features

* **noir:** added `distinct` keyword ([#1219](https://github.com/noir-lang/noir/issues/1219)) ([3a65f30](https://github.com/noir-lang/noir/commit/3a65f304c25e8239f9735ce1e6dee29d7eecc244))
* **noir:** added assert keyword ([#1227](https://github.com/noir-lang/noir/issues/1227)) ([0dc2cac](https://github.com/noir-lang/noir/commit/0dc2cac5bc26d277a0e6377fd774e0ec9c8d3531))
* Switch to aztec_backend that uses upstream BB & UltraPlonk ([#1114](https://github.com/noir-lang/noir/issues/1114)) ([f14fe0b](https://github.com/noir-lang/noir/commit/f14fe0b97e75eb5be39a48675149cf08d718abf6))


### Bug Fixes

* **wasm:** add std after dependencies ([#1245](https://github.com/noir-lang/noir/issues/1245)) ([55ef8a2](https://github.com/noir-lang/noir/commit/55ef8a2d3246a5edbf11a605c092b09151b120e6))

## [0.4.1](https://github.com/noir-lang/noir/compare/v0.4.0...v0.4.1) (2023-04-20)


### Features

* Add Poseidon-BN254 hash functions ([#1176](https://github.com/noir-lang/noir/issues/1176)) ([33feb2b](https://github.com/noir-lang/noir/commit/33feb2bcd71b1040d70d1f51a7377594db557c19))
* bump noir-source-resolver version ([#1182](https://github.com/noir-lang/noir/issues/1182)) ([750ed77](https://github.com/noir-lang/noir/commit/750ed7793f5a07bc361b56c66f041cb4097219e3))


### Bug Fixes

* Add checks for nop ([#1160](https://github.com/noir-lang/noir/issues/1160)) ([809b85f](https://github.com/noir-lang/noir/commit/809b85f751bd0e27ce8c4b38354bc051471d8522))
* allow comptime or non comptime fields in unconstrained for loops ([#1172](https://github.com/noir-lang/noir/issues/1172)) ([73df465](https://github.com/noir-lang/noir/commit/73df4653556a7d1c74d184e27ec5a8ca3be47af9))
* maintain ordering of return value witnesses when constructing ABI ([#1177](https://github.com/noir-lang/noir/issues/1177)) ([b799c8a](https://github.com/noir-lang/noir/commit/b799c8aa4491f4f17e248a50a154386803b6d712))
* **nargo:** restore `nargo codegen-verifier` functionality ([#1185](https://github.com/noir-lang/noir/issues/1185)) ([528a2a4](https://github.com/noir-lang/noir/commit/528a2a441cfe094885cc8f26ffba865f3a0b5c0c))
* **ssa:** set correct predecessors of IF join ([#1171](https://github.com/noir-lang/noir/issues/1171)) ([7628ed6](https://github.com/noir-lang/noir/commit/7628ed6aa0e430881bd5628c84342058fa0e2f78))

## [0.4.0](https://github.com/noir-lang/noir/compare/v0.3.2...v0.4.0) (2023-04-17)


### ⚠ BREAKING CHANGES

* remove outdated arkworks backend ([#1151](https://github.com/noir-lang/noir/issues/1151))
* **nargo:** define preprocessed artifacts for programs/contracts ([#1126](https://github.com/noir-lang/noir/issues/1126))
* **nargo:** use faster hash function for checking preprocessed keys ([#1094](https://github.com/noir-lang/noir/issues/1094))
* Fix returning of structs in ACIR ([#1058](https://github.com/noir-lang/noir/issues/1058))
* upgrade to acvm 0.8.0 ([#1047](https://github.com/noir-lang/noir/issues/1047))

### Features

* Add new `Vec` type to frontend ([#1103](https://github.com/noir-lang/noir/issues/1103)) ([e125157](https://github.com/noir-lang/noir/commit/e12515778913164a0a9673c3f0eb98b3c5b73a7b))
* Add storage slots to globals ([#1019](https://github.com/noir-lang/noir/issues/1019)) ([4190e11](https://github.com/noir-lang/noir/commit/4190e11732ae0757ac84d6dcdab78ade62a7cfe8))
* Allow arbitrary noir functions to be unconstrained ([#1044](https://github.com/noir-lang/noir/issues/1044)) ([ebc8a36](https://github.com/noir-lang/noir/commit/ebc8a36ebdf8b723baf9b5941ec2fa136ad0d2a1))
* Allow non-comptime field indices in unconstrained functions ([#1053](https://github.com/noir-lang/noir/issues/1053)) ([bc52612](https://github.com/noir-lang/noir/commit/bc5261230310fca5c84a27258935761d9836c912))
* Allow numeric generics to be referenced and add `map` ([#997](https://github.com/noir-lang/noir/issues/997)) ([34eab32](https://github.com/noir-lang/noir/commit/34eab32465ea195d53de29560e363303a36c73f6))
* Allow secret functions to use public parameters ([#1051](https://github.com/noir-lang/noir/issues/1051)) ([12c0668](https://github.com/noir-lang/noir/commit/12c0668421addb9c0718d60efdcbfe79311fb718))
* Allow structs and arrays as globals ([#1054](https://github.com/noir-lang/noir/issues/1054)) ([dadbd3c](https://github.com/noir-lang/noir/commit/dadbd3c033bd5e279e84f99bb579f91aff8b8213))
* Changes serialization for contract functions ([#1056](https://github.com/noir-lang/noir/issues/1056)) ([41e0020](https://github.com/noir-lang/noir/commit/41e00207b0eeae4d0285c617acac72c780cb0900))
* **compiler:** Allows specify entry_point source ([#1026](https://github.com/noir-lang/noir/issues/1026)) ([9789f89](https://github.com/noir-lang/noir/commit/9789f890fe9bfc014ba7a6b044c268c5dd40a658))
* dynamic array indexing ([#886](https://github.com/noir-lang/noir/issues/886)) ([aba1ed2](https://github.com/noir-lang/noir/commit/aba1ed229472f2cbb8677b08d54af629382514f3))
* Implement 'open' and 'unconstrained' keywords ([#1037](https://github.com/noir-lang/noir/issues/1037)) ([5a66dec](https://github.com/noir-lang/noir/commit/5a66dece860044dd23e287dae47070086a51018b))
* Implement `std::unsafe::zeroed` ([#1048](https://github.com/noir-lang/noir/issues/1048)) ([9a43f85](https://github.com/noir-lang/noir/commit/9a43f85a055f23e5746e6836fe11990f4c87bbdc))
* Implement arrays of structs ([#1068](https://github.com/noir-lang/noir/issues/1068)) ([f607150](https://github.com/noir-lang/noir/commit/f607150f34d5570ff2d86dddba2074f2c8c29b7e))
* import core logic in cli from `nargo` crate ([#1142](https://github.com/noir-lang/noir/issues/1142)) ([753a272](https://github.com/noir-lang/noir/commit/753a272cbdf32858e47d2fa4bd6c236521bbb2cf))
* make `noirc_driver` aware of contracts ([#999](https://github.com/noir-lang/noir/issues/999)) ([c21afca](https://github.com/noir-lang/noir/commit/c21afcaba738ad438cef6bd100a9eb25e7557bf3))
* Merge all contracts into one ABI ([#1033](https://github.com/noir-lang/noir/issues/1033)) ([473428c](https://github.com/noir-lang/noir/commit/473428cfc3109f4c03e6cff7b76f995daa6ef4fa))
* **nargo:** add `InvalidPackageError` and `DependencyResolutionError` error types. ([#1007](https://github.com/noir-lang/noir/issues/1007)) ([1e6761b](https://github.com/noir-lang/noir/commit/1e6761b490a38afe29a9eca085b1a806d8fdf59e))
* **nargo:** add skeleton of composite types in template input tomls ([#1104](https://github.com/noir-lang/noir/issues/1104)) ([1fb2756](https://github.com/noir-lang/noir/commit/1fb27566ca85fb3c5912308b99edb7a379a8b792))
* **nargo:** add test to example noir program ([#1039](https://github.com/noir-lang/noir/issues/1039)) ([f994c4f](https://github.com/noir-lang/noir/commit/f994c4f4813ba496f6a958a952691b650bf052e6))
* **nargo:** allow running `nargo` from any directory in package ([#1010](https://github.com/noir-lang/noir/issues/1010)) ([761fdb5](https://github.com/noir-lang/noir/commit/761fdb5ab96a2259883eb5b42157df466b05175d))
* **nargo:** define preprocessed artifacts for programs/contracts ([#1126](https://github.com/noir-lang/noir/issues/1126)) ([7528f59](https://github.com/noir-lang/noir/commit/7528f59d10dba5a56b9fa7cf979fdc93cacacb9b))
* **nargo:** print-acir command ([#1031](https://github.com/noir-lang/noir/issues/1031)) ([408d9c0](https://github.com/noir-lang/noir/commit/408d9c04e3a2fb10a54faee97d3e788f75a07cda))
* **nargo:** remove misleading quotes in generated `Prover.toml` ([#1087](https://github.com/noir-lang/noir/issues/1087)) ([57c817f](https://github.com/noir-lang/noir/commit/57c817fafe494c3d6a9cd56c7e266dad754b5c5b))
* **nargo:** split `nargo` into core and cli packages ([#1065](https://github.com/noir-lang/noir/issues/1065)) ([7c388f9](https://github.com/noir-lang/noir/commit/7c388f9103a96f4b2073def1bb1af7d18744f274))
* read-only array ([#899](https://github.com/noir-lang/noir/issues/899)) ([2e38ab0](https://github.com/noir-lang/noir/commit/2e38ab08c12b732331bb4dde18815dbb5c9e1398))
* **stdlib:** Implement Poseidon hash ([#768](https://github.com/noir-lang/noir/issues/768)) ([779ab66](https://github.com/noir-lang/noir/commit/779ab66413ad33a71ed9ca180ca1e5bd8ba3f285))


### Bug Fixes

* Avoid asserting in typechecker if struct field count is not correct ([#1036](https://github.com/noir-lang/noir/issues/1036)) ([b3d1d7f](https://github.com/noir-lang/noir/commit/b3d1d7fc6f30f30e6ec0effc547713a8de7a5486)), closes [#1028](https://github.com/noir-lang/noir/issues/1028)
* compiler identifying imported functions as being part of a contract ([#1112](https://github.com/noir-lang/noir/issues/1112)) ([61c38d2](https://github.com/noir-lang/noir/commit/61c38d2fd946697296905f267c49d18609835fcb))
* correct name in CLI output from `nargo_cli` to `nargo` ([74d7369](https://github.com/noir-lang/noir/commit/74d73696bdd042878cdfb06c8a781d575efc97fb))
* correct test for mutually exclusive feature flags ([#1085](https://github.com/noir-lang/noir/issues/1085)) ([eb5c917](https://github.com/noir-lang/noir/commit/eb5c917e4e5550229fd1fd174b9fd7e507058d25))
* crash when typechecking fields that don't exist ([#1070](https://github.com/noir-lang/noir/issues/1070)) ([a67e8c5](https://github.com/noir-lang/noir/commit/a67e8c5f3867c3704c74e0b53e74e8ac18dced0a))
* Fix returning of structs in ACIR ([#1058](https://github.com/noir-lang/noir/issues/1058)) ([91bd471](https://github.com/noir-lang/noir/commit/91bd47190402f0fe567dbfb6fcfa17b97c129905))
* **nargo:** correct logic for rejecting transitive local dependencies ([#1015](https://github.com/noir-lang/noir/issues/1015)) ([e2b8b65](https://github.com/noir-lang/noir/commit/e2b8b65834de1d6eeb87459f657257791cc9a289))
* **nargo:** correct name in CLI output from `nargo_cli` to `nargo` ([#1095](https://github.com/noir-lang/noir/issues/1095)) ([74d7369](https://github.com/noir-lang/noir/commit/74d73696bdd042878cdfb06c8a781d575efc97fb))
* **nargo:** give contract artifacts unique names to prevent overwrites ([#1158](https://github.com/noir-lang/noir/issues/1158)) ([1227b2c](https://github.com/noir-lang/noir/commit/1227b2c913153bebfc416990f833687abb466ec7))
* **nargo:** only search for `Nargo.toml` in commands which act on a Nargo package ([#1029](https://github.com/noir-lang/noir/issues/1029)) ([6e642b9](https://github.com/noir-lang/noir/commit/6e642b9cf2f54d5e593fd5ded9246a6c4a61b5f8))
* **nargo:** resolve local dependencies relative to root of depending package ([38bf571](https://github.com/noir-lang/noir/commit/38bf5719d1757d39c89ecee0a6653a5d9da29c21))
* Numeric generics with impls error ([#1148](https://github.com/noir-lang/noir/issues/1148)) ([5d6e4d0](https://github.com/noir-lang/noir/commit/5d6e4d0b13404bd0681c3fe508e1abad21522411))
* rationalise witness for constant values ([#984](https://github.com/noir-lang/noir/issues/984)) ([ab32365](https://github.com/noir-lang/noir/commit/ab32365793b640a0a1e7c359c36f739d981a2487))
* Resolve globals in types ([#1043](https://github.com/noir-lang/noir/issues/1043)) ([2badf14](https://github.com/noir-lang/noir/commit/2badf1412e4322ced1db74c540708534d452d019))


### Miscellaneous Chores

* **nargo:** use faster hash function for checking preprocessed keys ([#1094](https://github.com/noir-lang/noir/issues/1094)) ([a69758c](https://github.com/noir-lang/noir/commit/a69758c0dff98bb23539df9c13366ef5b23e6b0f))
* remove outdated arkworks backend ([#1151](https://github.com/noir-lang/noir/issues/1151)) ([bc8ed9a](https://github.com/noir-lang/noir/commit/bc8ed9aa0c207bc93ac18a210c7a7828b354e860))
* upgrade to acvm 0.8.0 ([#1047](https://github.com/noir-lang/noir/issues/1047)) ([63f958b](https://github.com/noir-lang/noir/commit/63f958b0d4122a9974d450d4d6439434440a320c))

## [0.3.2](https://github.com/noir-lang/noir/compare/v0.3.1...v0.3.2) (2023-03-16)


### Features

* **stdlib:** Implement elliptic curve primitives ([#964](https://github.com/noir-lang/noir/issues/964)) ([30d612d](https://github.com/noir-lang/noir/commit/30d612d3c1632c770ea2130be57c4f98ca3c6cae))


### Bug Fixes

* **nargo:** correct inconsistent file extension for ACIR hashes ([#994](https://github.com/noir-lang/noir/issues/994)) ([23c22d7](https://github.com/noir-lang/noir/commit/23c22d7849609fbe0ae0a13f2af6e295cce8e01f))
* Prevent calling contract functions from outside the contract ([#980](https://github.com/noir-lang/noir/issues/980)) ([21360e3](https://github.com/noir-lang/noir/commit/21360e3c1a3f1cae441d268f0ccaeb29e0490808))
* reverse slash direction in `StdLibAssets` prefix on windows ([#992](https://github.com/noir-lang/noir/issues/992)) ([65b7108](https://github.com/noir-lang/noir/commit/65b71084bade6afb63803537783d83bfdd858a6c))

## [0.3.1](https://github.com/noir-lang/noir/compare/v0.3.0...v0.3.1) (2023-03-13)


### Features

* add `nargo preprocess` command ([#912](https://github.com/noir-lang/noir/issues/912)) ([8922ceb](https://github.com/noir-lang/noir/commit/8922ceba977e2220b10def222fc728f67d0e4dc3))


### Bug Fixes

* Update backend dependency containing updated pk write fix  ([#956](https://github.com/noir-lang/noir/issues/956)) ([5d627a7](https://github.com/noir-lang/noir/commit/5d627a74a752bfc3c5ce0d51bf2d032594f9d7af))

## [0.3.0](https://github.com/noir-lang/noir/compare/v0.2.0...v0.3.0) (2023-03-13)


### ⚠ BREAKING CHANGES

* **nargo:** rename `contract` command to `codegen-verifier` ([#959](https://github.com/noir-lang/noir/issues/959))
* replace dummy ABIs with `FunctionSignature` type alias ([#930](https://github.com/noir-lang/noir/issues/930))
* **nargo:** save program ABI alongside ACIR ([#922](https://github.com/noir-lang/noir/issues/922))
* **nargo:** restrict `CliError` visibility to crate ([#911](https://github.com/noir-lang/noir/issues/911))
* prevent inconsistent language usage in `Driver` ([#881](https://github.com/noir-lang/noir/issues/881))
* **abi:** add explicit return type field to ABI. ([#865](https://github.com/noir-lang/noir/issues/865))
* **abi:** merge both abi encoding/decoding methods ([#862](https://github.com/noir-lang/noir/issues/862))
* **abi:** add an explicit mapping from ABI params to witness indices ([#851](https://github.com/noir-lang/noir/issues/851))
* Allow impls on primitive types ([#847](https://github.com/noir-lang/noir/issues/847))

### Features

* **abi:** add an explicit mapping from ABI params to witness indices ([#851](https://github.com/noir-lang/noir/issues/851)) ([5bd4bd5](https://github.com/noir-lang/noir/commit/5bd4bd5047e4bc9a67bd79ab2a2519dc0c92da42))
* **abi:** add explicit return type field to ABI. ([#865](https://github.com/noir-lang/noir/issues/865)) ([8ca5676](https://github.com/noir-lang/noir/commit/8ca5676ba68403fff8bd953fe7c2d2f7c8e62a09))
* **abi:** merge both abi encoding/decoding methods ([#862](https://github.com/noir-lang/noir/issues/862)) ([fecd32c](https://github.com/noir-lang/noir/commit/fecd32cc27b552eb47681618ba44894c635c7f8c))
* add support for reading boolean arrays from toml ([#900](https://github.com/noir-lang/noir/issues/900)) ([93d83bf](https://github.com/noir-lang/noir/commit/93d83bf24d9ee340de54bda3d3df80e48855ae66))
* Allow impls on primitive types ([#847](https://github.com/noir-lang/noir/issues/847)) ([479da0e](https://github.com/noir-lang/noir/commit/479da0e724dc34667baaabd8e37ce143193bf97e))
* **ci:** Publish noir_wasm when we cut a release ([#871](https://github.com/noir-lang/noir/issues/871)) ([5186ab9](https://github.com/noir-lang/noir/commit/5186ab97a0fc087413f6d217b87c77f693c574ac))
* **compile:** compile w/dependencies and options ([#965](https://github.com/noir-lang/noir/issues/965)) ([3f897f6](https://github.com/noir-lang/noir/commit/3f897f623d81ec31f0ed0495da45586ff88850b9))
* **compile:** Noir std lib embedded ([#973](https://github.com/noir-lang/noir/issues/973)) ([13b9069](https://github.com/noir-lang/noir/commit/13b906909ad1cbfed5608dd7d5ef2809d31324d8))
* Implement basic contracts ([#944](https://github.com/noir-lang/noir/issues/944)) ([8ba3ab2](https://github.com/noir-lang/noir/commit/8ba3ab2f3570870bf8528eaf6dd1377d9a52d546))
* Implement endianness specified versions of `to_bytes` `to_radix` and `to_bits` ([#914](https://github.com/noir-lang/noir/issues/914)) ([43abc6b](https://github.com/noir-lang/noir/commit/43abc6b5b9014135ea93d9007d634025e59e1d30))
* **nargo:** save program ABI alongside ACIR ([#922](https://github.com/noir-lang/noir/issues/922)) ([ddaf305](https://github.com/noir-lang/noir/commit/ddaf305634cf0d0f1b6046ab68e84268eb1fa088))
* separate contract/program compilation from IO ([#967](https://github.com/noir-lang/noir/issues/967)) ([c60f545](https://github.com/noir-lang/noir/commit/c60f5457a62ec52ec6240e6f7188e3f8fe81e44c))
* Silence output of prove and verify ([#892](https://github.com/noir-lang/noir/issues/892)) ([811b346](https://github.com/noir-lang/noir/commit/811b346a5a65f8ad061ebc88c9095dedd5eaa0bc))
* **ssa:** add location to ssa instructions ([#931](https://github.com/noir-lang/noir/issues/931)) ([356858b](https://github.com/noir-lang/noir/commit/356858b185e4e6500bbe45c27dddf15b125aaaae))
* update to ACVM 0.5.0 ([#902](https://github.com/noir-lang/noir/issues/902)) ([9b58da4](https://github.com/noir-lang/noir/commit/9b58da45ae7b1542f7e9c258d748ceae3f1960c2))


### Bug Fixes

* **abi:** ensure that return value is loaded from toml ([#883](https://github.com/noir-lang/noir/issues/883)) ([adba24c](https://github.com/noir-lang/noir/commit/adba24c7db27a30c9443811339e4eedbf12e4470))
* add more readable error for missing argument in toml ([#971](https://github.com/noir-lang/noir/issues/971)) ([e31f41f](https://github.com/noir-lang/noir/commit/e31f41f65cb264c95b84740f02b687140ee0a050))
* allow parsing strings from toml into booleans ([#894](https://github.com/noir-lang/noir/issues/894)) ([f729a00](https://github.com/noir-lang/noir/commit/f729a00e45f37e2cbb4654b48e8bab986e164423))
* check the argument count of generic types ([#970](https://github.com/noir-lang/noir/issues/970)) ([2688dc4](https://github.com/noir-lang/noir/commit/2688dc405968dcd9b7a9486cc9cabffd9698dce8))
* compute witness when println evaluated before input ([#891](https://github.com/noir-lang/noir/issues/891)) ([2727b34](https://github.com/noir-lang/noir/commit/2727b34f29d032b3d26ed41e538e7cc8d7d07770))
* correct type checking to handle `false` bools ([#893](https://github.com/noir-lang/noir/issues/893)) ([6c7aa2f](https://github.com/noir-lang/noir/commit/6c7aa2fc39c7caff1fee94888287f17101101e43))
* display command description in CLI for `nargo prove` ([#949](https://github.com/noir-lang/noir/issues/949)) ([2829af1](https://github.com/noir-lang/noir/commit/2829af1b9778f1b54bef18ae5d9748b7289ecb9c))
* evaluate constant division ([#909](https://github.com/noir-lang/noir/issues/909)) ([b91307b](https://github.com/noir-lang/noir/commit/b91307b43a5ecc6fea0edf59dee06d7e93b8f324))
* Fix multiple call of `to_le_bytes` ([#941](https://github.com/noir-lang/noir/issues/941)) ([2ee0119](https://github.com/noir-lang/noir/commit/2ee0119ac9b28ddbad560016c8151e29970bdfc5))
* generate valid toml when outputting nested structs  ([#936](https://github.com/noir-lang/noir/issues/936)) ([ba947a7](https://github.com/noir-lang/noir/commit/ba947a7c22720d90676422f9c29bd55f047e9edb))
* Improve member access error ([#940](https://github.com/noir-lang/noir/issues/940)) ([9b5b5f6](https://github.com/noir-lang/noir/commit/9b5b5f6ba8830f1c7d0eb46b0888f15f9fe6b5d7))
* **nargo:** Switch order of writing acir file and acir checksum file ([#895](https://github.com/noir-lang/noir/issues/895)) ([4fc94dc](https://github.com/noir-lang/noir/commit/4fc94dc010fda5496501991664c0853e5a8f6707))
* **nargo:** Use yml extension for bug report link presented upon panic ([#960](https://github.com/noir-lang/noir/issues/960)) ([f7b3711](https://github.com/noir-lang/noir/commit/f7b3711603536b1b1ad5246afa749087de688464))
* **nargo:** Use yml extension on the bug report link presented upon panic ([f7b3711](https://github.com/noir-lang/noir/commit/f7b3711603536b1b1ad5246afa749087de688464))
* **noir_wasm:** Update wasm ACIR serialization ([#898](https://github.com/noir-lang/noir/issues/898)) ([575436f](https://github.com/noir-lang/noir/commit/575436faacc75a945456748f252ac731107e5564))
* Optimize parser ([#869](https://github.com/noir-lang/noir/issues/869)) ([e927a39](https://github.com/noir-lang/noir/commit/e927a39dc3d6517f233509b8349dfd9c7f79471d))
* prevent inconsistent language usage in `Driver` ([48cda7a](https://github.com/noir-lang/noir/commit/48cda7a08b22afdde9f904632b502c53fb491ee6))
* prevent inconsistent language usage in `Driver` ([#881](https://github.com/noir-lang/noir/issues/881)) ([48cda7a](https://github.com/noir-lang/noir/commit/48cda7a08b22afdde9f904632b502c53fb491ee6))
* properly initialize `Evaluator` in test ([#863](https://github.com/noir-lang/noir/issues/863)) ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))
* properly initialize Evaluator in test ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))
* Remove uses of std::process::exit ([#963](https://github.com/noir-lang/noir/issues/963)) ([870ea46](https://github.com/noir-lang/noir/commit/870ea463583502db106d4c8b05ad5c02fb6f8428))
* **ssa:** fix the compile-time check for equality in acir-gen ([#904](https://github.com/noir-lang/noir/issues/904)) ([161e4fb](https://github.com/noir-lang/noir/commit/161e4fbfe17ef9ed6c237d6ea812a866fee2c74a))


### Miscellaneous Chores

* **nargo:** rename `contract` command to `codegen-verifier` ([#959](https://github.com/noir-lang/noir/issues/959)) ([2e63492](https://github.com/noir-lang/noir/commit/2e63492aadf17bda2906f22e10476834f497f664))
* **nargo:** restrict `CliError` visibility to crate ([#911](https://github.com/noir-lang/noir/issues/911)) ([ed0e1ab](https://github.com/noir-lang/noir/commit/ed0e1ab4c7a3461da1a3fd500335d146ce43176c))
* replace dummy ABIs with `FunctionSignature` type alias ([#930](https://github.com/noir-lang/noir/issues/930)) ([156125b](https://github.com/noir-lang/noir/commit/156125ba6b1c01804ea15305ba13eb9cc3203273))

## [0.2.0](https://github.com/noir-lang/noir/compare/v0.1.1...v0.2.0) (2023-02-16)


### ⚠ BREAKING CHANGES

* Make `abi` field non-optional in `CompiledProgram` ([#856](https://github.com/noir-lang/noir/issues/856))
* **nargo:** bump MSRV to 1.66.0 ([#799](https://github.com/noir-lang/noir/issues/799))

### Features

* **acvm:** Update to acvm 0.4.1 ([#779](https://github.com/noir-lang/noir/issues/779)) ([6f57e86](https://github.com/noir-lang/noir/commit/6f57e86c3d51191aa516a3b9315337b925810433))
* **ci:** Add concurrency group for rust workflow ([#806](https://github.com/noir-lang/noir/issues/806)) ([1b80f55](https://github.com/noir-lang/noir/commit/1b80f559599c2a7d7b8697f42f63db8e59d318c5))
* **ci:** Add concurrency group for rust workflow ([1b80f55](https://github.com/noir-lang/noir/commit/1b80f559599c2a7d7b8697f42f63db8e59d318c5))
* **ci:** Build binaries when a release is made ([#773](https://github.com/noir-lang/noir/issues/773)) ([a0c0c2c](https://github.com/noir-lang/noir/commit/a0c0c2c354b50c80eba425ba2f8c235015696c35))
* Impls with generics ([#798](https://github.com/noir-lang/noir/issues/798)) ([bea735d](https://github.com/noir-lang/noir/commit/bea735d98e162f42df5957781638101c1e6c75f6))
* **nargo:** add flag to verify created proofs ([#737](https://github.com/noir-lang/noir/issues/737)) ([e981c7c](https://github.com/noir-lang/noir/commit/e981c7ca0ab23073339869a7d45c04ae10fe1adf))
* **nargo:** add panic hook ([74cb340](https://github.com/noir-lang/noir/commit/74cb3407907c95a62bc7a72e62ba67c890f2a077))
* **nargo:** Add panic hook ([#850](https://github.com/noir-lang/noir/issues/850)) ([74cb340](https://github.com/noir-lang/noir/commit/74cb3407907c95a62bc7a72e62ba67c890f2a077))
* **nargo:** Update nargo to use preprocessing interface ([#765](https://github.com/noir-lang/noir/issues/765)) ([b3f1556](https://github.com/noir-lang/noir/commit/b3f1556558adcc1d510d23bb23a894b379d0eed3))
* **nargo:** Version info in nargo and wasm ([#802](https://github.com/noir-lang/noir/issues/802)) ([fd64be5](https://github.com/noir-lang/noir/commit/fd64be55fc905a032d53c9ac7a7f7b71da899c37))
* **ssa:** array sort ([#754](https://github.com/noir-lang/noir/issues/754)) ([32e9320](https://github.com/noir-lang/noir/commit/32e93202361490a051ec1931612c4d5a7f486e6a))
* **std_lib:** println statements ([#630](https://github.com/noir-lang/noir/issues/630)) ([d5d1be2](https://github.com/noir-lang/noir/commit/d5d1be2f3abc072e2f487e2e4fd68f9fb376abcc))
* **stdlib:** Add higher order array functions ([#833](https://github.com/noir-lang/noir/issues/833)) ([9c62fef](https://github.com/noir-lang/noir/commit/9c62fefb6b7b108ad5eb83971c89356429831a83))


### Bug Fixes

* avoid testing equality between unit values in acir_gen test ([#849](https://github.com/noir-lang/noir/issues/849)) ([c2b7230](https://github.com/noir-lang/noir/commit/c2b7230af2fdd3cee76bb0d72b0943d6782c322e))
* **ci:** Skip the title check if handling a merge group ([#790](https://github.com/noir-lang/noir/issues/790)) ([71b179c](https://github.com/noir-lang/noir/commit/71b179c4f812f773282a0911082dd759ad20c450))
* **nargo:** `nargo test` now only runs test functions defined in the current module ([#805](https://github.com/noir-lang/noir/issues/805)) ([c6293c9](https://github.com/noir-lang/noir/commit/c6293c9d1657a6937a95a10b931dbb6c3d9c94d7))
* operators issuing type errors when used with matching integer types arising from generic code ([#789](https://github.com/noir-lang/noir/issues/789)) ([932943a](https://github.com/noir-lang/noir/commit/932943a0f7af8f91ba55964ecc574e569a99508d))
* **ssa:** delete instructions with false predicate ([#760](https://github.com/noir-lang/noir/issues/760)) ([f329379](https://github.com/noir-lang/noir/commit/f3293793e7fd4a595971c24c4dcab9b0e7b921dd))
* **ssa:** synchronisation for functions ([#764](https://github.com/noir-lang/noir/issues/764)) ([615357a](https://github.com/noir-lang/noir/commit/615357af4173d767af87df9086bb9fb78fd749c6))


### Miscellaneous Chores

* Make `abi` field non-optional in `CompiledProgram` ([#856](https://github.com/noir-lang/noir/issues/856)) ([98acb5a](https://github.com/noir-lang/noir/commit/98acb5ad5609d89ea34481a8e8359449d0ca1344))
* **nargo:** bump MSRV to 1.66.0 ([#799](https://github.com/noir-lang/noir/issues/799)) ([59ff9e8](https://github.com/noir-lang/noir/commit/59ff9e897195aede863e3c166773c222e1bc7a54))

## [0.1.1](https://github.com/noir-lang/noir/compare/v0.1.0...v0.1.1) (2023-02-06)


### Features

* **ci:** Add workflow to validate PR title ([#730](https://github.com/noir-lang/noir/issues/730)) ([e5e8542](https://github.com/noir-lang/noir/commit/e5e85423946e52b431a32ee37c4967bef3c2fc88))
* **ci:** Change release workflow to use release-please ([950ca55](https://github.com/noir-lang/noir/commit/950ca5535ba52de3aafd861fd00a75d5c0bf0125))
* **docs:** Introduce Conventional Commits & release process docs ([#717](https://github.com/noir-lang/noir/issues/717)) ([950ca55](https://github.com/noir-lang/noir/commit/950ca5535ba52de3aafd861fd00a75d5c0bf0125))
* **nargo:** add `nargo execute` command ([#725](https://github.com/noir-lang/noir/issues/725)) ([9d6be60](https://github.com/noir-lang/noir/commit/9d6be60bbf2ef8cdeb272942fc2d3d94f5dda96f))
* **nargo:** Add `nargo test` command to run all unit tests ([#728](https://github.com/noir-lang/noir/issues/728)) ([2e1dc82](https://github.com/noir-lang/noir/commit/2e1dc823643c3c522eafdd38b5d92f6f431226f4))
* **nargo:** add option to save witness to file in execute command ([9d6be60](https://github.com/noir-lang/noir/commit/9d6be60bbf2ef8cdeb272942fc2d3d94f5dda96f))
* **nargo:** add support for testing noir libraries ([#752](https://github.com/noir-lang/noir/issues/752)) ([27bd2ac](https://github.com/noir-lang/noir/commit/27bd2ac26370400c9605262eeb12c2b47d94149e))
* **nargo:** Leverage rustls instead of openssl for downloads ([#691](https://github.com/noir-lang/noir/issues/691)) ([933809c](https://github.com/noir-lang/noir/commit/933809cc52029330c4823d330c088e0acb4e87c3))
