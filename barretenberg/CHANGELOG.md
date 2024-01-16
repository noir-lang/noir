# Changelog

## [0.18.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.17.0...barretenberg-v0.18.0) (2024-01-16)


### ⚠ BREAKING CHANGES

* Remove `Directive::Quotient` ([#4019](https://github.com/AztecProtocol/aztec-packages/issues/4019))
* implement keccakf1600 in brillig ([#3914](https://github.com/AztecProtocol/aztec-packages/issues/3914))
* add blake3 opcode to brillig ([#3913](https://github.com/AztecProtocol/aztec-packages/issues/3913))
* Remove opcode supported from the backend ([#3889](https://github.com/AztecProtocol/aztec-packages/issues/3889))

### Features

* Acir cleanup ([#3845](https://github.com/AztecProtocol/aztec-packages/issues/3845)) ([390b84c](https://github.com/AztecProtocol/aztec-packages/commit/390b84ced39ea8ed76c291019e63d18a772f7c3c))
* Add ACIR opcodes for ECADD and ECDOUBLE ([#3878](https://github.com/AztecProtocol/aztec-packages/issues/3878)) ([537630f](https://github.com/AztecProtocol/aztec-packages/commit/537630ffe1b3747a03aa821687e33db04b1fe3ad))
* Add blake3 opcode to brillig ([#3913](https://github.com/AztecProtocol/aztec-packages/issues/3913)) ([34fad0a](https://github.com/AztecProtocol/aztec-packages/commit/34fad0a76c58139b4b56f372aa6f39f897286b3a))
* Bench bb in pr's, docker shell utils ([#3561](https://github.com/AztecProtocol/aztec-packages/issues/3561)) ([5408919](https://github.com/AztecProtocol/aztec-packages/commit/54089190a4532988cec9f88d199263aeafa2c2f3))
* Benchmark protogalaxy prover ([#3958](https://github.com/AztecProtocol/aztec-packages/issues/3958)) ([5843722](https://github.com/AztecProtocol/aztec-packages/commit/5843722ff5e888bf858016c6d005af37fac42e1c))
* Benchmarks for basic functionality and IPA improvements ([#4004](https://github.com/AztecProtocol/aztec-packages/issues/4004)) ([fd1f619](https://github.com/AztecProtocol/aztec-packages/commit/fd1f619f443916c172b6311aa71a84153145ef4d))
* Bootstrap cache v2 ([#3876](https://github.com/AztecProtocol/aztec-packages/issues/3876)) ([331598d](https://github.com/AztecProtocol/aztec-packages/commit/331598d369ab9bb91dcc48d50bdd8df0684f0b79))
* Implement keccakf1600 in brillig ([#3914](https://github.com/AztecProtocol/aztec-packages/issues/3914)) ([a182381](https://github.com/AztecProtocol/aztec-packages/commit/a18238180cbd6c71f75fcfcb1a093ac29c839aeb))
* Parallel IPA ([#3882](https://github.com/AztecProtocol/aztec-packages/issues/3882)) ([7002a33](https://github.com/AztecProtocol/aztec-packages/commit/7002a332da3bb9a75d5164a068a2bd9ea1ad211a))
* Pil lookups w/ xor table example ([#3880](https://github.com/AztecProtocol/aztec-packages/issues/3880)) ([544d24e](https://github.com/AztecProtocol/aztec-packages/commit/544d24e419a604c4720988315239e365f06205b1))
* Poseidon2 stdlib impl ([#3551](https://github.com/AztecProtocol/aztec-packages/issues/3551)) ([50b4a72](https://github.com/AztecProtocol/aztec-packages/commit/50b4a728b4c20503f6ab56c07feaa29d767cec10))
* Protogalaxy Decider and complete folding tests ([#3657](https://github.com/AztecProtocol/aztec-packages/issues/3657)) ([cfdaf9c](https://github.com/AztecProtocol/aztec-packages/commit/cfdaf9c1980356764a0bed88bc01358b8e807bd0))
* Relations vs widgets benchmarking ([#3931](https://github.com/AztecProtocol/aztec-packages/issues/3931)) ([3af64ef](https://github.com/AztecProtocol/aztec-packages/commit/3af64eff3a32922849cb0fd1977ee89a6ea7cd07))
* Remove opcode supported from the backend ([#3889](https://github.com/AztecProtocol/aztec-packages/issues/3889)) ([1fd135c](https://github.com/AztecProtocol/aztec-packages/commit/1fd135cb61a0b0419a339743c2a4fa9890d62720))
* Reorganize acir composer ([#3957](https://github.com/AztecProtocol/aztec-packages/issues/3957)) ([e6232e8](https://github.com/AztecProtocol/aztec-packages/commit/e6232e8ded1fa731565b17b77b0b2be80b2ef6e2))
* Standalone calldata test ([#3842](https://github.com/AztecProtocol/aztec-packages/issues/3842)) ([7353a35](https://github.com/AztecProtocol/aztec-packages/commit/7353a358aa3f364d1d31fd00c73a9e1a4b6dff4e))


### Bug Fixes

* Bb unnecessary env var ([#3901](https://github.com/AztecProtocol/aztec-packages/issues/3901)) ([f127e5a](https://github.com/AztecProtocol/aztec-packages/commit/f127e5a4176d00e641c8f2308ebf105f415cb914))


### Miscellaneous

* Codegen acir opcodes after renaming arithmetic to assertzero ([#3896](https://github.com/AztecProtocol/aztec-packages/issues/3896)) ([c710ce1](https://github.com/AztecProtocol/aztec-packages/commit/c710ce19eaa3fbcf7c83957e7341a6ca10677ef1))
* Document `witness_buf_to_witness_data` ([#3940](https://github.com/AztecProtocol/aztec-packages/issues/3940)) ([fbaa726](https://github.com/AztecProtocol/aztec-packages/commit/fbaa72641c50cc7f05712e266416f12c4edf8fe9))
* Remove 'extern template's, expand macros ([#3953](https://github.com/AztecProtocol/aztec-packages/issues/3953)) ([5fe9908](https://github.com/AztecProtocol/aztec-packages/commit/5fe99085963cec32a2d411b95ab8887578a90253))
* Remove `Directive::Quotient` ([#4019](https://github.com/AztecProtocol/aztec-packages/issues/4019)) ([824d76f](https://github.com/AztecProtocol/aztec-packages/commit/824d76f363180821678238f1474a00520f781758))
* Reorganize benchmarks ([#3909](https://github.com/AztecProtocol/aztec-packages/issues/3909)) ([730766b](https://github.com/AztecProtocol/aztec-packages/commit/730766b07d9521c0ec6c0606042b506edbc5db48))

## [0.17.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.9...barretenberg-v0.17.0) (2024-01-09)


### ⚠ BREAKING CHANGES

* Remove aggregation objects from RecursionConstraint ([#3885](https://github.com/AztecProtocol/aztec-packages/issues/3885))
* Noir development branch (serialization changes) ([#3858](https://github.com/AztecProtocol/aztec-packages/issues/3858))
* Add Side effect counter struct for ordering ([#3608](https://github.com/AztecProtocol/aztec-packages/issues/3608))
* return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735))

### Features

* Adding option to set initial and max memory ([#3265](https://github.com/AztecProtocol/aztec-packages/issues/3265)) ([0ad75fe](https://github.com/AztecProtocol/aztec-packages/commit/0ad75fe745099119726976f964a92d1587f32fbf))
* **avm-main:** Pil -&gt; permutations ([#3650](https://github.com/AztecProtocol/aztec-packages/issues/3650)) ([c52acf6](https://github.com/AztecProtocol/aztec-packages/commit/c52acf64cf00443867f8f578a1c25cda49583faf))
* **avm-mini:** Call and return opcodes ([#3704](https://github.com/AztecProtocol/aztec-packages/issues/3704)) ([e534204](https://github.com/AztecProtocol/aztec-packages/commit/e534204c760db31eb1347cd76e85d151a1fb8305))
* **avm:** Add standalone jump opcode ([#3781](https://github.com/AztecProtocol/aztec-packages/issues/3781)) ([b1b2e7c](https://github.com/AztecProtocol/aztec-packages/commit/b1b2e7ca28ba56cf0bae0f906734df00458714b9))
* **avm:** VM circuit handles tagged memory ([#3725](https://github.com/AztecProtocol/aztec-packages/issues/3725)) ([739fe90](https://github.com/AztecProtocol/aztec-packages/commit/739fe90a50891d99b03a8f34da556c8725673f80)), closes [#3644](https://github.com/AztecProtocol/aztec-packages/issues/3644)
* Barretenberg doxygen CI ([#3818](https://github.com/AztecProtocol/aztec-packages/issues/3818)) ([022a918](https://github.com/AztecProtocol/aztec-packages/commit/022a918911817b1897fd69ea72da84054450c8cb))
* Bb uses goblin ([#3636](https://github.com/AztecProtocol/aztec-packages/issues/3636)) ([d093266](https://github.com/AztecProtocol/aztec-packages/commit/d09326636140dbd68d3efb8bc4ec2b6948e2bfe1))
* Correct circuit construction from acir ([#3757](https://github.com/AztecProtocol/aztec-packages/issues/3757)) ([a876ab8](https://github.com/AztecProtocol/aztec-packages/commit/a876ab8a61108be06bd5d884d727058e7e54a383))
* Goblin and eccvm bench ([#3606](https://github.com/AztecProtocol/aztec-packages/issues/3606)) ([1fe63b2](https://github.com/AztecProtocol/aztec-packages/commit/1fe63b2cf5b83fef576bb99294700743929d5ec7))
* Goblinize the final ecc ops in ZM ([#3741](https://github.com/AztecProtocol/aztec-packages/issues/3741)) ([3048d08](https://github.com/AztecProtocol/aztec-packages/commit/3048d0820c89f3bcce38913d3744cf5be1ece14f))
* Noir development branch (serialization changes) ([#3858](https://github.com/AztecProtocol/aztec-packages/issues/3858)) ([d2ae2cd](https://github.com/AztecProtocol/aztec-packages/commit/d2ae2cd529b0ef132c0b6c7c35938066c89d809c))
* ProverPolynomials owns its memory  ([#3560](https://github.com/AztecProtocol/aztec-packages/issues/3560)) ([a4aba00](https://github.com/AztecProtocol/aztec-packages/commit/a4aba0061929c96bf9cccb64916f96011688a3e1))
* Return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735)) ([bd5614c](https://github.com/AztecProtocol/aztec-packages/commit/bd5614c2ee04065e149d3df48f1ace9c0ce3858f))


### Bug Fixes

* CRS not needed for gate_count. Grumpkin not needed for non-goblin. ([#3872](https://github.com/AztecProtocol/aztec-packages/issues/3872)) ([8cda00d](https://github.com/AztecProtocol/aztec-packages/commit/8cda00d94946ed7e8dfc1dbafdefae3e6d1af682))
* Disable goblin bbjs tests ([#3836](https://github.com/AztecProtocol/aztec-packages/issues/3836)) ([1f5b2c6](https://github.com/AztecProtocol/aztec-packages/commit/1f5b2c606def0c7203cbd7497264c95bbfa708e1))
* Reenable goblin bbjs for a single test ([#3838](https://github.com/AztecProtocol/aztec-packages/issues/3838)) ([30e47a0](https://github.com/AztecProtocol/aztec-packages/commit/30e47a005c39ae0af80ef33b83251d04046191dc))
* Update toy to new master ([78cf525](https://github.com/AztecProtocol/aztec-packages/commit/78cf525dcacba77386779a74b6f806fba47f1bc7))


### Miscellaneous

* Add Side effect counter struct for ordering ([#3608](https://github.com/AztecProtocol/aztec-packages/issues/3608)) ([c58b197](https://github.com/AztecProtocol/aztec-packages/commit/c58b197512297a292cfddd253d8d951b207829a0))
* Align bb.js testing ([#3840](https://github.com/AztecProtocol/aztec-packages/issues/3840)) ([c489727](https://github.com/AztecProtocol/aztec-packages/commit/c4897270515f23891a32807dd2be046be12d5095))
* **avm:** Avm memory trace building ([#3835](https://github.com/AztecProtocol/aztec-packages/issues/3835)) ([b7766d6](https://github.com/AztecProtocol/aztec-packages/commit/b7766d68727c92f92abc91131a4332db25d805dd))
* Bring boxes back to CI. Build and run using docker/docker-compose. ([#3727](https://github.com/AztecProtocol/aztec-packages/issues/3727)) ([4a1c0df](https://github.com/AztecProtocol/aztec-packages/commit/4a1c0df76f26530521daaaa60945fead106b555e))
* Cleanup recursion interface ([#3744](https://github.com/AztecProtocol/aztec-packages/issues/3744)) ([fde0ac3](https://github.com/AztecProtocol/aztec-packages/commit/fde0ac3e96fe6e2edcdb1e6919d372e96181eda5))
* **dsl:** Abstract nested aggregation object from ACIR ([#3765](https://github.com/AztecProtocol/aztec-packages/issues/3765)) ([92f72e4](https://github.com/AztecProtocol/aztec-packages/commit/92f72e44d4b57a3078da6bd1bb39dd0f615785be))
* Remove aggregation objects from RecursionConstraint ([#3885](https://github.com/AztecProtocol/aztec-packages/issues/3885)) ([9a80008](https://github.com/AztecProtocol/aztec-packages/commit/9a80008c623a9d26e1b82c9e86561c304ef185f1))
* Remove HashToField128Security ACIR opcode ([#3631](https://github.com/AztecProtocol/aztec-packages/issues/3631)) ([1d6d3c9](https://github.com/AztecProtocol/aztec-packages/commit/1d6d3c94f327de1f20ef7d78302d3957db70019e))
* Use simple "flat" CRS. ([#3748](https://github.com/AztecProtocol/aztec-packages/issues/3748)) ([5c6c2ca](https://github.com/AztecProtocol/aztec-packages/commit/5c6c2caf212fb22856df41fd15464dda37e10dab))

## [0.16.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.8...barretenberg-v0.16.9) (2023-12-13)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.16.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.7...barretenberg-v0.16.8) (2023-12-13)


### Features

* Complete folding prover and verifier for ultra instances ([#3419](https://github.com/AztecProtocol/aztec-packages/issues/3419)) ([bb86ce9](https://github.com/AztecProtocol/aztec-packages/commit/bb86ce9a27e09b8a336af04b787b81d5f1d49ac8))
* Copy constructors for builders ([#3635](https://github.com/AztecProtocol/aztec-packages/issues/3635)) ([b82b0c5](https://github.com/AztecProtocol/aztec-packages/commit/b82b0c579c4a315c9b4eaf3e9726275633603b5a))
* Log-derivative based generic permutations for AVM ([#3428](https://github.com/AztecProtocol/aztec-packages/issues/3428)) ([379b5ad](https://github.com/AztecProtocol/aztec-packages/commit/379b5adc259ac69b01e61b852172cdfc87cf9350))
* Merge recursive verifier ([#3588](https://github.com/AztecProtocol/aztec-packages/issues/3588)) ([cdd9259](https://github.com/AztecProtocol/aztec-packages/commit/cdd92595c313617189a530e0bfda987db211ae6b))


### Bug Fixes

* Aztec sandbox compose fixes ([#3634](https://github.com/AztecProtocol/aztec-packages/issues/3634)) ([765a19c](https://github.com/AztecProtocol/aztec-packages/commit/765a19c3aad3a2793a764b970b7cc8a819094da7))
* Broken uint256_t implicit copy ([#3625](https://github.com/AztecProtocol/aztec-packages/issues/3625)) ([1a6b44d](https://github.com/AztecProtocol/aztec-packages/commit/1a6b44d67e077eb5904ab30255454693d6a1edac))


### Miscellaneous

* Nuke fib ([#3607](https://github.com/AztecProtocol/aztec-packages/issues/3607)) ([48e2e3d](https://github.com/AztecProtocol/aztec-packages/commit/48e2e3d261a7091cb0b87565ec8bc9ae595b3022))

## [0.16.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.6...barretenberg-v0.16.7) (2023-12-06)


### Features

* Encapsulated Goblin ([#3524](https://github.com/AztecProtocol/aztec-packages/issues/3524)) ([2f08423](https://github.com/AztecProtocol/aztec-packages/commit/2f08423e37942f991634fe6c45de52feb1f159cf))

## [0.16.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.5...barretenberg-v0.16.6) (2023-12-06)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.16.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.4...barretenberg-v0.16.5) (2023-12-06)


### Miscellaneous

* Trivial change roundup ([#3556](https://github.com/AztecProtocol/aztec-packages/issues/3556)) ([ff893b2](https://github.com/AztecProtocol/aztec-packages/commit/ff893b236091b480b6de18ebaab57c62dcdfe1d4))


### Documentation

* Add libstdc++-12-dev to setup instructions ([#3585](https://github.com/AztecProtocol/aztec-packages/issues/3585)) ([9773e8c](https://github.com/AztecProtocol/aztec-packages/commit/9773e8c3b4789f0dd6b5fdaf0f283b9bd7c9812f))

## [0.16.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.3...barretenberg-v0.16.4) (2023-12-05)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.16.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.2...barretenberg-v0.16.3) (2023-12-05)


### Miscellaneous

* CLI's startup time was pushing almost 2s. This gets the basic 'help' down to 0.16. ([#3529](https://github.com/AztecProtocol/aztec-packages/issues/3529)) ([396df13](https://github.com/AztecProtocol/aztec-packages/commit/396df13389cdcb8b8b0d5a92a4b3d1c2bffcb7a7))

## [0.16.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.1...barretenberg-v0.16.2) (2023-12-05)


### Features

* **AVM:** First version for mini AVM (ADD, RETURN, CALLDATACOPY) ([#3439](https://github.com/AztecProtocol/aztec-packages/issues/3439)) ([b3af146](https://github.com/AztecProtocol/aztec-packages/commit/b3af1463ed6b7858252ab4779f8c747a6de47363))
* Flavor refactor, reduce duplication ([#3407](https://github.com/AztecProtocol/aztec-packages/issues/3407)) ([8d6b013](https://github.com/AztecProtocol/aztec-packages/commit/8d6b01304d797f7cbb576b23a7e115390d113c79))
* New Poseidon2 circuit builder gates ([#3346](https://github.com/AztecProtocol/aztec-packages/issues/3346)) ([91cb369](https://github.com/AztecProtocol/aztec-packages/commit/91cb369aa7ecbf457965f53057cafa2c2e6f1214))
* New Poseidon2 relations ([#3406](https://github.com/AztecProtocol/aztec-packages/issues/3406)) ([14b9736](https://github.com/AztecProtocol/aztec-packages/commit/14b9736925c6da33133bd24ee283fb4c199082a5))
* Pull latest noir for brillig optimizations ([#3464](https://github.com/AztecProtocol/aztec-packages/issues/3464)) ([d356bac](https://github.com/AztecProtocol/aztec-packages/commit/d356bac740d203fbb9363a0127ca1d433358e029))
* Seperate pil files for sub machines ([#3454](https://github.com/AztecProtocol/aztec-packages/issues/3454)) ([d09d6f5](https://github.com/AztecProtocol/aztec-packages/commit/d09d6f5a5f2c7e2a58658a640a6a6d6ba4294701))


### Miscellaneous

* **avm:** Enable AVM unit tests in CI ([#3463](https://github.com/AztecProtocol/aztec-packages/issues/3463)) ([051dda9](https://github.com/AztecProtocol/aztec-packages/commit/051dda9c50f1d9f11f5063ddf51c9986a6998b43)), closes [#3461](https://github.com/AztecProtocol/aztec-packages/issues/3461)
* **bb:** Pointer_view to reference-based get_all ([#3495](https://github.com/AztecProtocol/aztec-packages/issues/3495)) ([50d7327](https://github.com/AztecProtocol/aztec-packages/commit/50d73271919306a05ac3a7c2e7d37363b6761248))
* **bb:** Reuse entities from GoblinUltra in GoblinUltraRecursive ([#3521](https://github.com/AztecProtocol/aztec-packages/issues/3521)) ([8259636](https://github.com/AztecProtocol/aztec-packages/commit/8259636c016c0adecb052f176e78444fb5481c38))
* Build the acir test vectors as part of CI. ([#3447](https://github.com/AztecProtocol/aztec-packages/issues/3447)) ([1a2d1f8](https://github.com/AztecProtocol/aztec-packages/commit/1a2d1f822d0e1fabd322c2c4d0473629edd56380))
* Field-agnostic and reusable transcript ([#3433](https://github.com/AztecProtocol/aztec-packages/issues/3433)) ([d78775a](https://github.com/AztecProtocol/aztec-packages/commit/d78775adb9574a3d76c3fca8cf940cdef460ae10))
* Optimise bb.js package size and sandox/cli dockerfiles to unbloat final containers. ([#3462](https://github.com/AztecProtocol/aztec-packages/issues/3462)) ([cb3db5d](https://github.com/AztecProtocol/aztec-packages/commit/cb3db5d0f1f8912f1a97258e5043eb0f69eff551))
* Pin node version in docker base images and bump nvmrc ([#3537](https://github.com/AztecProtocol/aztec-packages/issues/3537)) ([5d3895a](https://github.com/AztecProtocol/aztec-packages/commit/5d3895aefb7812eb6bd8017baf43533959ad69b4))
* Recursive verifier updates ([#3452](https://github.com/AztecProtocol/aztec-packages/issues/3452)) ([dbb4a12](https://github.com/AztecProtocol/aztec-packages/commit/dbb4a1205528bdd8217ea2d15ccf060e2aa9b7d2))
* Refactor `WitnessEntities` to be able to derive `WitnessCommitments` from it ([#3479](https://github.com/AztecProtocol/aztec-packages/issues/3479)) ([9c9b561](https://github.com/AztecProtocol/aztec-packages/commit/9c9b561f392de5fce11cefe4d72e4f33f2567f41))
* Transcript handled through shared_ptr ([#3434](https://github.com/AztecProtocol/aztec-packages/issues/3434)) ([30fca33](https://github.com/AztecProtocol/aztec-packages/commit/30fca3307ee7e33d81fd51c3d280c6362baef0b9))
* Typo fixes ([#3488](https://github.com/AztecProtocol/aztec-packages/issues/3488)) ([d9a44dc](https://github.com/AztecProtocol/aztec-packages/commit/d9a44dc2e655752e1c6503ac85b64169ec7e4754))

## [0.16.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.16.0...barretenberg-v0.16.1) (2023-11-28)


### Features

* Added poseidon2 hash function to barretenberg/crypto ([#3118](https://github.com/AztecProtocol/aztec-packages/issues/3118)) ([d47782b](https://github.com/AztecProtocol/aztec-packages/commit/d47782bb480f7e016dbc77cf962978ddca0632aa))


### Miscellaneous

* Point acir tests at noir master branch ([#3440](https://github.com/AztecProtocol/aztec-packages/issues/3440)) ([106e690](https://github.com/AztecProtocol/aztec-packages/commit/106e690993cdc10db85903d91af873c04744c05f))

## [0.16.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.15.1...barretenberg-v0.16.0) (2023-11-27)


### Features

* Goblin proof construction ([#3332](https://github.com/AztecProtocol/aztec-packages/issues/3332)) ([6a7ebb6](https://github.com/AztecProtocol/aztec-packages/commit/6a7ebb60e4ecf0ae0d047814e22ecd88c9c7528f))
* Noir subrepo. ([#3369](https://github.com/AztecProtocol/aztec-packages/issues/3369)) ([d94d88b](https://github.com/AztecProtocol/aztec-packages/commit/d94d88bf626ddbe41dd1b7fe3eb0f11619dde97a))


### Miscellaneous

* Deterministically deduplicate `cached_partial_non_native_field_multiplication` across wasm32 and native compilations ([#3425](https://github.com/AztecProtocol/aztec-packages/issues/3425)) ([5524933](https://github.com/AztecProtocol/aztec-packages/commit/55249336212764da4b85634e7d35e8fedb147619))
* Plumbs noir subrepo into yarn-project. ([#3420](https://github.com/AztecProtocol/aztec-packages/issues/3420)) ([63173c4](https://github.com/AztecProtocol/aztec-packages/commit/63173c45db127288bc4b079229239a650fc5d4be))
* Update path to acir artifacts ([#3426](https://github.com/AztecProtocol/aztec-packages/issues/3426)) ([f56f88d](https://github.com/AztecProtocol/aztec-packages/commit/f56f88de05a0ebfcc34c279ae869956a48baa0f4))

## [0.15.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.15.0...barretenberg-v0.15.1) (2023-11-21)


### Features

* **bb:** Add ability to write pk to file or stdout ([#3335](https://github.com/AztecProtocol/aztec-packages/issues/3335)) ([c99862c](https://github.com/AztecProtocol/aztec-packages/commit/c99862c9602d7d37f7fef348e9f014fb137adab1))
* DataBus PoC (UltraHonk as extension of Ultra) ([#3181](https://github.com/AztecProtocol/aztec-packages/issues/3181)) ([dd9dd84](https://github.com/AztecProtocol/aztec-packages/commit/dd9dd84e9cfc93f8605f28aa25fa36b0004052cb))
* Fold batching challenge (alpha) ([#3291](https://github.com/AztecProtocol/aztec-packages/issues/3291)) ([bc99a4f](https://github.com/AztecProtocol/aztec-packages/commit/bc99a4f644824727920b0b4a38ec5ba915d5c0ce))
* Open transcript polys as univariates in ECCVM ([#3331](https://github.com/AztecProtocol/aztec-packages/issues/3331)) ([436b22e](https://github.com/AztecProtocol/aztec-packages/commit/436b22e35bf8a41f78def237889f2afd2ca79830))
* ZM updates for Translator concatenated polys ([#3343](https://github.com/AztecProtocol/aztec-packages/issues/3343)) ([0e425db](https://github.com/AztecProtocol/aztec-packages/commit/0e425dbfc99af9fc2598a957acd8b71f3fd45fe9))


### Bug Fixes

* Bootstrap bbjs. ([#3337](https://github.com/AztecProtocol/aztec-packages/issues/3337)) ([06aedcb](https://github.com/AztecProtocol/aztec-packages/commit/06aedcbfd601e243d3486763c1306e20c1ae3688))
* Updating pedersen benchmarks ([#3211](https://github.com/AztecProtocol/aztec-packages/issues/3211)) ([7e89ff3](https://github.com/AztecProtocol/aztec-packages/commit/7e89ff363521dd65e0c9f0c098b3bacea33c2764))


### Miscellaneous

* All hashes in ts ([#3333](https://github.com/AztecProtocol/aztec-packages/issues/3333)) ([6307e12](https://github.com/AztecProtocol/aztec-packages/commit/6307e129770af7791dc5a477859b75ebb112a653))

## [0.15.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.14.2...barretenberg-v0.15.0) (2023-11-16)


### ⚠ BREAKING CHANGES

* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114))

### Features

* **bb:** Add msan preset ([#3284](https://github.com/AztecProtocol/aztec-packages/issues/3284)) ([bcf025c](https://github.com/AztecProtocol/aztec-packages/commit/bcf025ceef07fb2bf5b07f96e7818425ae59e79a))
* Protogalaxy combiner quotient ([#3245](https://github.com/AztecProtocol/aztec-packages/issues/3245)) ([db0f3ab](https://github.com/AztecProtocol/aztec-packages/commit/db0f3ab9b3d74e0527116a773bf11d26e6bf7736))
* Ultra honk arith from ultra ([#3274](https://github.com/AztecProtocol/aztec-packages/issues/3274)) ([ec2b805](https://github.com/AztecProtocol/aztec-packages/commit/ec2b805e5b35805e2c5e394ae2b6181865e22aa3))


### Bug Fixes

* Debug build ([#3283](https://github.com/AztecProtocol/aztec-packages/issues/3283)) ([aca2624](https://github.com/AztecProtocol/aztec-packages/commit/aca2624df2d07782f6879d32efc891318b985344))
* Fix block constraint key divergence bug. ([#3256](https://github.com/AztecProtocol/aztec-packages/issues/3256)) ([1c71a0c](https://github.com/AztecProtocol/aztec-packages/commit/1c71a0cf38cf463efe1964126a6a5741c27bd2eb))


### Miscellaneous

* **bb:** Remove -Wfatal-errors ([#3318](https://github.com/AztecProtocol/aztec-packages/issues/3318)) ([4229173](https://github.com/AztecProtocol/aztec-packages/commit/4229173e7d794ba7800b34dcc8565d7f3ea5525d))
* Clarify that barretenberg mirror should not take PRs ([#3303](https://github.com/AztecProtocol/aztec-packages/issues/3303)) ([13f1a1d](https://github.com/AztecProtocol/aztec-packages/commit/13f1a1d4f8cd12ac8f38e2d1a2c6715f2871f4c8))
* Clean up Plonk widgets ([#3305](https://github.com/AztecProtocol/aztec-packages/issues/3305)) ([4623d91](https://github.com/AztecProtocol/aztec-packages/commit/4623d916d5e8d048cf3c5e06f02d937cf91e6180))
* Explicitly instantiate Goblin translator relations ([#3239](https://github.com/AztecProtocol/aztec-packages/issues/3239)) ([e3b5fb0](https://github.com/AztecProtocol/aztec-packages/commit/e3b5fb0681839bd003804a9e066118dd4693502d))
* Plain struct flavor entities ([#3277](https://github.com/AztecProtocol/aztec-packages/issues/3277)) ([f109512](https://github.com/AztecProtocol/aztec-packages/commit/f1095124af96d2d69522c8677e5e02cd55063c99))
* Remove bn254 instantiation of eccvm plus naming changes ([#3330](https://github.com/AztecProtocol/aztec-packages/issues/3330)) ([23d1e2d](https://github.com/AztecProtocol/aztec-packages/commit/23d1e2d307757c42f6a070afcb22f800fae94555))
* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114)) ([87eeb71](https://github.com/AztecProtocol/aztec-packages/commit/87eeb715014996ec329de969df85684083b18f83))
* Revert build-debug folder for debug preset ([#3324](https://github.com/AztecProtocol/aztec-packages/issues/3324)) ([43a2e6b](https://github.com/AztecProtocol/aztec-packages/commit/43a2e6b68853d5c22fac4563949c83baf443827c))
* Towards plain struct flavor entities ([#3216](https://github.com/AztecProtocol/aztec-packages/issues/3216)) ([3ba89cf](https://github.com/AztecProtocol/aztec-packages/commit/3ba89cf6fe3821b1149f482ee28c5e0716878b15))
* Typo fixes based on cspell ([#3319](https://github.com/AztecProtocol/aztec-packages/issues/3319)) ([8ae44dd](https://github.com/AztecProtocol/aztec-packages/commit/8ae44dd702987db524ab5e3edd6545881614f56b))

## [0.14.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.14.1...barretenberg-v0.14.2) (2023-11-07)


### Features

* Run solidity tests for all acir artifacts ([#3161](https://github.com/AztecProtocol/aztec-packages/issues/3161)) ([d09f667](https://github.com/AztecProtocol/aztec-packages/commit/d09f66748fcbb7739b17940a36806abb72091ee1))

## [0.14.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.14.0...barretenberg-v0.14.1) (2023-11-07)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.14.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.13.1...barretenberg-v0.14.0) (2023-11-07)


### Features

* Gperftools ([#3096](https://github.com/AztecProtocol/aztec-packages/issues/3096)) ([ea2f9a7](https://github.com/AztecProtocol/aztec-packages/commit/ea2f9a72674ae7fd3e810a12026bfc26c693e1c1))


### Bug Fixes

* Cleanup gen_inner_proof_files.sh script. ([#3242](https://github.com/AztecProtocol/aztec-packages/issues/3242)) ([ee57e00](https://github.com/AztecProtocol/aztec-packages/commit/ee57e00da06a2daea571cac579a5f6ef9e039d5e))
* Temporary fix for bb prove w/ ram rom blocks ([#3215](https://github.com/AztecProtocol/aztec-packages/issues/3215)) ([af93a33](https://github.com/AztecProtocol/aztec-packages/commit/af93a33fdd5d73648d31b4e4f7347d29b8892405))


### Miscellaneous

* Clean up and refactor arithmetization ([#3164](https://github.com/AztecProtocol/aztec-packages/issues/3164)) ([0370b13](https://github.com/AztecProtocol/aztec-packages/commit/0370b135c723458852894363383bbe9275eb0e56))
* Move flavors ([#3188](https://github.com/AztecProtocol/aztec-packages/issues/3188)) ([f1ff849](https://github.com/AztecProtocol/aztec-packages/commit/f1ff849d90b3914bf8c24bf54ded8d98b7ffa961))
* Move honk/pcs ([#3187](https://github.com/AztecProtocol/aztec-packages/issues/3187)) ([3870ff8](https://github.com/AztecProtocol/aztec-packages/commit/3870ff8f829c29556d633693875cf30ce8d724eb))
* Move log deriv lookup accum to library ([#3226](https://github.com/AztecProtocol/aztec-packages/issues/3226)) ([189d1bb](https://github.com/AztecProtocol/aztec-packages/commit/189d1bbd6691d0237d69acb012238e97589ee257))
* Move sumcheck ([#3189](https://github.com/AztecProtocol/aztec-packages/issues/3189)) ([410cae3](https://github.com/AztecProtocol/aztec-packages/commit/410cae39aba1387571308567a8022cc51b6d25d1))
* Move transcripts ([#3176](https://github.com/AztecProtocol/aztec-packages/issues/3176)) ([7372d19](https://github.com/AztecProtocol/aztec-packages/commit/7372d19f64737eabfa917f7368a5bf99068f48d5))
* Split out relations, PG, Honk variants ([#3238](https://github.com/AztecProtocol/aztec-packages/issues/3238)) ([8abd39f](https://github.com/AztecProtocol/aztec-packages/commit/8abd39f5f8a434d96fe259df9c5940787bd705f1))


### Documentation

* Updated stale tree docs ([#3166](https://github.com/AztecProtocol/aztec-packages/issues/3166)) ([3d5c98c](https://github.com/AztecProtocol/aztec-packages/commit/3d5c98c3eeb76103c331bfcbefc4127ae39836c7))

## [0.13.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.13.0...barretenberg-v0.13.1) (2023-10-31)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.13.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.12.0...barretenberg-v0.13.0) (2023-10-31)


### Features

* Adding structure to Transcript ([#2937](https://github.com/AztecProtocol/aztec-packages/issues/2937)) ([db67aa1](https://github.com/AztecProtocol/aztec-packages/commit/db67aa1eb6ae9669d98301efbbb146d6265d58f4))
* Efficient ZM quotient computation ([#3016](https://github.com/AztecProtocol/aztec-packages/issues/3016)) ([ebda5fc](https://github.com/AztecProtocol/aztec-packages/commit/ebda5fcbc7321cb3f91b0c7a742b7cbd88a15179))
* Measure plonk rounds ([#3065](https://github.com/AztecProtocol/aztec-packages/issues/3065)) ([c8e1d8b](https://github.com/AztecProtocol/aztec-packages/commit/c8e1d8b9244c3955f0fea6a34a3cc28a81a29d2c))
* New script to output table of benchmarks for README pasting. ([#2780](https://github.com/AztecProtocol/aztec-packages/issues/2780)) ([6c20b45](https://github.com/AztecProtocol/aztec-packages/commit/6c20b45993ee9cbd319ab8351e2722e0c912f427))
* Pedersen in typescript. ([#3111](https://github.com/AztecProtocol/aztec-packages/issues/3111)) ([933f1b2](https://github.com/AztecProtocol/aztec-packages/commit/933f1b2c24a3a4bdaafd31e1158ba702ee9874c9))
* Protogalaxy folding of challenges ([#2935](https://github.com/AztecProtocol/aztec-packages/issues/2935)) ([7ed30e8](https://github.com/AztecProtocol/aztec-packages/commit/7ed30e83d2bea8399b7acd477c4dfc739417f96d))
* Zeromorph with concatenation (Goblin Translator part 10) ([#3006](https://github.com/AztecProtocol/aztec-packages/issues/3006)) ([70b0f17](https://github.com/AztecProtocol/aztec-packages/commit/70b0f17101f3b378df3e9a0247230b9ebf67239a))


### Miscellaneous

* Add stdlib tests for pedersen commitment ([#3075](https://github.com/AztecProtocol/aztec-packages/issues/3075)) ([87fa621](https://github.com/AztecProtocol/aztec-packages/commit/87fa621347e55f82e36c70515c1824161eee5282))
* Automatic c_binds for commit should return a point instead of an Fr element ([#3072](https://github.com/AztecProtocol/aztec-packages/issues/3072)) ([2e289a5](https://github.com/AztecProtocol/aztec-packages/commit/2e289a5d11d28496ac47220bede03268065e0cb7))
* Cleanup remaining mentions of `compress` with pedersen in cpp and ts ([#3074](https://github.com/AztecProtocol/aztec-packages/issues/3074)) ([52cf383](https://github.com/AztecProtocol/aztec-packages/commit/52cf3831794a6ab497c9a40f85859f4cc8ac4700))
* Remove endomorphism coefficient from ecc_add_gate ([#3115](https://github.com/AztecProtocol/aztec-packages/issues/3115)) ([d294987](https://github.com/AztecProtocol/aztec-packages/commit/d294987ad25fb69d2934dfade2bf7063ff64bef2))
* Remove unecessary calls to `pedersen__init` ([#3079](https://github.com/AztecProtocol/aztec-packages/issues/3079)) ([84f8db2](https://github.com/AztecProtocol/aztec-packages/commit/84f8db20f482242ac29a23eb4c8876f14f060b4c))
* Remove unused pedersen c_binds ([#3058](https://github.com/AztecProtocol/aztec-packages/issues/3058)) ([e71e5f9](https://github.com/AztecProtocol/aztec-packages/commit/e71e5f94ba920208e7cc9b2b1b9d62678b699812))
* Removes pedersen commit native pairs method ([#3073](https://github.com/AztecProtocol/aztec-packages/issues/3073)) ([69a34c7](https://github.com/AztecProtocol/aztec-packages/commit/69a34c72c9dccbd54072553ed1ecf0460b16db69))

## [0.12.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.11.1...barretenberg-v0.12.0) (2023-10-26)


### ⚠ BREAKING CHANGES

* remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033))

### Features

* Added correctness tests for several small relations in Goblin Translator (Goblin Translator part 8) ([#2963](https://github.com/AztecProtocol/aztec-packages/issues/2963)) ([4c83250](https://github.com/AztecProtocol/aztec-packages/commit/4c8325093e7d76158a767dcf2854f1cfd274c5ff))
* Correctness tests for decomposition and non-native field relations (Goblin Translator Part 9) ([#2981](https://github.com/AztecProtocol/aztec-packages/issues/2981)) ([cdc830d](https://github.com/AztecProtocol/aztec-packages/commit/cdc830dd8731d9f8fed85bb46b3ed6771796f526))
* Enable sol verifier tests in ci ([#2997](https://github.com/AztecProtocol/aztec-packages/issues/2997)) ([058de1e](https://github.com/AztecProtocol/aztec-packages/commit/058de1ea92b1c19f76867b93769d8de4bb9a6f55))
* Goblin Translator flavor and permutation correctness (Goblin Translator part 7) ([#2961](https://github.com/AztecProtocol/aztec-packages/issues/2961)) ([737f17f](https://github.com/AztecProtocol/aztec-packages/commit/737f17fdff5a213dd1424c4e668bce41b95b349a))


### Bug Fixes

* Fix clang-16 check ([#3030](https://github.com/AztecProtocol/aztec-packages/issues/3030)) ([7a5a8b3](https://github.com/AztecProtocol/aztec-packages/commit/7a5a8b3b79c18b45aa29eacc05e9bfb26090cc95))


### Miscellaneous

* **acir_tests:** Add script to regenerate double_verify_proof inputs ([#3005](https://github.com/AztecProtocol/aztec-packages/issues/3005)) ([9c4eab2](https://github.com/AztecProtocol/aztec-packages/commit/9c4eab27d6a8a774d49f40ccea92faf305caf500))
* Fix `pedersen_compress_with_hash_index` c_bind function ([#3054](https://github.com/AztecProtocol/aztec-packages/issues/3054)) ([a136f6e](https://github.com/AztecProtocol/aztec-packages/commit/a136f6e70725500739b518e1bfc96b680c3cb1b2))
* Proxy redundant `hash` methods ([#3046](https://github.com/AztecProtocol/aztec-packages/issues/3046)) ([df389b5](https://github.com/AztecProtocol/aztec-packages/commit/df389b5f593a202bc644479a6c3dff884b7d3652))
* Remove `pedersen_buffer_to_field` from c_bind ([#3045](https://github.com/AztecProtocol/aztec-packages/issues/3045)) ([de7e63b](https://github.com/AztecProtocol/aztec-packages/commit/de7e63bf7e1184333c1eaadf2387fef6bf163871))
* Remove pedersen hash oracle ([#3023](https://github.com/AztecProtocol/aztec-packages/issues/3023)) ([0e6958c](https://github.com/AztecProtocol/aztec-packages/commit/0e6958c94e6d00d4132f08baa2cd63141ff8aae7))
* Remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033)) ([a8ea391](https://github.com/AztecProtocol/aztec-packages/commit/a8ea391c95a9fe4fa26a3fa987f52114a40c664a))

## [0.11.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.11.0...barretenberg-v0.11.1) (2023-10-24)


### Features

* ProverPlookupAuxiliaryWidget kernel bench ([#2924](https://github.com/AztecProtocol/aztec-packages/issues/2924)) ([faffc39](https://github.com/AztecProtocol/aztec-packages/commit/faffc39a379c9f215978e4867c3d24dbc638f0b4))

## [0.11.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.10.1...barretenberg-v0.11.0) (2023-10-24)


### Features

* Pedersen hash in acir format ([#2990](https://github.com/AztecProtocol/aztec-packages/issues/2990)) ([2a4c548](https://github.com/AztecProtocol/aztec-packages/commit/2a4c548bc816a5f379ee841e26bb30411deef56b))


### Miscellaneous

* Update acir_tests reference branch ([#2993](https://github.com/AztecProtocol/aztec-packages/issues/2993)) ([91813a5](https://github.com/AztecProtocol/aztec-packages/commit/91813a55b8503c279ccd38b1d83463b97b86d064))

## [0.10.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.10.0...barretenberg-v0.10.1) (2023-10-24)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.10.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.9.0...barretenberg-v0.10.0) (2023-10-24)


### Features

* Goblin translator non-native field relation (Goblin Translator part 6) ([#2871](https://github.com/AztecProtocol/aztec-packages/issues/2871)) ([c4d8d96](https://github.com/AztecProtocol/aztec-packages/commit/c4d8d963171cf936242e04639154fccc86a0942f))
* Honk profiling by pass, tsan preset ([#2982](https://github.com/AztecProtocol/aztec-packages/issues/2982)) ([a1592fd](https://github.com/AztecProtocol/aztec-packages/commit/a1592fdcde661e09826852fc28bb4aa4c5521863))
* Protogalaxy Combiner ([#2436](https://github.com/AztecProtocol/aztec-packages/issues/2436)) ([a60c70d](https://github.com/AztecProtocol/aztec-packages/commit/a60c70dca1d920ad88511f77be3ad186afab7bdb))
* Protogalaxy perturbator! ([#2624](https://github.com/AztecProtocol/aztec-packages/issues/2624)) ([509dee6](https://github.com/AztecProtocol/aztec-packages/commit/509dee6108781f3dcd09b3c111be59f42798cac0))
* Refactor pedersen hash standard ([#2592](https://github.com/AztecProtocol/aztec-packages/issues/2592)) ([3085676](https://github.com/AztecProtocol/aztec-packages/commit/3085676dd8a68ac43abc3e5c7843ff437df91d7d))
* Widget benchmarking ([#2897](https://github.com/AztecProtocol/aztec-packages/issues/2897)) ([0e927e9](https://github.com/AztecProtocol/aztec-packages/commit/0e927e9233d7418b9fba4a0142f606e2f92a1f40))


### Bug Fixes

* Honk sumcheck performance ([#2925](https://github.com/AztecProtocol/aztec-packages/issues/2925)) ([5fbfe6e](https://github.com/AztecProtocol/aztec-packages/commit/5fbfe6eeccdb23f734fb36f30d1e33340f9fb07a))


### Miscellaneous

* Remove unused nix files ([#2933](https://github.com/AztecProtocol/aztec-packages/issues/2933)) ([3174f84](https://github.com/AztecProtocol/aztec-packages/commit/3174f84fe9d92b353d1b2c307ed5757ee941ce00))

## [0.9.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.14...barretenberg-v0.9.0) (2023-10-17)


### Features

* Bump msgpack ([#2884](https://github.com/AztecProtocol/aztec-packages/issues/2884)) ([d7b7fb1](https://github.com/AztecProtocol/aztec-packages/commit/d7b7fb1d70cfb6a592d4cf24c0da92ed9acc7d38))
* Download msgpack ([#2885](https://github.com/AztecProtocol/aztec-packages/issues/2885)) ([8ac8beb](https://github.com/AztecProtocol/aztec-packages/commit/8ac8bebaa8dad39df6f3d6f622e215574062ac52))

## [0.8.14](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.13...barretenberg-v0.8.14) (2023-10-13)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.13](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.12...barretenberg-v0.8.13) (2023-10-13)


### Bug Fixes

* Fix check_circuit in goblin translator (resulted in flimsy test) ([#2827](https://github.com/AztecProtocol/aztec-packages/issues/2827)) ([98b1679](https://github.com/AztecProtocol/aztec-packages/commit/98b16793b0e84360af8dc70934636d11d7bc7e29))

## [0.8.12](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.11...barretenberg-v0.8.12) (2023-10-13)


### Bug Fixes

* Fix rebuild pattern slashes. ([#2843](https://github.com/AztecProtocol/aztec-packages/issues/2843)) ([e32517e](https://github.com/AztecProtocol/aztec-packages/commit/e32517e9eae791b32f94b3816413392ccf0ba096))

## [0.8.11](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.10...barretenberg-v0.8.11) (2023-10-13)


### Features

* Goblin Translator Decomposition relation (Goblin Translator part 4) ([#2802](https://github.com/AztecProtocol/aztec-packages/issues/2802)) ([3c3cd9f](https://github.com/AztecProtocol/aztec-packages/commit/3c3cd9f62640b505b55916648df6ccddf524cdfc))
* Goblin Translator GenPermSort relation (Goblin Translator part 3) ([#2795](https://github.com/AztecProtocol/aztec-packages/issues/2795)) ([b36fdc4](https://github.com/AztecProtocol/aztec-packages/commit/b36fdc481d16e56fe244c5a10a5223199f9f2e6b))
* Goblin translator opcode constraint and accumulator transfer relations (Goblin Translator part 5) ([#2805](https://github.com/AztecProtocol/aztec-packages/issues/2805)) ([b3d1f28](https://github.com/AztecProtocol/aztec-packages/commit/b3d1f280913494322baee369e6ee4f04353891b3))
* Goblin Translator Permutation relation (Goblin Translator part 2) ([#2790](https://github.com/AztecProtocol/aztec-packages/issues/2790)) ([9a354c9](https://github.com/AztecProtocol/aztec-packages/commit/9a354c94c91f8f2927ca66d0de65b5b893066710))
* Integrate ZeroMorph into Honk ([#2774](https://github.com/AztecProtocol/aztec-packages/issues/2774)) ([ea86869](https://github.com/AztecProtocol/aztec-packages/commit/ea86869e92da3fbf921314fdbca31fdb85a6e274))
* Update goblin translator circuit builder (Goblin Translator part 1) ([#2764](https://github.com/AztecProtocol/aztec-packages/issues/2764)) ([32c69ae](https://github.com/AztecProtocol/aztec-packages/commit/32c69ae36ed431482d286e228fd830256e8bd1b5))


### Miscellaneous

* Change acir_tests branch to point to master ([#2815](https://github.com/AztecProtocol/aztec-packages/issues/2815)) ([73f229d](https://github.com/AztecProtocol/aztec-packages/commit/73f229d3123301818262439a2a98767146a1a58c))
* Remove Ultra Grumpkin flavor ([#2825](https://github.com/AztecProtocol/aztec-packages/issues/2825)) ([bde77b8](https://github.com/AztecProtocol/aztec-packages/commit/bde77b8e6e91fa734e06453e67a50597480b2ec1))
* Remove work queue from honk ([#2814](https://github.com/AztecProtocol/aztec-packages/issues/2814)) ([bca7d12](https://github.com/AztecProtocol/aztec-packages/commit/bca7d126d2ec583977ee5bdf77a90263d059dc44))
* Spell check ([#2817](https://github.com/AztecProtocol/aztec-packages/issues/2817)) ([4777a11](https://github.com/AztecProtocol/aztec-packages/commit/4777a113491c4c9901b4589a9a6cb1e1148c0288))

## [0.8.10](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.9...barretenberg-v0.8.10) (2023-10-11)


### Features

* Bb faster init ([#2776](https://github.com/AztecProtocol/aztec-packages/issues/2776)) ([c794533](https://github.com/AztecProtocol/aztec-packages/commit/c794533754a9706d362d0374209df9eb5b6bfdc7))
* LLVM xray presets ([#2525](https://github.com/AztecProtocol/aztec-packages/issues/2525)) ([23a1ee9](https://github.com/AztecProtocol/aztec-packages/commit/23a1ee91da6003d1b5798640c8ccecbd226beef7))
* Separate aggregation protocol ([#2736](https://github.com/AztecProtocol/aztec-packages/issues/2736)) ([ad16937](https://github.com/AztecProtocol/aztec-packages/commit/ad169374943ef49c32eabc66483a7be28a711565))
* Simplify relation containers ([#2619](https://github.com/AztecProtocol/aztec-packages/issues/2619)) ([99c5127](https://github.com/AztecProtocol/aztec-packages/commit/99c5127ac5c10e6637534870a689a95238ae997c))
* ZeroMorph ([#2664](https://github.com/AztecProtocol/aztec-packages/issues/2664)) ([a006e5a](https://github.com/AztecProtocol/aztec-packages/commit/a006e5a0e0a30f8dfe992e3ac8a05f6c276f9300))


### Miscellaneous

* Acir format cleanup ([#2779](https://github.com/AztecProtocol/aztec-packages/issues/2779)) ([5ea373f](https://github.com/AztecProtocol/aztec-packages/commit/5ea373f7d653f7322a108297113a2deb379e1400))
* Stop whinging about this ownership stuff. ([#2775](https://github.com/AztecProtocol/aztec-packages/issues/2775)) ([3dd6900](https://github.com/AztecProtocol/aztec-packages/commit/3dd6900f96a7dc855643be0e4aba0cfe9fa8a16e))
* Update ACIR serialisation format ([#2771](https://github.com/AztecProtocol/aztec-packages/issues/2771)) ([6d85527](https://github.com/AztecProtocol/aztec-packages/commit/6d855270f8c069edac62536ccc391a0cab764323))
* Use global crs in more places. Less pain. ([#2772](https://github.com/AztecProtocol/aztec-packages/issues/2772)) ([b819980](https://github.com/AztecProtocol/aztec-packages/commit/b8199802bad3c05ebe4d1ded5338a09a04e0ed7e))

## [0.8.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.8...barretenberg-v0.8.9) (2023-10-10)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.7...barretenberg-v0.8.8) (2023-10-09)


### Features

* GCC 13 preset ([#2623](https://github.com/AztecProtocol/aztec-packages/issues/2623)) ([4881414](https://github.com/AztecProtocol/aztec-packages/commit/4881414ceb30590674c244ef9bc4c8416eacd6bc))


### Bug Fixes

* Challenge generation update ([#2628](https://github.com/AztecProtocol/aztec-packages/issues/2628)) ([68c1fab](https://github.com/AztecProtocol/aztec-packages/commit/68c1fab51e3a339032b719ce966ed34787f33dab))


### Miscellaneous

* Bump ACIR deserializer ([#2675](https://github.com/AztecProtocol/aztec-packages/issues/2675)) ([502ee87](https://github.com/AztecProtocol/aztec-packages/commit/502ee872d6360bf4bc5b83c672eeb64c58944073))

## [0.8.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.6...barretenberg-v0.8.7) (2023-10-04)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.5...barretenberg-v0.8.6) (2023-10-04)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.4...barretenberg-v0.8.5) (2023-10-04)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.3...barretenberg-v0.8.4) (2023-10-04)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.2...barretenberg-v0.8.3) (2023-10-04)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.8.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.1...barretenberg-v0.8.2) (2023-10-04)


### Bug Fixes

* Include ignition data in package or save after 1st download ([#2591](https://github.com/AztecProtocol/aztec-packages/issues/2591)) ([d5e9f8b](https://github.com/AztecProtocol/aztec-packages/commit/d5e9f8be6bbcb8a88dfdec8fee8fe7cf439f6b19)), closes [#2445](https://github.com/AztecProtocol/aztec-packages/issues/2445)
* Make target architecture configurable, target westmere in GA. ([#2660](https://github.com/AztecProtocol/aztec-packages/issues/2660)) ([3cb9639](https://github.com/AztecProtocol/aztec-packages/commit/3cb9639ed1158e70b377aa49832eb650e5cd2930))

## [0.8.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.8.0...barretenberg-v0.8.1) (2023-10-03)


### Bug Fixes

* Add missing ecc doubling gate into ultra plonk and ultra honk  ([#2610](https://github.com/AztecProtocol/aztec-packages/issues/2610)) ([7cb7c58](https://github.com/AztecProtocol/aztec-packages/commit/7cb7c58444a087d81684afc6d5c2fc254357035e))


### Miscellaneous

* Update acir_tests script to point to master ([#2650](https://github.com/AztecProtocol/aztec-packages/issues/2650)) ([51d1e79](https://github.com/AztecProtocol/aztec-packages/commit/51d1e79c3463461864878d4d8f2e84d7e74b9c86))

## [0.8.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.10...barretenberg-v0.8.0) (2023-10-03)


### Features

* Barretenberg/crypto/blake3s supports compile-time hashing ([#2556](https://github.com/AztecProtocol/aztec-packages/issues/2556)) ([da05dd7](https://github.com/AztecProtocol/aztec-packages/commit/da05dd7ea41208aea42efe0aeb838e4d76e2d34a))
* **bb:** Add `bb --version` command ([#2482](https://github.com/AztecProtocol/aztec-packages/issues/2482)) ([530676f](https://github.com/AztecProtocol/aztec-packages/commit/530676f8ec53e63ba24f6fabc9097ae8f5db5fc6))
* **bb:** Avoid initializing CRS for `bb info` command ([#2425](https://github.com/AztecProtocol/aztec-packages/issues/2425)) ([d22c7b1](https://github.com/AztecProtocol/aztec-packages/commit/d22c7b1f69ea936c532fac68d19c6362f8a34be5))
* Consistent pedersen hash (work in progress) ([#1945](https://github.com/AztecProtocol/aztec-packages/issues/1945)) ([b4ad8f3](https://github.com/AztecProtocol/aztec-packages/commit/b4ad8f38250d82531439d6db33c8f81387c42496))
* Goblin op queue transcript aggregation ([#2257](https://github.com/AztecProtocol/aztec-packages/issues/2257)) ([b7f627a](https://github.com/AztecProtocol/aztec-packages/commit/b7f627a5e472d3dc691b799a5e3df508b685a272))
* Parallelization update for polynomials ([#2311](https://github.com/AztecProtocol/aztec-packages/issues/2311)) ([922fc99](https://github.com/AztecProtocol/aztec-packages/commit/922fc9912a4a88a41eef42fe64ca2b59d859b5b1))
* Update to protogalaxy interfaces ([#2498](https://github.com/AztecProtocol/aztec-packages/issues/2498)) ([9a3d265](https://github.com/AztecProtocol/aztec-packages/commit/9a3d2652d2614439017a6f47152efb9a177b7127))
* YML manifest. Simplify YBP. ([#2353](https://github.com/AztecProtocol/aztec-packages/issues/2353)) ([bf73bc3](https://github.com/AztecProtocol/aztec-packages/commit/bf73bc3e8fd0fd13193f9301073905682044a6c5))


### Bug Fixes

* **barretenberg:** Brittle headers caused error compiling for clang-16 on mainframe ([#2547](https://github.com/AztecProtocol/aztec-packages/issues/2547)) ([cc909da](https://github.com/AztecProtocol/aztec-packages/commit/cc909da0464003aee6d2ff4036ba59c321a5b617))
* Bb rebuild patterns ([#2499](https://github.com/AztecProtocol/aztec-packages/issues/2499)) ([868cceb](https://github.com/AztecProtocol/aztec-packages/commit/868cceb98c7fd6a8edd6710eba4d76ef58a68664))
* Fix working dir bug causing stdlib-tests to not run. ([#2495](https://github.com/AztecProtocol/aztec-packages/issues/2495)) ([6b3402c](https://github.com/AztecProtocol/aztec-packages/commit/6b3402c552292068dcdf74a920c65b2aad635441))
* Nightly subrepo mirror ([#2520](https://github.com/AztecProtocol/aztec-packages/issues/2520)) ([bedc8c8](https://github.com/AztecProtocol/aztec-packages/commit/bedc8c88cfc24a51806690f225a128f973c5845f))


### Miscellaneous

* BI build tweaks ([#2487](https://github.com/AztecProtocol/aztec-packages/issues/2487)) ([f8b6548](https://github.com/AztecProtocol/aztec-packages/commit/f8b65481eec99876007e521beecd671b9a18f19a))
* Kill Turbo ([#2442](https://github.com/AztecProtocol/aztec-packages/issues/2442)) ([c832825](https://github.com/AztecProtocol/aztec-packages/commit/c83282582536421ae67bbd936b3059597d908253))
* Provide cross compile to cjs. ([#2566](https://github.com/AztecProtocol/aztec-packages/issues/2566)) ([47d0d37](https://github.com/AztecProtocol/aztec-packages/commit/47d0d376727dfcb798af4ea019dfc23a9a57b6ca))
* Recursion todos ([#2516](https://github.com/AztecProtocol/aztec-packages/issues/2516)) ([2df107b](https://github.com/AztecProtocol/aztec-packages/commit/2df107b2da73217eb96d39c8ed880f76a2b3e4cd))
* Reenable some ultra honk composer tests ([#2417](https://github.com/AztecProtocol/aztec-packages/issues/2417)) ([31f4c32](https://github.com/AztecProtocol/aztec-packages/commit/31f4c32e2c4a3a91879e842ea2366eb167fdd510))
* Remove composer keyword from stdlib ([#2418](https://github.com/AztecProtocol/aztec-packages/issues/2418)) ([f3e7d91](https://github.com/AztecProtocol/aztec-packages/commit/f3e7d914e3b8b7f98eacde0dff12a51a04dde93e))
* Remove Standard Honk ([#2435](https://github.com/AztecProtocol/aztec-packages/issues/2435)) ([9b3ee45](https://github.com/AztecProtocol/aztec-packages/commit/9b3ee4579c0a13378eb27b5c24bf9b99a07de350))


### Documentation

* Fixed original minus underflow test ([#2472](https://github.com/AztecProtocol/aztec-packages/issues/2472)) ([0cf4bdc](https://github.com/AztecProtocol/aztec-packages/commit/0cf4bdc853d864fd4cf73d5af7e261ee2515c0d0))

## [0.7.10](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.9...barretenberg-v0.7.10) (2023-09-20)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.7.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.8...barretenberg-v0.7.9) (2023-09-19)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.7.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.7...barretenberg-v0.7.8) (2023-09-19)


### Features

* Allow tracing build system with [debug ci] ([#2389](https://github.com/AztecProtocol/aztec-packages/issues/2389)) ([ce311a9](https://github.com/AztecProtocol/aztec-packages/commit/ce311a9b44a8f0327235ccd3bb8f9a8fca97443e))

## [0.7.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.6...barretenberg-v0.7.7) (2023-09-18)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.7.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.5...barretenberg-v0.7.6) (2023-09-18)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.7.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.4...barretenberg-v0.7.5) (2023-09-15)


### Features

* Protogalaxy interfaces ([#2125](https://github.com/AztecProtocol/aztec-packages/issues/2125)) ([b45dd26](https://github.com/AztecProtocol/aztec-packages/commit/b45dd26214119f0c52c2c4f48ff11f650912fef9))

## [0.7.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.3...barretenberg-v0.7.4) (2023-09-15)


### Features

* Elliptic Curve Virtual Machine Circuit ([#1268](https://github.com/AztecProtocol/aztec-packages/issues/1268)) ([f85ecd9](https://github.com/AztecProtocol/aztec-packages/commit/f85ecd921271ec94b551992bcfe16c2b56f72d2e))

## [0.7.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.2...barretenberg-v0.7.3) (2023-09-15)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.7.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.1...barretenberg-v0.7.2) (2023-09-14)


### Features

* ASAN build ([#2307](https://github.com/AztecProtocol/aztec-packages/issues/2307)) ([274c89f](https://github.com/AztecProtocol/aztec-packages/commit/274c89f1916d8af2054d9773dc632f87bb3bf2fc))

## [0.7.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.7.0...barretenberg-v0.7.1) (2023-09-14)


### Miscellaneous

* Move barretenberg to top of repo. Make circuits build off barretenberg build. ([#2221](https://github.com/AztecProtocol/aztec-packages/issues/2221)) ([404ec34](https://github.com/AztecProtocol/aztec-packages/commit/404ec34d38e1a9c3fbe7a3cdb6e88c28f62f72e4))

## [0.7.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.7...barretenberg-v0.7.0) (2023-09-13)


### ⚠ BREAKING CHANGES

* **aztec-noir:** rename noir-aztec to aztec-noir ([#2071](https://github.com/AztecProtocol/aztec-packages/issues/2071))

### Features

* **build:** Use LTS version of ubuntu ([#2239](https://github.com/AztecProtocol/aztec-packages/issues/2239)) ([ce6671e](https://github.com/AztecProtocol/aztec-packages/commit/ce6671e6ab72fcdc8114df5b6a45f81c0086b19d))


### Bug Fixes

* **build:** Update ubuntu version used in Docker builds ([#2236](https://github.com/AztecProtocol/aztec-packages/issues/2236)) ([dbe80b7](https://github.com/AztecProtocol/aztec-packages/commit/dbe80b739e97474b29e6a4125ac0d2f16e248b32))
* Format barretenberg ([#2209](https://github.com/AztecProtocol/aztec-packages/issues/2209)) ([0801372](https://github.com/AztecProtocol/aztec-packages/commit/08013725091c7e80c1e83145ffbf3983cf1e7fe3))
* Msgpack blowup with bigger objects ([#2207](https://github.com/AztecProtocol/aztec-packages/issues/2207)) ([b909937](https://github.com/AztecProtocol/aztec-packages/commit/b909937ba53b896e11e6b65db08b8f2bb83218d5))
* Refactor constraints in scalar mul to use the high limb ([#2161](https://github.com/AztecProtocol/aztec-packages/issues/2161)) ([1d0e25d](https://github.com/AztecProtocol/aztec-packages/commit/1d0e25d9fad69aebccacf9f646e3291ea89716ca))


### Miscellaneous

* Add debugging to run_tests ([#2212](https://github.com/AztecProtocol/aztec-packages/issues/2212)) ([1c5e78a](https://github.com/AztecProtocol/aztec-packages/commit/1c5e78a4ac01bee4b785857447efdb02d8d9cb35))
* **aztec-noir:** Rename noir-aztec to aztec-noir ([#2071](https://github.com/AztecProtocol/aztec-packages/issues/2071)) ([e1e14d2](https://github.com/AztecProtocol/aztec-packages/commit/e1e14d2c7fb44d56b9a10a645676d3551830bb10))
* Update url for acir artifacts ([#2231](https://github.com/AztecProtocol/aztec-packages/issues/2231)) ([5e0abd3](https://github.com/AztecProtocol/aztec-packages/commit/5e0abd35dec449a665760e5ee51eeff89c76532c))

## [0.6.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.6...barretenberg-v0.6.7) (2023-09-11)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.5...barretenberg-v0.6.6) (2023-09-11)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.4...barretenberg-v0.6.5) (2023-09-08)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.3...barretenberg-v0.6.4) (2023-09-08)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.2...barretenberg-v0.6.3) (2023-09-08)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.6.1...barretenberg-v0.6.2) (2023-09-08)


### Miscellaneous

* **barretenberg:** Synchronize aztec-packages versions

## [0.6.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.5.2...barretenberg-v0.6.1) (2023-09-08)


### Bug Fixes

* Work around intermittent wasm webkit issue ([#2140](https://github.com/AztecProtocol/aztec-packages/issues/2140)) ([a9b0934](https://github.com/AztecProtocol/aztec-packages/commit/a9b09344c80d8628f95f859d4e2d455d61f9e7c6))


### Miscellaneous

* **master:** Release 0.5.2 ([#2141](https://github.com/AztecProtocol/aztec-packages/issues/2141)) ([451aad6](https://github.com/AztecProtocol/aztec-packages/commit/451aad6ea92ebced9839ca14baae10cee327be35))
* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))
* Release 0.6.1 ([1bd1a79](https://github.com/AztecProtocol/aztec-packages/commit/1bd1a79b0cefcd90306133aab141d992e8ea5fc3))

## [0.5.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg-v0.5.2...barretenberg-v0.5.2) (2023-09-08)


### Bug Fixes

* Work around intermittent wasm webkit issue ([#2140](https://github.com/AztecProtocol/aztec-packages/issues/2140)) ([a9b0934](https://github.com/AztecProtocol/aztec-packages/commit/a9b09344c80d8628f95f859d4e2d455d61f9e7c6))


### Miscellaneous

* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))

## [0.5.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.5.0...barretenberg-v0.5.1) (2023-09-05)


### Features

* Add `info` command to bb ([#2010](https://github.com/AztecProtocol/barretenberg/issues/2010)) ([2882d97](https://github.com/AztecProtocol/barretenberg/commit/2882d97f5165239badb328be80568e7d683c0465))
* **ci:** Use content hash in build system, restrict docs build to *.ts or *.cpp ([#1953](https://github.com/AztecProtocol/barretenberg/issues/1953)) ([297a20d](https://github.com/AztecProtocol/barretenberg/commit/297a20d7878a4aabab1cabf2cc5d2d67f9e969c5))


### Bug Fixes

* Adds Mac cross compile flags into barretenberg ([#1954](https://github.com/AztecProtocol/barretenberg/issues/1954)) ([0e17d97](https://github.com/AztecProtocol/barretenberg/commit/0e17d978a0cc6805b72646a8e36fd5267cbd6bcd))
* **bb.js:** (breaking change) bundles bb.js properly so that it works in the browser and in node ([#1855](https://github.com/AztecProtocol/barretenberg/issues/1855)) ([bc93a5f](https://github.com/AztecProtocol/barretenberg/commit/bc93a5f8510d0dc600343e7e613ab84380d3c225))
* **ci:** Incorrect content hash in some build targets ([#1973](https://github.com/AztecProtocol/barretenberg/issues/1973)) ([c6c469a](https://github.com/AztecProtocol/barretenberg/commit/c6c469aa5da7c6973f656ddf8af4fb20c3e8e4f6))
* Compilation on homebrew clang 16.06 ([#1937](https://github.com/AztecProtocol/barretenberg/issues/1937)) ([79c29ee](https://github.com/AztecProtocol/barretenberg/commit/79c29eebbdb78c1e9aa5b4a3da6207fbf93bdd10))
* Master ([#1981](https://github.com/AztecProtocol/barretenberg/issues/1981)) ([59a454e](https://github.com/AztecProtocol/barretenberg/commit/59a454ecf1611424893e1cb093774a23dde39310))
* Unify base64 interface between mac and linux (cherry-picked) ([#1968](https://github.com/AztecProtocol/barretenberg/issues/1968)) ([37ee120](https://github.com/AztecProtocol/barretenberg/commit/37ee1204eba280442b6941eff448d6ff15eb9f04))

## [0.5.0](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.6...barretenberg-v0.5.0) (2023-09-01)


### ⚠ BREAKING CHANGES

* update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/barretenberg/issues/1925))

### Bug Fixes

* Benchmark preset uses clang16 ([#1902](https://github.com/AztecProtocol/barretenberg/issues/1902)) ([cd0ff0e](https://github.com/AztecProtocol/barretenberg/commit/cd0ff0e2c049917ec47a110b45d76bed4c00ae2a))
* Reset keccak var inputs to 0 ([#1881](https://github.com/AztecProtocol/barretenberg/issues/1881)) ([23011ee](https://github.com/AztecProtocol/barretenberg/commit/23011ee1ea7f1b00b0f4194ebceedc75ea01c157))


### Miscellaneous Chores

* Update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/barretenberg/issues/1925)) ([5d8db8e](https://github.com/AztecProtocol/barretenberg/commit/5d8db8eb993334b43e24a51efba9c59e123320ab))

## [0.4.6](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.5...barretenberg-v0.4.6) (2023-08-29)


### Bug Fixes

* Truncate SRS size to the amount of points that we have downloaded ([#1862](https://github.com/AztecProtocol/barretenberg/issues/1862)) ([3bcf12b](https://github.com/AztecProtocol/barretenberg/commit/3bcf12b1a302280d5112475c5993b125e130209e))

## [0.4.5](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.4...barretenberg-v0.4.5) (2023-08-28)


### Bug Fixes

* Conditionally compile base64 command for bb binary ([#1851](https://github.com/AztecProtocol/barretenberg/issues/1851)) ([8f8b9f4](https://github.com/AztecProtocol/barretenberg/commit/8f8b9f46028a08342a3337db633782e5313e2763))

## [0.4.4](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.3...barretenberg-v0.4.4) (2023-08-28)


### Features

* Add ARM build for Mac + cleanup artifacts ([#1837](https://github.com/AztecProtocol/barretenberg/issues/1837)) ([2d2d5ea](https://github.com/AztecProtocol/barretenberg/commit/2d2d5ea33c512ab36c1214fb5bb90f80d8247469))

## [0.4.3](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.2...barretenberg-v0.4.3) (2023-08-23)


### Features

* **bb:** Use an environment variable to set the transcript URL ([#1750](https://github.com/AztecProtocol/barretenberg/issues/1750)) ([41d362e](https://github.com/AztecProtocol/barretenberg/commit/41d362e9c9ffeb763cd56ca8a9f8c4512b86c80c))


### Bug Fixes

* Clang version in README and subrepo edge case ([#1730](https://github.com/AztecProtocol/barretenberg/issues/1730)) ([74158c4](https://github.com/AztecProtocol/barretenberg/commit/74158c4e467d4b6ab90e7d5aeb9a28f04adc1d66))
* Download SRS using one canonical URL across the codebase ([#1748](https://github.com/AztecProtocol/barretenberg/issues/1748)) ([5c91de7](https://github.com/AztecProtocol/barretenberg/commit/5c91de7296e054f6d5ac3dca94ca85e06d496048))
* Proving fails when circuit has size &gt; ~500K ([#1739](https://github.com/AztecProtocol/barretenberg/issues/1739)) ([6d32383](https://github.com/AztecProtocol/barretenberg/commit/6d323838a525190618d608598357ee4608c46699))
* Revert clang check bootstrap.sh ([#1734](https://github.com/AztecProtocol/barretenberg/issues/1734)) ([65a38bc](https://github.com/AztecProtocol/barretenberg/commit/65a38bc045c66c5f64e87ba8c6e446945f2f0a24))
* Update barretenberg bootstrap.sh for mac ([#1732](https://github.com/AztecProtocol/barretenberg/issues/1732)) ([f21ac3e](https://github.com/AztecProtocol/barretenberg/commit/f21ac3e893b5d30f7a4ba8ca10e6fd70f5c617b4))

## [0.4.2](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.1...barretenberg-v0.4.2) (2023-08-21)


### Bug Fixes

* Remove automatic update to `AztecProtocol/dev-bb.js` ([#1712](https://github.com/AztecProtocol/barretenberg/issues/1712)) ([d883900](https://github.com/AztecProtocol/barretenberg/commit/d883900f9b297f659d14583ac93eede5160f9aae))

## [0.4.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.4.0...barretenberg-v0.4.1) (2023-08-21)


### Bug Fixes

* **bb:** Fix Typo ([#1709](https://github.com/AztecProtocol/barretenberg/issues/1709)) ([286d64e](https://github.com/AztecProtocol/barretenberg/commit/286d64e6036336314114f1d2a25273f4dabe36f4))

## [0.4.0](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.6...barretenberg-v0.4.0) (2023-08-21)


### ⚠ BREAKING CHANGES

* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/barretenberg/issues/1618))

### Features

* Add msgpack defs to remaining circuit types ([#1538](https://github.com/AztecProtocol/barretenberg/issues/1538)) ([e560e39](https://github.com/AztecProtocol/barretenberg/commit/e560e3955d039a93e2ed157c684ea36abd178d4b))
* Add workflow to output to dev-bb.js ([#1299](https://github.com/AztecProtocol/barretenberg/issues/1299)) ([25a54f1](https://github.com/AztecProtocol/barretenberg/commit/25a54f123e6f98dafef4cd882839106eadf6ab8d))
* Celer benchmark ([#1369](https://github.com/AztecProtocol/barretenberg/issues/1369)) ([8fd364a](https://github.com/AztecProtocol/barretenberg/commit/8fd364a3ff6e7b5f377ef5ec37649b47fe0a3e44))
* Honk recursive verifier Pt. 1 ([#1488](https://github.com/AztecProtocol/barretenberg/issues/1488)) ([030dace](https://github.com/AztecProtocol/barretenberg/commit/030dacebd9831ed938b546133373cad63e17ecd8))
* New stdlib Transcript  ([#1219](https://github.com/AztecProtocol/barretenberg/issues/1219)) ([1b9e077](https://github.com/AztecProtocol/barretenberg/commit/1b9e0770e7e470f2708eb6f96cd5ee831b84f4f4))


### Bug Fixes

* **acir:** When retrying failed ACIR tests it should not use the default CLI argument ([#1673](https://github.com/AztecProtocol/barretenberg/issues/1673)) ([ea4792d](https://github.com/AztecProtocol/barretenberg/commit/ea4792ddc9c23f7390f47cf78d4939cce6458a46))
* Align bbmalloc implementations ([#1513](https://github.com/AztecProtocol/barretenberg/issues/1513)) ([b92338d](https://github.com/AztecProtocol/barretenberg/commit/b92338d3c9de9d21a6933747a3f1479266d16f9e))
* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/barretenberg/issues/1618)) ([180cdc9](https://github.com/AztecProtocol/barretenberg/commit/180cdc9ac7cf9aa793d9774dc866ceb4e6ec3fbc))
* Bb sync take 2 ([#1669](https://github.com/AztecProtocol/barretenberg/issues/1669)) ([d3eebe4](https://github.com/AztecProtocol/barretenberg/commit/d3eebe46e5b702801c866d7dd073a0eeb9f475b7))
* Bin reference when installing package ([#678](https://github.com/AztecProtocol/barretenberg/issues/678)) ([c734295](https://github.com/AztecProtocol/barretenberg/commit/c734295a10d2c40ede773519664170880f28b2b7))
* Fix paths in `barretenberg` bootstrap.sh script ([#1662](https://github.com/AztecProtocol/barretenberg/issues/1662)) ([c8917cd](https://github.com/AztecProtocol/barretenberg/commit/c8917cd8ec415dafe5309ec0e90aba28184d8294))
* Fixed a failing test and added a small fuzzer ([#1384](https://github.com/AztecProtocol/barretenberg/issues/1384)) ([441e972](https://github.com/AztecProtocol/barretenberg/commit/441e972c88c5c314b4958e158f977f60a8c9e32d))
* Sync aztec master ([#680](https://github.com/AztecProtocol/barretenberg/issues/680)) ([3afc243](https://github.com/AztecProtocol/barretenberg/commit/3afc2438053f530e49fbebbdbadd8db8a630bb8c))

## [0.3.6](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.5...barretenberg-v0.3.6) (2023-08-08)


### Features

* Update release-please.yml ([#651](https://github.com/AztecProtocol/barretenberg/issues/651)) ([2795df6](https://github.com/AztecProtocol/barretenberg/commit/2795df6b705175a32fe2a6f18b3c572e297e277e))

## [0.3.5](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.4...barretenberg-v0.3.5) (2023-08-07)


### Features

* Celer benchmark ([#1369](https://github.com/AztecProtocol/barretenberg/issues/1369)) ([d4ade2a](https://github.com/AztecProtocol/barretenberg/commit/d4ade2a5f06a3abf3c9c2635946d7121cc2f64b4))
* Goblin Honk Composer/Prover/Verifier ([#1220](https://github.com/AztecProtocol/barretenberg/issues/1220)) ([970bb07](https://github.com/AztecProtocol/barretenberg/commit/970bb073763cc59552cd05dccf7f8fc63f58cef9))
* Goblin translator prototype ([#1249](https://github.com/AztecProtocol/barretenberg/issues/1249)) ([7738d74](https://github.com/AztecProtocol/barretenberg/commit/7738d74791acc0fa8b1b1d8bb2a77783ca900123))
* Internal keyword + lending contract and tests ([#978](https://github.com/AztecProtocol/barretenberg/issues/978)) ([e58ca4b](https://github.com/AztecProtocol/barretenberg/commit/e58ca4b332272fc57b2a5358bb5003bac79a8f5a))
* Minimal barretenberg .circleci ([#1352](https://github.com/AztecProtocol/barretenberg/issues/1352)) ([708e2e2](https://github.com/AztecProtocol/barretenberg/commit/708e2e2786de5dce5bfc770c54734e5862a436e5))


### Bug Fixes

* Bootstrap.sh git hook for monorepo ([#1256](https://github.com/AztecProtocol/barretenberg/issues/1256)) ([b22b8d5](https://github.com/AztecProtocol/barretenberg/commit/b22b8d5f42ddfae140068c3ce8b3053d4c8d1874))
* Build-system spot request cancellation ([#1339](https://github.com/AztecProtocol/barretenberg/issues/1339)) ([fc1d96a](https://github.com/AztecProtocol/barretenberg/commit/fc1d96a744a8d5a6cae06c408546c3638408551d))
* Fixing external benchmarks ([#1250](https://github.com/AztecProtocol/barretenberg/issues/1250)) ([0ea6a39](https://github.com/AztecProtocol/barretenberg/commit/0ea6a39950e8cd5ff7765031457c162d03ebae06))
* Fixing fuzzing build after composer splitting ([#1317](https://github.com/AztecProtocol/barretenberg/issues/1317)) ([946c23c](https://github.com/AztecProtocol/barretenberg/commit/946c23c52d45ddce973e453c40c048734e7f6937))
* Reinstate barretenberg-benchmark-aggregator ([#1330](https://github.com/AztecProtocol/barretenberg/issues/1330)) ([407a915](https://github.com/AztecProtocol/barretenberg/commit/407a915a94c7d83dec9e14a11ad0e3461fd2906d))
* Retry git submodule fetch ([#1371](https://github.com/AztecProtocol/barretenberg/issues/1371)) ([037dda3](https://github.com/AztecProtocol/barretenberg/commit/037dda3d254d56a20292d2bed5a9582d36c08427))

## [0.3.4](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.3...barretenberg-v0.3.4) (2023-07-25)


### Features

* Add Goblin Ultra Circuit builder ([#587](https://github.com/AztecProtocol/barretenberg/issues/587)) ([2d38c25](https://github.com/AztecProtocol/barretenberg/commit/2d38c252de8b867955da661181e51f1a5f28cbc6))
* Modify bb.js to be compatible with next.js ([#544](https://github.com/AztecProtocol/barretenberg/issues/544)) ([d384089](https://github.com/AztecProtocol/barretenberg/commit/d384089f60d1a6d5baeb0d3459556a310b790366))
* Support public inputs in Ultra Honk ([#581](https://github.com/AztecProtocol/barretenberg/issues/581)) ([9cd0a06](https://github.com/AztecProtocol/barretenberg/commit/9cd0a064b8258bf4f72dd9e1c5e8f85b074d1bbc))

## [0.3.3](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.2...barretenberg-v0.3.3) (2023-07-17)


### Features

* Bb and bb.js directly parse nargo bincode format. ([#610](https://github.com/AztecProtocol/barretenberg/issues/610)) ([d25e37a](https://github.com/AztecProtocol/barretenberg/commit/d25e37ad74b88dc45337b2a529ede3136dd4a699))
* Goblin work done in Valencia ([#569](https://github.com/AztecProtocol/barretenberg/issues/569)) ([57af751](https://github.com/AztecProtocol/barretenberg/commit/57af751646dc3c038fea24ada4e160f6d422845f))

## [0.3.2](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.1...barretenberg-v0.3.2) (2023-07-12)


### Features

* **msgpack:** Ability to specify NOSCHEMA for cbinds ([#605](https://github.com/AztecProtocol/barretenberg/issues/605)) ([8a4f5f1](https://github.com/AztecProtocol/barretenberg/commit/8a4f5f1d31e1d631c1cd3ed49c100858b58c56b2))

## [0.3.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg-v0.3.0...barretenberg-v0.3.1) (2023-07-11)


### Features

* Sentence case changelog titles ([#598](https://github.com/AztecProtocol/barretenberg/issues/598)) ([1466108](https://github.com/AztecProtocol/barretenberg/commit/146610857ae511e9cfb27f873f49cec2dd19ddad))

## 0.3.0 (2023-07-11)


### ⚠ BREAKING CHANGES

* Use circuit builders ([#501](https://github.com/AztecProtocol/barretenberg/issues/501))
* **dsl:** add hash index to pedersen constraint ([#436](https://github.com/AztecProtocol/barretenberg/issues/436))
* add support for ROM and RAM ACVM opcodes ([#417](https://github.com/AztecProtocol/barretenberg/issues/417))
* replace `MerkleMembershipConstraint` with`ComputeMerkleRootConstraint` ([#385](https://github.com/AztecProtocol/barretenberg/issues/385))
* Remove TOOLCHAIN logic and replace with CMake presets ([#162](https://github.com/AztecProtocol/barretenberg/issues/162))

### Features

* Add `get_sibling_path` method in MerkleTree ([#584](https://github.com/AztecProtocol/barretenberg/issues/584)) ([b3db9f8](https://github.com/AztecProtocol/barretenberg/commit/b3db9f8944e546cd9da9a1529e2562ee75e62369))
* Add `signature_verification_result` to schnorr stdlib ([#173](https://github.com/AztecProtocol/barretenberg/issues/173)) ([7ae381e](https://github.com/AztecProtocol/barretenberg/commit/7ae381e4c5a084efde18917569518c7d4040b653))
* Add equality and serialization to poly_triple ([#172](https://github.com/AztecProtocol/barretenberg/issues/172)) ([142b041](https://github.com/AztecProtocol/barretenberg/commit/142b041b2d3d090785f0e6f319fbf7504c751098))
* Add installation targets for libbarretenberg, wasm & headers ([#185](https://github.com/AztecProtocol/barretenberg/issues/185)) ([f2fdebe](https://github.com/AztecProtocol/barretenberg/commit/f2fdebe037d4d2d90761f98e28b4b0d3af9a0f63))
* Add Noir DSL with acir_format and turbo_proofs namespaces ([#198](https://github.com/AztecProtocol/barretenberg/issues/198)) ([54fab22](https://github.com/AztecProtocol/barretenberg/commit/54fab2217f437bb04a5e9fb71b271cf91b90c6e5))
* Add pkgconfig output for installed target ([#208](https://github.com/AztecProtocol/barretenberg/issues/208)) ([d85a365](https://github.com/AztecProtocol/barretenberg/commit/d85a365180ac2672bbd33bd8b799a1f154716ab3))
* add support for ROM and RAM ACVM opcodes ([#417](https://github.com/AztecProtocol/barretenberg/issues/417)) ([697fabb](https://github.com/AztecProtocol/barretenberg/commit/697fabb7cbeadb9264db5047e7fd36565dad8790))
* Allow bootstrap to work with linux + clang on ARM ([#131](https://github.com/AztecProtocol/barretenberg/issues/131)) ([52cb06b](https://github.com/AztecProtocol/barretenberg/commit/52cb06b445c73f2f324af6595af70ce9c130eb09))
* **api:** external cpp header for circuits ([#489](https://github.com/AztecProtocol/barretenberg/issues/489)) ([fbbb342](https://github.com/AztecProtocol/barretenberg/commit/fbbb34287fdef0e8fedb2e25c5431f17501ad653))
* **bb.js:** initial API ([#232](https://github.com/AztecProtocol/barretenberg/issues/232)) ([c860b02](https://github.com/AztecProtocol/barretenberg/commit/c860b02d80425de161af50acf33e94d94eb0659c))
* Benchmark suite update ([d7b1499](https://github.com/AztecProtocol/barretenberg/commit/d7b14993ac8d329664fd36e7b4aa083935b1d407))
* Benchmark suite update ([#508](https://github.com/AztecProtocol/barretenberg/issues/508)) ([d7b1499](https://github.com/AztecProtocol/barretenberg/commit/d7b14993ac8d329664fd36e7b4aa083935b1d407))
* CI to test aztec circuits with current commit of bberg ([#418](https://github.com/AztecProtocol/barretenberg/issues/418)) ([20a0873](https://github.com/AztecProtocol/barretenberg/commit/20a0873dcbfe4a862ad53a9c137030689a521a04))
* **dsl:** Add ECDSA secp256r1 verification ([#582](https://github.com/AztecProtocol/barretenberg/issues/582)) ([adc4c7b](https://github.com/AztecProtocol/barretenberg/commit/adc4c7b4eb634eae28dd28e25b94b93a5b49c80e))
* **dsl:** add hash index to pedersen constraint ([#436](https://github.com/AztecProtocol/barretenberg/issues/436)) ([e0b8804](https://github.com/AztecProtocol/barretenberg/commit/e0b8804b9418c7aa39e29e800fecb4ed15d73b80))
* **github:** add pull request template ([65f3e33](https://github.com/AztecProtocol/barretenberg/commit/65f3e3312061e7284c0dd0f0f89fa92ee92f9eac))
* **honk:** Shared relation arithmetic ([#514](https://github.com/AztecProtocol/barretenberg/issues/514)) ([0838474](https://github.com/AztecProtocol/barretenberg/commit/0838474e67469a6d91d6595d1ee23e1dea53863c))
* Improve barretenberg headers ([#201](https://github.com/AztecProtocol/barretenberg/issues/201)) ([4e03839](https://github.com/AztecProtocol/barretenberg/commit/4e03839a970a5d07dab7f86cd2b7166a09f5045a))
* Initial native version of bb binary. ([#524](https://github.com/AztecProtocol/barretenberg/issues/524)) ([4a1b532](https://github.com/AztecProtocol/barretenberg/commit/4a1b5322dc78921d253e6a374eba0b616ab788df))
* Make the circuit constructors field agnostic so we can check circuits on grumpkin ([#534](https://github.com/AztecProtocol/barretenberg/issues/534)) ([656d794](https://github.com/AztecProtocol/barretenberg/commit/656d7944f94f3da88250f3140838f3e32e9d0174))
* Multithreaded Sumcheck ([#556](https://github.com/AztecProtocol/barretenberg/issues/556)) ([c4094b1](https://github.com/AztecProtocol/barretenberg/commit/c4094b155ba9d8e914c3e6a5b0d7808945b1eeed))
* **nullifier_tree:** make empty nullifier tree leaves hash be 0 ([#360](https://github.com/AztecProtocol/barretenberg/issues/360)) ([#382](https://github.com/AztecProtocol/barretenberg/issues/382)) ([b85ab8d](https://github.com/AztecProtocol/barretenberg/commit/b85ab8d587b3e93db2aa0f1c4f012e58e5d97915))
* Optimize memory consumption of pedersen generators ([#413](https://github.com/AztecProtocol/barretenberg/issues/413)) ([d60b16a](https://github.com/AztecProtocol/barretenberg/commit/d60b16a14219fd4bd130ce4537c3e94bfa10128f))
* Parallelized folding in Gemini ([#550](https://github.com/AztecProtocol/barretenberg/issues/550)) ([3b962d3](https://github.com/AztecProtocol/barretenberg/commit/3b962d372491430871443fd1b95fd9e049e233c8))
* **pkg-config:** Add a bindir variable ([#239](https://github.com/AztecProtocol/barretenberg/issues/239)) ([611bf34](https://github.com/AztecProtocol/barretenberg/commit/611bf34bcc6f82969a6fe546bf0a7cbecda6d36d))
* Remove TOOLCHAIN logic and replace with CMake presets ([#162](https://github.com/AztecProtocol/barretenberg/issues/162)) ([09db0be](https://github.com/AztecProtocol/barretenberg/commit/09db0be3d09ee12b4b73b03abe8fa4565cdb6660))
* replace `MerkleMembershipConstraint` with`ComputeMerkleRootConstraint` ([#385](https://github.com/AztecProtocol/barretenberg/issues/385)) ([74dbce5](https://github.com/AztecProtocol/barretenberg/commit/74dbce5dfa126ecd6dbda7b758581752f7b6a389))
* Sort includes ([#571](https://github.com/AztecProtocol/barretenberg/issues/571)) ([dfa8736](https://github.com/AztecProtocol/barretenberg/commit/dfa8736136323e62a705066d25bef962a6a0b82d))
* Split plonk and honk tests ([#529](https://github.com/AztecProtocol/barretenberg/issues/529)) ([ba583ff](https://github.com/AztecProtocol/barretenberg/commit/ba583ff00509f636feae7b78304b115e34fc2357))
* Support nix package manager ([#234](https://github.com/AztecProtocol/barretenberg/issues/234)) ([19a72fe](https://github.com/AztecProtocol/barretenberg/commit/19a72fec0ff8d451fc94a9f5563202867a5f8147))
* **ts:** allow passing srs via env functions ([#260](https://github.com/AztecProtocol/barretenberg/issues/260)) ([ac78353](https://github.com/AztecProtocol/barretenberg/commit/ac7835304f4524039abf0a0df9ae85d905f55c86))
* **ultrahonk:** Added a simple filler table to minimize the amount of entries used to make UltraHonk polynomials non-zero ([b20b401](https://github.com/AztecProtocol/barretenberg/commit/b20b4012546c5b67623950d0fedb0974df8bf345))
* **ultrahonk:** Added a simple filler table to minimize the amount of entries used to make UltraHonk polynomials non-zero ([#531](https://github.com/AztecProtocol/barretenberg/issues/531)) ([b20b401](https://github.com/AztecProtocol/barretenberg/commit/b20b4012546c5b67623950d0fedb0974df8bf345))
* Utilize globally installed benchmark if available ([#152](https://github.com/AztecProtocol/barretenberg/issues/152)) ([fbc5027](https://github.com/AztecProtocol/barretenberg/commit/fbc502794e9bbdfda797b11ac71eba996d649722))
* Utilize globally installed gtest if available ([#151](https://github.com/AztecProtocol/barretenberg/issues/151)) ([efa18a6](https://github.com/AztecProtocol/barretenberg/commit/efa18a621917dc6c38f453825cadc76eb793a73c))
* Utilize globally installed leveldb if available ([#134](https://github.com/AztecProtocol/barretenberg/issues/134)) ([255dfb5](https://github.com/AztecProtocol/barretenberg/commit/255dfb52adca885b0a4e4380769a279922af49ff))
* Working UltraPlonk for Noir ([#299](https://github.com/AztecProtocol/barretenberg/issues/299)) ([d56dfbd](https://github.com/AztecProtocol/barretenberg/commit/d56dfbdfd74b438b3c8652e1ae8740de99f93ae5))


### Bug Fixes

* add NUM_RESERVED_GATES before fetching subgroup size in composer ([#539](https://github.com/AztecProtocol/barretenberg/issues/539)) ([fa11abf](https://github.com/AztecProtocol/barretenberg/commit/fa11abf0877314b03420d6f7ace1312df41cd50b))
* Adds `VERSION` file to release-please ([#542](https://github.com/AztecProtocol/barretenberg/issues/542)) ([31fb34c](https://github.com/AztecProtocol/barretenberg/commit/31fb34c307a4336414b1fd2076d96105a29b0e7b))
* Align native library object library with wasm ([#238](https://github.com/AztecProtocol/barretenberg/issues/238)) ([4fa6c0d](https://github.com/AztecProtocol/barretenberg/commit/4fa6c0d2d8c6309d53757ad54d3433d1d662de5f))
* Avoid bb.js memory issues. ([#578](https://github.com/AztecProtocol/barretenberg/issues/578)) ([96891de](https://github.com/AztecProtocol/barretenberg/commit/96891de21fd74ca33ea75ae97f73cada39a5d337))
* Avoid targeting honk test files when testing is disabled ([#125](https://github.com/AztecProtocol/barretenberg/issues/125)) ([e4a70ed](https://github.com/AztecProtocol/barretenberg/commit/e4a70edf2bb39d67095cbe21fff310372369a92d))
* BarycentricData instantiation time and unused code in secp curves ([#572](https://github.com/AztecProtocol/barretenberg/issues/572)) ([bc78bb0](https://github.com/AztecProtocol/barretenberg/commit/bc78bb00d273c756fa4f02967d219cd3fd788890))
* bbmalloc linker error ([#459](https://github.com/AztecProtocol/barretenberg/issues/459)) ([d4761c1](https://github.com/AztecProtocol/barretenberg/commit/d4761c11f30eeecbcb2485f50516bee71809bab1))
* Build on stock apple clang. ([#592](https://github.com/AztecProtocol/barretenberg/issues/592)) ([0ac4bc3](https://github.com/AztecProtocol/barretenberg/commit/0ac4bc36619f85c1b3a65d3f825ba5683cbbe30c))
* **build:** git add -f .yalc ([#265](https://github.com/AztecProtocol/barretenberg/issues/265)) ([7671192](https://github.com/AztecProtocol/barretenberg/commit/7671192c8a60ff0bc0f8ad3e14ac299ff780cc25))
* bump timeout on common test. ([c9bc87d](https://github.com/AztecProtocol/barretenberg/commit/c9bc87d29fa1325162cb1e7bf2db7cc85747fd9e))
* Check for wasm-opt during configure & run on post_build ([#175](https://github.com/AztecProtocol/barretenberg/issues/175)) ([1ff6af3](https://github.com/AztecProtocol/barretenberg/commit/1ff6af3cb6b7b4d3bb53bfbdcbf1c3a568e0fa86))
* check_circuit bug fix ([#510](https://github.com/AztecProtocol/barretenberg/issues/510)) ([4b156a3](https://github.com/AztecProtocol/barretenberg/commit/4b156a3648e6da9dfe292e354da9a27185d2aa9d))
* cleanup of include statements and dependencies ([#527](https://github.com/AztecProtocol/barretenberg/issues/527)) ([b288c24](https://github.com/AztecProtocol/barretenberg/commit/b288c2420bdc350658cd3776bad1eb087cc28d63))
* **cmake:** Remove leveldb dependency that was accidentally re-added ([#335](https://github.com/AztecProtocol/barretenberg/issues/335)) ([3534e2b](https://github.com/AztecProtocol/barretenberg/commit/3534e2bfcca46dbca30573286f43425dab6bc810))
* **dsl:** Use info instead of std::cout to log ([#323](https://github.com/AztecProtocol/barretenberg/issues/323)) ([486d738](https://github.com/AztecProtocol/barretenberg/commit/486d73842b4b7d6aa84fa12d7462fe52e892d416))
* Ecdsa Malleability Bug ([#512](https://github.com/AztecProtocol/barretenberg/issues/512)) ([5cf856c](https://github.com/AztecProtocol/barretenberg/commit/5cf856c5c29c9f9b8abb87d7bde23b4932711350))
* **ecdsa:** correct short weierstrass curve eqn  ([#567](https://github.com/AztecProtocol/barretenberg/issues/567)) ([386ec63](https://github.com/AztecProtocol/barretenberg/commit/386ec6372156d604e37e58490f1c7396077f84c4))
* Ensure barretenberg provides headers that Noir needs ([#200](https://github.com/AztecProtocol/barretenberg/issues/200)) ([0171a49](https://github.com/AztecProtocol/barretenberg/commit/0171a499a175f88a0ee3fcfd4de0f43ad0ebff85))
* Ensure TBB is optional using OPTIONAL_COMPONENTS ([#127](https://github.com/AztecProtocol/barretenberg/issues/127)) ([e3039b2](https://github.com/AztecProtocol/barretenberg/commit/e3039b26ea8aca4b8fdc4b2cbac6716ace261c76))
* Fixed the memory issue ([#509](https://github.com/AztecProtocol/barretenberg/issues/509)) ([107d438](https://github.com/AztecProtocol/barretenberg/commit/107d438ad96847e40f8e5374749b8cba820b2007))
* Increment CMakeList version on releases ([#536](https://github.com/AztecProtocol/barretenberg/issues/536)) ([b571411](https://github.com/AztecProtocol/barretenberg/commit/b571411a6d58f79e3e2553c3b1c81b4f186f2245))
* msgpack error ([#456](https://github.com/AztecProtocol/barretenberg/issues/456)) ([943d6d0](https://github.com/AztecProtocol/barretenberg/commit/943d6d07c57bea521c2593e892e839f25f82b289))
* msgpack variant_impl.hpp ([#462](https://github.com/AztecProtocol/barretenberg/issues/462)) ([b5838a6](https://github.com/AztecProtocol/barretenberg/commit/b5838a6c9fe456e832776da21379e41c0a2bbd5d))
* **nix:** Disable ASM & ADX when building in Nix ([#327](https://github.com/AztecProtocol/barretenberg/issues/327)) ([3bc724d](https://github.com/AztecProtocol/barretenberg/commit/3bc724d2163d29041bfa29a1e49625bab77289a2))
* **nix:** Use wasi-sdk 12 to provide barretenberg-wasm in overlay ([#315](https://github.com/AztecProtocol/barretenberg/issues/315)) ([4a06992](https://github.com/AztecProtocol/barretenberg/commit/4a069923f4a869f8c2315e6d3f738db6e66dcdfa))
* Pass brew omp location via LDFLAGS and CPPFLAGS ([#126](https://github.com/AztecProtocol/barretenberg/issues/126)) ([54141f1](https://github.com/AztecProtocol/barretenberg/commit/54141f12de9eee86220003b1f80d39a41795cedc))
* Remove leveldb_store from stdlib_merkle_tree ([#149](https://github.com/AztecProtocol/barretenberg/issues/149)) ([3ce5e7e](https://github.com/AztecProtocol/barretenberg/commit/3ce5e7e17ca7bb806373be833a44d55a8e584bda))
* Revert "fix: add NUM_RESERVED_GATES before fetching subgroup size in composer" ([#540](https://github.com/AztecProtocol/barretenberg/issues/540)) ([a9fbc39](https://github.com/AztecProtocol/barretenberg/commit/a9fbc3973f24680f676682d15c3a4cef0a1ab798))
* Revert generator changes that cause memory OOB access ([#338](https://github.com/AztecProtocol/barretenberg/issues/338)) ([500daf1](https://github.com/AztecProtocol/barretenberg/commit/500daf1ceb03771d2c01eaf1a86139a7ac1d814f))
* Soundness issue in bigfield's `evaluate_multiply_add` method ([#558](https://github.com/AztecProtocol/barretenberg/issues/558)) ([1a98ac6](https://github.com/AztecProtocol/barretenberg/commit/1a98ac64787a0e2904fd22043497a8d11afe5e6c))
* **srs:** Detect shasum utility when downloading lagrange ([#143](https://github.com/AztecProtocol/barretenberg/issues/143)) ([515604d](https://github.com/AztecProtocol/barretenberg/commit/515604dff83648e00106f35511aa567921599a78))
* Store lagrange forms of selector polys w/ Ultra ([#255](https://github.com/AztecProtocol/barretenberg/issues/255)) ([b121963](https://github.com/AztecProtocol/barretenberg/commit/b12196362497c8dfb3a64284d28de2d8ee7d730c))
* throw -&gt; throw_or_abort in sol gen ([#388](https://github.com/AztecProtocol/barretenberg/issues/388)) ([7cfe3f0](https://github.com/AztecProtocol/barretenberg/commit/7cfe3f055815e333ff8a8f1f30e8377c83d2182a))
* Trigger release-please ([#594](https://github.com/AztecProtocol/barretenberg/issues/594)) ([5042861](https://github.com/AztecProtocol/barretenberg/commit/5042861405df6b5659c0c32418720d8bdea81081))
* Update versioning in nix files when a release is made ([#549](https://github.com/AztecProtocol/barretenberg/issues/549)) ([1b3ff93](https://github.com/AztecProtocol/barretenberg/commit/1b3ff93e7ed8873583cdade95a860adb8823f1cd))
* **wasm:** Remove the CMAKE_STAGING_PREFIX variable from wasm preset ([#240](https://github.com/AztecProtocol/barretenberg/issues/240)) ([f2f8d1f](https://github.com/AztecProtocol/barretenberg/commit/f2f8d1f7a24ca73e30c981fd245c86f7f964abb7))
* Wrap each use of filesystem library in ifndef __wasm__ ([#181](https://github.com/AztecProtocol/barretenberg/issues/181)) ([0eae962](https://github.com/AztecProtocol/barretenberg/commit/0eae96293b4d2da6b6b23ae80ac132fb49f90915))


### Code Refactoring

* Use circuit builders ([#501](https://github.com/AztecProtocol/barretenberg/issues/501)) ([709a29c](https://github.com/AztecProtocol/barretenberg/commit/709a29c89a305be017270361780995353188035a))

## [0.2.0](https://github.com/AztecProtocol/barretenberg/compare/v0.1.0...v0.2.0) (2023-07-11)


### ⚠ BREAKING CHANGES

* Use circuit builders ([#501](https://github.com/AztecProtocol/barretenberg/issues/501))

### Features

* Add `get_sibling_path` method in MerkleTree ([#584](https://github.com/AztecProtocol/barretenberg/issues/584)) ([b3db9f8](https://github.com/AztecProtocol/barretenberg/commit/b3db9f8944e546cd9da9a1529e2562ee75e62369))
* **dsl:** Add ECDSA secp256r1 verification ([#582](https://github.com/AztecProtocol/barretenberg/issues/582)) ([adc4c7b](https://github.com/AztecProtocol/barretenberg/commit/adc4c7b4eb634eae28dd28e25b94b93a5b49c80e))
* Initial native version of bb binary. ([#524](https://github.com/AztecProtocol/barretenberg/issues/524)) ([4a1b532](https://github.com/AztecProtocol/barretenberg/commit/4a1b5322dc78921d253e6a374eba0b616ab788df))
* Make the circuit constructors field agnostic so we can check circuits on grumpkin ([#534](https://github.com/AztecProtocol/barretenberg/issues/534)) ([656d794](https://github.com/AztecProtocol/barretenberg/commit/656d7944f94f3da88250f3140838f3e32e9d0174))
* Multithreaded Sumcheck ([#556](https://github.com/AztecProtocol/barretenberg/issues/556)) ([c4094b1](https://github.com/AztecProtocol/barretenberg/commit/c4094b155ba9d8e914c3e6a5b0d7808945b1eeed))
* Optimize memory consumption of pedersen generators ([#413](https://github.com/AztecProtocol/barretenberg/issues/413)) ([d60b16a](https://github.com/AztecProtocol/barretenberg/commit/d60b16a14219fd4bd130ce4537c3e94bfa10128f))
* Parallelized folding in Gemini ([#550](https://github.com/AztecProtocol/barretenberg/issues/550)) ([3b962d3](https://github.com/AztecProtocol/barretenberg/commit/3b962d372491430871443fd1b95fd9e049e233c8))
* Sort includes ([#571](https://github.com/AztecProtocol/barretenberg/issues/571)) ([dfa8736](https://github.com/AztecProtocol/barretenberg/commit/dfa8736136323e62a705066d25bef962a6a0b82d))
* Split plonk and honk tests ([#529](https://github.com/AztecProtocol/barretenberg/issues/529)) ([ba583ff](https://github.com/AztecProtocol/barretenberg/commit/ba583ff00509f636feae7b78304b115e34fc2357))


### Bug Fixes

* add NUM_RESERVED_GATES before fetching subgroup size in composer ([#539](https://github.com/AztecProtocol/barretenberg/issues/539)) ([fa11abf](https://github.com/AztecProtocol/barretenberg/commit/fa11abf0877314b03420d6f7ace1312df41cd50b))
* Adds `VERSION` file to release-please ([#542](https://github.com/AztecProtocol/barretenberg/issues/542)) ([31fb34c](https://github.com/AztecProtocol/barretenberg/commit/31fb34c307a4336414b1fd2076d96105a29b0e7b))
* Avoid bb.js memory issues. ([#578](https://github.com/AztecProtocol/barretenberg/issues/578)) ([96891de](https://github.com/AztecProtocol/barretenberg/commit/96891de21fd74ca33ea75ae97f73cada39a5d337))
* BarycentricData instantiation time and unused code in secp curves ([#572](https://github.com/AztecProtocol/barretenberg/issues/572)) ([bc78bb0](https://github.com/AztecProtocol/barretenberg/commit/bc78bb00d273c756fa4f02967d219cd3fd788890))
* Build on stock apple clang. ([#592](https://github.com/AztecProtocol/barretenberg/issues/592)) ([0ac4bc3](https://github.com/AztecProtocol/barretenberg/commit/0ac4bc36619f85c1b3a65d3f825ba5683cbbe30c))
* bump timeout on common test. ([c9bc87d](https://github.com/AztecProtocol/barretenberg/commit/c9bc87d29fa1325162cb1e7bf2db7cc85747fd9e))
* check_circuit bug fix ([#510](https://github.com/AztecProtocol/barretenberg/issues/510)) ([4b156a3](https://github.com/AztecProtocol/barretenberg/commit/4b156a3648e6da9dfe292e354da9a27185d2aa9d))
* cleanup of include statements and dependencies ([#527](https://github.com/AztecProtocol/barretenberg/issues/527)) ([b288c24](https://github.com/AztecProtocol/barretenberg/commit/b288c2420bdc350658cd3776bad1eb087cc28d63))
* Ecdsa Malleability Bug ([#512](https://github.com/AztecProtocol/barretenberg/issues/512)) ([5cf856c](https://github.com/AztecProtocol/barretenberg/commit/5cf856c5c29c9f9b8abb87d7bde23b4932711350))
* **ecdsa:** correct short weierstrass curve eqn  ([#567](https://github.com/AztecProtocol/barretenberg/issues/567)) ([386ec63](https://github.com/AztecProtocol/barretenberg/commit/386ec6372156d604e37e58490f1c7396077f84c4))
* Increment CMakeList version on releases ([#536](https://github.com/AztecProtocol/barretenberg/issues/536)) ([b571411](https://github.com/AztecProtocol/barretenberg/commit/b571411a6d58f79e3e2553c3b1c81b4f186f2245))
* Revert "fix: add NUM_RESERVED_GATES before fetching subgroup size in composer" ([#540](https://github.com/AztecProtocol/barretenberg/issues/540)) ([a9fbc39](https://github.com/AztecProtocol/barretenberg/commit/a9fbc3973f24680f676682d15c3a4cef0a1ab798))
* Soundness issue in bigfield's `evaluate_multiply_add` method ([#558](https://github.com/AztecProtocol/barretenberg/issues/558)) ([1a98ac6](https://github.com/AztecProtocol/barretenberg/commit/1a98ac64787a0e2904fd22043497a8d11afe5e6c))
* Update versioning in nix files when a release is made ([#549](https://github.com/AztecProtocol/barretenberg/issues/549)) ([1b3ff93](https://github.com/AztecProtocol/barretenberg/commit/1b3ff93e7ed8873583cdade95a860adb8823f1cd))


### Code Refactoring

* Use circuit builders ([#501](https://github.com/AztecProtocol/barretenberg/issues/501)) ([709a29c](https://github.com/AztecProtocol/barretenberg/commit/709a29c89a305be017270361780995353188035a))

## 0.1.0 (2023-06-15)


### ⚠ BREAKING CHANGES

* **dsl:** add hash index to pedersen constraint ([#436](https://github.com/AztecProtocol/barretenberg/issues/436))
* add support for ROM and RAM ACVM opcodes ([#417](https://github.com/AztecProtocol/barretenberg/issues/417))
* replace `MerkleMembershipConstraint` with`ComputeMerkleRootConstraint` ([#385](https://github.com/AztecProtocol/barretenberg/issues/385))
* Remove TOOLCHAIN logic and replace with CMake presets ([#162](https://github.com/AztecProtocol/barretenberg/issues/162))

### Features

* Add `signature_verification_result` to schnorr stdlib ([#173](https://github.com/AztecProtocol/barretenberg/issues/173)) ([7ae381e](https://github.com/AztecProtocol/barretenberg/commit/7ae381e4c5a084efde18917569518c7d4040b653))
* Add equality and serialization to poly_triple ([#172](https://github.com/AztecProtocol/barretenberg/issues/172)) ([142b041](https://github.com/AztecProtocol/barretenberg/commit/142b041b2d3d090785f0e6f319fbf7504c751098))
* Add installation targets for libbarretenberg, wasm & headers ([#185](https://github.com/AztecProtocol/barretenberg/issues/185)) ([f2fdebe](https://github.com/AztecProtocol/barretenberg/commit/f2fdebe037d4d2d90761f98e28b4b0d3af9a0f63))
* Add Noir DSL with acir_format and turbo_proofs namespaces ([#198](https://github.com/AztecProtocol/barretenberg/issues/198)) ([54fab22](https://github.com/AztecProtocol/barretenberg/commit/54fab2217f437bb04a5e9fb71b271cf91b90c6e5))
* Add pkgconfig output for installed target ([#208](https://github.com/AztecProtocol/barretenberg/issues/208)) ([d85a365](https://github.com/AztecProtocol/barretenberg/commit/d85a365180ac2672bbd33bd8b799a1f154716ab3))
* add support for ROM and RAM ACVM opcodes ([#417](https://github.com/AztecProtocol/barretenberg/issues/417)) ([697fabb](https://github.com/AztecProtocol/barretenberg/commit/697fabb7cbeadb9264db5047e7fd36565dad8790))
* Allow bootstrap to work with linux + clang on ARM ([#131](https://github.com/AztecProtocol/barretenberg/issues/131)) ([52cb06b](https://github.com/AztecProtocol/barretenberg/commit/52cb06b445c73f2f324af6595af70ce9c130eb09))
* **api:** external cpp header for circuits ([#489](https://github.com/AztecProtocol/barretenberg/issues/489)) ([fbbb342](https://github.com/AztecProtocol/barretenberg/commit/fbbb34287fdef0e8fedb2e25c5431f17501ad653))
* **bb.js:** initial API ([#232](https://github.com/AztecProtocol/barretenberg/issues/232)) ([c860b02](https://github.com/AztecProtocol/barretenberg/commit/c860b02d80425de161af50acf33e94d94eb0659c))
* Benchmark suite update ([d7b1499](https://github.com/AztecProtocol/barretenberg/commit/d7b14993ac8d329664fd36e7b4aa083935b1d407))
* Benchmark suite update ([#508](https://github.com/AztecProtocol/barretenberg/issues/508)) ([d7b1499](https://github.com/AztecProtocol/barretenberg/commit/d7b14993ac8d329664fd36e7b4aa083935b1d407))
* CI to test aztec circuits with current commit of bberg ([#418](https://github.com/AztecProtocol/barretenberg/issues/418)) ([20a0873](https://github.com/AztecProtocol/barretenberg/commit/20a0873dcbfe4a862ad53a9c137030689a521a04))
* **dsl:** add hash index to pedersen constraint ([#436](https://github.com/AztecProtocol/barretenberg/issues/436)) ([e0b8804](https://github.com/AztecProtocol/barretenberg/commit/e0b8804b9418c7aa39e29e800fecb4ed15d73b80))
* **github:** add pull request template ([65f3e33](https://github.com/AztecProtocol/barretenberg/commit/65f3e3312061e7284c0dd0f0f89fa92ee92f9eac))
* **honk:** Shared relation arithmetic ([#514](https://github.com/AztecProtocol/barretenberg/issues/514)) ([0838474](https://github.com/AztecProtocol/barretenberg/commit/0838474e67469a6d91d6595d1ee23e1dea53863c))
* Improve barretenberg headers ([#201](https://github.com/AztecProtocol/barretenberg/issues/201)) ([4e03839](https://github.com/AztecProtocol/barretenberg/commit/4e03839a970a5d07dab7f86cd2b7166a09f5045a))
* **nullifier_tree:** make empty nullifier tree leaves hash be 0 ([#360](https://github.com/AztecProtocol/barretenberg/issues/360)) ([#382](https://github.com/AztecProtocol/barretenberg/issues/382)) ([b85ab8d](https://github.com/AztecProtocol/barretenberg/commit/b85ab8d587b3e93db2aa0f1c4f012e58e5d97915))
* **pkg-config:** Add a bindir variable ([#239](https://github.com/AztecProtocol/barretenberg/issues/239)) ([611bf34](https://github.com/AztecProtocol/barretenberg/commit/611bf34bcc6f82969a6fe546bf0a7cbecda6d36d))
* Remove TOOLCHAIN logic and replace with CMake presets ([#162](https://github.com/AztecProtocol/barretenberg/issues/162)) ([09db0be](https://github.com/AztecProtocol/barretenberg/commit/09db0be3d09ee12b4b73b03abe8fa4565cdb6660))
* replace `MerkleMembershipConstraint` with`ComputeMerkleRootConstraint` ([#385](https://github.com/AztecProtocol/barretenberg/issues/385)) ([74dbce5](https://github.com/AztecProtocol/barretenberg/commit/74dbce5dfa126ecd6dbda7b758581752f7b6a389))
* Support nix package manager ([#234](https://github.com/AztecProtocol/barretenberg/issues/234)) ([19a72fe](https://github.com/AztecProtocol/barretenberg/commit/19a72fec0ff8d451fc94a9f5563202867a5f8147))
* **ts:** allow passing srs via env functions ([#260](https://github.com/AztecProtocol/barretenberg/issues/260)) ([ac78353](https://github.com/AztecProtocol/barretenberg/commit/ac7835304f4524039abf0a0df9ae85d905f55c86))
* **ultrahonk:** Added a simple filler table to minimize the amount of entries used to make UltraHonk polynomials non-zero ([b20b401](https://github.com/AztecProtocol/barretenberg/commit/b20b4012546c5b67623950d0fedb0974df8bf345))
* **ultrahonk:** Added a simple filler table to minimize the amount of entries used to make UltraHonk polynomials non-zero ([#531](https://github.com/AztecProtocol/barretenberg/issues/531)) ([b20b401](https://github.com/AztecProtocol/barretenberg/commit/b20b4012546c5b67623950d0fedb0974df8bf345))
* Utilize globally installed benchmark if available ([#152](https://github.com/AztecProtocol/barretenberg/issues/152)) ([fbc5027](https://github.com/AztecProtocol/barretenberg/commit/fbc502794e9bbdfda797b11ac71eba996d649722))
* Utilize globally installed gtest if available ([#151](https://github.com/AztecProtocol/barretenberg/issues/151)) ([efa18a6](https://github.com/AztecProtocol/barretenberg/commit/efa18a621917dc6c38f453825cadc76eb793a73c))
* Utilize globally installed leveldb if available ([#134](https://github.com/AztecProtocol/barretenberg/issues/134)) ([255dfb5](https://github.com/AztecProtocol/barretenberg/commit/255dfb52adca885b0a4e4380769a279922af49ff))
* Working UltraPlonk for Noir ([#299](https://github.com/AztecProtocol/barretenberg/issues/299)) ([d56dfbd](https://github.com/AztecProtocol/barretenberg/commit/d56dfbdfd74b438b3c8652e1ae8740de99f93ae5))


### Bug Fixes

* Align native library object library with wasm ([#238](https://github.com/AztecProtocol/barretenberg/issues/238)) ([4fa6c0d](https://github.com/AztecProtocol/barretenberg/commit/4fa6c0d2d8c6309d53757ad54d3433d1d662de5f))
* Avoid targeting honk test files when testing is disabled ([#125](https://github.com/AztecProtocol/barretenberg/issues/125)) ([e4a70ed](https://github.com/AztecProtocol/barretenberg/commit/e4a70edf2bb39d67095cbe21fff310372369a92d))
* bbmalloc linker error ([#459](https://github.com/AztecProtocol/barretenberg/issues/459)) ([d4761c1](https://github.com/AztecProtocol/barretenberg/commit/d4761c11f30eeecbcb2485f50516bee71809bab1))
* **build:** git add -f .yalc ([#265](https://github.com/AztecProtocol/barretenberg/issues/265)) ([7671192](https://github.com/AztecProtocol/barretenberg/commit/7671192c8a60ff0bc0f8ad3e14ac299ff780cc25))
* Check for wasm-opt during configure & run on post_build ([#175](https://github.com/AztecProtocol/barretenberg/issues/175)) ([1ff6af3](https://github.com/AztecProtocol/barretenberg/commit/1ff6af3cb6b7b4d3bb53bfbdcbf1c3a568e0fa86))
* **cmake:** Remove leveldb dependency that was accidentally re-added ([#335](https://github.com/AztecProtocol/barretenberg/issues/335)) ([3534e2b](https://github.com/AztecProtocol/barretenberg/commit/3534e2bfcca46dbca30573286f43425dab6bc810))
* **dsl:** Use info instead of std::cout to log ([#323](https://github.com/AztecProtocol/barretenberg/issues/323)) ([486d738](https://github.com/AztecProtocol/barretenberg/commit/486d73842b4b7d6aa84fa12d7462fe52e892d416))
* Ensure barretenberg provides headers that Noir needs ([#200](https://github.com/AztecProtocol/barretenberg/issues/200)) ([0171a49](https://github.com/AztecProtocol/barretenberg/commit/0171a499a175f88a0ee3fcfd4de0f43ad0ebff85))
* Ensure TBB is optional using OPTIONAL_COMPONENTS ([#127](https://github.com/AztecProtocol/barretenberg/issues/127)) ([e3039b2](https://github.com/AztecProtocol/barretenberg/commit/e3039b26ea8aca4b8fdc4b2cbac6716ace261c76))
* Fixed the memory issue ([#509](https://github.com/AztecProtocol/barretenberg/issues/509)) ([107d438](https://github.com/AztecProtocol/barretenberg/commit/107d438ad96847e40f8e5374749b8cba820b2007))
* msgpack error ([#456](https://github.com/AztecProtocol/barretenberg/issues/456)) ([943d6d0](https://github.com/AztecProtocol/barretenberg/commit/943d6d07c57bea521c2593e892e839f25f82b289))
* msgpack variant_impl.hpp ([#462](https://github.com/AztecProtocol/barretenberg/issues/462)) ([b5838a6](https://github.com/AztecProtocol/barretenberg/commit/b5838a6c9fe456e832776da21379e41c0a2bbd5d))
* **nix:** Disable ASM & ADX when building in Nix ([#327](https://github.com/AztecProtocol/barretenberg/issues/327)) ([3bc724d](https://github.com/AztecProtocol/barretenberg/commit/3bc724d2163d29041bfa29a1e49625bab77289a2))
* **nix:** Use wasi-sdk 12 to provide barretenberg-wasm in overlay ([#315](https://github.com/AztecProtocol/barretenberg/issues/315)) ([4a06992](https://github.com/AztecProtocol/barretenberg/commit/4a069923f4a869f8c2315e6d3f738db6e66dcdfa))
* Pass brew omp location via LDFLAGS and CPPFLAGS ([#126](https://github.com/AztecProtocol/barretenberg/issues/126)) ([54141f1](https://github.com/AztecProtocol/barretenberg/commit/54141f12de9eee86220003b1f80d39a41795cedc))
* Remove leveldb_store from stdlib_merkle_tree ([#149](https://github.com/AztecProtocol/barretenberg/issues/149)) ([3ce5e7e](https://github.com/AztecProtocol/barretenberg/commit/3ce5e7e17ca7bb806373be833a44d55a8e584bda))
* Revert generator changes that cause memory OOB access ([#338](https://github.com/AztecProtocol/barretenberg/issues/338)) ([500daf1](https://github.com/AztecProtocol/barretenberg/commit/500daf1ceb03771d2c01eaf1a86139a7ac1d814f))
* **srs:** Detect shasum utility when downloading lagrange ([#143](https://github.com/AztecProtocol/barretenberg/issues/143)) ([515604d](https://github.com/AztecProtocol/barretenberg/commit/515604dff83648e00106f35511aa567921599a78))
* Store lagrange forms of selector polys w/ Ultra ([#255](https://github.com/AztecProtocol/barretenberg/issues/255)) ([b121963](https://github.com/AztecProtocol/barretenberg/commit/b12196362497c8dfb3a64284d28de2d8ee7d730c))
* throw -&gt; throw_or_abort in sol gen ([#388](https://github.com/AztecProtocol/barretenberg/issues/388)) ([7cfe3f0](https://github.com/AztecProtocol/barretenberg/commit/7cfe3f055815e333ff8a8f1f30e8377c83d2182a))
* **wasm:** Remove the CMAKE_STAGING_PREFIX variable from wasm preset ([#240](https://github.com/AztecProtocol/barretenberg/issues/240)) ([f2f8d1f](https://github.com/AztecProtocol/barretenberg/commit/f2f8d1f7a24ca73e30c981fd245c86f7f964abb7))
* Wrap each use of filesystem library in ifndef __wasm__ ([#181](https://github.com/AztecProtocol/barretenberg/issues/181)) ([0eae962](https://github.com/AztecProtocol/barretenberg/commit/0eae96293b4d2da6b6b23ae80ac132fb49f90915))
