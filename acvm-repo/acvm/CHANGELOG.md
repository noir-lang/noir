# Changelog

## [0.27.0](https://github.com/noir-lang/acvm/compare/acvm-v0.26.1...acvm-v0.27.0) (2023-09-19)


### ⚠ BREAKING CHANGES

* Separate barretenberg solver from generic blackbox solver code ([#554](https://github.com/noir-lang/acvm/issues/554))

### Miscellaneous Chores

* Separate barretenberg solver from generic blackbox solver code ([#554](https://github.com/noir-lang/acvm/issues/554)) ([a4b9772](https://github.com/noir-lang/acvm/commit/a4b97722a0892fe379ff075e6080675adafdce0e))

## [0.26.1](https://github.com/noir-lang/acvm/compare/acvm-v0.26.0...acvm-v0.26.1) (2023-09-12)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.26.0 to 0.26.1

## [0.26.0](https://github.com/noir-lang/acvm/compare/acvm-v0.25.0...acvm-v0.26.0) (2023-09-07)


### ⚠ BREAKING CHANGES

* Add a low and high limb to scalar mul opcode ([#532](https://github.com/noir-lang/acvm/issues/532))

### Miscellaneous Chores

* Add a low and high limb to scalar mul opcode ([#532](https://github.com/noir-lang/acvm/issues/532)) ([b054f66](https://github.com/noir-lang/acvm/commit/b054f66be9c73d4e02dbecdab80874a907f19242))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.25.0 to 0.26.0

## [0.25.0](https://github.com/noir-lang/acvm/compare/acvm-v0.24.1...acvm-v0.25.0) (2023-09-04)


### ⚠ BREAKING CHANGES

* Provide runtime callstacks for brillig failures and return errors in acvm_js ([#523](https://github.com/noir-lang/acvm/issues/523))

### Features

* Provide runtime callstacks for brillig failures and return errors in acvm_js ([#523](https://github.com/noir-lang/acvm/issues/523)) ([7ab7cff](https://github.com/noir-lang/acvm/commit/7ab7cff48a9aba61a97fad2a759fc8e55740b098))


### Bug Fixes

* initialize recursive proof output to zero ([#524](https://github.com/noir-lang/acvm/issues/524)) ([5453074](https://github.com/noir-lang/acvm/commit/545307457dd7634b20ea3977e2d2cc751eba06d2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.24.1 to 0.25.0

## [0.24.1](https://github.com/noir-lang/acvm/compare/acvm-v0.24.0...acvm-v0.24.1) (2023-09-03)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.24.0 to 0.24.1

## [0.24.0](https://github.com/noir-lang/acvm/compare/acvm-v0.23.0...acvm-v0.24.0) (2023-08-31)


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


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.23.0 to 0.24.0

## [0.23.0](https://github.com/noir-lang/acvm/compare/acvm-v0.22.0...acvm-v0.23.0) (2023-08-30)


### ⚠ BREAKING CHANGES

* Return an iterator from `new_locations()` instead of collecting ([#507](https://github.com/noir-lang/acvm/issues/507))
* **acvm:** remove `CommonReferenceString` trait and preprocess method ([#508](https://github.com/noir-lang/acvm/issues/508))
* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference

### Features

* **acvm_js:** Add `execute_circuit_with_black_box_solver` to prevent reinitialization of `BlackBoxFunctionSolver` ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))


### Miscellaneous Chores

* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))
* **acvm:** remove `CommonReferenceString` trait and preprocess method ([#508](https://github.com/noir-lang/acvm/issues/508)) ([3827dd3](https://github.com/noir-lang/acvm/commit/3827dd3ce487650843ba4df8337b423e39f97edf))
* Return an iterator from `new_locations()` instead of collecting ([#507](https://github.com/noir-lang/acvm/issues/507)) ([8d49a5c](https://github.com/noir-lang/acvm/commit/8d49a5c15b1e962cd59252467a20a922edadc2f2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.22.0 to 0.23.0

## [0.22.0](https://github.com/noir-lang/acvm/compare/acvm-v0.21.0...acvm-v0.22.0) (2023-08-18)


### ⚠ BREAKING CHANGES

* Switched from OpcodeLabel to OpcodeLocation and ErrorLocation ([#493](https://github.com/noir-lang/acvm/issues/493))
* **acvm:** check for index out-of-bounds on memory operations ([#468](https://github.com/noir-lang/acvm/issues/468))

### Features

* **acvm:** check for index out-of-bounds on memory operations ([#468](https://github.com/noir-lang/acvm/issues/468)) ([740468c](https://github.com/noir-lang/acvm/commit/740468c0a144f7179c38f615cfda31b2fcc77359))
* print error location with fmt ([#497](https://github.com/noir-lang/acvm/issues/497)) ([575a9e5](https://github.com/noir-lang/acvm/commit/575a9e50e97afb04a7b91799e06752cec3093f0b))
* Switched from OpcodeLabel to OpcodeLocation and ErrorLocation ([#493](https://github.com/noir-lang/acvm/issues/493)) ([27a5a93](https://github.com/noir-lang/acvm/commit/27a5a935849f8904e10056b08089f532a06962b8))


### Bug Fixes

* add opcode label to unsatisfied constrain string ([#482](https://github.com/noir-lang/acvm/issues/482)) ([cbbbe67](https://github.com/noir-lang/acvm/commit/cbbbe67b9a19a4a560b2dfa8f27ea1c6ebd61f28))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.21.0 to 0.22.0

## [0.21.0](https://github.com/noir-lang/acvm/compare/acvm-v0.20.1...acvm-v0.21.0) (2023-07-26)


### ⚠ BREAKING CHANGES

* **acir:** Remove `Block`, `RAM` and `ROM` opcodes ([#457](https://github.com/noir-lang/acvm/issues/457))
* **acvm:** Remove `OpcodeResolution` enum ([#400](https://github.com/noir-lang/acvm/issues/400))
* **acvm:** Support stepwise execution of ACIR ([#399](https://github.com/noir-lang/acvm/issues/399))

### Features

* **acvm:** Remove `OpcodeResolution` enum ([#400](https://github.com/noir-lang/acvm/issues/400)) ([d0ce48c](https://github.com/noir-lang/acvm/commit/d0ce48c506619a5560412ef6693bfa11036b501e))
* **acvm:** Support stepwise execution of ACIR ([#399](https://github.com/noir-lang/acvm/issues/399)) ([6a03950](https://github.com/noir-lang/acvm/commit/6a0395021779a2711353c2fe2948e09b5b538fc0))


### Miscellaneous Chores

* **acir:** Remove `Block`, `RAM` and `ROM` opcodes ([#457](https://github.com/noir-lang/acvm/issues/457)) ([8dd220a](https://github.com/noir-lang/acvm/commit/8dd220ae127baf6cc5a31d8ab7ffdeeb161f6109))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.20.1 to 0.21.0

## [0.20.1](https://github.com/noir-lang/acvm/compare/acvm-v0.20.0...acvm-v0.20.1) (2023-07-26)


### Features

* **stdlib:** Add fallback implementation of `Keccak256` black box function ([#445](https://github.com/noir-lang/acvm/issues/445)) ([f7ebb03](https://github.com/noir-lang/acvm/commit/f7ebb03653c971f119700ff8126d9eb5ff01be0f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.20.0 to 0.20.1

## [0.20.0](https://github.com/noir-lang/acvm/compare/acvm-v0.19.1...acvm-v0.20.0) (2023-07-20)


### ⚠ BREAKING CHANGES

* atomic memory opcodes ([#447](https://github.com/noir-lang/acvm/issues/447))

### Features

* atomic memory opcodes ([#447](https://github.com/noir-lang/acvm/issues/447)) ([3261c7a](https://github.com/noir-lang/acvm/commit/3261c7a2fd4f3a300bc5f39ef4febccd8a853560))
* **stdlib:** Add fallback implementation of `HashToField128Security` black box function ([#435](https://github.com/noir-lang/acvm/issues/435)) ([ed40f22](https://github.com/noir-lang/acvm/commit/ed40f228529e888d1960bfa70cb92b277e24b37f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.19.1 to 0.20.0

## [0.19.1](https://github.com/noir-lang/acvm/compare/acvm-v0.19.0...acvm-v0.19.1) (2023-07-17)


### Bug Fixes

* Remove panic when we divide 0/0 in quotient directive ([#437](https://github.com/noir-lang/acvm/issues/437)) ([9c8ff64](https://github.com/noir-lang/acvm/commit/9c8ff64ebf27a86787ae184e10ed9581041ec0ff))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.19.0 to 0.19.1

## [0.19.0](https://github.com/noir-lang/acvm/compare/acvm-v0.18.2...acvm-v0.19.0) (2023-07-15)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.18.2 to 0.19.0

## [0.18.2](https://github.com/noir-lang/acvm/compare/acvm-v0.18.1...acvm-v0.18.2) (2023-07-12)


### Features

* **acvm:** reexport `blackbox_solver` crate from `acvm` ([#431](https://github.com/noir-lang/acvm/issues/431)) ([517e942](https://github.com/noir-lang/acvm/commit/517e942b732d7107f6e064c6791917d1508229b3))
* **stdlib:** Add fallback implementation of `Blake2s` black box function ([#424](https://github.com/noir-lang/acvm/issues/424)) ([982d940](https://github.com/noir-lang/acvm/commit/982d94087d46092ce7a5e94dbd7e732195f58e42))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.18.1 to 0.18.2

## [0.18.1](https://github.com/noir-lang/acvm/compare/acvm-v0.18.0...acvm-v0.18.1) (2023-07-12)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.18.0 to 0.18.1

## [0.18.0](https://github.com/noir-lang/acvm/compare/acvm-v0.17.0...acvm-v0.18.0) (2023-07-12)


### ⚠ BREAKING CHANGES

* add backend-solvable blackboxes to brillig & unify implementations ([#422](https://github.com/noir-lang/acvm/issues/422))
* **acvm:** Remove `CircuitSimplifer` ([#421](https://github.com/noir-lang/acvm/issues/421))
* **acvm:** Add `circuit: &Circuit` to `eth_contract_from_vk` function signature ([#420](https://github.com/noir-lang/acvm/issues/420))
* Returns index of failing opcode and transformation mapping ([#412](https://github.com/noir-lang/acvm/issues/412))

### Features

* **acvm:** Add `circuit: &Circuit` to `eth_contract_from_vk` function signature ([#420](https://github.com/noir-lang/acvm/issues/420)) ([744e9da](https://github.com/noir-lang/acvm/commit/744e9da71f7ca477a5390a63f47211dd4dffb8b3))
* add backend-solvable blackboxes to brillig & unify implementations ([#422](https://github.com/noir-lang/acvm/issues/422)) ([093342e](https://github.com/noir-lang/acvm/commit/093342ea9481a311fa71343b8b7a22774788838a))
* Returns index of failing opcode and transformation mapping ([#412](https://github.com/noir-lang/acvm/issues/412)) ([79950e9](https://github.com/noir-lang/acvm/commit/79950e943f60e4082e1cf5ec4442aa67ea91aade))
* **stdlib:** Add fallback implementation of `SHA256` black box function ([#407](https://github.com/noir-lang/acvm/issues/407)) ([040369a](https://github.com/noir-lang/acvm/commit/040369adc8749fa5ec2edd255ff54c105c3140f5))


### Miscellaneous Chores

* **acvm:** Remove `CircuitSimplifer` ([#421](https://github.com/noir-lang/acvm/issues/421)) ([e07a56d](https://github.com/noir-lang/acvm/commit/e07a56d9c542a7f03ce156761054cd403de0bd23))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * brillig_vm bumped from 0.17.0 to 0.18.0

## [0.17.0](https://github.com/noir-lang/acvm/compare/acvm-v0.16.0...acvm-v0.17.0) (2023-07-07)


### ⚠ BREAKING CHANGES

* **acir:** add `EcdsaSecp256r1` blackbox function ([#408](https://github.com/noir-lang/acvm/issues/408))

### Features

* **acir:** add `EcdsaSecp256r1` blackbox function ([#408](https://github.com/noir-lang/acvm/issues/408)) ([9895817](https://github.com/noir-lang/acvm/commit/98958170c9fa9b4731e33b31cb494a72bb90549e))

## [0.16.0](https://github.com/noir-lang/acvm/compare/acvm-v0.15.1...acvm-v0.16.0) (2023-07-06)


### ⚠ BREAKING CHANGES

* **acvm:** replace `PartialWitnessGeneratorStatus` with `ACVMStatus` ([#410](https://github.com/noir-lang/acvm/issues/410))
* **acir:** revert changes to `SchnorrVerify` opcode ([#409](https://github.com/noir-lang/acvm/issues/409))
* **acvm:** Replace `PartialWitnessGenerator` trait with `BlackBoxFunctionSolver` ([#378](https://github.com/noir-lang/acvm/issues/378))
* **acvm:** Encapsulate internal state of ACVM within a struct ([#384](https://github.com/noir-lang/acvm/issues/384))
* remove unused `OpcodeResolutionError::IncorrectNumFunctionArguments` variant ([#397](https://github.com/noir-lang/acvm/issues/397))
* **acir:** Remove `Oracle` opcode ([#368](https://github.com/noir-lang/acvm/issues/368))
* **acir:** Use fixed length data structures in black box function inputs/outputs where possible. ([#386](https://github.com/noir-lang/acvm/issues/386))

### Features

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

## [0.15.1](https://github.com/noir-lang/acvm/compare/acvm-v0.15.0...acvm-v0.15.1) (2023-06-20)


### Features

* **brillig:** Allow dynamic-size foreign calls ([#370](https://github.com/noir-lang/acvm/issues/370)) ([5ba0349](https://github.com/noir-lang/acvm/commit/5ba0349420cc1b20113cb5e96490a0808a769757))

## [0.15.0](https://github.com/noir-lang/acvm/compare/acvm-v0.14.2...acvm-v0.15.0) (2023-06-15)


### ⚠ BREAKING CHANGES

* **brillig:** Accept multiple inputs/outputs for foreign calls ([#367](https://github.com/noir-lang/acvm/issues/367))
* **acvm:** Make internals of ACVM private ([#353](https://github.com/noir-lang/acvm/issues/353))

### Features

* Add method to generate updated `Brillig` opcode from `UnresolvedBrilligCall` ([#363](https://github.com/noir-lang/acvm/issues/363)) ([fda5dbe](https://github.com/noir-lang/acvm/commit/fda5dbe57c28dc4bc28dfd8fe0a4a8ba29635393))
* **brillig:** Accept multiple inputs/outputs for foreign calls ([#367](https://github.com/noir-lang/acvm/issues/367)) ([78d62b2](https://github.com/noir-lang/acvm/commit/78d62b2d7c1c8b884e1f3fe7983e6e5029700e70))


### Miscellaneous Chores

* **acvm:** Make internals of ACVM private ([#353](https://github.com/noir-lang/acvm/issues/353)) ([c902a01](https://github.com/noir-lang/acvm/commit/c902a01639033665d106e2d9f4e5c7070af8c0bb))

## [0.14.2](https://github.com/noir-lang/acvm/compare/acvm-v0.14.1...acvm-v0.14.2) (2023-06-08)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions

## [0.14.1](https://github.com/noir-lang/acvm/compare/acvm-v0.14.0...acvm-v0.14.1) (2023-06-07)


### Features

* Re-use intermediate variables created during width reduction, with proper scale. ([#343](https://github.com/noir-lang/acvm/issues/343)) ([6bd0baa](https://github.com/noir-lang/acvm/commit/6bd0baa4bc9ac204e7710ec6d17d1752d2e924c0))

## [0.14.0](https://github.com/noir-lang/acvm/compare/acvm-v0.13.3...acvm-v0.14.0) (2023-06-06)


### ⚠ BREAKING CHANGES

* **acir:** Verify Proof ([#291](https://github.com/noir-lang/acvm/issues/291))

### Features

* **acir:** Verify Proof ([#291](https://github.com/noir-lang/acvm/issues/291)) ([9f34428](https://github.com/noir-lang/acvm/commit/9f34428b7084c7c38de401a16ca76e748d8b1d77))

## [0.13.3](https://github.com/noir-lang/acvm/compare/acvm-v0.13.2...acvm-v0.13.3) (2023-06-05)


### Bug Fixes

* Empty commit to trigger release-please ([e8f0748](https://github.com/noir-lang/acvm/commit/e8f0748042ef505d59ab63266d3c36c5358ee30d))

## [0.13.2](https://github.com/noir-lang/acvm/compare/acvm-v0.13.1...acvm-v0.13.2) (2023-06-02)


### Bug Fixes

* re-use intermediate vars during width reduction ([#278](https://github.com/noir-lang/acvm/issues/278)) ([5b32920](https://github.com/noir-lang/acvm/commit/5b32920263c4481c60faf0b84f0031aa8149b6b2))

## [0.13.1](https://github.com/noir-lang/acvm/compare/acvm-v0.13.0...acvm-v0.13.1) (2023-06-01)


### Bug Fixes

* **brillig:** Proper error handling for Brillig failures ([#329](https://github.com/noir-lang/acvm/issues/329)) ([cffa110](https://github.com/noir-lang/acvm/commit/cffa110c8df30ee3dd8b635d38b17b1fcd54b03e))
* **ci:** Correct typo to avoid `undefined` in changelogs ([#333](https://github.com/noir-lang/acvm/issues/333)) ([d3424c0](https://github.com/noir-lang/acvm/commit/d3424c04fd303c9cbe25d03118d8b358cbb84b83))

## [0.13.0](https://github.com/noir-lang/acvm/compare/acvm-v0.12.0...acvm-v0.13.0) (2023-06-01)


### ⚠ BREAKING CHANGES

* added hash index to pedersen ([#281](https://github.com/noir-lang/acvm/issues/281))
* Add variable length keccak opcode ([#314](https://github.com/noir-lang/acvm/issues/314))
* Remove AES opcode ([#302](https://github.com/noir-lang/acvm/issues/302))
* **acir, acvm:** Remove ComputeMerkleRoot opcode #296
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
* Reorganize code related to `PartialWitnessGenerator` ([#287](https://github.com/noir-lang/acvm/issues/287)) ([b9d61a1](https://github.com/noir-lang/acvm/commit/b9d61a16210d70e350a7e953951362c94f497f89))

## [0.12.0](https://github.com/noir-lang/acvm/compare/acvm-v0.11.0...acvm-v0.12.0) (2023-05-17)


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


### Miscellaneous Chores

* **acvm:** Backend trait must implement Debug ([#275](https://github.com/noir-lang/acvm/issues/275)) ([3288b4c](https://github.com/noir-lang/acvm/commit/3288b4c7eb01f5621e577d5ff9e7c92c7757e021))
* **acvm:** expose separate solvers for AND and XOR opcodes ([#266](https://github.com/noir-lang/acvm/issues/266)) ([84b5d18](https://github.com/noir-lang/acvm/commit/84b5d18d29a111a42bfc1c3d122129c8f062c3db))
* **acvm:** rename `hash_to_field128_security` to `hash_to_field_128_security` ([#271](https://github.com/noir-lang/acvm/issues/271)) ([fad9af2](https://github.com/noir-lang/acvm/commit/fad9af27fb102fa34bf7511f8ed7b16b3ec2d115))
* allow backends to specify support for all opcode variants ([#273](https://github.com/noir-lang/acvm/issues/273)) ([efd37fe](https://github.com/noir-lang/acvm/commit/efd37fedcbbabb3fac810e662731439e07fef49a))
* remove `OpcodeResolutionError::UnexpectedOpcode` ([#274](https://github.com/noir-lang/acvm/issues/274)) ([0e71aac](https://github.com/noir-lang/acvm/commit/0e71aac7aa85b3e9142972a26ba122c2c7c51d9b))
* remove deprecated circuit hash functions ([#288](https://github.com/noir-lang/acvm/issues/288)) ([1a22c75](https://github.com/noir-lang/acvm/commit/1a22c752de3354a2a6d34892331ab6623b24c0b0))

## [0.11.0](https://github.com/noir-lang/acvm/compare/acvm-v0.10.3...acvm-v0.11.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248))

### Features

* **acvm:** Add generic error for failing to solve an opcode ([#251](https://github.com/noir-lang/acvm/issues/251)) ([bc89528](https://github.com/noir-lang/acvm/commit/bc8952820de610e585d505decfac6e590bbb1a35))
* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248)) ([45c45f7](https://github.com/noir-lang/acvm/commit/45c45f7cdb79c3ccb0373ca0e698b282d4dabc39))
* Add Keccak Hash function ([#259](https://github.com/noir-lang/acvm/issues/259)) ([443c734](https://github.com/noir-lang/acvm/commit/443c73482eeef6cc42a1a254bf0d7706698ee353))

## [0.10.3](https://github.com/noir-lang/acvm/compare/acvm-v0.10.2...acvm-v0.10.3) (2023-04-28)


### Bug Fixes

* add default feature flag to ACVM crate ([#245](https://github.com/noir-lang/acvm/issues/245)) ([455fddb](https://github.com/noir-lang/acvm/commit/455fddbc19af81cb01d54e29cad199691e1a1d98))

## [0.10.2](https://github.com/noir-lang/acvm/compare/acvm-v0.10.1...acvm-v0.10.2) (2023-04-28)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions

## [0.10.1](https://github.com/noir-lang/acvm/compare/acvm-v0.10.0...acvm-v0.10.1) (2023-04-28)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions

## [0.10.0](https://github.com/noir-lang/acvm/compare/acvm-v0.9.0...acvm-v0.10.0) (2023-04-26)


### ⚠ BREAKING CHANGES

* return `Result<OpcodeResolution, OpcodeResolutionError>` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238))
* **acvm:** have all black box functions return `Result<OpcodeResolution, OpcodeResolutionError>` ([#237](https://github.com/noir-lang/acvm/issues/237))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223))
* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190))

### Features

* **acvm:** have all black box functions return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` ([#237](https://github.com/noir-lang/acvm/issues/237)) ([e8e93fd](https://github.com/noir-lang/acvm/commit/e8e93fda0db18f0d266dd1aacbb53ec787992dc9))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230)) ([198fb69](https://github.com/noir-lang/acvm/commit/198fb69e90a5ed3c0a8716d888b4dc6c2f9b18aa))
* Add range opcode optimization ([#219](https://github.com/noir-lang/acvm/issues/219)) ([7abe6e5](https://github.com/noir-lang/acvm/commit/7abe6e5f6d6fea379c3748a910afd00db066eb45))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223)) ([00282dc](https://github.com/noir-lang/acvm/commit/00282dc5e2b03947bf709a088d829f3e0ba80eed))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213)) ([e877bed](https://github.com/noir-lang/acvm/commit/e877bed2cca76bd486e9bed66b4230e65a01f0a2))
* return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238)) ([15d3c5a](https://github.com/noir-lang/acvm/commit/15d3c5a9be2dd92f266fcb7e672da17cada9fec5))


### Bug Fixes

* prevent `bn254` feature flag always being enabled ([#225](https://github.com/noir-lang/acvm/issues/225)) ([82eee6a](https://github.com/noir-lang/acvm/commit/82eee6ab08ae480f04904ca8571fd88f4466c000))


### Miscellaneous Chores

* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220)) ([64bb346](https://github.com/noir-lang/acvm/commit/64bb346524428a0ce196826ea1e5ccde08ad6201))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190)) ([a619df6](https://github.com/noir-lang/acvm/commit/a619df614bbb9b2518b788b42a7553b069823a0f))

## [0.9.0](https://github.com/noir-lang/acvm/compare/acvm-v0.8.1...acvm-v0.9.0) (2023-04-07)


### ⚠ BREAKING CHANGES

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185))
* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180))

### Features

* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180)) ([fd28657](https://github.com/noir-lang/acvm/commit/fd28657426260ce3c53517b75a27eb5c4a74e234))


### Miscellaneous Chores

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185)) ([ee59c9e](https://github.com/noir-lang/acvm/commit/ee59c9efe9a54ff6b97e4daaebf64f3e327e97d9))

## [0.8.1](https://github.com/noir-lang/acvm/compare/acvm-v0.8.0...acvm-v0.8.1) (2023-03-30)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions

## [0.8.0](https://github.com/noir-lang/acvm/compare/acvm-v0.7.1...acvm-v0.8.0) (2023-03-28)


### Miscellaneous Chores

* **acvm:** Synchronize acvm versions

## [0.7.1](https://github.com/noir-lang/acvm/compare/acvm-v0.7.0...acvm-v0.7.1) (2023-03-27)


### Bug Fixes

* **pwg:** stall instead of fail for unassigned black box ([#154](https://github.com/noir-lang/acvm/issues/154)) ([412a1a6](https://github.com/noir-lang/acvm/commit/412a1a60b434bef53e12d37c3b2bb3d51a317994))

## [0.7.0](https://github.com/noir-lang/acvm/compare/acvm-v0.6.0...acvm-v0.7.0) (2023-03-23)


### ⚠ BREAKING CHANGES

* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149))
* **acir:** Add RAM and ROM opcodes
* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142))

### Features

* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56)) ([5f358a9](https://github.com/noir-lang/acvm/commit/5f358a97aaa81d87956e182cd8a6d60de75f9752))
* **acir:** Add RAM and ROM opcodes ([73e9f25](https://github.com/noir-lang/acvm/commit/73e9f25dd87b2ca91245e93d2445eadc0f522fac))
* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149)) ([88ee2f8](https://github.com/noir-lang/acvm/commit/88ee2f89f37abf5dd1d9f91b4d2eed44dc651348))


### Miscellaneous Chores

* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140)) ([35dd181](https://github.com/noir-lang/acvm/commit/35dd181102203df17eef510666b327ef41f4b036))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142)) ([85dd6e8](https://github.com/noir-lang/acvm/commit/85dd6e85bfba85bfb97651f7e30e1f75deb986d5))

## [0.6.0](https://github.com/noir-lang/acvm/compare/acvm-v0.5.0...acvm-v0.6.0) (2023-03-03)


### ⚠ BREAKING CHANGES

* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114))

### Features

* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114)) ([097cfb0](https://github.com/noir-lang/acvm/commit/097cfb069291705ddb4bf1fca77ddcef21dbbd08))

## [0.5.0](https://github.com/noir-lang/acvm/compare/acvm-v0.4.1...acvm-v0.5.0) (2023-02-22)


### ⚠ BREAKING CHANGES

* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96))
* **acvm:** add `eth_contract_from_vk` to `SmartContract
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108))
* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88))

### Features

* **acvm:** add `eth_contract_from_vk` to `SmartContract ([#113](https://github.com/noir-lang/acvm/issues/113)) ([373c18f](https://github.com/noir-lang/acvm/commit/373c18fc05edf673cfec9e8bbb78bd7d7514999e))
* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96)) ([f57ba57](https://github.com/noir-lang/acvm/commit/f57ba57c2bb2597edf2b02fb1321c69cf11993ee))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108)) ([af56ca9](https://github.com/noir-lang/acvm/commit/af56ca9da06068c650c66e76bfd09e65eb0ec213))
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111)) ([39b8a41](https://github.com/noir-lang/acvm/commit/39b8a41293e567971f700f61103852cb987a8d16))


### Bug Fixes

* Clean up Log Directive hex output  ([#97](https://github.com/noir-lang/acvm/issues/97)) ([d23c735](https://github.com/noir-lang/acvm/commit/d23c7352523ffb42f3e8f4229b61f9803ab78a7e))


### Miscellaneous Chores

* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58)) ([2427a27](https://github.com/noir-lang/acvm/commit/2427a275048e598c6d651cce8348a4c55148f235))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88)) ([9329307](https://github.com/noir-lang/acvm/commit/9329307e054de202cfc55207162ad952b70d515e))
