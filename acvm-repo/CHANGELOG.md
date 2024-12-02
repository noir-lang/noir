# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.55.0](https://github.com/noir-lang/noir/compare/v0.54.0...v0.55.0) (2024-11-19)


### ⚠ BREAKING CHANGES

* Remove `recursive` from ACIR format; add them to API and CLI (https://github.com/AztecProtocol/aztec-packages/pull/9479)
* **avm/brillig:** revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408)
* use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416)
* remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338)
* replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350)
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327))
* remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107)
* remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245)
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057)
* remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104)
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989)
* **avm:** remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030)
* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Encode static error strings in the ABI (https://github.com/AztecProtocol/aztec-packages/pull/9552) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Ensure that generated ACIR is solvable ([#6415](https://github.com/noir-lang/noir/issues/6415)) ([b473d99](https://github.com/noir-lang/noir/commit/b473d99b2b70b595596b8392617256dbaf5d5642))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate databus in the private kernels (https://github.com/AztecProtocol/aztec-packages/pull/9028) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Parallelize DIE pass (https://github.com/AztecProtocol/aztec-packages/pull/9933) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327)) ([4d87c9a](https://github.com/noir-lang/noir/commit/4d87c9ac78b48b4bd0ae81316df28aab390d004e))
* Remove 'single use' intermediate variables ([#6268](https://github.com/noir-lang/noir/issues/6268)) ([ec75e8e](https://github.com/noir-lang/noir/commit/ec75e8ec59e0f2a2169aea67372411ede4074d09))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Simplify constant MSM calls in SSA ([#6547](https://github.com/noir-lang/noir/issues/6547)) ([f291e37](https://github.com/noir-lang/noir/commit/f291e3702589a5cd043acfded5e187f56ec765cc))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Stop with HeapVector (https://github.com/AztecProtocol/aztec-packages/pull/9810) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8934) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9034) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9099) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9275) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9711) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* **test:** Fuzz test poseidon2 hash equivalence ([#6265](https://github.com/noir-lang/noir/issues/6265)) ([f61ba03](https://github.com/noir-lang/noir/commit/f61ba037c6726e19be4f894d9447fe396df95417))
* **test:** Fuzz test stdlib hash functions ([#6233](https://github.com/noir-lang/noir/issues/6233)) ([1a2ca46](https://github.com/noir-lang/noir/commit/1a2ca46af0d1c05813dbe28670a2bc39b79e4c9f))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Discard optimisation that would change execution ordering or that is related to call outputs ([#6461](https://github.com/noir-lang/noir/issues/6461)) ([b8654f7](https://github.com/noir-lang/noir/commit/b8654f700b218cc09c5381af65df11ead9ffcdaf))
* Display every bit in integer tokens ([#6360](https://github.com/noir-lang/noir/issues/6360)) ([b985fdf](https://github.com/noir-lang/noir/commit/b985fdf6e635570b8db3af83d9ec14e7cd749062))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Homogeneous input points for EC ADD ([#6241](https://github.com/noir-lang/noir/issues/6241)) ([f6a7306](https://github.com/noir-lang/noir/commit/f6a7306436ea1a37ec7f3b884721b50467e9a063))
* Reject invalid expression with in CLI parser ([#6287](https://github.com/noir-lang/noir/issues/6287)) ([052aee8](https://github.com/noir-lang/noir/commit/052aee80ff3e1e4fd2ca45310d7bb8b980af126a))
* Remove need for duplicate attributes on each function (https://github.com/AztecProtocol/aztec-packages/pull/9244) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Take blackbox function outputs into account when merging expressions ([#6532](https://github.com/noir-lang/noir/issues/6532)) ([713df69](https://github.com/noir-lang/noir/commit/713df69aad56fc5aaefd5d140275a3217de4d866))
* **tests:** Prevent EOF error while running test programs ([#6455](https://github.com/noir-lang/noir/issues/6455)) ([358e381](https://github.com/noir-lang/noir/commit/358e38107edbc4f40c97b88196456d82f5557e3f))
* Typing of artifacts (https://github.com/AztecProtocol/aztec-packages/pull/9581) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))


### Miscellaneous Chores

* Remove `recursive` from ACIR format; add them to API and CLI (https://github.com/AztecProtocol/aztec-packages/pull/9479) ([7dd71c1](https://github.com/noir-lang/noir/commit/7dd71c15cbcbf025ba049b506c94924903b32754))
* Remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))


### Code Refactoring

* **avm:** Remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))

## [0.54.0](https://github.com/noir-lang/noir/compare/v0.53.0...v0.54.0) (2024-11-08)


### ⚠ BREAKING CHANGES

* **avm/brillig:** revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408)
* use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416)
* remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338)
* replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350)
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327))
* remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107)
* remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245)
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057)
* remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104)
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989)
* **avm:** remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030)
* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Ensure that generated ACIR is solvable ([#6415](https://github.com/noir-lang/noir/issues/6415)) ([b473d99](https://github.com/noir-lang/noir/commit/b473d99b2b70b595596b8392617256dbaf5d5642))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate databus in the private kernels (https://github.com/AztecProtocol/aztec-packages/pull/9028) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327)) ([4d87c9a](https://github.com/noir-lang/noir/commit/4d87c9ac78b48b4bd0ae81316df28aab390d004e))
* Remove 'single use' intermediate variables ([#6268](https://github.com/noir-lang/noir/issues/6268)) ([ec75e8e](https://github.com/noir-lang/noir/commit/ec75e8ec59e0f2a2169aea67372411ede4074d09))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8934) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9034) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9099) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9275) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* **test:** Fuzz test poseidon2 hash equivalence ([#6265](https://github.com/noir-lang/noir/issues/6265)) ([f61ba03](https://github.com/noir-lang/noir/commit/f61ba037c6726e19be4f894d9447fe396df95417))
* **test:** Fuzz test stdlib hash functions ([#6233](https://github.com/noir-lang/noir/issues/6233)) ([1a2ca46](https://github.com/noir-lang/noir/commit/1a2ca46af0d1c05813dbe28670a2bc39b79e4c9f))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Discard optimisation that would change execution ordering or that is related to call outputs ([#6461](https://github.com/noir-lang/noir/issues/6461)) ([b8654f7](https://github.com/noir-lang/noir/commit/b8654f700b218cc09c5381af65df11ead9ffcdaf))
* Display every bit in integer tokens ([#6360](https://github.com/noir-lang/noir/issues/6360)) ([b985fdf](https://github.com/noir-lang/noir/commit/b985fdf6e635570b8db3af83d9ec14e7cd749062))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Homogeneous input points for EC ADD ([#6241](https://github.com/noir-lang/noir/issues/6241)) ([f6a7306](https://github.com/noir-lang/noir/commit/f6a7306436ea1a37ec7f3b884721b50467e9a063))
* Reject invalid expression with in CLI parser ([#6287](https://github.com/noir-lang/noir/issues/6287)) ([052aee8](https://github.com/noir-lang/noir/commit/052aee80ff3e1e4fd2ca45310d7bb8b980af126a))
* Remove need for duplicate attributes on each function (https://github.com/AztecProtocol/aztec-packages/pull/9244) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **tests:** Prevent EOF error while running test programs ([#6455](https://github.com/noir-lang/noir/issues/6455)) ([358e381](https://github.com/noir-lang/noir/commit/358e38107edbc4f40c97b88196456d82f5557e3f))


### Miscellaneous Chores

* Remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))


### Code Refactoring

* **avm:** Remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))

## [0.53.0](https://github.com/noir-lang/noir/compare/v0.52.0...v0.53.0) (2024-10-31)


### ⚠ BREAKING CHANGES

* **avm/brillig:** revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408)
* use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416)
* remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338)
* replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350)
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327))
* remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107)
* remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245)
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057)
* remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104)
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989)
* **avm:** remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030)
* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Revert/rethrow oracle (https://github.com/AztecProtocol/aztec-packages/pull/9408) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate databus in the private kernels (https://github.com/AztecProtocol/aztec-packages/pull/9028) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **profiler:** New flamegraph command that profiles the opcodes executed ([#6327](https://github.com/noir-lang/noir/issues/6327)) ([4d87c9a](https://github.com/noir-lang/noir/commit/4d87c9ac78b48b4bd0ae81316df28aab390d004e))
* Remove 'single use' intermediate variables ([#6268](https://github.com/noir-lang/noir/issues/6268)) ([ec75e8e](https://github.com/noir-lang/noir/commit/ec75e8ec59e0f2a2169aea67372411ede4074d09))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8934) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9034) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9099) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9275) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* **test:** Fuzz test poseidon2 hash equivalence ([#6265](https://github.com/noir-lang/noir/issues/6265)) ([f61ba03](https://github.com/noir-lang/noir/commit/f61ba037c6726e19be4f894d9447fe396df95417))
* **test:** Fuzz test stdlib hash functions ([#6233](https://github.com/noir-lang/noir/issues/6233)) ([1a2ca46](https://github.com/noir-lang/noir/commit/1a2ca46af0d1c05813dbe28670a2bc39b79e4c9f))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Display every bit in integer tokens ([#6360](https://github.com/noir-lang/noir/issues/6360)) ([b985fdf](https://github.com/noir-lang/noir/commit/b985fdf6e635570b8db3af83d9ec14e7cd749062))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Homogeneous input points for EC ADD ([#6241](https://github.com/noir-lang/noir/issues/6241)) ([f6a7306](https://github.com/noir-lang/noir/commit/f6a7306436ea1a37ec7f3b884721b50467e9a063))
* Reject invalid expression with in CLI parser ([#6287](https://github.com/noir-lang/noir/issues/6287)) ([052aee8](https://github.com/noir-lang/noir/commit/052aee80ff3e1e4fd2ca45310d7bb8b980af126a))
* Remove need for duplicate attributes on each function (https://github.com/AztecProtocol/aztec-packages/pull/9244) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))


### Miscellaneous Chores

* Remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove noir_js_backend_barretenberg (https://github.com/AztecProtocol/aztec-packages/pull/9338) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Replace usage of vector in keccakf1600 input with array (https://github.com/AztecProtocol/aztec-packages/pull/9350) ([3925228](https://github.com/noir-lang/noir/commit/392522880e102e275ebcf42f16651a8ffa0bbbd2))
* Use Brillig opcode when possible for less-than operations on fields (https://github.com/AztecProtocol/aztec-packages/pull/9416) ([321a493](https://github.com/noir-lang/noir/commit/321a493216e19a2f077007c3447a3030db0df0d0))


### Code Refactoring

* **avm:** Remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))

## [0.52.0](https://github.com/noir-lang/noir/compare/v0.51.0...v0.52.0) (2024-10-22)


### ⚠ BREAKING CHANGES

* remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107)
* remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245)
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057)
* remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104)
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989)
* **avm:** remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030)
* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)
* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Brillig and AVM default all uninitialized memory cells to Field 0 (https://github.com/AztecProtocol/aztec-packages/pull/9057) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Brillig with a stack and conditional inlining (https://github.com/AztecProtocol/aztec-packages/pull/8989) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate databus in the private kernels (https://github.com/AztecProtocol/aztec-packages/pull/9028) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8934) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9034) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9099) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/9275) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* **test:** Fuzz test poseidon2 hash equivalence ([#6265](https://github.com/noir-lang/noir/issues/6265)) ([f61ba03](https://github.com/noir-lang/noir/commit/f61ba037c6726e19be4f894d9447fe396df95417))
* **test:** Fuzz test stdlib hash functions ([#6233](https://github.com/noir-lang/noir/issues/6233)) ([1a2ca46](https://github.com/noir-lang/noir/commit/1a2ca46af0d1c05813dbe28670a2bc39b79e4c9f))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Homogeneous input points for EC ADD ([#6241](https://github.com/noir-lang/noir/issues/6241)) ([f6a7306](https://github.com/noir-lang/noir/commit/f6a7306436ea1a37ec7f3b884721b50467e9a063))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Reject invalid expression with in CLI parser ([#6287](https://github.com/noir-lang/noir/issues/6287)) ([052aee8](https://github.com/noir-lang/noir/commit/052aee80ff3e1e4fd2ca45310d7bb8b980af126a))
* Remove need for duplicate attributes on each function (https://github.com/AztecProtocol/aztec-packages/pull/9244) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))


### Miscellaneous Chores

* Remove keccak256 opcode from ACIR/Brillig (https://github.com/AztecProtocol/aztec-packages/pull/9104) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen commitment (https://github.com/AztecProtocol/aztec-packages/pull/9107) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))
* Remove pedersen hash opcode (https://github.com/AztecProtocol/aztec-packages/pull/9245) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))


### Code Refactoring

* **avm:** Remove CMOV opcode (https://github.com/AztecProtocol/aztec-packages/pull/9030) ([70dcf4a](https://github.com/noir-lang/noir/commit/70dcf4a25dcad10daeb427f0887d3a0bf10c9916))

## [0.51.0](https://github.com/noir-lang/noir/compare/v0.50.0...v0.51.0) (2024-10-03)


### ⚠ BREAKING CHANGES

* remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571)
* add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570)
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)
* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add support for u1 in the avm, ToRadix's radix arg is a memory addr (https://github.com/AztecProtocol/aztec-packages/pull/8570) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize allocating immediate amounts of memory (https://github.com/AztecProtocol/aztec-packages/pull/8579) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Remove sha256 opcode (https://github.com/AztecProtocol/aztec-packages/pull/4571) ([e8bbce7](https://github.com/noir-lang/noir/commit/e8bbce71fde3fc7af410c30920c2a547389d8248))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Runtime brillig bigint id assignment ([#5369](https://github.com/noir-lang/noir/issues/5369)) ([a8928dd](https://github.com/noir-lang/noir/commit/a8928ddcffcae15babf7aa5aff0e462e4549552e))

## [0.50.0](https://github.com/noir-lang/noir/compare/v0.49.0...v0.50.0) (2024-09-13)


### ⚠ BREAKING CHANGES

* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488)
* **avm:** variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441)
* **avm/brillig:** take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388)
* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)
* add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205))
* restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180))

### Features

* (bb) 128-bit challenges (https://github.com/AztecProtocol/aztec-packages/pull/8406) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* Add assertions for ACVM `FunctionInput` `bit_size` ([#5864](https://github.com/noir-lang/noir/issues/5864)) ([8712f4c](https://github.com/noir-lang/noir/commit/8712f4c20d23f3809bcfb03f2e3ba0e5ace20a1d))
* Add Not instruction in brillig (https://github.com/AztecProtocol/aztec-packages/pull/8488) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Add recursive aggregation object to proving/verification keys (https://github.com/AztecProtocol/aztec-packages/pull/6770) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Add reusable procedures to brillig generation (https://github.com/AztecProtocol/aztec-packages/pull/7981) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205)) ([14adafc](https://github.com/noir-lang/noir/commit/14adafc965fa9c833e096ec037e086aae67703ad))
* Added indirect const instruction (https://github.com/AztecProtocol/aztec-packages/pull/8065) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Adding aggregation to honk and rollup (https://github.com/AztecProtocol/aztec-packages/pull/7466) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Automate verify_honk_proof input generation (https://github.com/AztecProtocol/aztec-packages/pull/8092) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **avm/brillig:** Take addresses in calldatacopy (https://github.com/AztecProtocol/aztec-packages/pull/8388) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* **avm:** Variants for SET opcode (https://github.com/AztecProtocol/aztec-packages/pull/8441) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Change the layout of arrays and vectors to be a single pointer (https://github.com/AztecProtocol/aztec-packages/pull/8448) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Hook up secondary calldata column in dsl (https://github.com/AztecProtocol/aztec-packages/pull/7759) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make token transfer be recursive (https://github.com/AztecProtocol/aztec-packages/pull/7730) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` (https://github.com/AztecProtocol/aztec-packages/pull/6131) ([ff67e14](https://github.com/noir-lang/noir/commit/ff67e145d086bf6fdf58fb5e57927033e52e03d3))
* New test programs for wasm benchmarking (https://github.com/AztecProtocol/aztec-packages/pull/8389) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Note hashes as points (https://github.com/AztecProtocol/aztec-packages/pull/7618) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize constant array handling in brillig_gen (https://github.com/AztecProtocol/aztec-packages/pull/7661) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Optimize to_radix (https://github.com/AztecProtocol/aztec-packages/pull/8073) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Pass calldata ids to the backend (https://github.com/AztecProtocol/aztec-packages/pull/7875) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Poseidon2 gates for Ultra arithmetisation (https://github.com/AztecProtocol/aztec-packages/pull/7494) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* **profiler:** Add support for brillig functions in opcodes-flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7698) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Removing superfluous call to MSM (https://github.com/AztecProtocol/aztec-packages/pull/7708) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Report gates and VKs of private protocol circuits with megahonk (https://github.com/AztecProtocol/aztec-packages/pull/7722) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180)) ([bdb2bc6](https://github.com/noir-lang/noir/commit/bdb2bc608ea8fd52d46545a38b68dd2558b28110))
* Separate runtimes of SSA functions before inlining ([#5121](https://github.com/noir-lang/noir/issues/5121)) ([69eca9b](https://github.com/noir-lang/noir/commit/69eca9b8671fa54192bef814dd584fdb5387a5f7))
* Simplify constant calls to `poseidon2_permutation`, `schnorr_verify` and `embedded_curve_add` ([#5140](https://github.com/noir-lang/noir/issues/5140)) ([2823ba7](https://github.com/noir-lang/noir/commit/2823ba7242db788ca1d7f6e7a48be2f1de62f278))
* Small optimization in toradix (https://github.com/AztecProtocol/aztec-packages/pull/8040) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7743) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7862) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7945) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7958) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8008) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8093) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8125) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8237) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8423) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8435) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8466) ([3c3ed1e](https://github.com/noir-lang/noir/commit/3c3ed1e3d28946a02071c524dd128afe131bc3da))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8482) ([d4832ec](https://github.com/noir-lang/noir/commit/d4832ece9d3ad16544afea49cc7caf40501a2cc3))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8512) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/8526) ([95e19ab](https://github.com/noir-lang/noir/commit/95e19ab9486ad054241b6e53e40e55bdba9dc7e5))
* TXE nr deployments, dependency cleanup for CLI (https://github.com/AztecProtocol/aztec-packages/pull/7548) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Unify all acir recursion constraints based on RecursionConstraint and proof_type (https://github.com/AztecProtocol/aztec-packages/pull/7993) ([5c4f19f](https://github.com/noir-lang/noir/commit/5c4f19f097dd3704522996330c961bf0a2db8d99))


### Bug Fixes

* Add support for nested arrays returned by oracles ([#5132](https://github.com/noir-lang/noir/issues/5132)) ([f846879](https://github.com/noir-lang/noir/commit/f846879dd038328bd0a1d39a72b448ef52a1002b))
* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Avoid unnecessarily splitting expressions with multiplication terms with a shared term ([#5291](https://github.com/noir-lang/noir/issues/5291)) ([19884f1](https://github.com/noir-lang/noir/commit/19884f161dfc7d7ce75dd2c404b8ef39cdad2240))
* **debugger:** Update the debugger to handle the new Brillig debug metadata format ([#5706](https://github.com/noir-lang/noir/issues/5706)) ([a31f82e](https://github.com/noir-lang/noir/commit/a31f82e598def60d00c65b79b8c5411f8aa832aa))
* Deflatten databus visibilities (https://github.com/AztecProtocol/aztec-packages/pull/7761) ([4ea25db](https://github.com/noir-lang/noir/commit/4ea25dbde87488e758139619a3ce4edf93c6ebd6))
* Do not duplicate redundant Brillig debug metadata ([#5696](https://github.com/noir-lang/noir/issues/5696)) ([e4f7dbe](https://github.com/noir-lang/noir/commit/e4f7dbe63b55807b3ff0b4d6f47a8b7f847299fb))
* Export brillig names in contract functions (https://github.com/AztecProtocol/aztec-packages/pull/8212) ([f0c2686](https://github.com/noir-lang/noir/commit/f0c268606a71381ab4504396695a0adb9b3258b6))
* Handle multiple entry points for Brillig call stack resolution after metadata deduplication ([#5788](https://github.com/noir-lang/noir/issues/5788)) ([38fe9dd](https://github.com/noir-lang/noir/commit/38fe9dda111952fdb894df90a319c087382edfc9))
* Handle struct with nested arrays in oracle return values ([#5244](https://github.com/noir-lang/noir/issues/5244)) ([a30814f](https://github.com/noir-lang/noir/commit/a30814f1f767bf874cd7e2969f5061c68f16b9a7))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Restrict keccak256_injective test input to 8 bits ([#5977](https://github.com/noir-lang/noir/issues/5977)) ([a1b1346](https://github.com/noir-lang/noir/commit/a1b1346bf7525c508fd390393c307475cc2345d7))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Runtime brillig bigint id assignment ([#5369](https://github.com/noir-lang/noir/issues/5369)) ([a8928dd](https://github.com/noir-lang/noir/commit/a8928ddcffcae15babf7aa5aff0e462e4549552e))

## [0.49.0](https://github.com/noir-lang/noir/compare/v0.48.0...v0.49.0) (2024-08-06)


### ⚠ BREAKING CHANGES

* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)
* add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205))
* restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180))
* switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283)
* specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239)
* remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995)
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016)
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)

### Features

* `multi_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6097) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* `variable_base_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6039) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir_gen:** Width aware ACIR gen addition ([#5493](https://github.com/noir-lang/noir/issues/5493)) ([85fa592](https://github.com/noir-lang/noir/commit/85fa592fdef3b8589ce03b232e1b51565837b540))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Activate return_data in ACIR opcodes ([#5080](https://github.com/noir-lang/noir/issues/5080)) ([c9fda3c](https://github.com/noir-lang/noir/commit/c9fda3c7fd4575bfe7d457e8d4230e071f0129a0))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add native rust implementation of schnorr signature verification ([#5053](https://github.com/noir-lang/noir/issues/5053)) ([fab1c35](https://github.com/noir-lang/noir/commit/fab1c3567d731ea7902635a7a020a8d14f94fd27))
* Add native rust implementations of pedersen functions ([#4871](https://github.com/noir-lang/noir/issues/4871)) ([fb039f7](https://github.com/noir-lang/noir/commit/fb039f74df23aea39bc0593a5d538d82b4efadf0))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205)) ([14adafc](https://github.com/noir-lang/noir/commit/14adafc965fa9c833e096ec037e086aae67703ad))
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Avoid heap allocs when going to/from field (https://github.com/AztecProtocol/aztec-packages/pull/7547) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Dynamic assertion payloads v2 (https://github.com/AztecProtocol/aztec-packages/pull/5949) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Increase default expression width to 4 ([#4995](https://github.com/noir-lang/noir/issues/4995)) ([f01d309](https://github.com/noir-lang/noir/commit/f01d3090759a5ff0f1f83c5616d22890c6bd76be))
* Integrate new proving systems in e2e (https://github.com/AztecProtocol/aztec-packages/pull/6971) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Make ACVM generic across fields ([#5114](https://github.com/noir-lang/noir/issues/5114)) ([70f374c](https://github.com/noir-lang/noir/commit/70f374c06642962d8f2b95b80f8c938fcf7761d7))
* Make Brillig do integer arithmetic operations using u128 instead of Bigint (https://github.com/AztecProtocol/aztec-packages/pull/7518) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Move abi demonomorphizer to noir_codegen and use noir_codegen in protocol types (https://github.com/AztecProtocol/aztec-packages/pull/6302) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Move to_radix to a blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6294) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` (https://github.com/AztecProtocol/aztec-packages/pull/6131) ([ff67e14](https://github.com/noir-lang/noir/commit/ff67e145d086bf6fdf58fb5e57927033e52e03d3))
* Parsing non-string assertion payloads in noir js (https://github.com/AztecProtocol/aztec-packages/pull/6079) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Private Kernel Recursion (https://github.com/AztecProtocol/aztec-packages/pull/6278) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Proper padding in ts AES and constrained AES in body and header computations (https://github.com/AztecProtocol/aztec-packages/pull/6269) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Remove conditional compilation of `bn254_blackbox_solver` ([#5058](https://github.com/noir-lang/noir/issues/5058)) ([9420d7c](https://github.com/noir-lang/noir/commit/9420d7c2ba6bbbf5ecb9a066837c505310955b6c))
* Remove external blackbox solver from acir simulator (https://github.com/AztecProtocol/aztec-packages/pull/6586) ([a40a9a5](https://github.com/noir-lang/noir/commit/a40a9a55571deed386688fb84260bdf2794d4d38))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180)) ([bdb2bc6](https://github.com/noir-lang/noir/commit/bdb2bc608ea8fd52d46545a38b68dd2558b28110))
* Separate runtimes of SSA functions before inlining ([#5121](https://github.com/noir-lang/noir/issues/5121)) ([69eca9b](https://github.com/noir-lang/noir/commit/69eca9b8671fa54192bef814dd584fdb5387a5f7))
* Set aztec private functions to be recursive (https://github.com/AztecProtocol/aztec-packages/pull/6192) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6280) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6332) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6573) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7432) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7444) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7454) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7577) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7583) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* ToRadix BB + avm transpiler support (https://github.com/AztecProtocol/aztec-packages/pull/6330) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Typing return values of embedded_curve_ops (https://github.com/AztecProtocol/aztec-packages/pull/7413) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Add support for nested arrays returned by oracles ([#5132](https://github.com/noir-lang/noir/issues/5132)) ([f846879](https://github.com/noir-lang/noir/commit/f846879dd038328bd0a1d39a72b448ef52a1002b))
* Add trailing extra arguments for backend in gates_flamegraph (https://github.com/AztecProtocol/aztec-packages/pull/7472) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Avoid unnecessarily splitting expressions with multiplication terms with a shared term ([#5291](https://github.com/noir-lang/noir/issues/5291)) ([19884f1](https://github.com/noir-lang/noir/commit/19884f161dfc7d7ce75dd2c404b8ef39cdad2240))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Check for public args in aztec functions (https://github.com/AztecProtocol/aztec-packages/pull/6355) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Handle struct with nested arrays in oracle return values ([#5244](https://github.com/noir-lang/noir/issues/5244)) ([a30814f](https://github.com/noir-lang/noir/commit/a30814f1f767bf874cd7e2969f5061c68f16b9a7))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Revert "feat: Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7512)" (https://github.com/AztecProtocol/aztec-packages/pull/7558) ([daad75c](https://github.com/noir-lang/noir/commit/daad75c26d19ae707b90a7424b77dab9937e8575))
* Runtime brillig bigint id assignment ([#5369](https://github.com/noir-lang/noir/issues/5369)) ([a8928dd](https://github.com/noir-lang/noir/commit/a8928ddcffcae15babf7aa5aff0e462e4549552e))
* Temporarily revert to_radix blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6304) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))


### Miscellaneous Chores

* Remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))

## [0.48.0](https://github.com/noir-lang/noir/compare/v0.47.0...v0.48.0) (2024-07-18)


### ⚠ BREAKING CHANGES

* constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222)
* add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205))
* restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180))
* switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283)
* specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239)
* remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995)
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016)
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)

### Features

* `multi_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6097) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* `variable_base_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6039) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Activate return_data in ACIR opcodes ([#5080](https://github.com/noir-lang/noir/issues/5080)) ([c9fda3c](https://github.com/noir-lang/noir/commit/c9fda3c7fd4575bfe7d457e8d4230e071f0129a0))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add native rust implementation of schnorr signature verification ([#5053](https://github.com/noir-lang/noir/issues/5053)) ([fab1c35](https://github.com/noir-lang/noir/commit/fab1c3567d731ea7902635a7a020a8d14f94fd27))
* Add native rust implementations of pedersen functions ([#4871](https://github.com/noir-lang/noir/issues/4871)) ([fb039f7](https://github.com/noir-lang/noir/commit/fb039f74df23aea39bc0593a5d538d82b4efadf0))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205)) ([14adafc](https://github.com/noir-lang/noir/commit/14adafc965fa9c833e096ec037e086aae67703ad))
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Constant inputs for blackbox (https://github.com/AztecProtocol/aztec-packages/pull/7222) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Dynamic assertion payloads v2 (https://github.com/AztecProtocol/aztec-packages/pull/5949) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Increase default expression width to 4 ([#4995](https://github.com/noir-lang/noir/issues/4995)) ([f01d309](https://github.com/noir-lang/noir/commit/f01d3090759a5ff0f1f83c5616d22890c6bd76be))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Make ACVM generic across fields ([#5114](https://github.com/noir-lang/noir/issues/5114)) ([70f374c](https://github.com/noir-lang/noir/commit/70f374c06642962d8f2b95b80f8c938fcf7761d7))
* Move abi demonomorphizer to noir_codegen and use noir_codegen in protocol types (https://github.com/AztecProtocol/aztec-packages/pull/6302) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Move to_radix to a blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6294) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` (https://github.com/AztecProtocol/aztec-packages/pull/6131) ([ff67e14](https://github.com/noir-lang/noir/commit/ff67e145d086bf6fdf58fb5e57927033e52e03d3))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Parsing non-string assertion payloads in noir js (https://github.com/AztecProtocol/aztec-packages/pull/6079) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Private Kernel Recursion (https://github.com/AztecProtocol/aztec-packages/pull/6278) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Proper padding in ts AES and constrained AES in body and header computations (https://github.com/AztecProtocol/aztec-packages/pull/6269) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Remove conditional compilation of `bn254_blackbox_solver` ([#5058](https://github.com/noir-lang/noir/issues/5058)) ([9420d7c](https://github.com/noir-lang/noir/commit/9420d7c2ba6bbbf5ecb9a066837c505310955b6c))
* Remove external blackbox solver from acir simulator (https://github.com/AztecProtocol/aztec-packages/pull/6586) ([a40a9a5](https://github.com/noir-lang/noir/commit/a40a9a55571deed386688fb84260bdf2794d4d38))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180)) ([bdb2bc6](https://github.com/noir-lang/noir/commit/bdb2bc608ea8fd52d46545a38b68dd2558b28110))
* Separate runtimes of SSA functions before inlining ([#5121](https://github.com/noir-lang/noir/issues/5121)) ([69eca9b](https://github.com/noir-lang/noir/commit/69eca9b8671fa54192bef814dd584fdb5387a5f7))
* Set aztec private functions to be recursive (https://github.com/AztecProtocol/aztec-packages/pull/6192) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6280) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6332) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6573) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7392) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/7400) ([fb97bb9](https://github.com/noir-lang/noir/commit/fb97bb9b795c9d7af395b82fd6f0ea8111d59c11))
* ToRadix BB + avm transpiler support (https://github.com/AztecProtocol/aztec-packages/pull/6330) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Add support for nested arrays returned by oracles ([#5132](https://github.com/noir-lang/noir/issues/5132)) ([f846879](https://github.com/noir-lang/noir/commit/f846879dd038328bd0a1d39a72b448ef52a1002b))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Avoid unnecessarily splitting expressions with multiplication terms with a shared term ([#5291](https://github.com/noir-lang/noir/issues/5291)) ([19884f1](https://github.com/noir-lang/noir/commit/19884f161dfc7d7ce75dd2c404b8ef39cdad2240))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Check for public args in aztec functions (https://github.com/AztecProtocol/aztec-packages/pull/6355) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Handle struct with nested arrays in oracle return values ([#5244](https://github.com/noir-lang/noir/issues/5244)) ([a30814f](https://github.com/noir-lang/noir/commit/a30814f1f767bf874cd7e2969f5061c68f16b9a7))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Move BigInt modulus checks to runtime in brillig ([#5374](https://github.com/noir-lang/noir/issues/5374)) ([741d339](https://github.com/noir-lang/noir/commit/741d33991f8e2918bf092c354ca56047e0274533))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Runtime brillig bigint id assignment ([#5369](https://github.com/noir-lang/noir/issues/5369)) ([a8928dd](https://github.com/noir-lang/noir/commit/a8928ddcffcae15babf7aa5aff0e462e4549552e))
* Temporarily revert to_radix blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6304) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))


### Miscellaneous Chores

* Remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))

## [0.47.0](https://github.com/noir-lang/noir/compare/v0.46.0...v0.47.0) (2024-06-17)


### ⚠ BREAKING CHANGES

* add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205))
* restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180))
* switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283)
* specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239)
* remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995)
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016)
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)

### Features

* `multi_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6097) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* `variable_base_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6039) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Activate return_data in ACIR opcodes ([#5080](https://github.com/noir-lang/noir/issues/5080)) ([c9fda3c](https://github.com/noir-lang/noir/commit/c9fda3c7fd4575bfe7d457e8d4230e071f0129a0))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add native rust implementation of schnorr signature verification ([#5053](https://github.com/noir-lang/noir/issues/5053)) ([fab1c35](https://github.com/noir-lang/noir/commit/fab1c3567d731ea7902635a7a020a8d14f94fd27))
* Add native rust implementations of pedersen functions ([#4871](https://github.com/noir-lang/noir/issues/4871)) ([fb039f7](https://github.com/noir-lang/noir/commit/fb039f74df23aea39bc0593a5d538d82b4efadf0))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add session id to foreign call RPC requests ([#5205](https://github.com/noir-lang/noir/issues/5205)) ([14adafc](https://github.com/noir-lang/noir/commit/14adafc965fa9c833e096ec037e086aae67703ad))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Dynamic assertion payloads v2 (https://github.com/AztecProtocol/aztec-packages/pull/5949) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Increase default expression width to 4 ([#4995](https://github.com/noir-lang/noir/issues/4995)) ([f01d309](https://github.com/noir-lang/noir/commit/f01d3090759a5ff0f1f83c5616d22890c6bd76be))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Make ACVM generic across fields ([#5114](https://github.com/noir-lang/noir/issues/5114)) ([70f374c](https://github.com/noir-lang/noir/commit/70f374c06642962d8f2b95b80f8c938fcf7761d7))
* Move abi demonomorphizer to noir_codegen and use noir_codegen in protocol types (https://github.com/AztecProtocol/aztec-packages/pull/6302) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Move to_radix to a blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6294) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` (https://github.com/AztecProtocol/aztec-packages/pull/6131) ([ff67e14](https://github.com/noir-lang/noir/commit/ff67e145d086bf6fdf58fb5e57927033e52e03d3))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Parsing non-string assertion payloads in noir js (https://github.com/AztecProtocol/aztec-packages/pull/6079) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Private Kernel Recursion (https://github.com/AztecProtocol/aztec-packages/pull/6278) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Proper padding in ts AES and constrained AES in body and header computations (https://github.com/AztecProtocol/aztec-packages/pull/6269) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Remove conditional compilation of `bn254_blackbox_solver` ([#5058](https://github.com/noir-lang/noir/issues/5058)) ([9420d7c](https://github.com/noir-lang/noir/commit/9420d7c2ba6bbbf5ecb9a066837c505310955b6c))
* Remove external blackbox solver from acir simulator (https://github.com/AztecProtocol/aztec-packages/pull/6586) ([a40a9a5](https://github.com/noir-lang/noir/commit/a40a9a55571deed386688fb84260bdf2794d4d38))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Restrict noir word size to u32 ([#5180](https://github.com/noir-lang/noir/issues/5180)) ([bdb2bc6](https://github.com/noir-lang/noir/commit/bdb2bc608ea8fd52d46545a38b68dd2558b28110))
* Separate runtimes of SSA functions before inlining ([#5121](https://github.com/noir-lang/noir/issues/5121)) ([69eca9b](https://github.com/noir-lang/noir/commit/69eca9b8671fa54192bef814dd584fdb5387a5f7))
* Set aztec private functions to be recursive (https://github.com/AztecProtocol/aztec-packages/pull/6192) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Specify databus arrays for BB (https://github.com/AztecProtocol/aztec-packages/pull/6239) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Switch `bb` over to read ACIR from nargo artifacts (https://github.com/AztecProtocol/aztec-packages/pull/6283) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6280) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6332) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/6573) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* ToRadix BB + avm transpiler support (https://github.com/AztecProtocol/aztec-packages/pull/6330) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Add support for nested arrays returned by oracles ([#5132](https://github.com/noir-lang/noir/issues/5132)) ([f846879](https://github.com/noir-lang/noir/commit/f846879dd038328bd0a1d39a72b448ef52a1002b))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Check for public args in aztec functions (https://github.com/AztecProtocol/aztec-packages/pull/6355) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Temporarily revert to_radix blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6304) ([436bbda](https://github.com/noir-lang/noir/commit/436bbdaadb2a294b94f93e53d7d3cad3859c7e46))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.46.0](https://github.com/noir-lang/noir/compare/v0.45.0...v0.46.0) (2024-05-20)


### ⚠ BREAKING CHANGES

* remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995)
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016)
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))

### Features

* `multi_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6097) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* `variable_base_scalar_mul` blackbox func (https://github.com/AztecProtocol/aztec-packages/pull/6039) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add native rust implementations of pedersen functions ([#4871](https://github.com/noir-lang/noir/issues/4871)) ([fb039f7](https://github.com/noir-lang/noir/commit/fb039f74df23aea39bc0593a5d538d82b4efadf0))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* AES blackbox (https://github.com/AztecProtocol/aztec-packages/pull/6016) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Dynamic assertion payloads v2 (https://github.com/AztecProtocol/aztec-packages/pull/5949) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Increase default expression width to 4 ([#4995](https://github.com/noir-lang/noir/issues/4995)) ([f01d309](https://github.com/noir-lang/noir/commit/f01d3090759a5ff0f1f83c5616d22890c6bd76be))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Parsing non-string assertion payloads in noir js (https://github.com/AztecProtocol/aztec-packages/pull/6079) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Set aztec private functions to be recursive (https://github.com/AztecProtocol/aztec-packages/pull/6192) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove `Opcode::Brillig` from ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5995) ([73a635e](https://github.com/noir-lang/noir/commit/73a635e5086cf3407f9846ce39807cd15b4e485a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.45.0](https://github.com/noir-lang/noir/compare/v0.44.0...v0.45.0) (2024-05-03)


### ⚠ BREAKING CHANGES

* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907))
* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))

### Features

* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Bit shift is restricted to u8 right operand ([#4907](https://github.com/noir-lang/noir/issues/4907)) ([c4b0369](https://github.com/noir-lang/noir/commit/c4b03691feca17ef268acab523292f3051f672ea))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Handle `BrilligCall` opcodes in the debugger ([#4897](https://github.com/noir-lang/noir/issues/4897)) ([b380dc4](https://github.com/noir-lang/noir/commit/b380dc44de5c9f8de278ece3d531ebbc2c9238ba))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.44.0](https://github.com/noir-lang/noir/compare/v0.43.0...v0.44.0) (2024-04-24)


### ⚠ BREAKING CHANGES

* contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687)
* change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374)
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620)
* trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732)
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709)
* remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617)
* storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387)
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616)
* contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386)
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))

### Features

* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Brillig stdlib ([#4848](https://github.com/noir-lang/noir/issues/4848)) ([0c8175c](https://github.com/noir-lang/noir/commit/0c8175cb539efd9427c73ae5af0d48abe688ebab))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Add predicate to call opcode (https://github.com/AztecProtocol/aztec-packages/pull/5616) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **acir:** BrilligCall opcode  (https://github.com/AztecProtocol/aztec-packages/pull/5709) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add return values to aztec fns (https://github.com/AztecProtocol/aztec-packages/pull/5389) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **avm:** Integrate AVM with initializers (https://github.com/AztecProtocol/aztec-packages/pull/5469) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Brillig heterogeneous memory cells (https://github.com/AztecProtocol/aztec-packages/pull/5608) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig pointer codegen and execution (https://github.com/AztecProtocol/aztec-packages/pull/5737) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Change backend width to 4 (https://github.com/AztecProtocol/aztec-packages/pull/5374) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Contract interfaces and better function calls (https://github.com/AztecProtocol/aztec-packages/pull/5687) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Contract_abi-exports (https://github.com/AztecProtocol/aztec-packages/pull/5386) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Impl of missing functionality in new key store (https://github.com/AztecProtocol/aztec-packages/pull/5750) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* **nargo:** Handle call stacks for multiple Acir calls ([#4711](https://github.com/noir-lang/noir/issues/4711)) ([5b23171](https://github.com/noir-lang/noir/commit/5b231714740447d82cde7cdbe65d4a8b46a31df4))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Restore hashing args via slice for performance (https://github.com/AztecProtocol/aztec-packages/pull/5539) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **simulator:** Fetch return values at circuit execution (https://github.com/AztecProtocol/aztec-packages/pull/5642) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Storage_layout and `#[aztec(storage)]` (https://github.com/AztecProtocol/aztec-packages/pull/5387) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5572) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5619) ([2bd006a](https://github.com/noir-lang/noir/commit/2bd006ae07499e8702b0fa9565855f0a5ef1a589))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5697) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5794) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5814) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5935) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5955) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5999) ([1b867b1](https://github.com/noir-lang/noir/commit/1b867b121fba5db3087ca845b4934e6732b23fd1))
* Trap with revert data (https://github.com/AztecProtocol/aztec-packages/pull/5732) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Use fixed size arrays in black box functions where sizes are known (https://github.com/AztecProtocol/aztec-packages/pull/5620) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Variable length returns (https://github.com/AztecProtocol/aztec-packages/pull/5633) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Avoid huge unrolling in hash_args (https://github.com/AztecProtocol/aztec-packages/pull/5703) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Catch panics from EC point creation (e.g. the point is at infinity) ([#4790](https://github.com/noir-lang/noir/issues/4790)) ([645dba1](https://github.com/noir-lang/noir/commit/645dba192f16ef34018828186ffb297422a8dc73))
* Don't reuse brillig with slice arguments (https://github.com/AztecProtocol/aztec-packages/pull/5800) ([0f9ae0a](https://github.com/noir-lang/noir/commit/0f9ae0ac1d68714b56ba4524aedcc67212494f1b))
* Issue 4682 and add solver for unconstrained bigintegers ([#4729](https://github.com/noir-lang/noir/issues/4729)) ([e4d33c1](https://github.com/noir-lang/noir/commit/e4d33c126a2795d9aaa6048d4e91b64cb4bbe4f2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Proper field inversion for bigints ([#4802](https://github.com/noir-lang/noir/issues/4802)) ([b46d0e3](https://github.com/noir-lang/noir/commit/b46d0e39f4252f8bbaa987f88d112e4c233b3d61))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove fixed-length keccak256 (https://github.com/AztecProtocol/aztec-packages/pull/5617) ([305bcdc](https://github.com/noir-lang/noir/commit/305bcdcbd01cb84dbaac900f14cb6cf867f83bda))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.43.0](https://github.com/noir-lang/noir/compare/v0.42.0...v0.43.0) (2024-04-10)


### ⚠ BREAKING CHANGES

* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395)
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))

### Features

* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR (https://github.com/AztecProtocol/aztec-packages/pull/5341) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **acvm_js:** Execute program  ([#4694](https://github.com/noir-lang/noir/issues/4694)) ([386f6d0](https://github.com/noir-lang/noir/commit/386f6d0a5822912db878285cb001032a7c0ff622))
* **acvm:** Execute multiple circuits  (https://github.com/AztecProtocol/aztec-packages/pull/5380) ([a0f7474](https://github.com/noir-lang/noir/commit/a0f7474ae6bd74132efdb945d2eb2383f3913cce))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Brillig typed memory (https://github.com/AztecProtocol/aztec-packages/pull/5395) ([0bc18c4](https://github.com/noir-lang/noir/commit/0bc18c4f78171590dd58bded959f68f53a44cc8c))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable ([#4708](https://github.com/noir-lang/noir/issues/4708)) ([8fea405](https://github.com/noir-lang/noir/commit/8fea40576f262bd5bb588923c0660d8967404e56))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.42.0](https://github.com/noir-lang/noir/compare/v0.41.0...v0.42.0) (2024-03-25)


### ⚠ BREAKING CHANGES

* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149)
* automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508)
* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773)
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175)
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Acir call opcode (https://github.com/AztecProtocol/aztec-packages/pull/4773) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **acir:** Program and witness stack structure (https://github.com/AztecProtocol/aztec-packages/pull/5149) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add CMOV instruction to brillig and brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5308) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* Automatic NoteInterface and NoteGetterOptions auto select (https://github.com/AztecProtocol/aztec-packages/pull/4508) ([13eb71b](https://github.com/noir-lang/noir/commit/13eb71b8de44eb6aad9c37943ad06fc73db589f5))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **avm:** Brillig CONST of size &gt; u128 (https://github.com/AztecProtocol/aztec-packages/pull/5217) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Brillig IR refactor (https://github.com/AztecProtocol/aztec-packages/pull/5233) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Check initializer msg.sender matches deployer from address preimage (https://github.com/AztecProtocol/aztec-packages/pull/5222) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Initial Earthly CI (https://github.com/AztecProtocol/aztec-packages/pull/5069) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* New brillig field operations and refactor of binary operations (https://github.com/AztecProtocol/aztec-packages/pull/5208) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Signed integer division and modulus in brillig gen (https://github.com/AztecProtocol/aztec-packages/pull/5279) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))
* Support contracts with no constructor (https://github.com/AztecProtocol/aztec-packages/pull/5175) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5234) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))
* Sync from noir (https://github.com/AztecProtocol/aztec-packages/pull/5286) ([c3c9e19](https://github.com/noir-lang/noir/commit/c3c9e19a20d61272a04b95fd6c7d34cc4cb96e45))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.41.0](https://github.com/noir-lang/noir/compare/v0.40.0...v0.41.0) (2024-03-11)


### ⚠ BREAKING CHANGES

* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898)
* move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479)
* note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500)
* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add poseidon2 opcode implementation for acvm/brillig, and Noir ([#4398](https://github.com/noir-lang/noir/issues/4398)) ([10e8292](https://github.com/noir-lang/noir/commit/10e82920798380f50046e52db4a20ca205191ab7))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Backpropagate constants in ACIR during optimization ([#3926](https://github.com/noir-lang/noir/issues/3926)) ([aad0da0](https://github.com/noir-lang/noir/commit/aad0da024c69663f42e6913e674682d5864b26ae))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Internal as a macro (https://github.com/AztecProtocol/aztec-packages/pull/4898) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Note type ids (https://github.com/AztecProtocol/aztec-packages/pull/4500) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))
* Sync from aztec-packages ([#4483](https://github.com/noir-lang/noir/issues/4483)) ([fe8f277](https://github.com/noir-lang/noir/commit/fe8f2776ccfde29209a2c3fc162311c99e4f59be))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))
* Noir test incorrect reporting (https://github.com/AztecProtocol/aztec-packages/pull/4925) ([5f57ebb](https://github.com/noir-lang/noir/commit/5f57ebb7ff4b810802f90699a10f4325ef904f2e))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Move noir out of yarn-project (https://github.com/AztecProtocol/aztec-packages/pull/4479) ([78ef013](https://github.com/noir-lang/noir/commit/78ef0134b82e76a73dadb6c7975def22290e3a1a))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.40.0](https://github.com/noir-lang/noir/compare/v0.39.0...v0.40.0) (2024-02-12)


### ⚠ BREAKING CHANGES

* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221)
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Add instrumentation for tracking variables in debugging ([#4122](https://github.com/noir-lang/noir/issues/4122)) ([c58d691](https://github.com/noir-lang/noir/commit/c58d69141b54a918cd1675400c00bfd48720f896))
* Add support for overriding expression width ([#4117](https://github.com/noir-lang/noir/issues/4117)) ([c8026d5](https://github.com/noir-lang/noir/commit/c8026d557d535b10fe455165d6445076df7a03de))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Allow variables and stack trace inspection in the debugger ([#4184](https://github.com/noir-lang/noir/issues/4184)) ([bf263fc](https://github.com/noir-lang/noir/commit/bf263fc8d843940f328a90f6366edd2671fb2682))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Evaluation of dynamic assert messages ([#4101](https://github.com/noir-lang/noir/issues/4101)) ([c284e01](https://github.com/noir-lang/noir/commit/c284e01bfe20ceae4414dc123624b5cbb8b66d09))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Remove replacement of boolean range opcodes with `AssertZero` opcodes ([#4107](https://github.com/noir-lang/noir/issues/4107)) ([dac0e87](https://github.com/noir-lang/noir/commit/dac0e87ee3be3446b92bbb12ef4832fd493fcee3))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))
* Sync commits from `aztec-packages` ([#4144](https://github.com/noir-lang/noir/issues/4144)) ([0205d3b](https://github.com/noir-lang/noir/commit/0205d3b4ad0cf5ffd775a43eb5af273a772cf138))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))
* Remove panic from `init_log_level` in `acvm_js` ([#4195](https://github.com/noir-lang/noir/issues/4195)) ([2e26530](https://github.com/noir-lang/noir/commit/2e26530bf53006c1ed4fee310bcaa905c95dd95b))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([158c8ce](https://github.com/noir-lang/noir/commit/158c8cec7f0dc698042e9512001dd2c9d6b40bcc))

## [0.39.0](https://github.com/noir-lang/noir/compare/v0.38.0...v0.39.0) (2024-01-22)


### ⚠ BREAKING CHANGES

* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Breaking changes from aztec-packages ([#3955](https://github.com/noir-lang/noir/issues/3955)) ([5be049e](https://github.com/noir-lang/noir/commit/5be049eee6c342649462282ee04f6411e6ea392c))
* Remove range constraints from witnesses which are constrained to be constants ([#3928](https://github.com/noir-lang/noir/issues/3928)) ([afe9c7a](https://github.com/noir-lang/noir/commit/afe9c7a38bb9d4245205d3aa46d4ce23d70a5671))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))
* Sync `aztec-packages` ([#4011](https://github.com/noir-lang/noir/issues/4011)) ([fee2452](https://github.com/noir-lang/noir/commit/fee24523c427c27f0bdaf98ea09a852a2da3e94c))
* Sync commits from `aztec-packages` ([#4068](https://github.com/noir-lang/noir/issues/4068)) ([7a8f3a3](https://github.com/noir-lang/noir/commit/7a8f3a33b57875e681e3d81e667e3570a1cdbdcc))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))
* Return error rather instead of panicking on invalid circuit ([#3976](https://github.com/noir-lang/noir/issues/3976)) ([67201bf](https://github.com/noir-lang/noir/commit/67201bfc21a9c8858aa86be9cd47d463fb78d925))


### Miscellaneous Chores

* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))
* Rename Arithmetic opcode to AssertZero ([#3840](https://github.com/noir-lang/noir/issues/3840)) ([836f171](https://github.com/noir-lang/noir/commit/836f17145c2901060706294461c2d282dd121b3e))

## [0.38.0](https://github.com/noir-lang/noir/compare/v0.37.1...v0.38.0) (2023-12-18)


### ⚠ BREAKING CHANGES

* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841))
* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805))

### Features

* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))


### Miscellaneous Chores

* Remove partial backend feature ([#3805](https://github.com/noir-lang/noir/issues/3805)) ([0383100](https://github.com/noir-lang/noir/commit/0383100853a80a5b28b797cdfeae0d271f1b7805))
* Remove unused methods on ACIR opcodes ([#3841](https://github.com/noir-lang/noir/issues/3841)) ([9e5d0e8](https://github.com/noir-lang/noir/commit/9e5d0e813d61a0bfb5ee68174ed287c5a20f1579))

## [0.37.1](https://github.com/noir-lang/noir/compare/v0.37.0...v0.37.1) (2023-12-15)


### Features

* Aztec-packages ([#3754](https://github.com/noir-lang/noir/issues/3754)) ([c043265](https://github.com/noir-lang/noir/commit/c043265e550b59bd4296504826fe15d3ce3e9ad2))
* Speed up transformation of debug messages ([#3815](https://github.com/noir-lang/noir/issues/3815)) ([2a8af1e](https://github.com/noir-lang/noir/commit/2a8af1e4141ffff61547ee1c2837a6392bd5db48))


### Bug Fixes

* Deserialize odd length hex literals ([#3747](https://github.com/noir-lang/noir/issues/3747)) ([4000fb2](https://github.com/noir-lang/noir/commit/4000fb279221eb07187d657bfaa7f1c7b311abf2))

## [0.37.0](https://github.com/noir-lang/noir/compare/v0.36.0...v0.37.0) (2023-12-01)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add `FieldElement::from&lt;usize&gt;` implementation ([#3647](https://github.com/noir-lang/noir/issues/3647)) ([8b7c5aa](https://github.com/noir-lang/noir/commit/8b7c5aa5311f4e6811438f67bd552b641b13fc9a))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Aztec-packages ([#3599](https://github.com/noir-lang/noir/issues/3599)) ([2cd6dc3](https://github.com/noir-lang/noir/commit/2cd6dc39e3a956aa5dff721d47aaf1921f98fded))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.36.0](https://github.com/noir-lang/noir/compare/v0.35.0...v0.36.0) (2023-12-01)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add `FieldElement::from&lt;usize&gt;` implementation ([#3647](https://github.com/noir-lang/noir/issues/3647)) ([8b7c5aa](https://github.com/noir-lang/noir/commit/8b7c5aa5311f4e6811438f67bd552b641b13fc9a))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Aztec-packages ([#3599](https://github.com/noir-lang/noir/issues/3599)) ([2cd6dc3](https://github.com/noir-lang/noir/commit/2cd6dc39e3a956aa5dff721d47aaf1921f98fded))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.35.0](https://github.com/noir-lang/noir/compare/v0.34.0...v0.35.0) (2023-11-28)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.34.0](https://github.com/noir-lang/noir/compare/v0.33.0...v0.34.0) (2023-11-22)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Add debugger commands to introspect (and modify) the current state ([#3391](https://github.com/noir-lang/noir/issues/3391)) ([9e1ad85](https://github.com/noir-lang/noir/commit/9e1ad858cf8a1d9aba0137abe6a749267498bfaf))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.33.0](https://github.com/noir-lang/noir/compare/v0.32.0...v0.33.0) (2023-11-07)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.32.0](https://github.com/noir-lang/noir/compare/v0.31.0...v0.32.0) (2023-11-07)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.31.0](https://github.com/noir-lang/noir/compare/v0.30.0...v0.31.0) (2023-11-02)


### ⚠ BREAKING CHANGES

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345))
* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Extract Brillig VM to allow step debugging ([#3259](https://github.com/noir-lang/noir/issues/3259)) ([f6431f9](https://github.com/noir-lang/noir/commit/f6431f99711f15a96a4f7fed2f413daece94b5e1))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Refactor debugger and separate core from UI ([#3308](https://github.com/noir-lang/noir/issues/3308)) ([8466810](https://github.com/noir-lang/noir/commit/846681079ab7295b201480a5c8baebc45e858c6f))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))


### Miscellaneous Chores

* Move circuit serialization circuit into acir ([#3345](https://github.com/noir-lang/noir/issues/3345)) ([122119b](https://github.com/noir-lang/noir/commit/122119b7377cec1b7c42c586c64b69b3bdf4d539))

## [0.30.0](https://github.com/noir-lang/noir/compare/v0.29.0...v0.30.0) (2023-10-25)


### ⚠ BREAKING CHANGES

* expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Expose pedersen hash in acir and bb solver ([#3269](https://github.com/noir-lang/noir/issues/3269)) ([0108b6c](https://github.com/noir-lang/noir/commit/0108b6c1e8dc0dfc766ab3c4944deae9354dec36))
* Implement euclidean division and signed division in terms of `AcirVar`s ([#3230](https://github.com/noir-lang/noir/issues/3230)) ([b8b7782](https://github.com/noir-lang/noir/commit/b8b77825410c0e1f95549259a51e2c40de1ec342))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Replace boolean range constraints with arithmetic opcodes ([#3234](https://github.com/noir-lang/noir/issues/3234)) ([949222c](https://github.com/noir-lang/noir/commit/949222c20d9e65152e3814d02da1c4c41ffc23a5))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* Switch to new pedersen implementation ([#3151](https://github.com/noir-lang/noir/issues/3151)) ([35fb3f7](https://github.com/noir-lang/noir/commit/35fb3f7076d52db7ca3bef0a70a3dbccaf82f58d))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))

## [0.29.0](https://github.com/noir-lang/noir/compare/v0.28.0...v0.29.0) (2023-10-20)


### ⚠ BREAKING CHANGES

* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872))
* **wasm:** improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* **acvm:** Separate ACVM optimizations and transformations ([#2979](https://github.com/noir-lang/noir/issues/2979)) ([5865d1a](https://github.com/noir-lang/noir/commit/5865d1a1bca16e1853663c71f893ff81fa3f7185))
* Add ACIR serializer C++ codegen ([#2961](https://github.com/noir-lang/noir/issues/2961)) ([7556982](https://github.com/noir-lang/noir/commit/7556982dbebe25eaa17240abbe270b771b55de45))
* Add conditional compilation of methods based on the underlying field being used  ([#3045](https://github.com/noir-lang/noir/issues/3045)) ([2e008e2](https://github.com/noir-lang/noir/commit/2e008e2438795bbc41b0641e830378b76bf2e194))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))
* Pass ACIR to ACVM by reference rather than passing ownership ([#2872](https://github.com/noir-lang/noir/issues/2872)) ([b3a9c34](https://github.com/noir-lang/noir/commit/b3a9c343993ce3207de62106bda6cb2b2ef3de50))
* Pass brillig bytecode to VM by reference ([#3030](https://github.com/noir-lang/noir/issues/3030)) ([4ee290b](https://github.com/noir-lang/noir/commit/4ee290b8b6f75bc1974a5750248570eeca8d244e))
* Save Brillig execution state in ACVM ([#3026](https://github.com/noir-lang/noir/issues/3026)) ([88682da](https://github.com/noir-lang/noir/commit/88682da87ffc9e26da5c9e4b5a4d8e62a6ee43c6))
* Solve `fixed_base_scalar_mul` black box functions in rust ([#3153](https://github.com/noir-lang/noir/issues/3153)) ([1c1afbc](https://github.com/noir-lang/noir/commit/1c1afbcddf0b5fdb39f00ad28ae90caf699d1265))
* **wasm:** Improve and simplify wasm compiler interface ([#2976](https://github.com/noir-lang/noir/issues/2976)) ([1b5124b](https://github.com/noir-lang/noir/commit/1b5124bc74f7ac5360db04b34d1b7b2284061fd3))


### Bug Fixes

* ACIR optimizer should update assertion messages ([#3010](https://github.com/noir-lang/noir/issues/3010)) ([758b6b6](https://github.com/noir-lang/noir/commit/758b6b62918907c1a39f3090a77419003551745e))
* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))
* Determinism of fallback transformer ([#3100](https://github.com/noir-lang/noir/issues/3100)) ([12daad1](https://github.com/noir-lang/noir/commit/12daad19c902caf5ee9e2eb4b6847bde5a924353))
* Fix method `program_counter`, change method signature ([#3012](https://github.com/noir-lang/noir/issues/3012)) ([5ea522b](https://github.com/noir-lang/noir/commit/5ea522b840ca0f6f90d02ca00f0de32f515d450f))
* Minor problems with `aztec` publishing ([#3095](https://github.com/noir-lang/noir/issues/3095)) ([0fc8f20](https://github.com/noir-lang/noir/commit/0fc8f20b8b87d033d27ce18db039399c17f81837))
* Prevent duplicated assert message transformation ([#3038](https://github.com/noir-lang/noir/issues/3038)) ([082a6d0](https://github.com/noir-lang/noir/commit/082a6d02dad67a25692bed15c340a16a848a320e))
* Return error rather than panicking on unreadable circuits ([#3179](https://github.com/noir-lang/noir/issues/3179)) ([d4f61d3](https://github.com/noir-lang/noir/commit/d4f61d3d51d515e40a5fd02d35315889f841bf53))

## [0.28.0](https://github.com/noir-lang/noir/compare/v0.27.4...v0.28.0) (2023-10-03)


### ⚠ BREAKING CHANGES

* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935))

### Features

* **acvm_js:** Export black box solver functions ([#2812](https://github.com/noir-lang/noir/issues/2812)) ([da8a98e](https://github.com/noir-lang/noir/commit/da8a98ed312fe69cb0bdb8f9d0a70ee7a981398f))
* Maintain shape of foreign call arguments ([#2935](https://github.com/noir-lang/noir/issues/2935)) ([f7869e6](https://github.com/noir-lang/noir/commit/f7869e6fb492b617e776e538ac4babfa56261d26))


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))

## [0.27.4](https://github.com/noir-lang/noir/compare/v0.27.3...v0.27.4) (2023-09-28)


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))

## [0.27.3](https://github.com/noir-lang/noir/compare/v0.27.2...v0.27.3) (2023-09-27)


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))

## [0.27.2](https://github.com/noir-lang/noir/compare/v0.27.1...v0.27.2) (2023-09-27)


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))

## [0.27.1](https://github.com/noir-lang/noir/compare/v0.27.0...v0.27.1) (2023-09-26)


### Bug Fixes

* **acvm:** Return false rather than panicking on invalid ECDSA signatures ([#2783](https://github.com/noir-lang/noir/issues/2783)) ([155abc0](https://github.com/noir-lang/noir/commit/155abc0d99fff41c79163c16bf297d41e5dff0fa))

## [0.27.0](https://github.com/noir-lang/acvm/compare/root-v0.26.1...root-v0.27.0) (2023-09-19)


### ⚠ BREAKING CHANGES

* Separate barretenberg solver from generic blackbox solver code ([#554](https://github.com/noir-lang/acvm/issues/554))

### Features

* **acir:** add method on `Circuit` to return assert message ([#551](https://github.com/noir-lang/acvm/issues/551)) ([ee18cde](https://github.com/noir-lang/acvm/commit/ee18cde3537b2be6714061af0bc9ef3793929f7f))


### Bug Fixes

* bump brillig_vm version in release please ([#556](https://github.com/noir-lang/acvm/issues/556)) ([f6c7823](https://github.com/noir-lang/acvm/commit/f6c7823b3be2b85a6ce8dc4af7a3b57ee7a577db))
* use the exact version for the hex crate ([#546](https://github.com/noir-lang/acvm/issues/546)) ([2a546e5](https://github.com/noir-lang/acvm/commit/2a546e5b5cc9f39737ad81f8e96d58313a74eced))


### Miscellaneous Chores

* Separate barretenberg solver from generic blackbox solver code ([#554](https://github.com/noir-lang/acvm/issues/554)) ([a4b9772](https://github.com/noir-lang/acvm/commit/a4b97722a0892fe379ff075e6080675adafdce0e))

## [0.26.1](https://github.com/noir-lang/acvm/compare/root-v0.26.0...root-v0.26.1) (2023-09-12)


### Bug Fixes

* Implements handling of the high limb during fixed base scalar multiplication ([#535](https://github.com/noir-lang/acvm/issues/535)) ([551504a](https://github.com/noir-lang/acvm/commit/551504aa572d3f9d56b5576d25ce1211296ee488))

## [0.26.0](https://github.com/noir-lang/acvm/compare/root-v0.25.0...root-v0.26.0) (2023-09-07)


### ⚠ BREAKING CHANGES

* Add a low and high limb to scalar mul opcode ([#532](https://github.com/noir-lang/acvm/issues/532))

### Miscellaneous Chores

* Add a low and high limb to scalar mul opcode ([#532](https://github.com/noir-lang/acvm/issues/532)) ([b054f66](https://github.com/noir-lang/acvm/commit/b054f66be9c73d4e02dbecdab80874a907f19242))

## [0.25.0](https://github.com/noir-lang/acvm/compare/root-v0.24.1...root-v0.25.0) (2023-09-04)


### ⚠ BREAKING CHANGES

* Provide runtime callstacks for brillig failures and return errors in acvm_js ([#523](https://github.com/noir-lang/acvm/issues/523))

### Features

* Provide runtime callstacks for brillig failures and return errors in acvm_js ([#523](https://github.com/noir-lang/acvm/issues/523)) ([7ab7cff](https://github.com/noir-lang/acvm/commit/7ab7cff48a9aba61a97fad2a759fc8e55740b098))


### Bug Fixes

* initialize recursive proof output to zero ([#524](https://github.com/noir-lang/acvm/issues/524)) ([5453074](https://github.com/noir-lang/acvm/commit/545307457dd7634b20ea3977e2d2cc751eba06d2))

## [0.24.1](https://github.com/noir-lang/acvm/compare/root-v0.24.0...root-v0.24.1) (2023-09-03)


### Bug Fixes

* Add WASI 20 `_initialize` call to `acvm_backend.wasm` binary ([#518](https://github.com/noir-lang/acvm/issues/518)) ([ec6ab0c](https://github.com/noir-lang/acvm/commit/ec6ab0c6fb2753209abe1e03a449873e255ffd76))

## [0.24.0](https://github.com/noir-lang/acvm/compare/root-v0.23.0...root-v0.24.0) (2023-08-31)


### ⚠ BREAKING CHANGES

* **acvm:** Remove the `Backend` trait ([#514](https://github.com/noir-lang/acvm/issues/514))
* **acir:** Remove unused `Directive` opcodes ([#510](https://github.com/noir-lang/acvm/issues/510))
* **acir:** Add predicate to MemoryOp ([#503](https://github.com/noir-lang/acvm/issues/503))
* **acvm:** Remove unused arguments from `Backend` trait ([#511](https://github.com/noir-lang/acvm/issues/511))
* Assertion messages embedded in the circuit ([#484](https://github.com/noir-lang/acvm/issues/484))

### Features

* **acir:** Add predicate to MemoryOp ([#503](https://github.com/noir-lang/acvm/issues/503)) ([ca9eebe](https://github.com/noir-lang/acvm/commit/ca9eebe34e61adabf97318c8ccaf60c8a424aafd))
* Assertion messages embedded in the circuit ([#484](https://github.com/noir-lang/acvm/issues/484)) ([06b97c5](https://github.com/noir-lang/acvm/commit/06b97c51041e16651cf8b2be8bc18214e276c6c9))


### Miscellaneous Chores

* **acir:** Remove unused `Directive` opcodes ([#510](https://github.com/noir-lang/acvm/issues/510)) ([cfd8cbf](https://github.com/noir-lang/acvm/commit/cfd8cbf58307511ac0cc9106c299695c2ca779de))
* **acvm:** Remove the `Backend` trait ([#514](https://github.com/noir-lang/acvm/issues/514)) ([681535d](https://github.com/noir-lang/acvm/commit/681535da52815a4a164ee4f48f7b48329664af98))
* **acvm:** Remove unused arguments from `Backend` trait ([#511](https://github.com/noir-lang/acvm/issues/511)) ([ae65355](https://github.com/noir-lang/acvm/commit/ae65355afb7df98c71f81d5a54e89f39f9333920))

## [0.23.0](https://github.com/noir-lang/acvm/compare/root-v0.22.0...root-v0.23.0) (2023-08-30)


### ⚠ BREAKING CHANGES

* Return an iterator from `new_locations()` instead of collecting ([#507](https://github.com/noir-lang/acvm/issues/507))
* **acvm:** remove `CommonReferenceString` trait and preprocess method ([#508](https://github.com/noir-lang/acvm/issues/508))
* **acvm:** Remove `BlackBoxFunctionSolver` from `Backend` trait ([#494](https://github.com/noir-lang/acvm/issues/494))
* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference

### Features

* **acvm_js:** Add `execute_circuit_with_black_box_solver` to prevent reinitialization of `BlackBoxFunctionSolver` ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))
* Expose a `BlackBoxFunctionSolver` containing a barretenberg wasm from `blackbox_solver` ([#494](https://github.com/noir-lang/acvm/issues/494)) ([a1d4b71](https://github.com/noir-lang/acvm/commit/a1d4b71256dfbf1e883e770dd9c45479235aa860))


### Miscellaneous Chores

* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))
* **acvm:** Remove `BlackBoxFunctionSolver` from `Backend` trait ([#494](https://github.com/noir-lang/acvm/issues/494)) ([a1d4b71](https://github.com/noir-lang/acvm/commit/a1d4b71256dfbf1e883e770dd9c45479235aa860))
* **acvm:** remove `CommonReferenceString` trait and preprocess method ([#508](https://github.com/noir-lang/acvm/issues/508)) ([3827dd3](https://github.com/noir-lang/acvm/commit/3827dd3ce487650843ba4df8337b423e39f97edf))
* Return an iterator from `new_locations()` instead of collecting ([#507](https://github.com/noir-lang/acvm/issues/507)) ([8d49a5c](https://github.com/noir-lang/acvm/commit/8d49a5c15b1e962cd59252467a20a922edadc2f2))

## [0.22.0](https://github.com/noir-lang/acvm/compare/root-v0.21.0...root-v0.22.0) (2023-08-18)


### ⚠ BREAKING CHANGES

* Switched from OpcodeLabel to OpcodeLocation and ErrorLocation ([#493](https://github.com/noir-lang/acvm/issues/493))
* **acvm:** check for index out-of-bounds on memory operations ([#468](https://github.com/noir-lang/acvm/issues/468))

### Features

* **acvm:** check for index out-of-bounds on memory operations ([#468](https://github.com/noir-lang/acvm/issues/468)) ([740468c](https://github.com/noir-lang/acvm/commit/740468c0a144f7179c38f615cfda31b2fcc77359))
* print error location with fmt ([#497](https://github.com/noir-lang/acvm/issues/497)) ([575a9e5](https://github.com/noir-lang/acvm/commit/575a9e50e97afb04a7b91799e06752cec3093f0b))
* Switched from OpcodeLabel to OpcodeLocation and ErrorLocation ([#493](https://github.com/noir-lang/acvm/issues/493)) ([27a5a93](https://github.com/noir-lang/acvm/commit/27a5a935849f8904e10056b08089f532a06962b8))


### Bug Fixes

* add opcode label to unsatisfied constrain string ([#482](https://github.com/noir-lang/acvm/issues/482)) ([cbbbe67](https://github.com/noir-lang/acvm/commit/cbbbe67b9a19a4a560b2dfa8f27ea1c6ebd61f28))

## [0.21.0](https://github.com/noir-lang/acvm/compare/root-v0.20.1...root-v0.21.0) (2023-07-26)


### ⚠ BREAKING CHANGES

* **acir:** Remove `Block`, `RAM` and `ROM` opcodes ([#457](https://github.com/noir-lang/acvm/issues/457))
* **acvm:** Remove `OpcodeResolution` enum ([#400](https://github.com/noir-lang/acvm/issues/400))
* **acvm:** Support stepwise execution of ACIR ([#399](https://github.com/noir-lang/acvm/issues/399))

### Features

* **acvm:** Remove `OpcodeResolution` enum ([#400](https://github.com/noir-lang/acvm/issues/400)) ([d0ce48c](https://github.com/noir-lang/acvm/commit/d0ce48c506619a5560412ef6693bfa11036b501e))
* **acvm:** Support stepwise execution of ACIR ([#399](https://github.com/noir-lang/acvm/issues/399)) ([6a03950](https://github.com/noir-lang/acvm/commit/6a0395021779a2711353c2fe2948e09b5b538fc0))


### Miscellaneous Chores

* **acir:** Remove `Block`, `RAM` and `ROM` opcodes ([#457](https://github.com/noir-lang/acvm/issues/457)) ([8dd220a](https://github.com/noir-lang/acvm/commit/8dd220ae127baf6cc5a31d8ab7ffdeeb161f6109))

## [0.20.1](https://github.com/noir-lang/acvm/compare/root-v0.20.0...root-v0.20.1) (2023-07-26)


### Features

* add optimizations to fallback black box functions on booleans ([#446](https://github.com/noir-lang/acvm/issues/446)) ([2cfb2a8](https://github.com/noir-lang/acvm/commit/2cfb2a8cf911a81eedbd9da13ab2c616abd67f83))
* **stdlib:** Add fallback implementation of `Keccak256` black box function ([#445](https://github.com/noir-lang/acvm/issues/445)) ([f7ebb03](https://github.com/noir-lang/acvm/commit/f7ebb03653c971f119700ff8126d9eb5ff01be0f))

## [0.20.0](https://github.com/noir-lang/acvm/compare/root-v0.19.1...root-v0.20.0) (2023-07-20)


### ⚠ BREAKING CHANGES

* atomic memory opcodes ([#447](https://github.com/noir-lang/acvm/issues/447))

### Features

* atomic memory opcodes ([#447](https://github.com/noir-lang/acvm/issues/447)) ([3261c7a](https://github.com/noir-lang/acvm/commit/3261c7a2fd4f3a300bc5f39ef4febccd8a853560))
* **brillig:** Support integers which fit inside a `FieldElement` ([#403](https://github.com/noir-lang/acvm/issues/403)) ([f992412](https://github.com/noir-lang/acvm/commit/f992412617ade875fa26fe3a2cc3c06dbcad503b))
* **stdlib:** Add fallback implementation of `HashToField128Security` black box function ([#435](https://github.com/noir-lang/acvm/issues/435)) ([ed40f22](https://github.com/noir-lang/acvm/commit/ed40f228529e888d1960bfa70cb92b277e24b37f))

## [0.19.1](https://github.com/noir-lang/acvm/compare/root-v0.19.0...root-v0.19.1) (2023-07-17)


### Bug Fixes

* Remove panic when we divide 0/0 in quotient directive ([#437](https://github.com/noir-lang/acvm/issues/437)) ([9c8ff64](https://github.com/noir-lang/acvm/commit/9c8ff64ebf27a86787ae184e10ed9581041ec0ff))

## [0.19.0](https://github.com/noir-lang/acvm/compare/root-v0.18.2...root-v0.19.0) (2023-07-15)


### ⚠ BREAKING CHANGES

* move to bincode and GzEncoding for artifacts ([#436](https://github.com/noir-lang/acvm/issues/436))

### Features

* move to bincode and GzEncoding for artifacts ([#436](https://github.com/noir-lang/acvm/issues/436)) ([4683240](https://github.com/noir-lang/acvm/commit/46832400a8bc20135a8a895ab9477b14449734d9))

## [0.18.2](https://github.com/noir-lang/acvm/compare/root-v0.18.1...root-v0.18.2) (2023-07-12)


### Features

* **acvm:** reexport `blackbox_solver` crate from `acvm` ([#431](https://github.com/noir-lang/acvm/issues/431)) ([517e942](https://github.com/noir-lang/acvm/commit/517e942b732d7107f6e064c6791917d1508229b3))
* **stdlib:** Add fallback implementation of `Blake2s` black box function ([#424](https://github.com/noir-lang/acvm/issues/424)) ([982d940](https://github.com/noir-lang/acvm/commit/982d94087d46092ce7a5e94dbd7e732195f58e42))

## [0.18.1](https://github.com/noir-lang/acvm/compare/root-v0.18.0...root-v0.18.1) (2023-07-12)


### Bug Fixes

* Crate publishing order ([#428](https://github.com/noir-lang/acvm/issues/428)) ([4f69cb5](https://github.com/noir-lang/acvm/commit/4f69cb5782435a2fcf45bb0985e1bb0eb944b194))

## [0.18.0](https://github.com/noir-lang/acvm/compare/root-v0.17.0...root-v0.18.0) (2023-07-12)


### ⚠ BREAKING CHANGES

* add backend-solvable blackboxes to brillig & unify implementations ([#422](https://github.com/noir-lang/acvm/issues/422))
* **acvm:** Remove `CircuitSimplifer` ([#421](https://github.com/noir-lang/acvm/issues/421))
* **acvm:** Add `circuit: &Circuit` to `eth_contract_from_vk` function signature ([#420](https://github.com/noir-lang/acvm/issues/420))
* Returns index of failing opcode and transformation mapping ([#412](https://github.com/noir-lang/acvm/issues/412))

### Features

* **acvm:** Add `circuit: &Circuit` to `eth_contract_from_vk` function signature ([#420](https://github.com/noir-lang/acvm/issues/420)) ([744e9da](https://github.com/noir-lang/acvm/commit/744e9da71f7ca477a5390a63f47211dd4dffb8b3))
* add backend-solvable blackboxes to brillig & unify implementations ([#422](https://github.com/noir-lang/acvm/issues/422)) ([093342e](https://github.com/noir-lang/acvm/commit/093342ea9481a311fa71343b8b7a22774788838a))
* derive PartialOrd, Ord, and Hash on RegisterIndex ([#425](https://github.com/noir-lang/acvm/issues/425)) ([7f6b0dc](https://github.com/noir-lang/acvm/commit/7f6b0dc138c4e11d2b5847f0c9603979cc43493a))
* Returns index of failing opcode and transformation mapping ([#412](https://github.com/noir-lang/acvm/issues/412)) ([79950e9](https://github.com/noir-lang/acvm/commit/79950e943f60e4082e1cf5ec4442aa67ea91aade))
* **stdlib:** Add fallback implementation of `SHA256` black box function ([#407](https://github.com/noir-lang/acvm/issues/407)) ([040369a](https://github.com/noir-lang/acvm/commit/040369adc8749fa5ec2edd255ff54c105c3140f5))


### Miscellaneous Chores

* **acvm:** Remove `CircuitSimplifier` ([#421](https://github.com/noir-lang/acvm/issues/421)) ([e07a56d](https://github.com/noir-lang/acvm/commit/e07a56d9c542a7f03ce156761054cd403de0bd23))

## [0.17.0](https://github.com/noir-lang/acvm/compare/root-v0.16.0...root-v0.17.0) (2023-07-07)


### ⚠ BREAKING CHANGES

* **acir:** add `EcdsaSecp256r1` blackbox function ([#408](https://github.com/noir-lang/acvm/issues/408))

### Features

* **acir:** add `EcdsaSecp256r1` blackbox function ([#408](https://github.com/noir-lang/acvm/issues/408)) ([9895817](https://github.com/noir-lang/acvm/commit/98958170c9fa9b4731e33b31cb494a72bb90549e))

## [0.16.0](https://github.com/noir-lang/acvm/compare/root-v0.15.1...root-v0.16.0) (2023-07-06)


### ⚠ BREAKING CHANGES

* **acvm:** replace `PartialWitnessGeneratorStatus` with `ACVMStatus` ([#410](https://github.com/noir-lang/acvm/issues/410))
* **acir:** revert changes to `SchnorrVerify` opcode ([#409](https://github.com/noir-lang/acvm/issues/409))
* **acvm:** Replace `PartialWitnessGenerator` trait with `BlackBoxFunctionSolver` ([#378](https://github.com/noir-lang/acvm/issues/378))
* **acvm:** Encapsulate internal state of ACVM within a struct ([#384](https://github.com/noir-lang/acvm/issues/384))
* remove unused `OpcodeResolutionError::IncorrectNumFunctionArguments` variant ([#397](https://github.com/noir-lang/acvm/issues/397))
* **acir:** Remove `Oracle` opcode ([#368](https://github.com/noir-lang/acvm/issues/368))
* **acir:** Use fixed length data structures in black box function inputs/outputs where possible. ([#386](https://github.com/noir-lang/acvm/issues/386))
* **acir:** Implement `Add` trait for `Witness` & make output of `Mul` on `Expression` optional ([#393](https://github.com/noir-lang/acvm/issues/393))

### Features

* **acir:** Implement `Add` trait for `Witness` & make output of `Mul` on `Expression` optional ([#393](https://github.com/noir-lang/acvm/issues/393)) ([5bcdfc6](https://github.com/noir-lang/acvm/commit/5bcdfc62e4936922135add171d60a948922581ff))
* **acir:** Remove `Oracle` opcode ([#368](https://github.com/noir-lang/acvm/issues/368)) ([63354df](https://github.com/noir-lang/acvm/commit/63354df1fe47a4f1128b91641d1b66dfc1281794))
* **acir:** Use fixed length data structures in black box function inputs/outputs where possible. ([#386](https://github.com/noir-lang/acvm/issues/386)) ([b139d4d](https://github.com/noir-lang/acvm/commit/b139d4d566c715009465a430aab0fb819aacab4f))
* **acvm:** Derive `Copy` for `Language` ([#406](https://github.com/noir-lang/acvm/issues/406)) ([69a6c22](https://github.com/noir-lang/acvm/commit/69a6c224d80be556ac5388ffeb7a02424df22031))
* **acvm:** Encapsulate internal state of ACVM within a struct ([#384](https://github.com/noir-lang/acvm/issues/384)) ([84d4867](https://github.com/noir-lang/acvm/commit/84d4867b2d97097d451d59174781555dafd2591f))
* **acvm:** Replace `PartialWitnessGenerator` trait with `BlackBoxFunctionSolver` ([#378](https://github.com/noir-lang/acvm/issues/378)) ([73fbc95](https://github.com/noir-lang/acvm/commit/73fbc95942b0039565c93719809975f66dc9ec53))
* **acvm:** replace `PartialWitnessGeneratorStatus` with `ACVMStatus` ([#410](https://github.com/noir-lang/acvm/issues/410)) ([fc3240d](https://github.com/noir-lang/acvm/commit/fc3240d456d0128f6eb42096beb8b7a586ea48da))
* **brillig:** implemented first blackbox functions ([#401](https://github.com/noir-lang/acvm/issues/401)) ([62d40f7](https://github.com/noir-lang/acvm/commit/62d40f7c03cd1102f615b8d565f82496962db637))


### Bug Fixes

* **acir:** revert changes to `SchnorrVerify` opcode ([#409](https://github.com/noir-lang/acvm/issues/409)) ([f1c7940](https://github.com/noir-lang/acvm/commit/f1c7940f4ac618c7b440b6ed30199f85cbe72cca))


### Miscellaneous Chores

* remove unused `OpcodeResolutionError::IncorrectNumFunctionArguments` variant ([#397](https://github.com/noir-lang/acvm/issues/397)) ([d1368d0](https://github.com/noir-lang/acvm/commit/d1368d041eb42d265a4ef385e066b82bc36d0743))

## [0.15.1](https://github.com/noir-lang/acvm/compare/root-v0.15.0...root-v0.15.1) (2023-06-20)


### Features

* **brillig:** Allow dynamic-size foreign calls ([#370](https://github.com/noir-lang/acvm/issues/370)) ([5ba0349](https://github.com/noir-lang/acvm/commit/5ba0349420cc1b20113cb5e96490a0808a769757))


### Bug Fixes

* **brillig:** remove register initialization check ([#392](https://github.com/noir-lang/acvm/issues/392)) ([1a53143](https://github.com/noir-lang/acvm/commit/1a531438b5c1ab7ce8c4bd599dda3515bdd5cfcd))

## [0.15.0](https://github.com/noir-lang/acvm/compare/root-v0.14.2...root-v0.15.0) (2023-06-15)


### ⚠ BREAKING CHANGES

* **brillig:** Accept multiple inputs/outputs for foreign calls ([#367](https://github.com/noir-lang/acvm/issues/367))
* **acvm:** Make internals of ACVM private ([#353](https://github.com/noir-lang/acvm/issues/353))

### Features

* Add method to generate updated `Brillig` opcode from `UnresolvedBrilligCall` ([#363](https://github.com/noir-lang/acvm/issues/363)) ([fda5dbe](https://github.com/noir-lang/acvm/commit/fda5dbe57c28dc4bc28dfd8fe0a4a8ba29635393))
* **brillig:** Accept multiple inputs/outputs for foreign calls ([#367](https://github.com/noir-lang/acvm/issues/367)) ([78d62b2](https://github.com/noir-lang/acvm/commit/78d62b2d7c1c8b884e1f3fe7983e6e5029700e70))
* **brillig:** Set `VMStatus` to `Failure` rather than panicking on invalid foreign call response ([#375](https://github.com/noir-lang/acvm/issues/375)) ([c49d82c](https://github.com/noir-lang/acvm/commit/c49d82c99c73c60e264585ed201af2b6a2b7ee0f))


### Bug Fixes

* **brillig:** Correct signed division implementation ([#356](https://github.com/noir-lang/acvm/issues/356)) ([4eefda0](https://github.com/noir-lang/acvm/commit/4eefda01e7b371035314f77631df4687608b4782))
* **brillig:** Explicitly wrap on arithmetic operations ([#365](https://github.com/noir-lang/acvm/issues/365)) ([c0544a9](https://github.com/noir-lang/acvm/commit/c0544a99930d3c8d534376c8f8a91645a39aecf8))


### Miscellaneous Chores

* **acvm:** Make internals of ACVM private ([#353](https://github.com/noir-lang/acvm/issues/353)) ([c902a01](https://github.com/noir-lang/acvm/commit/c902a01639033665d106e2d9f4e5c7070af8c0bb))

## [0.14.2](https://github.com/noir-lang/acvm/compare/root-v0.14.1...root-v0.14.2) (2023-06-08)


### Bug Fixes

* **brillig:** expand memory with zeroes on store ([#350](https://github.com/noir-lang/acvm/issues/350)) ([4d2dadd](https://github.com/noir-lang/acvm/commit/4d2dadd3acd9dc25f0feae865b74cbaea7250f3d))

## [0.14.1](https://github.com/noir-lang/acvm/compare/root-v0.14.0...root-v0.14.1) (2023-06-07)


### Features

* Re-use intermediate variables created during width reduction, with proper scale. ([#343](https://github.com/noir-lang/acvm/issues/343)) ([6bd0baa](https://github.com/noir-lang/acvm/commit/6bd0baa4bc9ac204e7710ec6d17d1752d2e924c0))

## [0.14.0](https://github.com/noir-lang/acvm/compare/root-v0.13.3...root-v0.14.0) (2023-06-06)


### ⚠ BREAKING CHANGES

* **acir:** Verify Proof ([#291](https://github.com/noir-lang/acvm/issues/291))

### Features

* **acir:** Verify Proof ([#291](https://github.com/noir-lang/acvm/issues/291)) ([9f34428](https://github.com/noir-lang/acvm/commit/9f34428b7084c7c38de401a16ca76e748d8b1d77))

## [0.13.3](https://github.com/noir-lang/acvm/compare/root-v0.13.2...root-v0.13.3) (2023-06-05)


### Bug Fixes

* Empty commit to trigger release-please ([e8f0748](https://github.com/noir-lang/acvm/commit/e8f0748042ef505d59ab63266d3c36c5358ee30d))

## [0.13.2](https://github.com/noir-lang/acvm/compare/root-v0.13.1...root-v0.13.2) (2023-06-02)


### Bug Fixes

* re-use intermediate vars during width reduction ([#278](https://github.com/noir-lang/acvm/issues/278)) ([5b32920](https://github.com/noir-lang/acvm/commit/5b32920263c4481c60faf0b84f0031aa8149b6b2))

## [0.13.1](https://github.com/noir-lang/acvm/compare/root-v0.13.0...root-v0.13.1) (2023-06-01)


### Bug Fixes

* **brillig:** Proper error handling for Brillig failures ([#329](https://github.com/noir-lang/acvm/issues/329)) ([cffa110](https://github.com/noir-lang/acvm/commit/cffa110c8df30ee3dd8b635d38b17b1fcd54b03e))
* **ci:** Add brillig_vm to release-please & link versions ([#332](https://github.com/noir-lang/acvm/issues/332)) ([84bd22e](https://github.com/noir-lang/acvm/commit/84bd22eea46cdfef3a5dbf534b878e819d44f755))
* **ci:** Correct typo to avoid `undefined` in changelogs ([#333](https://github.com/noir-lang/acvm/issues/333)) ([d3424c0](https://github.com/noir-lang/acvm/commit/d3424c04fd303c9cbe25d03118d8b358cbb84b83))

## [0.13.0](https://github.com/noir-lang/acvm/compare/root-v0.12.0...root-v0.13.0) (2023-06-01)


### ⚠ BREAKING CHANGES

* added hash index to pedersen ([#281](https://github.com/noir-lang/acvm/issues/281))
* Add variable length keccak opcode ([#314](https://github.com/noir-lang/acvm/issues/314))
* Remove AES opcode ([#302](https://github.com/noir-lang/acvm/issues/302))
* **acir, acvm:** Remove ComputeMerkleRoot opcode #296
* Remove manual serialization of `Opcode`s in favour of `serde` ([#286](https://github.com/noir-lang/acvm/issues/286))
* Remove backend solvable methods from the interface and solve them in ACVM ([#264](https://github.com/noir-lang/acvm/issues/264))
* Reorganize code related to `PartialWitnessGenerator` ([#287](https://github.com/noir-lang/acvm/issues/287))

### Features

* **acir, acvm:** Remove ComputeMerkleRoot opcode [#296](https://github.com/noir-lang/acvm/issues/296) ([8b3923e](https://github.com/noir-lang/acvm/commit/8b3923e191e4ac399400025496e8bb4453734040))
* Add `Brillig` opcode to introduce custom non-determinism to ACVM ([#152](https://github.com/noir-lang/acvm/issues/152)) ([3c6740a](https://github.com/noir-lang/acvm/commit/3c6740af75125afc8ebb4379f781f8274015e2e2))
* Add variable length keccak opcode ([#314](https://github.com/noir-lang/acvm/issues/314)) ([7bfd169](https://github.com/noir-lang/acvm/commit/7bfd1695b6f119cd70fce4866314c9bb4991eaab))
* added hash index to pedersen ([#281](https://github.com/noir-lang/acvm/issues/281)) ([61820b6](https://github.com/noir-lang/acvm/commit/61820b651900aac8d9557b4b9477ed0e1763c124))
* Remove backend solvable methods from the interface and solve them in ACVM ([#264](https://github.com/noir-lang/acvm/issues/264)) ([69916cb](https://github.com/noir-lang/acvm/commit/69916cbdd928875b2e8fe4775f2251f71c3f3c92))


### Bug Fixes

* Allow async functions without send on async trait ([#292](https://github.com/noir-lang/acvm/issues/292)) ([9f9fc21](https://github.com/noir-lang/acvm/commit/9f9fc216a6d09ca97352ffd365bfd347e94ad8eb))


### Miscellaneous Chores

* Remove AES opcode ([#302](https://github.com/noir-lang/acvm/issues/302)) ([a429a54](https://github.com/noir-lang/acvm/commit/a429a5422d6f001b6db0d0a0f30c79ec0f96de89))
* Remove manual serialization of `Opcode`s in favour of `serde` ([#286](https://github.com/noir-lang/acvm/issues/286)) ([8a3812f](https://github.com/noir-lang/acvm/commit/8a3812fe6ed3b267692284bdcd909d9dd32b9747))
* Reorganize code related to `PartialWitnessGenerator` ([#287](https://github.com/noir-lang/acvm/issues/287)) ([b9d61a1](https://github.com/noir-lang/acvm/commit/b9d61a16210d70e350a7e953951362c94f497f89))

## [0.12.0](https://github.com/noir-lang/acvm/compare/root-v0.11.0...root-v0.12.0) (2023-05-17)


### ⚠ BREAKING CHANGES

* remove deprecated circuit hash functions ([#288](https://github.com/noir-lang/acvm/issues/288))
* allow backends to specify support for all opcode variants ([#273](https://github.com/noir-lang/acvm/issues/273))
* **acvm:** Add CommonReferenceString backend trait ([#231](https://github.com/noir-lang/acvm/issues/231))
* Introduce WitnessMap data structure to avoid leaking internal structure ([#252](https://github.com/noir-lang/acvm/issues/252))
* use struct variants for blackbox function calls ([#269](https://github.com/noir-lang/acvm/issues/269))
* **acvm:** Backend trait must implement Debug ([#275](https://github.com/noir-lang/acvm/issues/275))
* remove `OpcodeResolutionError::UnexpectedOpcode` ([#274](https://github.com/noir-lang/acvm/issues/274))
* **acvm:** rename `hash_to_field128_security` to `hash_to_field_128_security` ([#271](https://github.com/noir-lang/acvm/issues/271))
* **acvm:** update black box solver interfaces to match `pwg:black_box::solve` ([#268](https://github.com/noir-lang/acvm/issues/268))
* **acvm:** expose separate solvers for AND and XOR opcodes ([#266](https://github.com/noir-lang/acvm/issues/266))
* **acvm:** Simplification pass for ACIR ([#151](https://github.com/noir-lang/acvm/issues/151))
* Remove `solve` from PWG trait & introduce separate solvers for each blackbox ([#257](https://github.com/noir-lang/acvm/issues/257))

### Features

* **acvm:** Add CommonReferenceString backend trait ([#231](https://github.com/noir-lang/acvm/issues/231)) ([eeddcf1](https://github.com/noir-lang/acvm/commit/eeddcf179880f246383f7f67a11e589269c4e3ff))
* **acvm:** Simplification pass for ACIR ([#151](https://github.com/noir-lang/acvm/issues/151)) ([7bc42c6](https://github.com/noir-lang/acvm/commit/7bc42c62b6e095f838b781c87cbb1ecd2af5f179))
* **acvm:** update black box solver interfaces to match `pwg:black_box::solve` ([#268](https://github.com/noir-lang/acvm/issues/268)) ([0098b7d](https://github.com/noir-lang/acvm/commit/0098b7d9640076d970e6c15d5fd6f368eb1513ff))
* Introduce WitnessMap data structure to avoid leaking internal structure ([#252](https://github.com/noir-lang/acvm/issues/252)) ([b248e60](https://github.com/noir-lang/acvm/commit/b248e606dd69c25d33ae77c5c5c0541adbf80cd6))
* Remove `solve` from PWG trait & introduce separate solvers for each blackbox ([#257](https://github.com/noir-lang/acvm/issues/257)) ([3f3dd74](https://github.com/noir-lang/acvm/commit/3f3dd7460b27ab06b55dfc3fe5dd733f08e30a9f))
* use struct variants for blackbox function calls ([#269](https://github.com/noir-lang/acvm/issues/269)) ([a83333b](https://github.com/noir-lang/acvm/commit/a83333b9e270dfcfd40a36271896840ec0201bc4))


### Bug Fixes

* **acir:** Hide variants of WitnessMapError and export it from package ([#283](https://github.com/noir-lang/acvm/issues/283)) ([bbd9ab7](https://github.com/noir-lang/acvm/commit/bbd9ab7ca5be3fb31f3e141fee2522704852f5de))


### Miscellaneous Chores

* **acvm:** Backend trait must implement Debug ([#275](https://github.com/noir-lang/acvm/issues/275)) ([3288b4c](https://github.com/noir-lang/acvm/commit/3288b4c7eb01f5621e577d5ff9e7c92c7757e021))
* **acvm:** expose separate solvers for AND and XOR opcodes ([#266](https://github.com/noir-lang/acvm/issues/266)) ([84b5d18](https://github.com/noir-lang/acvm/commit/84b5d18d29a111a42bfc1c3d122129c8f062c3db))
* **acvm:** rename `hash_to_field128_security` to `hash_to_field_128_security` ([#271](https://github.com/noir-lang/acvm/issues/271)) ([fad9af2](https://github.com/noir-lang/acvm/commit/fad9af27fb102fa34bf7511f8ed7b16b3ec2d115))
* allow backends to specify support for all opcode variants ([#273](https://github.com/noir-lang/acvm/issues/273)) ([efd37fe](https://github.com/noir-lang/acvm/commit/efd37fedcbbabb3fac810e662731439e07fef49a))
* remove `OpcodeResolutionError::UnexpectedOpcode` ([#274](https://github.com/noir-lang/acvm/issues/274)) ([0e71aac](https://github.com/noir-lang/acvm/commit/0e71aac7aa85b3e9142972a26ba122c2c7c51d9b))
* remove deprecated circuit hash functions ([#288](https://github.com/noir-lang/acvm/issues/288)) ([1a22c75](https://github.com/noir-lang/acvm/commit/1a22c752de3354a2a6d34892331ab6623b24c0b0))

## [0.11.0](https://github.com/noir-lang/acvm/compare/root-v0.10.3...root-v0.11.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248))

### Features

* **acvm:** Add generic error for failing to solve an opcode ([#251](https://github.com/noir-lang/acvm/issues/251)) ([bc89528](https://github.com/noir-lang/acvm/commit/bc8952820de610e585d505decfac6e590bbb1a35))
* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248)) ([45c45f7](https://github.com/noir-lang/acvm/commit/45c45f7cdb79c3ccb0373ca0e698b282d4dabc39))
* Add Keccak Hash function ([#259](https://github.com/noir-lang/acvm/issues/259)) ([443c734](https://github.com/noir-lang/acvm/commit/443c73482eeef6cc42a1a254bf0d7706698ee353))


### Bug Fixes

* **acir:** Fix `Expression` multiplication to correctly handle degree 1 terms ([#255](https://github.com/noir-lang/acvm/issues/255)) ([e399396](https://github.com/noir-lang/acvm/commit/e399396f7e06deb6b831517af17018607df3f252))

## [0.10.3](https://github.com/noir-lang/acvm/compare/root-v0.10.2...root-v0.10.3) (2023-04-28)


### Bug Fixes

* add default feature flag to ACVM crate ([#245](https://github.com/noir-lang/acvm/issues/245)) ([455fddb](https://github.com/noir-lang/acvm/commit/455fddbc19af81cb01d54e29cad199691e1a1d98))

## [0.10.2](https://github.com/noir-lang/acvm/compare/root-v0.10.1...root-v0.10.2) (2023-04-28)


### Bug Fixes

* add default flag to `acvm_stdlib` ([#242](https://github.com/noir-lang/acvm/issues/242)) ([83b6fa8](https://github.com/noir-lang/acvm/commit/83b6fa8302569add7e3ac8481b2fd2a6a1ff3576))

## [0.10.1](https://github.com/noir-lang/acvm/compare/root-v0.10.0...root-v0.10.1) (2023-04-28)


### Bug Fixes

* **acir:** add `bn254` as default feature flag ([#240](https://github.com/noir-lang/acvm/issues/240)) ([e56973d](https://github.com/noir-lang/acvm/commit/e56973d8dc1745fe9bb844ec8347acd4d836d42f))

## [0.10.0](https://github.com/noir-lang/acvm/compare/root-v0.9.0...root-v0.10.0) (2023-04-26)


### ⚠ BREAKING CHANGES

* return `Result<OpcodeResolution, OpcodeResolutionError>` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238))
* **acvm:** have all black box functions return `Result<OpcodeResolution, OpcodeResolutionError>` ([#237](https://github.com/noir-lang/acvm/issues/237))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230))
* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223))
* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213))
* organize operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190))

### Features

* **acvm:** have all black box functions return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` ([#237](https://github.com/noir-lang/acvm/issues/237)) ([e8e93fd](https://github.com/noir-lang/acvm/commit/e8e93fda0db18f0d266dd1aacbb53ec787992dc9))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230)) ([198fb69](https://github.com/noir-lang/acvm/commit/198fb69e90a5ed3c0a8716d888b4dc6c2f9b18aa))
* Add range opcode optimization ([#219](https://github.com/noir-lang/acvm/issues/219)) ([7abe6e5](https://github.com/noir-lang/acvm/commit/7abe6e5f6d6fea379c3748a910afd00db066eb45))
* implement `add_mul` on `Expression` ([#207](https://github.com/noir-lang/acvm/issues/207)) ([f156e18](https://github.com/noir-lang/acvm/commit/f156e18cf7a0f1a99bbe1683b8e75fec8325e6dd))
* implement `FieldElement::from&lt;bool&gt;()` ([#203](https://github.com/noir-lang/acvm/issues/203)) ([476cfa2](https://github.com/noir-lang/acvm/commit/476cfa247fddb515c64c2801c6868357c9375294))
* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233)) ([74bfee8](https://github.com/noir-lang/acvm/commit/74bfee80e0ff0d205aee1eea548c97ade8bd0e41))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223)) ([00282dc](https://github.com/noir-lang/acvm/commit/00282dc5e2b03947bf709a088d829f3e0ba80eed))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213)) ([e877bed](https://github.com/noir-lang/acvm/commit/e877bed2cca76bd486e9bed66b4230e65a01f0a2))
* return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238)) ([15d3c5a](https://github.com/noir-lang/acvm/commit/15d3c5a9be2dd92f266fcb7e672da17cada9fec5))


### Bug Fixes

* prevent `bn254` feature flag always being enabled ([#225](https://github.com/noir-lang/acvm/issues/225)) ([82eee6a](https://github.com/noir-lang/acvm/commit/82eee6ab08ae480f04904ca8571fd88f4466c000))


### Miscellaneous Chores

* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220)) ([64bb346](https://github.com/noir-lang/acvm/commit/64bb346524428a0ce196826ea1e5ccde08ad6201))
* organize operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190)) ([a619df6](https://github.com/noir-lang/acvm/commit/a619df614bbb9b2518b788b42a7553b069823a0f))

## [0.9.0](https://github.com/noir-lang/acvm/compare/root-v0.8.1...root-v0.9.0) (2023-04-07)


### ⚠ BREAKING CHANGES

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185))
* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180))

### Features

* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180)) ([fd28657](https://github.com/noir-lang/acvm/commit/fd28657426260ce3c53517b75a27eb5c4a74e234))


### Bug Fixes

* Add test for Out of Memory  ([#188](https://github.com/noir-lang/acvm/issues/188)) ([c3db985](https://github.com/noir-lang/acvm/commit/c3db985893e7e59ea04005bb3a57eda5c6ce28c7))


### Miscellaneous Chores

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185)) ([ee59c9e](https://github.com/noir-lang/acvm/commit/ee59c9efe9a54ff6b97e4daaebf64f3e327e97d9))

## [0.8.1](https://github.com/noir-lang/acvm/compare/root-v0.8.0...root-v0.8.1) (2023-03-30)


### Bug Fixes

* unwraps if inputs is zero ([#171](https://github.com/noir-lang/acvm/issues/171)) ([10a3bb2](https://github.com/noir-lang/acvm/commit/10a3bb2a9930ccf422b3f08227aae07775686860))

## [0.8.0](https://github.com/noir-lang/acvm/compare/root-v0.7.1...root-v0.8.0) (2023-03-28)


### ⚠ BREAKING CHANGES

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156))

### Bug Fixes

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156)) ([1cc2b7f](https://github.com/noir-lang/acvm/commit/1cc2b7f2179cecc338fe0def72bb2dd17eaed0cd))

## [0.7.1](https://github.com/noir-lang/acvm/compare/root-v0.7.0...root-v0.7.1) (2023-03-27)


### Bug Fixes

* **pwg:** stall instead of fail for unassigned black box ([#154](https://github.com/noir-lang/acvm/issues/154)) ([412a1a6](https://github.com/noir-lang/acvm/commit/412a1a60b434bef53e12d37c3b2bb3d51a317994))

## [0.7.0](https://github.com/noir-lang/acvm/compare/root-v0.6.0...root-v0.7.0) (2023-03-23)


### ⚠ BREAKING CHANGES

* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149))
* **acir:** Add RAM and ROM opcodes
* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56))
* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142))

### Features

* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56)) ([5f358a9](https://github.com/noir-lang/acvm/commit/5f358a97aaa81d87956e182cd8a6d60de75f9752))
* **acir:** Add RAM and ROM opcodes ([73e9f25](https://github.com/noir-lang/acvm/commit/73e9f25dd87b2ca91245e93d2445eadc0f522fac))
* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149)) ([88ee2f8](https://github.com/noir-lang/acvm/commit/88ee2f89f37abf5dd1d9f91b4d2eed44dc651348))


### Miscellaneous Chores

* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145)) ([bbb6d92](https://github.com/noir-lang/acvm/commit/bbb6d92e25c43dd33b12f5fcd639fc9ad2a9c9d8))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140)) ([35dd181](https://github.com/noir-lang/acvm/commit/35dd181102203df17eef510666b327ef41f4b036))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142)) ([85dd6e8](https://github.com/noir-lang/acvm/commit/85dd6e85bfba85bfb97651f7e30e1f75deb986d5))

## [0.6.0](https://github.com/noir-lang/acvm/compare/root-v0.5.0...root-v0.6.0) (2023-03-03)


### ⚠ BREAKING CHANGES

* **acir:** rename `term_addition` to `push_addition_term`
* **acir:** rename `term_multiplication` to `push_multiplication_term` ([#122](https://github.com/noir-lang/acvm/issues/122))
* **acir:** remove `UnknownWitness` ([#123](https://github.com/noir-lang/acvm/issues/123))
* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114))

### Features

* **acir:** add useful methods from `noirc_evaluator` onto `Expression` ([#125](https://github.com/noir-lang/acvm/issues/125)) ([d3d5f89](https://github.com/noir-lang/acvm/commit/d3d5f8917482ce5649602695829862a5df4ea712))
* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114)) ([097cfb0](https://github.com/noir-lang/acvm/commit/097cfb069291705ddb4bf1fca77ddcef21dbbd08))


### Bug Fixes

* **acir:** correctly display expressions with non-unit coefficients ([d3d5f89](https://github.com/noir-lang/acvm/commit/d3d5f8917482ce5649602695829862a5df4ea712))
* **ci:** publish acvm_stdlib before acvm ([#117](https://github.com/noir-lang/acvm/issues/117)) ([ca6defc](https://github.com/noir-lang/acvm/commit/ca6defc9bb5f51241b2fc4d9cd732f9678b4688f))


### Miscellaneous Chores

* **acir:** remove `UnknownWitness` ([#123](https://github.com/noir-lang/acvm/issues/123)) ([9f002c7](https://github.com/noir-lang/acvm/commit/9f002c7b49a5cf222d4a01732cc4917a47690863))
* **acir:** rename `term_addition` to `push_addition_term` ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))
* **acir:** rename `term_multiplication` to `push_multiplication_term` ([#122](https://github.com/noir-lang/acvm/issues/122)) ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))

## [0.5.0](https://github.com/noir-lang/acvm/compare/root-v0.4.1...root-v0.5.0) (2023-02-22)


### ⚠ BREAKING CHANGES

* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96))
* **acvm:** add `eth_contract_from_vk` to `SmartContract
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108))
* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99))
* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58))
* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91))
* reorganize compiler in terms of optimizers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88))

### Features

* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91)) ([b909146](https://github.com/noir-lang/acvm/commit/b9091461e199bacdd073cc9b31f03dade0b4fb2d))
* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99)) ([53666b7](https://github.com/noir-lang/acvm/commit/53666b782d89c65cd755f9e4ded2c9cf5a141e46))
* **acvm:** add `eth_contract_from_vk` to `SmartContract ([#113](https://github.com/noir-lang/acvm/issues/113)) ([373c18f](https://github.com/noir-lang/acvm/commit/373c18fc05edf673cfec9e8bbb78bd7d7514999e))
* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96)) ([f57ba57](https://github.com/noir-lang/acvm/commit/f57ba57c2bb2597edf2b02fb1321c69cf11993ee))
* **ci:** Add release workflow ([#89](https://github.com/noir-lang/acvm/issues/89)) ([db8e828](https://github.com/noir-lang/acvm/commit/db8e828341f59241ef7f437c908277fb8fbca9e3))
* **ci:** Publish crates upon release ([#104](https://github.com/noir-lang/acvm/issues/104)) ([b265920](https://github.com/noir-lang/acvm/commit/b265920bc1b0c776d20326a0b74fc635c22af4b9))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108)) ([af56ca9](https://github.com/noir-lang/acvm/commit/af56ca9da06068c650c66e76bfd09e65eb0ec213))
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111)) ([39b8a41](https://github.com/noir-lang/acvm/commit/39b8a41293e567971f700f61103852cb987a8d16))
* Update Arkworks' dependencies on `acir_field` ([#69](https://github.com/noir-lang/acvm/issues/69)) ([65d6130](https://github.com/noir-lang/acvm/commit/65d61307a12f25e04afad2d50e4c4db5ce97dd8c))


### Bug Fixes

* **ci:** Update dependency versions in the workspace file ([#103](https://github.com/noir-lang/acvm/issues/103)) ([9acc266](https://github.com/noir-lang/acvm/commit/9acc266c7dc5a6ad2fa9c466cc82cb81d984b7ed))
* Clean up Log Directive hex output  ([#97](https://github.com/noir-lang/acvm/issues/97)) ([d23c735](https://github.com/noir-lang/acvm/commit/d23c7352523ffb42f3e8f4229b61f9803ab78a7e))


### Miscellaneous Chores

* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58)) ([2427a27](https://github.com/noir-lang/acvm/commit/2427a275048e598c6d651cce8348a4c55148f235))
* reorganize compiler in terms of optimizers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88)) ([9329307](https://github.com/noir-lang/acvm/commit/9329307e054de202cfc55207162ad952b70d515e))

## [0.4.1] - 2023-02-08

### Added

### Fixed

- Removed duplicated logic in match branch

### Changed

### Removed

## [0.4.0] - 2023-02-08

### Added

- Add log directive
- Expose `acir_field` through `acir` crate
- Add permutation directive
- Add preprocess methods to ACVM interface

### Fixed

### Changed

- Changed spellings of many functions to be correct using spellchecker

### Removed

## [0.3.1] - 2023-01-18

### Added

### Fixed

### Changed

- ACVM compile method now returns an Error for when a function cannot be reduced to arithmetic gates

- Backtrack changes from noir-lang/noir/587

### Removed

## [0.3.0] - 2022-12-31

### Added

- Added stdlib module to hold all of the standard opcodes
- added `read` , `write` methods for circuit

### Fixed

### Changed

- XOR, Range and AND gates are no longer special case. They are now another opcode in the GadgetCall
- Move fallback module to `stdlib`
- Optimizer code and any other passes will live in acvm. acir is solely for defining the IR now.
- ACIR passes now live under the compiler parent module
- Moved opcode module in acir crate to circuit/opcode
- Rename GadgetCall to BlackBoxFuncCall
- Rename opcode file to blackbox_functions . Similarly OPCODE is now BlackBoxFunc
- Renamed GateResolution::UnsupportedOpcode to GateResolution::UnsupportedBlackBoxFunc
- Renamed GadgetDefinition to FuncDefinition
- Rename GadgetInput to FunctionInput
- Rename Gate -> Opcode . Similarly gate.rs is now opcodes.rs
- Rename CustomGate::supports_gate -> CustomGate::supports_opcode
- Rename GateResolution to OpcodeResolution
- Rename Split directive to ToBits
- Field element printing function was modified to uses ascii superscript numbers and ascii multiplication
- Refactor the way we print ACIR (This is a first draft and will change with more feedback)
- Rename `solve_gadget_call` trait method on ProofSystemCompile to `solve_blackbox_function_call`
- API for `compile` now requires a function pointer which tells us whether a blackbox function is supported
- Renamed Directive::Oddrange to Directive::OddRange
- Renamed FieldElement::to_bytes to FieldElement::to_be_bytes

### Removed

- Selector struct has been removed as it is no longer being used. It is also not being used by Noir.
- CustomGate trait -- There is a method in the ProofSystemCompiler Trait that backends can use to indicate whether
they support a particular black box function
- Remove OpcodeResolution enum from pwg. The happy case is strictly when the witness has been solved

## [0.2.1] - 2022-12-23

- Removed ToBits and ToBytes opcode
