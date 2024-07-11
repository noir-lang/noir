# Changelog

## [0.46.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.46.3...aztec-packages-v0.46.4) (2024-07-11)


### Features

* Configure world-state to follow the proven chain only ([#7430](https://github.com/AztecProtocol/aztec-packages/issues/7430)) ([2e41ac7](https://github.com/AztecProtocol/aztec-packages/commit/2e41ac7d6b3c0e9e0bf02a8687f9c3d7446a08c6))


### Bug Fixes

* Missing secrets in docs publish ([#7445](https://github.com/AztecProtocol/aztec-packages/issues/7445)) ([840a4b9](https://github.com/AztecProtocol/aztec-packages/commit/840a4b987f363626667b48febd46787f13a0f917))
* Use simulated circuit for tiny reset ([#7442](https://github.com/AztecProtocol/aztec-packages/issues/7442)) ([f79a7c0](https://github.com/AztecProtocol/aztec-packages/commit/f79a7c0d85e11bf7e2b59a033ef58dac31d4f77b))


### Miscellaneous

* Apply where statement to impls instead of fns ([#7433](https://github.com/AztecProtocol/aztec-packages/issues/7433)) ([bb201f2](https://github.com/AztecProtocol/aztec-packages/commit/bb201f2fc8543cf752e2b5d5ec7ec15d3e7cdac5))
* **avm:** Codegen cleanup ([#7439](https://github.com/AztecProtocol/aztec-packages/issues/7439)) ([e31887e](https://github.com/AztecProtocol/aztec-packages/commit/e31887e0091e31fcec59b8c792ec6af36d835f04))
* **proving:** Post honk branch fixes ([#7435](https://github.com/AztecProtocol/aztec-packages/issues/7435)) ([86eafa0](https://github.com/AztecProtocol/aztec-packages/commit/86eafa0ca43645252852d1aa4def33de86156ff6))

## [0.46.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.46.2...aztec-packages-v0.46.3) (2024-07-11)


### Features

* Add CLI argument for debugging comptime blocks (https://github.com/noir-lang/noir/pull/5192) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Add reset tiny and optimize tail ([#7422](https://github.com/AztecProtocol/aztec-packages/issues/7422)) ([399917b](https://github.com/AztecProtocol/aztec-packages/commit/399917b3e6916805bb55596b47183e44700fe8f5))
* **avm:** Calldatacopy and return gadget ([#7415](https://github.com/AztecProtocol/aztec-packages/issues/7415)) ([ec39e4e](https://github.com/AztecProtocol/aztec-packages/commit/ec39e4e2ffecb6d6e355eb3963008b710cc11d2c)), closes [#7381](https://github.com/AztecProtocol/aztec-packages/issues/7381) [#7211](https://github.com/AztecProtocol/aztec-packages/issues/7211)
* **avm:** Make ProverPolynomials::get_row return references ([#7419](https://github.com/AztecProtocol/aztec-packages/issues/7419)) ([108fc5f](https://github.com/AztecProtocol/aztec-packages/commit/108fc5f92e44b027b38fa31614e14f2b7a9f650a))
* Integrate new proving systems in e2e ([#6971](https://github.com/AztecProtocol/aztec-packages/issues/6971)) ([723a0c1](https://github.com/AztecProtocol/aztec-packages/commit/723a0c10c9010f3869f103c77f71950efbf7106c))
* Lsp rename/find-all-references for struct members (https://github.com/noir-lang/noir/pull/5443) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* MSM sorting ([#7351](https://github.com/AztecProtocol/aztec-packages/issues/7351)) ([5cbdc54](https://github.com/AztecProtocol/aztec-packages/commit/5cbdc549f0ab137ab4fa601e20d80699871faaf4))
* **optimization:** Deduplicate more instructions (https://github.com/noir-lang/noir/pull/5457) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Prefix operator overload trait dispatch (https://github.com/noir-lang/noir/pull/5423) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Remove proof from L1 Rollup process ([#7347](https://github.com/AztecProtocol/aztec-packages/issues/7347)) ([2645eab](https://github.com/AztecProtocol/aztec-packages/commit/2645eab19bac030835c959eb01f8f3af27f89adf)), closes [#7346](https://github.com/AztecProtocol/aztec-packages/issues/7346)
* Remove ram tables in note_getter ([#7434](https://github.com/AztecProtocol/aztec-packages/issues/7434)) ([fd67da3](https://github.com/AztecProtocol/aztec-packages/commit/fd67da35da3949bf112392d7cf1d512c5eed23eb))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5467) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Typing return values of embedded_curve_ops ([#7413](https://github.com/AztecProtocol/aztec-packages/issues/7413)) ([db96077](https://github.com/AztecProtocol/aztec-packages/commit/db960772abfead018e4f6da55ae259c6b3d574ef))


### Bug Fixes

* **avm:** Fixes AVM full tests and decrease timeout to 35 minutes ([#7438](https://github.com/AztecProtocol/aztec-packages/issues/7438)) ([2a7494b](https://github.com/AztecProtocol/aztec-packages/commit/2a7494baec4396b9fa62f0a9c240b4b02f23fb1d))
* Memory init with no other ops gate counting ([#7427](https://github.com/AztecProtocol/aztec-packages/issues/7427)) ([e7177ba](https://github.com/AztecProtocol/aztec-packages/commit/e7177ba0f96c1da3edbcdffdaaf88c128bbdd719))
* Pass secrets to ci-arm.yml ([#7436](https://github.com/AztecProtocol/aztec-packages/issues/7436)) ([619501d](https://github.com/AztecProtocol/aztec-packages/commit/619501df5aa4c544cd8607dae5d4e20595109b2f))
* Remove compile-time error for invalid indices (https://github.com/noir-lang/noir/pull/5466) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Using different generators in private refund ([#7414](https://github.com/AztecProtocol/aztec-packages/issues/7414)) ([59b92ca](https://github.com/AztecProtocol/aztec-packages/commit/59b92ca0f72ca3705dd6933b304897c91edc81c3)), closes [#7320](https://github.com/AztecProtocol/aztec-packages/issues/7320)


### Miscellaneous

* **bb:** Fix double increment ([#7428](https://github.com/AztecProtocol/aztec-packages/issues/7428)) ([7870a58](https://github.com/AztecProtocol/aztec-packages/commit/7870a5815dc759aed7097dc9eb5ab8e10b3a1865))
* **boxes:** Adding an init command for an empty project ([#7398](https://github.com/AztecProtocol/aztec-packages/issues/7398)) ([a6a605d](https://github.com/AztecProtocol/aztec-packages/commit/a6a605d3cd83b2f4b8e47722ff262382a7a2ea1d))
* Bump bb to 0.45.1 (https://github.com/noir-lang/noir/pull/5469) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Disable flaky cheat code test ([7b8c2ba](https://github.com/AztecProtocol/aztec-packages/commit/7b8c2ba14600f4e51896bec15c6e7f3286885050))
* Document EmbeddedCurvePoint (https://github.com/noir-lang/noir/pull/5468) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))
* Minimize usage of get_row in inverse computation ([#7431](https://github.com/AztecProtocol/aztec-packages/issues/7431)) ([f177887](https://github.com/AztecProtocol/aztec-packages/commit/f1778876eac8ef65edd06c49d1ddf2429d6583e5))
* Private refund cleanup ([#7403](https://github.com/AztecProtocol/aztec-packages/issues/7403)) ([ebec8ff](https://github.com/AztecProtocol/aztec-packages/commit/ebec8ff48900b48be3fce6bc9a52bcb566c79a7f))
* Replace relative paths to noir-protocol-circuits ([842f6d1](https://github.com/AztecProtocol/aztec-packages/commit/842f6d1978aaeba9d39e3433f06f2f402145b754))
* Unbundle `check_array_is_initialized` (https://github.com/noir-lang/noir/pull/5451) ([97ecff5](https://github.com/AztecProtocol/aztec-packages/commit/97ecff5ea76a0da878bdccc453b121147f726ec4))

## [0.46.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.46.1...aztec-packages-v0.46.2) (2024-07-10)


### Features

* Optimize assert_split_sorted_transformed_value_arrays ([#7417](https://github.com/AztecProtocol/aztec-packages/issues/7417)) ([4355b3f](https://github.com/AztecProtocol/aztec-packages/commit/4355b3f3084696c54f6cd44aed7baf6f4caa925d))


### Bug Fixes

* Updated docs ([#7418](https://github.com/AztecProtocol/aztec-packages/issues/7418)) ([ad3da14](https://github.com/AztecProtocol/aztec-packages/commit/ad3da14eb715c1ec4a1e3b5ffc3d792eb738e404))


### Miscellaneous

* **docs:** Cleanup voting tutorial and deployment guide ([#7406](https://github.com/AztecProtocol/aztec-packages/issues/7406)) ([60cead2](https://github.com/AztecProtocol/aztec-packages/commit/60cead28dbdc397e80dfa14e180d01e53c202f15))

## [0.46.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.46.0...aztec-packages-v0.46.1) (2024-07-10)


### Features

* Apply `no_predicates` in stdlib (https://github.com/noir-lang/noir/pull/5454) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Lsp rename/find-all-references for globals (https://github.com/noir-lang/noir/pull/5415) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Lsp rename/find-all-references for local variables (https://github.com/noir-lang/noir/pull/5439) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Remove duplicated array reads at constant indices (https://github.com/noir-lang/noir/pull/5445) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Remove redundant `EnableSideEffects` instructions (https://github.com/noir-lang/noir/pull/5440) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))


### Bug Fixes

* Account for the expected kind when resolving turbofish generics (https://github.com/noir-lang/noir/pull/5448) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Added bb to noir-projects deps ([#7412](https://github.com/AztecProtocol/aztec-packages/issues/7412)) ([6d3ed3a](https://github.com/AztecProtocol/aztec-packages/commit/6d3ed3a5269f1354c8c722232fd6f1d46ac7a245))
* Fix issue with unresolved results (https://github.com/noir-lang/noir/pull/5453) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))
* Prevent `no_predicates` from removing predicates in calling function (https://github.com/noir-lang/noir/pull/5452) ([6237d96](https://github.com/AztecProtocol/aztec-packages/commit/6237d96a0bc23a5ed656e7ba172fb57facd9c807))


### Miscellaneous

* Replace relative paths to noir-protocol-circuits ([db45302](https://github.com/AztecProtocol/aztec-packages/commit/db453026efa29dddc973f507b002c1c4fd1a3676))
* Replace usage of `GrumpkinPoint` with `EmbeddedCurvePoint` ([#7382](https://github.com/AztecProtocol/aztec-packages/issues/7382)) ([5279695](https://github.com/AztecProtocol/aztec-packages/commit/52796958738f8f1eb90d9691ff489d189f9bce90))
* Replace usage of `GrumpkinPrivateKey` with `EmbeddedCurveScalar` ([#7384](https://github.com/AztecProtocol/aztec-packages/issues/7384)) ([a917198](https://github.com/AztecProtocol/aztec-packages/commit/a917198c6a17063414087419d8cb1de93e6dc21e))

## [0.46.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.45.1...aztec-packages-v0.46.0) (2024-07-09)


### ⚠ BREAKING CHANGES

* constant inputs for blackbox ([#7222](https://github.com/AztecProtocol/aztec-packages/issues/7222))

### Features

* Add more slice methods to the stdlib (https://github.com/noir-lang/noir/pull/5424) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Constant inputs for blackbox ([#7222](https://github.com/AztecProtocol/aztec-packages/issues/7222)) ([9f9ded2](https://github.com/AztecProtocol/aztec-packages/commit/9f9ded2b99980b3b40fce9b55e72c91df1dc3d72))
* Detect subgraphs that are completely independent from inputs or outputs (https://github.com/noir-lang/noir/pull/5402) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Lsp "go to definition" for modules (https://github.com/noir-lang/noir/pull/5406) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Lsp rename/find-all-references for traits (https://github.com/noir-lang/noir/pull/5409) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Lsp rename/find-all-references for type aliases (https://github.com/noir-lang/noir/pull/5414) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5408) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5432) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* TXE docs and usability ([#7305](https://github.com/AztecProtocol/aztec-packages/issues/7305)) ([6b2a351](https://github.com/AztecProtocol/aztec-packages/commit/6b2a351910f4e95ade690157de0574cb9a10b4e3))
* Unquote multiple items from annotations (https://github.com/noir-lang/noir/pull/5441) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Use BB code hash for vk caching ([#7396](https://github.com/AztecProtocol/aztec-packages/issues/7396)) ([b03f1a2](https://github.com/AztecProtocol/aztec-packages/commit/b03f1a2e3927207b314bc16b3bf90e727088541f))
* VK tree ([#6914](https://github.com/AztecProtocol/aztec-packages/issues/6914)) ([8631237](https://github.com/AztecProtocol/aztec-packages/commit/863123729fed0f4a150e634f52da06ac6b581162))


### Bug Fixes

* Added a comment to clarify how to launch TXE ([#7402](https://github.com/AztecProtocol/aztec-packages/issues/7402)) ([1ca48a4](https://github.com/AztecProtocol/aztec-packages/commit/1ca48a4355370644dceb6680643680f7e8cd5228))
* Added missing dep ([#7405](https://github.com/AztecProtocol/aztec-packages/issues/7405)) ([1cb968a](https://github.com/AztecProtocol/aztec-packages/commit/1cb968a6b9e1293cdb14d1555fd3d5bba8cae937))
* Allow importing notes from other contracts and inject them in the macros ([#7349](https://github.com/AztecProtocol/aztec-packages/issues/7349)) ([586c0b0](https://github.com/AztecProtocol/aztec-packages/commit/586c0b019d0c2c462188f03c9c31d25a46e35eb6))
* Change panic to error in interpreter (https://github.com/noir-lang/noir/pull/5446) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Complete call stacks with no_predicates (https://github.com/noir-lang/noir/pull/5418) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Correct range for overlfowing/underflowing integer assignment (https://github.com/noir-lang/noir/pull/5416) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* **e2e snapshots:** Be specific about ipv4 localhost ([#7350](https://github.com/AztecProtocol/aztec-packages/issues/7350)) ([e2bbf06](https://github.com/AztecProtocol/aztec-packages/commit/e2bbf06310657637178fd1cc62f00f5db5c97c15))
* Fields fromstring not working as intended ([#7365](https://github.com/AztecProtocol/aztec-packages/issues/7365)) ([633eb6b](https://github.com/AztecProtocol/aztec-packages/commit/633eb6b73e5e5c782e871f826435fcb9850271eb))
* Included argshash computation in public call_interfaces and cleanup ([#7354](https://github.com/AztecProtocol/aztec-packages/issues/7354)) ([13e9b94](https://github.com/AztecProtocol/aztec-packages/commit/13e9b9435cbdab7d0ccd75e20ef6a155345bb842))
* Lsp find struct reference in return locations and paths (https://github.com/noir-lang/noir/pull/5404) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Lsp struct rename/reference difference (https://github.com/noir-lang/noir/pull/5411) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Privacy leak in private refund ([#7358](https://github.com/AztecProtocol/aztec-packages/issues/7358)) ([e05f6f0](https://github.com/AztecProtocol/aztec-packages/commit/e05f6f03263cd8760c8490389458e3c158ad24b3)), closes [#7321](https://github.com/AztecProtocol/aztec-packages/issues/7321)


### Miscellaneous

* Add remaining slice methods to the interpreter (https://github.com/noir-lang/noir/pull/5422) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* Add test and benchmarks for poseidon2 (https://github.com/noir-lang/noir/pull/5386) ([c7b1ae4](https://github.com/AztecProtocol/aztec-packages/commit/c7b1ae40593c24530723f344111459a51ad5f0e5))
* **avm:** Avoid including flavor where possible ([#7361](https://github.com/AztecProtocol/aztec-packages/issues/7361)) ([dbdffd6](https://github.com/AztecProtocol/aztec-packages/commit/dbdffd60b12aa5152fbd2da7d20abc8550d33cef))
* **avm:** Better log_derivative_inverse_round ([#7360](https://github.com/AztecProtocol/aztec-packages/issues/7360)) ([6329833](https://github.com/AztecProtocol/aztec-packages/commit/63298337162b80c8d9b82c94760a0fb7be0fe940))
* **avm:** Make stats thread safe ([#7393](https://github.com/AztecProtocol/aztec-packages/issues/7393)) ([894ac3b](https://github.com/AztecProtocol/aztec-packages/commit/894ac3b904b8753f2820c7170d70e491201e8ede))
* **avm:** Smaller prover ([#7359](https://github.com/AztecProtocol/aztec-packages/issues/7359)) ([7d8c833](https://github.com/AztecProtocol/aztec-packages/commit/7d8c833f94f5c796cb146e6fb5a961e471163ec0))
* **avm:** Smaller transcript ([#7357](https://github.com/AztecProtocol/aztec-packages/issues/7357)) ([3952a44](https://github.com/AztecProtocol/aztec-packages/commit/3952a444629fc03616089c27d0e037240db7b4e9))
* **bb:** Do not instantiate Relation ([#7389](https://github.com/AztecProtocol/aztec-packages/issues/7389)) ([d9cbf4c](https://github.com/AztecProtocol/aztec-packages/commit/d9cbf4c289d3b3952f84540dadf35e0b410eef2a))
* Counters ([#7342](https://github.com/AztecProtocol/aztec-packages/issues/7342)) ([819f370](https://github.com/AztecProtocol/aztec-packages/commit/819f37002a253cdba8c46daac5d68f64fa11f19c))
* **docs:** Fix link to docker ([#7395](https://github.com/AztecProtocol/aztec-packages/issues/7395)) ([ad4a401](https://github.com/AztecProtocol/aztec-packages/commit/ad4a401ad4c56734043ddfd12bf8cf126e43988f))
* **docs:** Update token tutorial ([#7241](https://github.com/AztecProtocol/aztec-packages/issues/7241)) ([0414eb5](https://github.com/AztecProtocol/aztec-packages/commit/0414eb50fff3fbaf12a15bfc2e111b480bb750a4))
* Dummy workflow (https://github.com/noir-lang/noir/pull/5438) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Extracts refactoring of reading error payloads from [#5403](https://github.com/AztecProtocol/aztec-packages/issues/5403) (https://github.com/noir-lang/noir/pull/5413) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Generate declaration files for VKs ([#7391](https://github.com/AztecProtocol/aztec-packages/issues/7391)) ([7c96636](https://github.com/AztecProtocol/aztec-packages/commit/7c966365b93fd6cbd8827ed7c44511513e1567fc))
* Merge `BarretenbergVerifierBackend` and `BarretenbergBackend` (https://github.com/noir-lang/noir/pull/5399) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Move SSA checks to a new folder (https://github.com/noir-lang/noir/pull/5434) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Notify Noir team on changes to the stdlib ([#7387](https://github.com/AztecProtocol/aztec-packages/issues/7387)) ([061aac3](https://github.com/AztecProtocol/aztec-packages/commit/061aac33a6713f62a73c5d1920687903c5fb499f))
* Optimize private call stack item hash for gate count ([#7285](https://github.com/AztecProtocol/aztec-packages/issues/7285)) ([783d9b6](https://github.com/AztecProtocol/aztec-packages/commit/783d9b67205dcf8ba67a3eb989db8bc70a4b67f7))
* Optimize public call stack item hashing ([#7330](https://github.com/AztecProtocol/aztec-packages/issues/7330)) ([4a5093c](https://github.com/AztecProtocol/aztec-packages/commit/4a5093cd6639305a7e4c7876da94dcf7cb71ebc2))
* Refactor logic around inlining `no_predicates` functions (https://github.com/noir-lang/noir/pull/5433) ([6137a10](https://github.com/AztecProtocol/aztec-packages/commit/6137a10c9a1d052c1e80ba43aaa5a13f9bf08fee))
* Replace relative paths to noir-protocol-circuits ([8510269](https://github.com/AztecProtocol/aztec-packages/commit/8510269f007afe8136eb0d77667fbb44b8148ce1))
* Replace relative paths to noir-protocol-circuits ([9eb8d1c](https://github.com/AztecProtocol/aztec-packages/commit/9eb8d1c940dbda5c6efe931450264198e4e7738d))
* Sync external contributions ([#6984](https://github.com/AztecProtocol/aztec-packages/issues/6984)) ([a265b29](https://github.com/AztecProtocol/aztec-packages/commit/a265b29ff473927dbc2fb0beb5c9bf69b7ef1ca9))


### Documentation

* Making private refunds smooth-brain friendly ([#7343](https://github.com/AztecProtocol/aztec-packages/issues/7343)) ([533c937](https://github.com/AztecProtocol/aztec-packages/commit/533c9378df19b3a4c3d5d2f9489e64d1fa2e4fb8))
* Update overview.md (fix link text) ([#7344](https://github.com/AztecProtocol/aztec-packages/issues/7344)) ([79e5856](https://github.com/AztecProtocol/aztec-packages/commit/79e5856bb065d0fdd9b484ab9c78c1e978d4cacb))

## [0.45.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.45.0...aztec-packages-v0.45.1) (2024-07-04)


### Features

* Add support for fieldable in events ([#7310](https://github.com/AztecProtocol/aztec-packages/issues/7310)) ([694cebc](https://github.com/AztecProtocol/aztec-packages/commit/694cebc10c5713b2fa82ac6a29961bf44b026046)), closes [#6951](https://github.com/AztecProtocol/aztec-packages/issues/6951)
* **avm:** Use template engine for codegen ([#7299](https://github.com/AztecProtocol/aztec-packages/issues/7299)) ([d4359a3](https://github.com/AztecProtocol/aztec-packages/commit/d4359a34668fc389a8a5c5a8b8493f0dc8a5d4ae))
* Build releases for `aarch64-unknown-linux-gnu` target (https://github.com/noir-lang/noir/pull/5289) ([2ae17f2](https://github.com/AztecProtocol/aztec-packages/commit/2ae17f2177380244f695575c169cc591496cf3ad))
* Create codeql.yml ([#7318](https://github.com/AztecProtocol/aztec-packages/issues/7318)) ([11fcfd2](https://github.com/AztecProtocol/aztec-packages/commit/11fcfd22f9c322a07d4b75a440c2fb0baa53e305))
* Deploy l1 contracts on devnet ([#7306](https://github.com/AztecProtocol/aztec-packages/issues/7306)) ([b8eef86](https://github.com/AztecProtocol/aztec-packages/commit/b8eef86228e8ccd7a2917d1e89a6994b11c18db5))
* Implement trait dispatch in the comptime interpreter (https://github.com/noir-lang/noir/pull/5376) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Lsp "find all references" (https://github.com/noir-lang/noir/pull/5395) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* Lsp rename struct (https://github.com/noir-lang/noir/pull/5380) ([2ae17f2](https://github.com/AztecProtocol/aztec-packages/commit/2ae17f2177380244f695575c169cc591496cf3ad))
* **lsp:** Allow function rename (https://github.com/noir-lang/noir/pull/4294) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Remove note hash nullifier counter. ([#7294](https://github.com/AztecProtocol/aztec-packages/issues/7294)) ([c0c9144](https://github.com/AztecProtocol/aztec-packages/commit/c0c9144a961da895c4885af2e59e22617325e61a))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5387) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5401) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))


### Bug Fixes

* Align reset variant sizes after constants changes ([#7340](https://github.com/AztecProtocol/aztec-packages/issues/7340)) ([e431b6f](https://github.com/AztecProtocol/aztec-packages/commit/e431b6f64937e8fba357f890bdc3042adf17cb44))
* Correctly detect signed/unsigned integer overflows/underflows (https://github.com/noir-lang/noir/pull/5375) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* **docs:** Fix broken docs link to gihtub (https://github.com/noir-lang/noir/pull/5398) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* **docs:** Fix colour leak ([#7289](https://github.com/AztecProtocol/aztec-packages/issues/7289)) ([d3388d4](https://github.com/AztecProtocol/aztec-packages/commit/d3388d46f0c1c3a4c9f6a15abeae41f764039c10))
* Don't panic when using undefined variables in the interpreter (https://github.com/noir-lang/noir/pull/5381) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* Go to definition from `use` statement (https://github.com/noir-lang/noir/pull/5390) ([2ae17f2](https://github.com/AztecProtocol/aztec-packages/commit/2ae17f2177380244f695575c169cc591496cf3ad))
* Go to definition from aliased use (https://github.com/noir-lang/noir/pull/5396) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* ICE when using a comptime let variable in runtime code (https://github.com/noir-lang/noir/pull/5391) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* Include artifacts in noir-contracts package.json ([#7339](https://github.com/AztecProtocol/aztec-packages/issues/7339)) ([7dd87c7](https://github.com/AztecProtocol/aztec-packages/commit/7dd87c7f73dd618c5b3fd1737ba0c915559df70b))
* Only create d.ts files for contract artifacts ([#7307](https://github.com/AztecProtocol/aztec-packages/issues/7307)) ([b5e2a67](https://github.com/AztecProtocol/aztec-packages/commit/b5e2a6724084cfa576aeded78164e4dee222366f))
* Remove event selector from unencrypted log ([#7309](https://github.com/AztecProtocol/aztec-packages/issues/7309)) ([c6eb734](https://github.com/AztecProtocol/aztec-packages/commit/c6eb73429cf8ed451d13d7f2701841a3eab36d16))
* Remove panics in the interpreter when a builtin fails to type check (https://github.com/noir-lang/noir/pull/5382) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Replace expects in interpreter with errors (https://github.com/noir-lang/noir/pull/5383) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Replace std::HashMap with FxHashMap to fix frontend indeterminism (https://github.com/noir-lang/noir/pull/5385) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))
* Truncate flamegraph text to the right ([#7333](https://github.com/AztecProtocol/aztec-packages/issues/7333)) ([b7c6593](https://github.com/AztecProtocol/aztec-packages/commit/b7c6593ccd5b2d6247d65a37d57b4c2d790981bb))


### Miscellaneous

* Add bb-pilcom to rust analyzer ([#7317](https://github.com/AztecProtocol/aztec-packages/issues/7317)) ([694e68e](https://github.com/AztecProtocol/aztec-packages/commit/694e68e369494b0a3515f0f3eb5ee637f72968ee))
* ARGS_HASH constants 64 -&gt; 16 ([#7284](https://github.com/AztecProtocol/aztec-packages/issues/7284)) ([c19029a](https://github.com/AztecProtocol/aztec-packages/commit/c19029a9a2b331510856a12d51483336fa783e2f))
* **avm:** Basic stat collection ([#7283](https://github.com/AztecProtocol/aztec-packages/issues/7283)) ([adf2331](https://github.com/AztecProtocol/aztec-packages/commit/adf233153d02ea5d4dcaa20357138bd2e91dc8d8))
* **avm:** Less code in prover and verifier ([#7302](https://github.com/AztecProtocol/aztec-packages/issues/7302)) ([f401a9a](https://github.com/AztecProtocol/aztec-packages/commit/f401a9afc21af84105123393a77d87587a9a34dc))
* **avm:** Migrate lookups and permutations ([#7335](https://github.com/AztecProtocol/aztec-packages/issues/7335)) ([56fe4fe](https://github.com/AztecProtocol/aztec-packages/commit/56fe4febc179aa6b329f2a2f3a2d8675d039909f))
* **avm:** Migrate to template engine ([#7316](https://github.com/AztecProtocol/aztec-packages/issues/7316)) ([0fbfe11](https://github.com/AztecProtocol/aztec-packages/commit/0fbfe111329640b27684e6c55a45d08bb17e8e5a))
* **avm:** Re-ordering routines by opcode order ([#7298](https://github.com/AztecProtocol/aztec-packages/issues/7298)) ([4bb512d](https://github.com/AztecProtocol/aztec-packages/commit/4bb512d6935ac03338a920ec1dbc810d77e0c58c))
* **avm:** Remove shifts from full row ([#7327](https://github.com/AztecProtocol/aztec-packages/issues/7327)) ([4d641ee](https://github.com/AztecProtocol/aztec-packages/commit/4d641ee74a109030f63119ab9c0a71b5d2e2e7a6))
* Charge for more l2 gas costs ([#7157](https://github.com/AztecProtocol/aztec-packages/issues/7157)) ([3ab00c4](https://github.com/AztecProtocol/aztec-packages/commit/3ab00c4903015d3f9e1e3fa1164ab71a88737257))
* **docs:** Remove persona boxes from the landing page (https://github.com/noir-lang/noir/pull/5400) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* Nuking "new" from names ([#7273](https://github.com/AztecProtocol/aztec-packages/issues/7273)) ([b12c6cb](https://github.com/AztecProtocol/aztec-packages/commit/b12c6cb54ec6b39baed6e6bb06ecf4ace3eeede5))
* Refactor conversion between `FieldElement` and signed integers (https://github.com/noir-lang/noir/pull/5397) ([10076d9](https://github.com/AztecProtocol/aztec-packages/commit/10076d9663dcf40ac712df69e3a71a1bb54866e2))
* Replace relative paths to noir-protocol-circuits ([0124665](https://github.com/AztecProtocol/aztec-packages/commit/0124665e226e4f2d97f25fa497e14592a1d2b6e0))
* Replace relative paths to noir-protocol-circuits ([4007885](https://github.com/AztecProtocol/aztec-packages/commit/4007885b050fe37c2b798772382fe7462b220d9c))
* Update flamegraph script link ([#7329](https://github.com/AztecProtocol/aztec-packages/issues/7329)) ([0ec83ee](https://github.com/AztecProtocol/aztec-packages/commit/0ec83eefbf83d568b37011d31afefc1874880286))
* Use `mod.nr` files in stdlib (https://github.com/noir-lang/noir/pull/5379) ([ccfa69c](https://github.com/AztecProtocol/aztec-packages/commit/ccfa69c24c94d657f7d0881203ada133ed5d2ef9))

## [0.45.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.44.0...aztec-packages-v0.45.0) (2024-07-02)


### ⚠ BREAKING CHANGES

* error on too large integer value (https://github.com/noir-lang/noir/pull/5371)
* rename struct-specific TypeDefinition -> StructDefinition (https://github.com/noir-lang/noir/pull/5356)
* extend storage read oracle to receive address and block number ([#7243](https://github.com/AztecProtocol/aztec-packages/issues/7243))
* split storage access oracles ([#7237](https://github.com/AztecProtocol/aztec-packages/issues/7237))
* remove `dep::` prefix (https://github.com/noir-lang/noir/pull/4946)

### Features

* `mod.nr` entrypoint (https://github.com/noir-lang/noir/pull/5039) ([bb5cbab](https://github.com/AztecProtocol/aztec-packages/commit/bb5cbab945cfd61f6a0da79f8874a0fcdc59361a))
* `static_assert` builtin (https://github.com/noir-lang/noir/pull/5342) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Add `map`, `fold`, `reduce`, `any`, and `all` for slices (https://github.com/noir-lang/noir/pull/5331) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Add `set` and `set_unchecked` methods to `Vec` and `BoundedVec` (https://github.com/noir-lang/noir/pull/5241) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add BoundedVec::map (https://github.com/noir-lang/noir/pull/5250) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add fuzzer for Noir programs (https://github.com/noir-lang/noir/pull/5251) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add new lenses for encryted notes ([#7238](https://github.com/AztecProtocol/aztec-packages/issues/7238)) ([c07cf2c](https://github.com/AztecProtocol/aztec-packages/commit/c07cf2cf2b004dba46a3138a1f64f207b6ee537f))
* Add opcodes flamegraph and refactor gates flamegraph ([#7282](https://github.com/AztecProtocol/aztec-packages/issues/7282)) ([df3b27b](https://github.com/AztecProtocol/aztec-packages/commit/df3b27b8c603845598bf966100be3a21e8e442db))
* Add outgoing keys support to getEvents ([#7239](https://github.com/AztecProtocol/aztec-packages/issues/7239)) ([77c304e](https://github.com/AztecProtocol/aztec-packages/commit/77c304ee70de3cf47f68b45c35c776a31d61af46))
* Add support for wildcard types (https://github.com/noir-lang/noir/pull/5275) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **avm:** Calldata gadget preliminaries ([#7227](https://github.com/AztecProtocol/aztec-packages/issues/7227)) ([79e8588](https://github.com/AztecProtocol/aztec-packages/commit/79e85883c90465cf2ff6e1a2d7af0e5d4d3e111c))
* Build simple dictionary from inspecting ACIR program (https://github.com/noir-lang/noir/pull/5264) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Constant Honk proof sizes ([#6954](https://github.com/AztecProtocol/aztec-packages/issues/6954)) ([17c8d3a](https://github.com/AztecProtocol/aztec-packages/commit/17c8d3a00f3a2e500d5caa1fb438504bcd357e8a))
* Disable nargo color output if stderr is tty (https://github.com/noir-lang/noir/pull/5346) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* **docs:** Macros explainer ([#7172](https://github.com/AztecProtocol/aztec-packages/issues/7172)) ([bb2ebfc](https://github.com/AztecProtocol/aztec-packages/commit/bb2ebfce8edae9e851c7c8fb9eb1d50673f4bec6))
* Error on too large integer value (https://github.com/noir-lang/noir/pull/5371) ([bb5cbab](https://github.com/AztecProtocol/aztec-packages/commit/bb5cbab945cfd61f6a0da79f8874a0fcdc59361a))
* Example of private token transfer event ([#7242](https://github.com/AztecProtocol/aztec-packages/issues/7242)) ([99ce26f](https://github.com/AztecProtocol/aztec-packages/commit/99ce26f568b5210ac800889b28d396aa9c9d7e3e))
* **experimental:** Implement macro calls & splicing into `Expr` values (https://github.com/noir-lang/noir/pull/5203) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Extend storage read oracle to receive address and block number ([#7243](https://github.com/AztecProtocol/aztec-packages/issues/7243)) ([153b201](https://github.com/AztecProtocol/aztec-packages/commit/153b2010c5d79f308779370d240dfaa2a086ca3c))
* **frontend:** Explicit numeric generics and type kinds (https://github.com/noir-lang/noir/pull/5155) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* **frontend:** Where clause on impl (https://github.com/noir-lang/noir/pull/5320) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Function selector opcode in AVM ([#7244](https://github.com/AztecProtocol/aztec-packages/issues/7244)) ([dde47e9](https://github.com/AztecProtocol/aztec-packages/commit/dde47e927ebe5606a272a35dd8c4f4876369b244))
* Implement comptime support for `array_len` builtin (https://github.com/noir-lang/noir/pull/5272) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Implement comptime support for `as_slice` builtin (https://github.com/noir-lang/noir/pull/5276) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Insert trait impls into the program from type annotations (https://github.com/noir-lang/noir/pull/5327) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Let `should_fail_with` check that the failure reason contains the expected message (https://github.com/noir-lang/noir/pull/5319) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Make macros operate on token streams instead of AST nodes (https://github.com/noir-lang/noir/pull/5301) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Private refunds ([#7226](https://github.com/AztecProtocol/aztec-packages/issues/7226)) ([6fafff6](https://github.com/AztecProtocol/aztec-packages/commit/6fafff6e0ccda9d1e07beb5a5e8638f75b0345c2))
* Remove `dep::` prefix (https://github.com/noir-lang/noir/pull/4946) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Remove event selector in logs from public context ([#7192](https://github.com/AztecProtocol/aztec-packages/issues/7192)) ([646d45a](https://github.com/AztecProtocol/aztec-packages/commit/646d45a0cb92634909fb38d0478181c8d1d814af))
* Rename struct-specific TypeDefinition -&gt; StructDefinition (https://github.com/noir-lang/noir/pull/5356) ([bb5cbab](https://github.com/AztecProtocol/aztec-packages/commit/bb5cbab945cfd61f6a0da79f8874a0fcdc59361a))
* Run `comptime` code from annotations on a type definition (https://github.com/noir-lang/noir/pull/5256) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Split storage access oracles ([#7237](https://github.com/AztecProtocol/aztec-packages/issues/7237)) ([51f7d65](https://github.com/AztecProtocol/aztec-packages/commit/51f7d65d69eede9508f44224db554d5185298509))
* **stdlib:** Update stdlib to use explicit numeric generics (https://github.com/noir-lang/noir/pull/5306) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Store shared mutable hash ([#7169](https://github.com/AztecProtocol/aztec-packages/issues/7169)) ([868606e](https://github.com/AztecProtocol/aztec-packages/commit/868606e6c2c7b71043eabec7cc7b1eb8240fe4b3))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5242) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5340) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5347) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5377) ([bb5cbab](https://github.com/AztecProtocol/aztec-packages/commit/bb5cbab945cfd61f6a0da79f8874a0fcdc59361a))
* TXE fixes to avm opcodes and missing oracles, forced ci failure ([#7252](https://github.com/AztecProtocol/aztec-packages/issues/7252)) ([de303e2](https://github.com/AztecProtocol/aztec-packages/commit/de303e22e1a1a1115da444cabbe6155833b207b4))
* Unconstrained variants for event emission ([#7251](https://github.com/AztecProtocol/aztec-packages/issues/7251)) ([6d093e3](https://github.com/AztecProtocol/aztec-packages/commit/6d093e3cb3ed2b81eebf2a7d923f7487b95749cd))
* Unify unencrypted log emission and decoding ([#7232](https://github.com/AztecProtocol/aztec-packages/issues/7232)) ([354dba2](https://github.com/AztecProtocol/aztec-packages/commit/354dba2ae23a33419360e0983e325ce76939872d))
* Update rebuild script ([#7225](https://github.com/AztecProtocol/aztec-packages/issues/7225)) ([af59247](https://github.com/AztecProtocol/aztec-packages/commit/af592474c1d57c9d7886763d04afeb793f98efe3))
* Use runtime loops for brillig array initialization (https://github.com/noir-lang/noir/pull/5243) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Wonky rollups ([#7189](https://github.com/AztecProtocol/aztec-packages/issues/7189)) ([1de3746](https://github.com/AztecProtocol/aztec-packages/commit/1de3746bb691e2e26e9f5c7a90b4437d4433cd48))


### Bug Fixes

* Add more thorough check for whether a type is valid when passing it from constrained code to unconstrained code (https://github.com/noir-lang/noir/pull/5009) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add support for nested arrays returned by oracles (https://github.com/noir-lang/noir/pull/5132) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Address compiler warnings coming from stdlib (https://github.com/noir-lang/noir/pull/5351) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Avoid duplicating constant arrays (https://github.com/noir-lang/noir/pull/5287) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Avoid panic in type system (https://github.com/noir-lang/noir/pull/5332) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Avoid unnecessarily splitting expressions with multiplication terms with a shared term (https://github.com/noir-lang/noir/pull/5291) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Benchmark prover e2e test with proving ([#7175](https://github.com/AztecProtocol/aztec-packages/issues/7175)) ([431c14c](https://github.com/AztecProtocol/aztec-packages/commit/431c14ccca8bcbdeba51061cad6f6e01f054dd86))
* Devnet deployment issues ([#7197](https://github.com/AztecProtocol/aztec-packages/issues/7197)) ([9cf4904](https://github.com/AztecProtocol/aztec-packages/commit/9cf49048eefd1f02d22c6b4a8db100b863f39f84))
* Disable `if` optimization (https://github.com/noir-lang/noir/pull/5240) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **docs:** Check for already deployed account contracts in token bridge tutorial ([#7234](https://github.com/AztecProtocol/aztec-packages/issues/7234)) ([d9efaf7](https://github.com/AztecProtocol/aztec-packages/commit/d9efaf792b921bdabcfc24bea150130c59b3644c))
* **docs:** Historical reference library updates ([#7166](https://github.com/AztecProtocol/aztec-packages/issues/7166)) ([b3409c4](https://github.com/AztecProtocol/aztec-packages/commit/b3409c48b5d116698a67b4ceb52bd2fb4ee3c8ad))
* Don't benchmark the "prove" command as it doesn't exist anymore (https://github.com/noir-lang/noir/pull/5323) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Don't lazily elaborate functions (https://github.com/noir-lang/noir/pull/5282) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **elaborator:** Fix duplicate methods error (https://github.com/noir-lang/noir/pull/5225) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **elaborator:** Fix regression introduced by lazy-global changes (https://github.com/noir-lang/noir/pull/5223) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Error when a local function is called in a comptime context (https://github.com/noir-lang/noir/pull/5334) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Fix authwit package ([#7204](https://github.com/AztecProtocol/aztec-packages/issues/7204)) ([98ccd41](https://github.com/AztecProtocol/aztec-packages/commit/98ccd4152ae8ed3e187f4bd8e18927d70627ff04))
* Fix incorrect return type being applied to stdlib functions `modulus_be_bytes()`, `modulus_be_bits()`, etc. (https://github.com/noir-lang/noir/pull/5278) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Fix test to accomodate new max read requests ([#7286](https://github.com/AztecProtocol/aztec-packages/issues/7286)) ([a023367](https://github.com/AztecProtocol/aztec-packages/commit/a023367e20a11195e6d2b490f5c09a87997bd2ba))
* Fix tokenization of unquoted types in macros (https://github.com/noir-lang/noir/pull/5326) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Fix usage of `#[abi(tag)]` attribute with elaborator (https://github.com/noir-lang/noir/pull/5298) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Handle struct with nested arrays in oracle return values (https://github.com/noir-lang/noir/pull/5244) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Ignore calls to `Intrinsic::AsWitness` during brillig codegen (https://github.com/noir-lang/noir/pull/5350) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Implement generic functions in the interpreter (https://github.com/noir-lang/noir/pull/5330) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* **nargo_fmt:** Account for spaces before the generic list of a function (https://github.com/noir-lang/noir/pull/5303) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Replace panic in monomorphization with an error (https://github.com/noir-lang/noir/pull/5305) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Reran pil-&gt;cpp codegen & encode_and_encrypt_event_with_randomness fix ([#7247](https://github.com/AztecProtocol/aztec-packages/issues/7247)) ([fa15a45](https://github.com/AztecProtocol/aztec-packages/commit/fa15a450408181ffc50946ee56c4ae0fd8c5a61f))
* Runtime brillig bigint id assignment (https://github.com/noir-lang/noir/pull/5369) ([bb5cbab](https://github.com/AztecProtocol/aztec-packages/commit/bb5cbab945cfd61f6a0da79f8874a0fcdc59361a))
* Skip emission of brillig calls which will never be executed (https://github.com/noir-lang/noir/pull/5314) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* TS LSP being slow ([#7181](https://github.com/AztecProtocol/aztec-packages/issues/7181)) ([e934e87](https://github.com/AztecProtocol/aztec-packages/commit/e934e872d5a2fb3ca46646436de25777a33c4737))
* Update `in_contract` flag before handling function metadata in elaborator (https://github.com/noir-lang/noir/pull/5292) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Use proper serialization in `AbiValue` (https://github.com/noir-lang/noir/pull/5270) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))


### Miscellaneous

* `static_assert` error message fix and split into is-dynamic and is-false (https://github.com/noir-lang/noir/pull/5353) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Add back Pedersen blackbox functions (revert PR 5221) (https://github.com/noir-lang/noir/pull/5318) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add log_hash as input in log emission in private context ([#7249](https://github.com/AztecProtocol/aztec-packages/issues/7249)) ([8b3dfe9](https://github.com/AztecProtocol/aztec-packages/commit/8b3dfe9dabc19e24f759b1a6c8ed14e2d9874149))
* Add no predicate to poseidon2 (https://github.com/noir-lang/noir/pull/5252) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add no-predicate to hash implementations (https://github.com/noir-lang/noir/pull/5253) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Add property tests for ABI encoding (https://github.com/noir-lang/noir/pull/5216) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Address TODO in `compat.nr` (https://github.com/noir-lang/noir/pull/5339) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* **avm-transpiler:** Better error messages ([#7217](https://github.com/AztecProtocol/aztec-packages/issues/7217)) ([27051ad](https://github.com/AztecProtocol/aztec-packages/commit/27051ad988b98d1b4a60064b2492cd987b1df7ac))
* **avm:** Remove trailing minus zero in codegen ([#7185](https://github.com/AztecProtocol/aztec-packages/issues/7185)) ([f3c8166](https://github.com/AztecProtocol/aztec-packages/commit/f3c81661688cc04b64a389d8fd72484ca8580a05))
* Avoid building contracts when producing gates report ([#7136](https://github.com/AztecProtocol/aztec-packages/issues/7136)) ([25507e6](https://github.com/AztecProtocol/aztec-packages/commit/25507e63e6a629a8a16ad47434141a95bbb0e102))
* Bump `bb` to 0.43.0 (https://github.com/noir-lang/noir/pull/5321) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Bundle SSA Evaluator Options (https://github.com/noir-lang/noir/pull/5317) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **ci:** Trigger a noir sync every morning at 8am ([#7280](https://github.com/AztecProtocol/aztec-packages/issues/7280)) ([412c016](https://github.com/AztecProtocol/aztec-packages/commit/412c0160073d642056855c649d9c59ad1ae100f3))
* Copy across typo PR script from aztec-packages (https://github.com/noir-lang/noir/pull/5235) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Create separate crate just for noir artifacts (https://github.com/noir-lang/noir/pull/5162) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* **docs:** Fixing trailing slash issue (https://github.com/noir-lang/noir/pull/5233) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Fix examples (https://github.com/noir-lang/noir/pull/5357) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Fix migration notes ([#7279](https://github.com/AztecProtocol/aztec-packages/issues/7279)) ([51d93eb](https://github.com/AztecProtocol/aztec-packages/commit/51d93eb3020ea8f8902c814d0f8dd74640535a90))
* Fix negative tests in AVM circuit for context input lookups ([#7261](https://github.com/AztecProtocol/aztec-packages/issues/7261)) ([ad2f654](https://github.com/AztecProtocol/aztec-packages/commit/ad2f654eb2589dff118c3e104c4f91825ee7f739))
* Fixing all relative paths (https://github.com/noir-lang/noir/pull/5220) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Generate PIL constants from via constants gen ([#7258](https://github.com/AztecProtocol/aztec-packages/issues/7258)) ([244ef7e](https://github.com/AztecProtocol/aztec-packages/commit/244ef7e5a6871443444df88c28a1c2a7430d6db1))
* Gets rid of unencrypted emit in private_context ([#7236](https://github.com/AztecProtocol/aztec-packages/issues/7236)) ([3e6d88e](https://github.com/AztecProtocol/aztec-packages/commit/3e6d88e53c3e4c0777e152393d5310b5607baa0a))
* Improve authwit comments/docs ([#7180](https://github.com/AztecProtocol/aztec-packages/issues/7180)) ([051ab9e](https://github.com/AztecProtocol/aztec-packages/commit/051ab9e3d4eda170c775f683762c81b6876c61ca))
* Misc cleanup in simulator ([#7203](https://github.com/AztecProtocol/aztec-packages/issues/7203)) ([eb00830](https://github.com/AztecProtocol/aztec-packages/commit/eb00830fce7afae60447bec5383349d7f490e4d7))
* Optimize the elaborator (https://github.com/noir-lang/noir/pull/5230) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Parse macros (https://github.com/noir-lang/noir/pull/5229) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Pedersen commitment in Noir (https://github.com/noir-lang/noir/pull/5221) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Pedersen hash in Noir (https://github.com/noir-lang/noir/pull/5217) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Private tail circuits ([#7148](https://github.com/AztecProtocol/aztec-packages/issues/7148)) ([9e67e7d](https://github.com/AztecProtocol/aztec-packages/commit/9e67e7d8c47004763df8fdbee9f278a75db193a0))
* Pull out change to expression splitting from sync PR ([#7215](https://github.com/AztecProtocol/aztec-packages/issues/7215)) ([b4f50a5](https://github.com/AztecProtocol/aztec-packages/commit/b4f50a5bf03babd83c1f467d2792b50e334bf5a7))
* Pull out foreign call nested array changes ([#7216](https://github.com/AztecProtocol/aztec-packages/issues/7216)) ([1faaaf5](https://github.com/AztecProtocol/aztec-packages/commit/1faaaf53bb7461d1806a79822ded1ecefe01b59b))
* Pull out noir-lang/noir[#5120](https://github.com/AztecProtocol/aztec-packages/issues/5120) ([#7205](https://github.com/AztecProtocol/aztec-packages/issues/7205)) ([c5dc094](https://github.com/AztecProtocol/aztec-packages/commit/c5dc0946f4d300df5c6a70026e102de8e69f020b))
* Pull out pedersen generator builtin from sync PR ([#7210](https://github.com/AztecProtocol/aztec-packages/issues/7210)) ([412f02e](https://github.com/AztecProtocol/aztec-packages/commit/412f02eb05321db1bbc60902e31ab50d743541d6))
* Pull out SSA changes from sync PR ([#7209](https://github.com/AztecProtocol/aztec-packages/issues/7209)) ([141e137](https://github.com/AztecProtocol/aztec-packages/commit/141e137b06b7f9aa324705e716aac2157312aacb))
* Push code related to ABI gen into `noirc_driver` (https://github.com/noir-lang/noir/pull/5218) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Redo typo PR by dropbigfish (https://github.com/noir-lang/noir/pull/5234) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Reduce note and nullifier constants ([#7255](https://github.com/AztecProtocol/aztec-packages/issues/7255)) ([4637304](https://github.com/AztecProtocol/aztec-packages/commit/463730458de2397d66ec90fedfeee61700c426a4))
* Refactor test case generation in build.rs (https://github.com/noir-lang/noir/pull/5280) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Refactor to use `mod.nr` support ([#7259](https://github.com/AztecProtocol/aztec-packages/issues/7259)) ([cda45db](https://github.com/AztecProtocol/aztec-packages/commit/cda45dba624e519ace66fd2e75b85d29a0f6eb9f))
* Release Noir(0.31.0) (https://github.com/noir-lang/noir/pull/5166) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Remove `is_unconstrained_fn` field from elaborator (https://github.com/noir-lang/noir/pull/5335) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Remove 4738 ref ([#7254](https://github.com/AztecProtocol/aztec-packages/issues/7254)) ([97d997c](https://github.com/AztecProtocol/aztec-packages/commit/97d997c851fe319f864e2826903ffa7d8677d701))
* Remove a log file ([#7201](https://github.com/AztecProtocol/aztec-packages/issues/7201)) ([83bb218](https://github.com/AztecProtocol/aztec-packages/commit/83bb2180cfacd33298b5b3346140453566f3cf8e))
* Remove commented code ([#7231](https://github.com/AztecProtocol/aztec-packages/issues/7231)) ([2740d60](https://github.com/AztecProtocol/aztec-packages/commit/2740d600c0d4a18ce90df24e334e572a80233832))
* Remove panic for unimplemented trait dispatch (https://github.com/noir-lang/noir/pull/5329) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Replace `is_bn254` implementation to not rely on truncation of literals (https://github.com/noir-lang/noir/pull/5247) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Replace `regression_5202` with more manageably sized program (https://github.com/noir-lang/noir/pull/5345) ([eb9e9f6](https://github.com/AztecProtocol/aztec-packages/commit/eb9e9f6f2b3952760822faaacb7e851e936e0800))
* Replace cached `in_contract` with `in_contract()` method (https://github.com/noir-lang/noir/pull/5324) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Replace logical operators with bitwise in `DebugToString` (https://github.com/noir-lang/noir/pull/5236) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Replace relative paths to noir-protocol-circuits ([e83b07b](https://github.com/AztecProtocol/aztec-packages/commit/e83b07bf813001c144c61c61174c6adf816cc991))
* Replace relative paths to noir-protocol-circuits ([eca8587](https://github.com/AztecProtocol/aztec-packages/commit/eca858775f8f84455cc0a20d9f9fb828cf342b68))
* Replace relative paths to noir-protocol-circuits ([b9ddf43](https://github.com/AztecProtocol/aztec-packages/commit/b9ddf43faa0184692917d543e39507192b2ac64b))
* Replace relative paths to noir-protocol-circuits ([6f817e8](https://github.com/AztecProtocol/aztec-packages/commit/6f817e86b61aea78d9f4132ecf4c3ed2f96b4e5c))
* Replace relative paths to noir-protocol-circuits ([f9bf0a4](https://github.com/AztecProtocol/aztec-packages/commit/f9bf0a4d8ea7591abbb092f1a44b3ff6bcab7af7))
* Replicate noir-lang/noir[#4946](https://github.com/AztecProtocol/aztec-packages/issues/4946) ([#7202](https://github.com/AztecProtocol/aztec-packages/issues/7202)) ([b5c07d8](https://github.com/AztecProtocol/aztec-packages/commit/b5c07d8507c783ebb440cb32a897416627b71ec1))
* Simplify compilation flow to write to file immediately (https://github.com/noir-lang/noir/pull/5265) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Split off fuzzer, abi changes and `noirc_artifacts` from sync ([#7208](https://github.com/AztecProtocol/aztec-packages/issues/7208)) ([255d752](https://github.com/AztecProtocol/aztec-packages/commit/255d752594dd5372c75934a784e995ae4899e431))
* Thread generics through ACIR/brillig gen (https://github.com/noir-lang/noir/pull/5120) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))
* Use `push_err` more in elaborator (https://github.com/noir-lang/noir/pull/5336) ([f2abb4e](https://github.com/AztecProtocol/aztec-packages/commit/f2abb4e9deb05437666db9c27cd0d49c2ec9ac3d))
* Use options.limit as upper limit for note-getter loop ([#7253](https://github.com/AztecProtocol/aztec-packages/issues/7253)) ([8ff669b](https://github.com/AztecProtocol/aztec-packages/commit/8ff669b63f302447e099dc52dea248c2ca914043))
* Use prefix op_ for every instruction in avm_trace.hpp ([#7214](https://github.com/AztecProtocol/aztec-packages/issues/7214)) ([7ed7558](https://github.com/AztecProtocol/aztec-packages/commit/7ed75586cd5deb8aff3730a80cb29c642495bbff))
* Use the elaborator by default (https://github.com/noir-lang/noir/pull/5246) ([ed815a3](https://github.com/AztecProtocol/aztec-packages/commit/ed815a3713fc311056a8bd0a616945f12d9be2a8))

## [0.44.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.43.0...aztec-packages-v0.44.0) (2024-06-26)


### ⚠ BREAKING CHANGES

* make note_getter return BoundedVec instead of an Option array ([#7050](https://github.com/AztecProtocol/aztec-packages/issues/7050))
* TXE ([#6985](https://github.com/AztecProtocol/aztec-packages/issues/6985))

### Features

* Add macro impls for events ([#7081](https://github.com/AztecProtocol/aztec-packages/issues/7081)) ([c13dd9f](https://github.com/AztecProtocol/aztec-packages/commit/c13dd9fbc9f5390dc4613e5739d3ec3120430087))
* Add OpenTelemetry to node ([#7102](https://github.com/AztecProtocol/aztec-packages/issues/7102)) ([6bf2b72](https://github.com/AztecProtocol/aztec-packages/commit/6bf2b7269fddb5bd7fe4c567710146b4969d2845))
* Added prove_output_all flow for honk ([#6869](https://github.com/AztecProtocol/aztec-packages/issues/6869)) ([7bd7c66](https://github.com/AztecProtocol/aztec-packages/commit/7bd7c66de6adb1f703509e9a8e8911ff0ec2025c))
* **avm:** Add ECC ops to avm_proving_test ([#7058](https://github.com/AztecProtocol/aztec-packages/issues/7058)) ([7f62a90](https://github.com/AztecProtocol/aztec-packages/commit/7f62a901c848020058a58f6e772d495566416e0b))
* **avm:** Cpp msm changes ([#7056](https://github.com/AztecProtocol/aztec-packages/issues/7056)) ([f9c8f20](https://github.com/AztecProtocol/aztec-packages/commit/f9c8f20f1bd6af0003960b19f61b737e3bad5f1e))
* **avm:** Include bb-pilcom in monorepo ([#7098](https://github.com/AztecProtocol/aztec-packages/issues/7098)) ([0442158](https://github.com/AztecProtocol/aztec-packages/commit/044215814ac75df37c1d3ce3a6852b4ac49e6065))
* Constrain event encryption and unify note and event emit api ([#7171](https://github.com/AztecProtocol/aztec-packages/issues/7171)) ([5c3772f](https://github.com/AztecProtocol/aztec-packages/commit/5c3772f09e812a05882039ad888089c238f14001)), closes [#7160](https://github.com/AztecProtocol/aztec-packages/issues/7160)
* Conventional lookups using log-deriv ([#7020](https://github.com/AztecProtocol/aztec-packages/issues/7020)) ([6f1212f](https://github.com/AztecProtocol/aztec-packages/commit/6f1212ff0d6bb7a326e571da2d49cfac75a8e5de))
* Devnet deployments ([#7024](https://github.com/AztecProtocol/aztec-packages/issues/7024)) ([fa70876](https://github.com/AztecProtocol/aztec-packages/commit/fa70876a17b981e6ffa4bece390186b1231ba4fe))
* Do not discard logs on revert since the kernel has pruned revertible logs. ([#7076](https://github.com/AztecProtocol/aztec-packages/issues/7076)) ([366fb21](https://github.com/AztecProtocol/aztec-packages/commit/366fb210e4cdb63fb872567506b6c12e57a0508b)), closes [#4712](https://github.com/AztecProtocol/aztec-packages/issues/4712)
* **docs:** Publish PDF of protocol specs + remove links to pages in item lists in protocol specs ([#6684](https://github.com/AztecProtocol/aztec-packages/issues/6684)) ([367e3cf](https://github.com/AztecProtocol/aztec-packages/commit/367e3cf55f0281658c3da3ccd2b6cc87b707cb92))
* Enable merge recursive verifier in Goblin recursive verifier ([#7182](https://github.com/AztecProtocol/aztec-packages/issues/7182)) ([9b4f56c](https://github.com/AztecProtocol/aztec-packages/commit/9b4f56c89fb17eb3497987e6f9198441a4e89c56))
* Flamegraph helper script ([#7077](https://github.com/AztecProtocol/aztec-packages/issues/7077)) ([8630c8f](https://github.com/AztecProtocol/aztec-packages/commit/8630c8f2018c711b84997dc4727d5123ad616107))
* Full test skips public simulation ([#7186](https://github.com/AztecProtocol/aztec-packages/issues/7186)) ([4c1997f](https://github.com/AztecProtocol/aztec-packages/commit/4c1997fdc989fc12c0fb50f690558fccab8e0cd1))
* Make note_getter return BoundedVec instead of an Option array ([#7050](https://github.com/AztecProtocol/aztec-packages/issues/7050)) ([f9ac0fc](https://github.com/AztecProtocol/aztec-packages/commit/f9ac0fca40a9d7273ec2adddbfbe961f86595f56))
* **p2p:** More comprehensive peer management, dial retries, persistence fix ([#6953](https://github.com/AztecProtocol/aztec-packages/issues/6953)) ([cdd1cbd](https://github.com/AztecProtocol/aztec-packages/commit/cdd1cbd2ff5a8aceb52bde44c24462fed0808890))
* Private authwit with static call ([#7073](https://github.com/AztecProtocol/aztec-packages/issues/7073)) ([9c52d47](https://github.com/AztecProtocol/aztec-packages/commit/9c52d474146177b83f78ab9f12d9d41a03678838))
* Several updates in SMT verification module ([#7105](https://github.com/AztecProtocol/aztec-packages/issues/7105)) ([41b21f1](https://github.com/AztecProtocol/aztec-packages/commit/41b21f179ead617203c6d77b080e4f8b0065e06c))
* Shplonk revival in ECCVM ([#7164](https://github.com/AztecProtocol/aztec-packages/issues/7164)) ([34eb5a0](https://github.com/AztecProtocol/aztec-packages/commit/34eb5a01d34e5ff5d1414fff53ca0623d83bde5d))
* Throwing errors in `BufferReader` when out of bounds ([#7149](https://github.com/AztecProtocol/aztec-packages/issues/7149)) ([bf4a986](https://github.com/AztecProtocol/aztec-packages/commit/bf4a986cb07c923b606944ff48960b2df2ea7c6e))
* Track spans ([#7129](https://github.com/AztecProtocol/aztec-packages/issues/7129)) ([924c3f8](https://github.com/AztecProtocol/aztec-packages/commit/924c3f8809b30d16e81eed5e467aa79ee7074f77))
* TXE ([#6985](https://github.com/AztecProtocol/aztec-packages/issues/6985)) ([109624f](https://github.com/AztecProtocol/aztec-packages/commit/109624f127dc8da6d9d963b3af9250237be0d4e4))
* TXE 2: Electric boogaloo ([#7154](https://github.com/AztecProtocol/aztec-packages/issues/7154)) ([bb38246](https://github.com/AztecProtocol/aztec-packages/commit/bb38246d09ee0e5430d31ade32bc28da688b4a84))


### Bug Fixes

* **avm:** Fix unencryptedlog c++ deser ([#7194](https://github.com/AztecProtocol/aztec-packages/issues/7194)) ([89a99af](https://github.com/AztecProtocol/aztec-packages/commit/89a99af4ff2ea79c276ff379a3cdd1b8cae18d15))
* **avm:** Re-enable ext call test ([#7147](https://github.com/AztecProtocol/aztec-packages/issues/7147)) ([33ccf1b](https://github.com/AztecProtocol/aztec-packages/commit/33ccf1b61260868e6bb027b7838ef530717bff01))
* **avm:** Reenable tag error sload ([#7153](https://github.com/AztecProtocol/aztec-packages/issues/7153)) ([fd92d46](https://github.com/AztecProtocol/aztec-packages/commit/fd92d467eee51638b896852b789c8fae17e0689c))
* **avm:** Update codegen ([#7178](https://github.com/AztecProtocol/aztec-packages/issues/7178)) ([1d29708](https://github.com/AztecProtocol/aztec-packages/commit/1d29708bb6136184c27c1dc4f632b277cf3e4e64))
* Bug fixing bench prover test ([#7135](https://github.com/AztecProtocol/aztec-packages/issues/7135)) ([13678be](https://github.com/AztecProtocol/aztec-packages/commit/13678be4e1c76d20a804a04b0fa82f68aeca38ae)), closes [#7080](https://github.com/AztecProtocol/aztec-packages/issues/7080)
* **ci:** Don't run npm_deploy l1-contracts ([#7187](https://github.com/AztecProtocol/aztec-packages/issues/7187)) ([80d26d8](https://github.com/AztecProtocol/aztec-packages/commit/80d26d883154b81f0f92664d61c907bcfe46509b))
* **ci:** Move osxcross from build image ([#7151](https://github.com/AztecProtocol/aztec-packages/issues/7151)) ([7746363](https://github.com/AztecProtocol/aztec-packages/commit/77463638133113de074a6030954a2be9954638e1))
* Enable log filtering with the DEBUG variable ([#7150](https://github.com/AztecProtocol/aztec-packages/issues/7150)) ([33798b6](https://github.com/AztecProtocol/aztec-packages/commit/33798b6b8d32b88ce2719b4c6dd7ac9024a88c7f))
* Export event selector and replace function selector with event selector where appropriate ([#7095](https://github.com/AztecProtocol/aztec-packages/issues/7095)) ([fcc15fa](https://github.com/AztecProtocol/aztec-packages/commit/fcc15faffac98ab844dbad51e949c24114a8bcf0)), closes [#7089](https://github.com/AztecProtocol/aztec-packages/issues/7089)
* False decryption fix ([#7066](https://github.com/AztecProtocol/aztec-packages/issues/7066)) ([48d9df4](https://github.com/AztecProtocol/aztec-packages/commit/48d9df4ff227c08a6e66f21c0286bc6349151671))
* Fix bug for a unit test in full proving mode repated to MSM ([#7104](https://github.com/AztecProtocol/aztec-packages/issues/7104)) ([e37809b](https://github.com/AztecProtocol/aztec-packages/commit/e37809bdcdcf76f89f68403ee75aaf6d32c79a94))


### Miscellaneous

* `destroy_note(...)` optimization ([#7103](https://github.com/AztecProtocol/aztec-packages/issues/7103)) ([0770011](https://github.com/AztecProtocol/aztec-packages/commit/0770011139d698ffa466605f0c10f0f5fe965da3))
* Add avm team as codeowners to more repo files ([#7196](https://github.com/AztecProtocol/aztec-packages/issues/7196)) ([9be0ad6](https://github.com/AztecProtocol/aztec-packages/commit/9be0ad6b41a69c35ad9737d60da7a16300b87642))
* **avm:** Remove avm prefix from pil and executor ([#7099](https://github.com/AztecProtocol/aztec-packages/issues/7099)) ([b502fcd](https://github.com/AztecProtocol/aztec-packages/commit/b502fcd500dcb10945b29d00f86e290bec63cce3))
* **avm:** Renamings and comments ([#7128](https://github.com/AztecProtocol/aztec-packages/issues/7128)) ([ed2f98e](https://github.com/AztecProtocol/aztec-packages/commit/ed2f98ee9d7f540fbdfae09a0117bf22aaf6ebc7))
* **avm:** Separate some fixed tables ([#7163](https://github.com/AztecProtocol/aztec-packages/issues/7163)) ([1d4a9a2](https://github.com/AztecProtocol/aztec-packages/commit/1d4a9a29ad37543aa0058bd43fa533f19a91e019))
* **ci:** Add new e2e base target ([#7179](https://github.com/AztecProtocol/aztec-packages/issues/7179)) ([26fc599](https://github.com/AztecProtocol/aztec-packages/commit/26fc59965b4bd95ba8b06ec3139e3fc44e5c3495))
* Create workflow for full AVM tests ([#7051](https://github.com/AztecProtocol/aztec-packages/issues/7051)) ([a0b9c4b](https://github.com/AztecProtocol/aztec-packages/commit/a0b9c4b4383f448549c04567cd9c9264ce4240dc)), closes [#6643](https://github.com/AztecProtocol/aztec-packages/issues/6643)
* **docs:** Fix migration notes ([#7195](https://github.com/AztecProtocol/aztec-packages/issues/7195)) ([88efda0](https://github.com/AztecProtocol/aztec-packages/commit/88efda0b40b3a90c4a3b09badaaec8678edd0da7))
* **docs:** Moving tutorials and quick starts around, spinning off codespaces page ([#6777](https://github.com/AztecProtocol/aztec-packages/issues/6777)) ([1542fa6](https://github.com/AztecProtocol/aztec-packages/commit/1542fa699e32ef88c1a8b9ba3d1f74315a5bb63e))
* Fix migration notes ([#7133](https://github.com/AztecProtocol/aztec-packages/issues/7133)) ([14917d3](https://github.com/AztecProtocol/aztec-packages/commit/14917d3d8c5da1f6038597d9129c6051a2fd94de))
* Fix noir-projects dockerfile for CircleCI ([#7093](https://github.com/AztecProtocol/aztec-packages/issues/7093)) ([52ce25d](https://github.com/AztecProtocol/aztec-packages/commit/52ce25d1abcc5a8cff7ec360acf23806cb317b57))
* Increase the timeout of the runner for full AVM workflow to 70 minutes ([#7183](https://github.com/AztecProtocol/aztec-packages/issues/7183)) ([9aabc32](https://github.com/AztecProtocol/aztec-packages/commit/9aabc324040e84be9d61644d33948c818553f422))
* Indirects and read/write slices ([#7082](https://github.com/AztecProtocol/aztec-packages/issues/7082)) ([d5e80ee](https://github.com/AztecProtocol/aztec-packages/commit/d5e80ee9b6298f7edf39b99ff51ae7cc26b8cbd8))
* Minor naming cleanup ([#7144](https://github.com/AztecProtocol/aztec-packages/issues/7144)) ([20e2492](https://github.com/AztecProtocol/aztec-packages/commit/20e249278d3c161e72901d61bb259cb1d297e44c))
* Note hashes cleanup + optimization ([#7132](https://github.com/AztecProtocol/aztec-packages/issues/7132)) ([edd6d3f](https://github.com/AztecProtocol/aztec-packages/commit/edd6d3ffdddd6aa29c6638e5fab87aadf5d89d09))
* Note hashing gate optimizations ([#7130](https://github.com/AztecProtocol/aztec-packages/issues/7130)) ([81a2580](https://github.com/AztecProtocol/aztec-packages/commit/81a258003f93335eba2d9d07ffc4f0feca935956))
* **powdr:** Update to latest and add logging ([#7152](https://github.com/AztecProtocol/aztec-packages/issues/7152)) ([f500f2e](https://github.com/AztecProtocol/aztec-packages/commit/f500f2eeca5abc731ad942e956b8a6b56c6922f5))
* Reads the return data ([#6669](https://github.com/AztecProtocol/aztec-packages/issues/6669)) ([ef85542](https://github.com/AztecProtocol/aztec-packages/commit/ef8554268c175e6349474883c6072e3979fe45c0))
* Refactor AVM simulator's side-effect tracing ([#7091](https://github.com/AztecProtocol/aztec-packages/issues/7091)) ([9495413](https://github.com/AztecProtocol/aztec-packages/commit/94954131ea61bb6b58efe4e9f8b4e1f489f53fa9))
* Remove stray files ([#7158](https://github.com/AztecProtocol/aztec-packages/issues/7158)) ([29398de](https://github.com/AztecProtocol/aztec-packages/commit/29398de7625f70e0efdcc8f9acdda656337c56db))
* Remove unneeded public input folding ([#7094](https://github.com/AztecProtocol/aztec-packages/issues/7094)) ([c30dc38](https://github.com/AztecProtocol/aztec-packages/commit/c30dc3856cec038d8c53af34cf82e08b0cb456aa))
* Replace relative paths to noir-protocol-circuits ([f7e4392](https://github.com/AztecProtocol/aztec-packages/commit/f7e439257380b5061e561870addb5be68d753cfc))
* Replace relative paths to noir-protocol-circuits ([886f7b1](https://github.com/AztecProtocol/aztec-packages/commit/886f7b1cb062c7bf2322122c7881f99b0cd58313))
* Replace relative paths to noir-protocol-circuits ([b1081f8](https://github.com/AztecProtocol/aztec-packages/commit/b1081f80d1891f70295a850dc1995a2e63d880bf))
* Replace relative paths to noir-protocol-circuits ([c0989eb](https://github.com/AztecProtocol/aztec-packages/commit/c0989eb159ed9fcb2eaacf57d2c209cc071b5669))
* Replace relative paths to noir-protocol-circuits ([525bbe7](https://github.com/AztecProtocol/aztec-packages/commit/525bbe750a8154d98355694ad9e1f3e33275ab5b))
* Replace relative paths to noir-protocol-circuits ([67bcd82](https://github.com/AztecProtocol/aztec-packages/commit/67bcd8279f0d8921437a6f37e6ec3f25a69471a5))
* Take the PCS out of Zeromorph and refactor tests ([#7078](https://github.com/AztecProtocol/aztec-packages/issues/7078)) ([e192678](https://github.com/AztecProtocol/aztec-packages/commit/e19267872bae6fa2df258a1e363f1ba2f2f47922))
* Track avm proving time ([#7084](https://github.com/AztecProtocol/aztec-packages/issues/7084)) ([59df722](https://github.com/AztecProtocol/aztec-packages/commit/59df72249a8db2b6d1cf0c7908836041c84a54c1))
* Ultra flavor cleanup ([#7070](https://github.com/AztecProtocol/aztec-packages/issues/7070)) ([77761c6](https://github.com/AztecProtocol/aztec-packages/commit/77761c670f2d516ab486de0f7bde036ff00ebd99))

## [0.43.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.42.0...aztec-packages-v0.43.0) (2024-06-18)


### ⚠ BREAKING CHANGES

* remove `distinct` keyword (https://github.com/noir-lang/noir/pull/5219)
* remove `param_witnesses` and `return_witnesses` from ABI (https://github.com/noir-lang/noir/pull/5154)
* add session id to foreign call RPC requests (https://github.com/noir-lang/noir/pull/5205)
* make options.limit a compile-time constant ([#7027](https://github.com/AztecProtocol/aztec-packages/issues/7027))
* restrict noir word size to u32 (https://github.com/noir-lang/noir/pull/5180)
* separate proving from `noir_js` (https://github.com/noir-lang/noir/pull/5072)

### Features

* `pxe.addNullifiedNote(...)` ([#6948](https://github.com/AztecProtocol/aztec-packages/issues/6948)) ([42a4b1c](https://github.com/AztecProtocol/aztec-packages/commit/42a4b1c6f000886b8b63e2fd6b0b218a29cb820c))
* Add data dir to pxe container ([#6874](https://github.com/AztecProtocol/aztec-packages/issues/6874)) ([504fea2](https://github.com/AztecProtocol/aztec-packages/commit/504fea2e330ad66ec269ddff581b7448c008f9ca))
* Add ENFORCE_FEES sequencer config ([#6949](https://github.com/AztecProtocol/aztec-packages/issues/6949)) ([46dcb98](https://github.com/AztecProtocol/aztec-packages/commit/46dcb985e98ca26ee2dd3d2ec98976f1d8f27ba7))
* Add gate profiler for noir circuits ([#7004](https://github.com/AztecProtocol/aztec-packages/issues/7004)) ([a2f6876](https://github.com/AztecProtocol/aztec-packages/commit/a2f687687559d15fde52abce54838f6e144a0aa4))
* Add node to devnet ([#6898](https://github.com/AztecProtocol/aztec-packages/issues/6898)) ([acc534c](https://github.com/AztecProtocol/aztec-packages/commit/acc534c339ad05d548f8f287e4bd6051201cb1f6))
* Add session id to foreign call RPC requests (https://github.com/noir-lang/noir/pull/5205) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Add standard form function to biggroup ([#6899](https://github.com/AztecProtocol/aztec-packages/issues/6899)) ([3e44be5](https://github.com/AztecProtocol/aztec-packages/commit/3e44be538e5c7f0e7269c1e5c0820f7bc6e83734))
* Add utils::collapse ([#7016](https://github.com/AztecProtocol/aztec-packages/issues/7016)) ([2d19ad9](https://github.com/AztecProtocol/aztec-packages/commit/2d19ad9af6130aeaf6621f239c4119c3126dd7c6))
* Affine_element read/write with proper handling of point at infinity ([#6963](https://github.com/AztecProtocol/aztec-packages/issues/6963)) ([c6cbe39](https://github.com/AztecProtocol/aztec-packages/commit/c6cbe39eed23dc845aef898e937e99de43f71675))
* Auth registry ([#7035](https://github.com/AztecProtocol/aztec-packages/issues/7035)) ([cea0b3b](https://github.com/AztecProtocol/aztec-packages/commit/cea0b3b29c2f7c37eb07c226a06534f92518cea6))
* Auto-gen p2p private key ([#6910](https://github.com/AztecProtocol/aztec-packages/issues/6910)) ([0fc9677](https://github.com/AztecProtocol/aztec-packages/commit/0fc9677b2db0b126e6b604b387735a29e295ff05))
* Avm e2e nested call + alu fix + cast fix ([#6974](https://github.com/AztecProtocol/aztec-packages/issues/6974)) ([b150b61](https://github.com/AztecProtocol/aztec-packages/commit/b150b610153e380a93240914c95887f88b56fa94))
* **avm-simulator:** Msm blackbox ([#7048](https://github.com/AztecProtocol/aztec-packages/issues/7048)) ([0ce27e0](https://github.com/AztecProtocol/aztec-packages/commit/0ce27e05c4c099167d0d98300f6d73ced22639ad))
* **avm:** Add get_contract_instance ([#6871](https://github.com/AztecProtocol/aztec-packages/issues/6871)) ([b3a86bf](https://github.com/AztecProtocol/aztec-packages/commit/b3a86bf72343d1060ce58a11f139e05ba2a75754))
* **avm:** Deserialise execution hints in bb main ([#6848](https://github.com/AztecProtocol/aztec-packages/issues/6848)) ([d3be85f](https://github.com/AztecProtocol/aztec-packages/commit/d3be85f57c34aa88e732ea115239f3bed1e7aa16))
* **avm:** E2e proving of storage ([#6967](https://github.com/AztecProtocol/aztec-packages/issues/6967)) ([6a7be0c](https://github.com/AztecProtocol/aztec-packages/commit/6a7be0c434934175bb6da1f3525c025b3f743824))
* **avm:** E2e send l1 msg ([#6880](https://github.com/AztecProtocol/aztec-packages/issues/6880)) ([deb972d](https://github.com/AztecProtocol/aztec-packages/commit/deb972d3f13a92d34a6f91074b072fb66d247f64))
* **avm:** Gas remaining range check and handling of out of gas ([#6944](https://github.com/AztecProtocol/aztec-packages/issues/6944)) ([5647571](https://github.com/AztecProtocol/aztec-packages/commit/56475716e05973e6b493de427f32eee71c0f8f6a)), closes [#6902](https://github.com/AztecProtocol/aztec-packages/issues/6902)
* **avm:** Get contract instance now works e2e with avm proving ([#6911](https://github.com/AztecProtocol/aztec-packages/issues/6911)) ([662187d](https://github.com/AztecProtocol/aztec-packages/commit/662187d1d6960b734a71aaf365e7f20d471dc4c9))
* **avm:** Indirect support for kernel output opcodes ([#6962](https://github.com/AztecProtocol/aztec-packages/issues/6962)) ([f330bff](https://github.com/AztecProtocol/aztec-packages/commit/f330bffa80b6da5f037cea3cf469ef1c7b6d9d03))
* **avm:** Indirect support for kernel read opcodes ([#6940](https://github.com/AztecProtocol/aztec-packages/issues/6940)) ([ccc474d](https://github.com/AztecProtocol/aztec-packages/commit/ccc474d9d0cd10faf857bc1ec6571dc25306a531))
* **avm:** L2gasleft and dagasleft opcodes ([#6884](https://github.com/AztecProtocol/aztec-packages/issues/6884)) ([fbab612](https://github.com/AztecProtocol/aztec-packages/commit/fbab612b17dfe0e95ead1a592b7bc9fe6ca5415d))
* **avm:** Nullifier non exist ([#6877](https://github.com/AztecProtocol/aztec-packages/issues/6877)) ([05697f2](https://github.com/AztecProtocol/aztec-packages/commit/05697f289d3b97def74f45cd839a58a8a077c3fa))
* **avm:** Plumb externalcall hints ([#6890](https://github.com/AztecProtocol/aztec-packages/issues/6890)) ([3a97f08](https://github.com/AztecProtocol/aztec-packages/commit/3a97f08c457472bd701200adfa45d61554fd3867))
* **avm:** Plumb start side effect counter in circuit ([#7007](https://github.com/AztecProtocol/aztec-packages/issues/7007)) ([fa8f12f](https://github.com/AztecProtocol/aztec-packages/commit/fa8f12f93a8d94604a4382de444501fac310dbb8))
* **avm:** Revert opcode ([#6909](https://github.com/AztecProtocol/aztec-packages/issues/6909)) ([620d3da](https://github.com/AztecProtocol/aztec-packages/commit/620d3dacc853c71e808ef58001eb4c8584fa59d9))
* **avm:** Support preserving BB working dir for better debugging ([#6990](https://github.com/AztecProtocol/aztec-packages/issues/6990)) ([a9688f0](https://github.com/AztecProtocol/aztec-packages/commit/a9688f058252cb1c4714cfb06bd2cf30c6ac0268))
* **avm:** Use hints in gas accounting (circuit) ([#6895](https://github.com/AztecProtocol/aztec-packages/issues/6895)) ([c3746f5](https://github.com/AztecProtocol/aztec-packages/commit/c3746f5d6ae38bc448d00834d91a7ddd7b901e64))
* **bb:** Stack traces for check_circuit ([#6851](https://github.com/AztecProtocol/aztec-packages/issues/6851)) ([eb35e62](https://github.com/AztecProtocol/aztec-packages/commit/eb35e627445c72ee07fafb3652076349302e7fa1))
* **cli:** Publicly deploy a pre-initialized account ([#6960](https://github.com/AztecProtocol/aztec-packages/issues/6960)) ([e671935](https://github.com/AztecProtocol/aztec-packages/commit/e67193585fe967106a013d266e00e94d20d31b32))
* Constrain note encryption ([#6432](https://github.com/AztecProtocol/aztec-packages/issues/6432)) ([e59f4d3](https://github.com/AztecProtocol/aztec-packages/commit/e59f4d3cee4b27248d26111fc6fda2f0e55a7d54))
* Contract storage reads serialize with side effect counter ([#6961](https://github.com/AztecProtocol/aztec-packages/issues/6961)) ([db49ed5](https://github.com/AztecProtocol/aztec-packages/commit/db49ed57d1d4165ce47e6af01b6fd67239121aa4))
* **docs:** Add uniswap back in as a reference and fix links ([#7074](https://github.com/AztecProtocol/aztec-packages/issues/7074)) ([a4d1df6](https://github.com/AztecProtocol/aztec-packages/commit/a4d1df6d2900185a9c57af44e6f0c3ca80df7c9b))
* **docs:** Nits ([#6187](https://github.com/AztecProtocol/aztec-packages/issues/6187)) ([d025496](https://github.com/AztecProtocol/aztec-packages/commit/d0254960b1712b717e156b428aa05721702ec4a8))
* Ecadd op code ([#6906](https://github.com/AztecProtocol/aztec-packages/issues/6906)) ([03a9064](https://github.com/AztecProtocol/aztec-packages/commit/03a9064b308fbf5541f4f763e1ad1e05f60e1fff))
* Estimate tx size ([#6928](https://github.com/AztecProtocol/aztec-packages/issues/6928)) ([1fa7d84](https://github.com/AztecProtocol/aztec-packages/commit/1fa7d84b6bf176d4585c333747ac4a61b8743e36))
* Flows and tests for the tube component ([#6934](https://github.com/AztecProtocol/aztec-packages/issues/6934)) ([4b45438](https://github.com/AztecProtocol/aztec-packages/commit/4b454386a35f4b0cd4c6a9b8003c55e55e50b592))
* Gas token self deploys ([#6956](https://github.com/AztecProtocol/aztec-packages/issues/6956)) ([ecd7614](https://github.com/AztecProtocol/aztec-packages/commit/ecd7614d0a52d277862aef97e81b68b8f66bc2c0))
* Implement println in the comptime interpreter (https://github.com/noir-lang/noir/pull/5197) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Increase max L2 to L1 msgs ([#6959](https://github.com/AztecProtocol/aztec-packages/issues/6959)) ([875fb2d](https://github.com/AztecProtocol/aztec-packages/commit/875fb2d507368b15c9672526c52b92039ef558d3))
* Make options.limit a compile-time constant ([#7027](https://github.com/AztecProtocol/aztec-packages/issues/7027)) ([78cd640](https://github.com/AztecProtocol/aztec-packages/commit/78cd640dd2d5b281a921140b915a294eaa44f6f0))
* Nuking last hardcoded note type ids ([#7069](https://github.com/AztecProtocol/aztec-packages/issues/7069)) ([a23fd0b](https://github.com/AztecProtocol/aztec-packages/commit/a23fd0ba1604a5308fd77ed45a5b1d20da13f405)), closes [#5833](https://github.com/AztecProtocol/aztec-packages/issues/5833)
* Place return value witnesses directly after function arguments (https://github.com/noir-lang/noir/pull/5142) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Poor man's CLI block explorer ([#6946](https://github.com/AztecProtocol/aztec-packages/issues/6946)) ([2b79df6](https://github.com/AztecProtocol/aztec-packages/commit/2b79df673e7a23886052990c85dc6ca530537e9f))
* Poor man's fernet ([#6918](https://github.com/AztecProtocol/aztec-packages/issues/6918)) ([19c2a97](https://github.com/AztecProtocol/aztec-packages/commit/19c2a97784c917da212e76f3307d47e1beb8099f))
* Private kernel output validator ([#6892](https://github.com/AztecProtocol/aztec-packages/issues/6892)) ([0435e9a](https://github.com/AztecProtocol/aztec-packages/commit/0435e9a76f158b72690f34025a2723a29a3c7816))
* Processing outgoing ([#6766](https://github.com/AztecProtocol/aztec-packages/issues/6766)) ([4da66fd](https://github.com/AztecProtocol/aztec-packages/commit/4da66fdfb3d0686b5ed917e947869b9c2cef14a8))
* Pxe can filter on emitted events ([#6947](https://github.com/AztecProtocol/aztec-packages/issues/6947)) ([ee45fda](https://github.com/AztecProtocol/aztec-packages/commit/ee45fdafd837e4070b627aaac630e2f985531c97))
* Restrict noir word size to u32 (https://github.com/noir-lang/noir/pull/5180) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Separate proving from `noir_js` (https://github.com/noir-lang/noir/pull/5072) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Separate runtimes of SSA functions before inlining (https://github.com/noir-lang/noir/pull/5121) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* SMT Standard Circuit separation ([#6904](https://github.com/AztecProtocol/aztec-packages/issues/6904)) ([f970732](https://github.com/AztecProtocol/aztec-packages/commit/f9707321bdd107e3c7116cafd89fd570224e89ef))
* SMT Verification Module Update ([#6849](https://github.com/AztecProtocol/aztec-packages/issues/6849)) ([6c98529](https://github.com/AztecProtocol/aztec-packages/commit/6c985299d796b8c711794395518c3b3a0f41e775))
* SMT Verifier for Ultra Arithmetization ([#7067](https://github.com/AztecProtocol/aztec-packages/issues/7067)) ([6692ac8](https://github.com/AztecProtocol/aztec-packages/commit/6692ac831ab980d9623442236c21b499a7238966))
* Splitting event log functionality ([#6921](https://github.com/AztecProtocol/aztec-packages/issues/6921)) ([8052bc6](https://github.com/AztecProtocol/aztec-packages/commit/8052bc64ee53e27f364438ecee057e2c9c1b3583))
* Standard form for cycle_group ([#6915](https://github.com/AztecProtocol/aztec-packages/issues/6915)) ([e6cba16](https://github.com/AztecProtocol/aztec-packages/commit/e6cba16ef82428b115d527eabe237122e269aa32))
* Standardize pedersen functions to return `EmbeddedCurvePoint` (https://github.com/noir-lang/noir/pull/5190) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Storing outgoing + API for outgoing ([#7022](https://github.com/AztecProtocol/aztec-packages/issues/7022)) ([8281ec6](https://github.com/AztecProtocol/aztec-packages/commit/8281ec6dcd60a08a20da86fb9805dda8e9581764))
* Support casting in globals (https://github.com/noir-lang/noir/pull/5164) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Support disabling aztec vm in non-wasm builds  ([#6965](https://github.com/AztecProtocol/aztec-packages/issues/6965)) ([f7a46c0](https://github.com/AztecProtocol/aztec-packages/commit/f7a46c0d8de2e58b7e76576a76eb85f52b266966))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5222) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Track timeout status of proving jobs ([#6868](https://github.com/AztecProtocol/aztec-packages/issues/6868)) ([7306176](https://github.com/AztecProtocol/aztec-packages/commit/7306176c80d1d80c032c3eed38a2008d545fb025))


### Bug Fixes

* ALU pil relation TWO_LINE_OP_NO_OVERLAP ([#6968](https://github.com/AztecProtocol/aztec-packages/issues/6968)) ([4ba553b](https://github.com/AztecProtocol/aztec-packages/commit/4ba553ba3170838de3b6c4cf47b609b0198443d0))
* AVM / aztec-up CircleCI issues ([#7045](https://github.com/AztecProtocol/aztec-packages/issues/7045)) ([3f5d380](https://github.com/AztecProtocol/aztec-packages/commit/3f5d380f72d5ae819b2718ef9fbdfaec6b9a0e4d))
* **avm:** Bugfix related to pc increment in calldatacopy of avm circuit ([#6891](https://github.com/AztecProtocol/aztec-packages/issues/6891)) ([5fe59d2](https://github.com/AztecProtocol/aztec-packages/commit/5fe59d2ed96a5b966efc9e3619c87b4a23c502f4))
* **avm:** Correctly generate public inputs in verifier ([#7018](https://github.com/AztecProtocol/aztec-packages/issues/7018)) ([4c4c17f](https://github.com/AztecProtocol/aztec-packages/commit/4c4c17f804b8735dc017bbae171117ca15df25cc))
* Aztec-builder port issue ([#7068](https://github.com/AztecProtocol/aztec-packages/issues/7068)) ([729e69a](https://github.com/AztecProtocol/aztec-packages/commit/729e69ae2cd773ee176935b7d4644db95dd62668))
* Biggroup batch mul handles collisions ([#6780](https://github.com/AztecProtocol/aztec-packages/issues/6780)) ([e61c40e](https://github.com/AztecProtocol/aztec-packages/commit/e61c40e9c3e71f50c2d6a6c8a1688b6a8ddd4ba8))
* Bugfix for Keccak opcode related to reading bytes from input ([#6989](https://github.com/AztecProtocol/aztec-packages/issues/6989)) ([5713f4e](https://github.com/AztecProtocol/aztec-packages/commit/5713f4e25ef8bf09cb91632bd210cd46bb7a77c3))
* Correct docker-compose vars ([#6945](https://github.com/AztecProtocol/aztec-packages/issues/6945)) ([d492ac8](https://github.com/AztecProtocol/aztec-packages/commit/d492ac80e346572a371df84a6cebc4409b634a8d))
* Declare volume ([#6881](https://github.com/AztecProtocol/aztec-packages/issues/6881)) ([1e38115](https://github.com/AztecProtocol/aztec-packages/commit/1e381159bb1d407dec8a4926242ecd54ec38e787))
* Dirty merge 6880 ([#6905](https://github.com/AztecProtocol/aztec-packages/issues/6905)) ([fc6ec3f](https://github.com/AztecProtocol/aztec-packages/commit/fc6ec3fc7371b2506e7409a7d24ce37f25803fac))
* Do not fail if rollup contract does not support turns ([#6938](https://github.com/AztecProtocol/aztec-packages/issues/6938)) ([5e6fe68](https://github.com/AztecProtocol/aztec-packages/commit/5e6fe68e088483203655522e0242c7e3230297b5))
* Docker compose env vars ([#6926](https://github.com/AztecProtocol/aztec-packages/issues/6926)) ([14e0c1d](https://github.com/AztecProtocol/aztec-packages/commit/14e0c1df512555900cd49fff2d9070b489984e2f))
* **docs:** Fix avm instruction set table ([#7061](https://github.com/AztecProtocol/aztec-packages/issues/7061)) ([fcbd44b](https://github.com/AztecProtocol/aztec-packages/commit/fcbd44b43e8a5dd459b543aaa25158d7f1dcc050))
* **docs:** Remove prefix in link to code snippet source ([#6878](https://github.com/AztecProtocol/aztec-packages/issues/6878)) ([0e8e772](https://github.com/AztecProtocol/aztec-packages/commit/0e8e772c05c81c0b5cf6a2d047a5587c6c2e1a9c))
* **elaborator:** Invert unconstrained check (https://github.com/noir-lang/noir/pull/5176) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* **elaborator:** Lazily elaborate globals (https://github.com/noir-lang/noir/pull/5191) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Ensure changes in aztec up directory are deployed ([#7031](https://github.com/AztecProtocol/aztec-packages/issues/7031)) ([e673fd4](https://github.com/AztecProtocol/aztec-packages/commit/e673fd4784147c365d9191a997049c53a7d6d67f)), closes [#6932](https://github.com/AztecProtocol/aztec-packages/issues/6932)
* Error for allocate instructions in acir-gen (https://github.com/noir-lang/noir/pull/5200) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* **experimental elaborator:** Clear generics after elaborating type aliases (https://github.com/noir-lang/noir/pull/5136) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **experimental elaborator:** Fix `impl Trait` when `--use-elaborator` is selected (https://github.com/noir-lang/noir/pull/5138) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **experimental elaborator:** Fix definition kind of globals and tuple patterns with `--use-elaborator` flag (https://github.com/noir-lang/noir/pull/5139) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **experimental elaborator:** Fix frontend tests when `--use-elaborator` flag is specified (https://github.com/noir-lang/noir/pull/5145) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **experimental elaborator:** Fix global values used in the elaborator (https://github.com/noir-lang/noir/pull/5135) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **experimental elaborator:** Fix globals which use function calls (https://github.com/noir-lang/noir/pull/5172) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Expose node port ([#6917](https://github.com/AztecProtocol/aztec-packages/issues/6917)) ([131af88](https://github.com/AztecProtocol/aztec-packages/commit/131af8806a453b851403b0eb7cba855bc2c0cc43))
* Fix avm unit test with proving by passing the public_inputs ([#7062](https://github.com/AztecProtocol/aztec-packages/issues/7062)) ([2d7c097](https://github.com/AztecProtocol/aztec-packages/commit/2d7c097d7a6606101354736d69bd0bbbe6f005bf))
* Fix client ivc incorrect srs size issue and parallelise srs generation for grumpkin ([#6913](https://github.com/AztecProtocol/aztec-packages/issues/6913)) ([f015736](https://github.com/AztecProtocol/aztec-packages/commit/f01573641728d6cc62da36189a22fa813713fd82))
* Fix for the flaky issue (I hope) ([#6923](https://github.com/AztecProtocol/aztec-packages/issues/6923)) ([39747b9](https://github.com/AztecProtocol/aztec-packages/commit/39747b933a13aa08f25c5074207f9d92489d5e3d))
* Fix panic in `get_global_let_statement` (https://github.com/noir-lang/noir/pull/5177) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Fixing 0 naf ([#6950](https://github.com/AztecProtocol/aztec-packages/issues/6950)) ([d35ee2e](https://github.com/AztecProtocol/aztec-packages/commit/d35ee2ed87967a5161ef52d892856900a55de0b9))
* **frontend:** Resolve object types from method calls a single time (https://github.com/noir-lang/noir/pull/5131) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Initialize side_effect_counter based on the initial value passed to builder ([#7017](https://github.com/AztecProtocol/aztec-packages/issues/7017)) ([46d166b](https://github.com/AztecProtocol/aztec-packages/commit/46d166b0f1d16d801e056d3195546970cddda1a8))
* **p2p:** Remove p2p datastore persistence for now ([#6879](https://github.com/AztecProtocol/aztec-packages/issues/6879)) ([ce7f0e2](https://github.com/AztecProtocol/aztec-packages/commit/ce7f0e244621a599796e8d26fd37540b541ca0d3))
* Pxe waits for node to go up ([#6933](https://github.com/AztecProtocol/aztec-packages/issues/6933)) ([06f03fd](https://github.com/AztecProtocol/aztec-packages/commit/06f03fdf73d9374b5d18d33cf1480f7748db016f))
* Register account contract before recipient ([#6855](https://github.com/AztecProtocol/aztec-packages/issues/6855)) ([dfea1c7](https://github.com/AztecProtocol/aztec-packages/commit/dfea1c79f57564af3be83a0b3244374f74834571))
* Revert "chore: add arm64 version of aztec-nargo image" ([#7039](https://github.com/AztecProtocol/aztec-packages/issues/7039)) ([25d12da](https://github.com/AztecProtocol/aztec-packages/commit/25d12da45c1c36e8c5b77a8c81baea3bb365d2c6))
* SimulateTx does not prove ([#6930](https://github.com/AztecProtocol/aztec-packages/issues/6930)) ([d3d6b9e](https://github.com/AztecProtocol/aztec-packages/commit/d3d6b9ebfa81267b28ebf361fdba310071963492))
* Stop squashing storage accesses in avm simulator - all need to be validated in kernel ([#7036](https://github.com/AztecProtocol/aztec-packages/issues/7036)) ([6ffc4b4](https://github.com/AztecProtocol/aztec-packages/commit/6ffc4b4455a0613c933de0ec7528774186f53bee))
* Update docker-compopse env vars ([#6943](https://github.com/AztecProtocol/aztec-packages/issues/6943)) ([80d1121](https://github.com/AztecProtocol/aztec-packages/commit/80d1121f270363a3da1e4200f41283f822357f92))
* Updating max update requests per tx ([#6783](https://github.com/AztecProtocol/aztec-packages/issues/6783)) ([55b1cf7](https://github.com/AztecProtocol/aztec-packages/commit/55b1cf7056ab3d630b2ed98d1d99c43b39feb587))
* Use predicate for curve operations (https://github.com/noir-lang/noir/pull/5076) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Wrapping in signed division (https://github.com/noir-lang/noir/pull/5134) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))


### Miscellaneous

* Add arm64 version of aztec-nargo image ([#7034](https://github.com/AztecProtocol/aztec-packages/issues/7034)) ([2a41e84](https://github.com/AztecProtocol/aztec-packages/commit/2a41e8415bac26b122c0da2672bf3077aae4eda2))
* Add more lints related to oracle calls (https://github.com/noir-lang/noir/pull/5193) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Add negative tests for cast and U128 multiplication related to TWO_LINE_OP_NO_OVERLAP ([#7041](https://github.com/AztecProtocol/aztec-packages/issues/7041)) ([7f14ca1](https://github.com/AztecProtocol/aztec-packages/commit/7f14ca122032a56eb322e34ee0290845e75a925a)), closes [#6969](https://github.com/AztecProtocol/aztec-packages/issues/6969)
* Add some property tests to ACVM crates (https://github.com/noir-lang/noir/pull/5215) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Add transfer to undeployed account ([#7015](https://github.com/AztecProtocol/aztec-packages/issues/7015)) ([46324b9](https://github.com/AztecProtocol/aztec-packages/commit/46324b96343a9b603413843480211d05abdf4556))
* Add transferFrom migration notes ([#7079](https://github.com/AztecProtocol/aztec-packages/issues/7079)) ([d4921a0](https://github.com/AztecProtocol/aztec-packages/commit/d4921a032a56eb0ba464b0e505f6ac11cb41502d))
* Automate necessary changes to outward sync into noir-lang/noir ([#7049](https://github.com/AztecProtocol/aztec-packages/issues/7049)) ([449e41c](https://github.com/AztecProtocol/aztec-packages/commit/449e41c1ffbded4b64bf9cde7a97fd8670d7647c))
* **avm:** Add a TS prover test suite for each avm context function ([#6957](https://github.com/AztecProtocol/aztec-packages/issues/6957)) ([f745696](https://github.com/AztecProtocol/aztec-packages/commit/f745696270a440ce45a33b1f72996e47dacdaf74))
* **avm:** Add bytecode size metrics ([#7042](https://github.com/AztecProtocol/aztec-packages/issues/7042)) ([555d97a](https://github.com/AztecProtocol/aztec-packages/commit/555d97af65cfb1b8d30b32d5b9b9a23b9b446f9e))
* **avm:** Add debugging info and trace dump ([#6979](https://github.com/AztecProtocol/aztec-packages/issues/6979)) ([e11f880](https://github.com/AztecProtocol/aztec-packages/commit/e11f88004e2c31cb2b2ae376095513e94584a4dc))
* **avm:** Add tag checking and missing indirects ([#6936](https://github.com/AztecProtocol/aztec-packages/issues/6936)) ([48be80c](https://github.com/AztecProtocol/aztec-packages/commit/48be80c4f9cd21885b21cb9c8202e956d537e595))
* **avm:** Add TS bb prover tests for hashing opcodes ([#6970](https://github.com/AztecProtocol/aztec-packages/issues/6970)) ([312718a](https://github.com/AztecProtocol/aztec-packages/commit/312718a6946651470c7c97e42414bfc654355d24))
* **avm:** Enable tag checking and some proving tests ([#6966](https://github.com/AztecProtocol/aztec-packages/issues/6966)) ([b19daa4](https://github.com/AztecProtocol/aztec-packages/commit/b19daa44f034e50109e53363e691493534e7d3f1))
* **avm:** Fix proving for kernel tests ([#7033](https://github.com/AztecProtocol/aztec-packages/issues/7033)) ([f5e1106](https://github.com/AztecProtocol/aztec-packages/commit/f5e1106bcaa9558ac0a953de06d4fafd09fb1fe8))
* **avm:** Fix struct serialization and factory ([#6903](https://github.com/AztecProtocol/aztec-packages/issues/6903)) ([bee2646](https://github.com/AztecProtocol/aztec-packages/commit/bee2646d0274806e8fd1a74bd321620c8c8bd0d9))
* **avm:** Gas alignments with simulator ([#6873](https://github.com/AztecProtocol/aztec-packages/issues/6873)) ([54339d4](https://github.com/AztecProtocol/aztec-packages/commit/54339d48861a91429e996177713f46952ffbd808)), closes [#6860](https://github.com/AztecProtocol/aztec-packages/issues/6860)
* **avm:** Modify unit test to have a calldatacopy over 4 elements ([#6893](https://github.com/AztecProtocol/aztec-packages/issues/6893)) ([9f5b113](https://github.com/AztecProtocol/aztec-packages/commit/9f5b11345dc5dd055442eaf7673227fe7cbaf262))
* **avm:** Remove unused public context inputs ([#7028](https://github.com/AztecProtocol/aztec-packages/issues/7028)) ([f7a0921](https://github.com/AztecProtocol/aztec-packages/commit/f7a0921bb4bb26a3abba9c298a7f50e49248d711))
* Avoid `bn254_blackbox_solver` polluting feature flags (https://github.com/noir-lang/noir/pull/5141) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Avoid manual creation of contract artifact in wasm (https://github.com/noir-lang/noir/pull/5117) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Bb repo warning ([#7023](https://github.com/AztecProtocol/aztec-packages/issues/7023)) ([c3d7053](https://github.com/AztecProtocol/aztec-packages/commit/c3d70537c5558ba451a43e403bab067940aa48b6))
* **bb:** Hide `debug()` logs under `--debug` flag ([#7008](https://github.com/AztecProtocol/aztec-packages/issues/7008)) ([a8c3c3f](https://github.com/AztecProtocol/aztec-packages/commit/a8c3c3fcf35b7c464006c481230afcb11b9952dc))
* Break out helper methods for writing foreign call results (https://github.com/noir-lang/noir/pull/5181) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* **ci:** Don't raise MSRV issue if workflow cancelled (https://github.com/noir-lang/noir/pull/5143) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Custom jest Field equality ([#7012](https://github.com/AztecProtocol/aztec-packages/issues/7012)) ([1a198b8](https://github.com/AztecProtocol/aztec-packages/commit/1a198b8d53397f89f9fe6299d9ec5cb42ce245b2))
* Default to using bn254 in `noirc_frontend` (https://github.com/noir-lang/noir/pull/5144) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* **docs:** Fix incorrect docs github link in footer (https://github.com/noir-lang/noir/pull/5206) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* **docs:** Fixing the breadcrumb issue ([#6605](https://github.com/AztecProtocol/aztec-packages/issues/6605)) ([2624c26](https://github.com/AztecProtocol/aztec-packages/commit/2624c264fd266e090eec1b79654005b4dcd057de))
* **docs:** Supplement Noir Debugger's dependency versions (https://github.com/noir-lang/noir/pull/5199) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* **docs:** Update docs homepage (https://github.com/noir-lang/noir/pull/5198) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Enable skipped ordering tests since AVM properly updates side-effect counter for nested calls ([#7064](https://github.com/AztecProtocol/aztec-packages/issues/7064)) ([5ff5ffb](https://github.com/AztecProtocol/aztec-packages/commit/5ff5ffb83ae55c6f12af6e5271e399f4aeaa4737)), closes [#6471](https://github.com/AztecProtocol/aztec-packages/issues/6471)
* **experimental elaborator:** Handle `comptime` expressions in the elaborator (https://github.com/noir-lang/noir/pull/5169) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Fix issue [#6929](https://github.com/AztecProtocol/aztec-packages/issues/6929) (off-by-one error in `UltraCircuitBuilder::create_range_constraint`) ([#6931](https://github.com/AztecProtocol/aztec-packages/issues/6931)) ([16deef6](https://github.com/AztecProtocol/aztec-packages/commit/16deef6a83a9fe41e1f865e79e17c2f671604bb0))
* Fix migration notes ([#7075](https://github.com/AztecProtocol/aztec-packages/issues/7075)) ([ac75f8c](https://github.com/AztecProtocol/aztec-packages/commit/ac75f8cf2019dd00f80c81259c30737a042a4b9b))
* Granular public simulation benchmarks ([#6924](https://github.com/AztecProtocol/aztec-packages/issues/6924)) ([b70bc98](https://github.com/AztecProtocol/aztec-packages/commit/b70bc98c948c51053560e8948a43b65159a95b58))
* Inline `FieldElement.is_negative` and document (https://github.com/noir-lang/noir/pull/5214) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Lookups cleanup/documentation ([#7002](https://github.com/AztecProtocol/aztec-packages/issues/7002)) ([92b1349](https://github.com/AztecProtocol/aztec-packages/commit/92b1349ba671e87e948bf9248c5133accde9091f))
* Loosen trait bounds on impls depending on `AcirField` (https://github.com/noir-lang/noir/pull/5115) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Make `nargo` crate and debug info generic (https://github.com/noir-lang/noir/pull/5184) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Mark all oracles as unconstrained ([#7032](https://github.com/AztecProtocol/aztec-packages/issues/7032)) ([7a68be4](https://github.com/AztecProtocol/aztec-packages/commit/7a68be4bc31114853d8c25549029c74afd9a8f37))
* Move acir docs to code declaration (https://github.com/noir-lang/noir/pull/5040) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Move gas bridge initialization into L1 contracts deployment ([#6912](https://github.com/AztecProtocol/aztec-packages/issues/6912)) ([26a1fc4](https://github.com/AztecProtocol/aztec-packages/commit/26a1fc4bcec04434b61651e2f527938a14f3ac3a))
* Move implementation of bitwise operations into `blackbox_solver` (https://github.com/noir-lang/noir/pull/5209) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Note emission ([#7003](https://github.com/AztecProtocol/aztec-packages/issues/7003)) ([10048da](https://github.com/AztecProtocol/aztec-packages/commit/10048da5ce7edfe850d03ee97505ed72552c1dca))
* Note processor cleanup ([#6870](https://github.com/AztecProtocol/aztec-packages/issues/6870)) ([315c46e](https://github.com/AztecProtocol/aztec-packages/commit/315c46e3804718bcaf7337da887548755984ca67))
* Opcodes l2gasleft and dagasleft return value with tag ff ([#6896](https://github.com/AztecProtocol/aztec-packages/issues/6896)) ([5890845](https://github.com/AztecProtocol/aztec-packages/commit/5890845e8f9b278b2a5c5c930eb28ec0aba74ebc))
* Parallelise compilation of contracts and protocol circuits ([#7009](https://github.com/AztecProtocol/aztec-packages/issues/7009)) ([86a3314](https://github.com/AztecProtocol/aztec-packages/commit/86a33140f9a65e518003b3f4c60f97d132f85b89))
* Remove `distinct` keyword (https://github.com/noir-lang/noir/pull/5219) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Remove `param_witnesses` and `return_witnesses` from ABI (https://github.com/noir-lang/noir/pull/5154) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Remove deprecated functions ([#7029](https://github.com/AztecProtocol/aztec-packages/issues/7029)) ([bc80e85](https://github.com/AztecProtocol/aztec-packages/commit/bc80e8575b5e60c3a45a7631e445c79774d20a49))
* Remove hir to ast pass (https://github.com/noir-lang/noir/pull/5147) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Remove old public storage access ordering hack ([#7063](https://github.com/AztecProtocol/aztec-packages/issues/7063)) ([bf6b8b8](https://github.com/AztecProtocol/aztec-packages/commit/bf6b8b86d78ce6ee5e863dc0a43e78c36b6b35a8))
* Remove stale comment (https://github.com/noir-lang/noir/pull/5179) ([12af650](https://github.com/AztecProtocol/aztec-packages/commit/12af650f0d27c37dca06bb329bf76a5574534d78))
* Remove unused `new_variables` argument from `resolve_type_inner` (https://github.com/noir-lang/noir/pull/5148) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Rename p2p vars ([#6916](https://github.com/AztecProtocol/aztec-packages/issues/6916)) ([ae7d757](https://github.com/AztecProtocol/aztec-packages/commit/ae7d75764fc704daae67be882e0e9f09a0a9407c))
* Replace relative paths to noir-protocol-circuits ([8f7b865](https://github.com/AztecProtocol/aztec-packages/commit/8f7b8656940354df38bb623bc6d8941ab98f3e5d))
* Replace relative paths to noir-protocol-circuits ([91e1554](https://github.com/AztecProtocol/aztec-packages/commit/91e155472951908e455fff9279118f0b9be0900b))
* Replace relative paths to noir-protocol-circuits ([f4fed13](https://github.com/AztecProtocol/aztec-packages/commit/f4fed131a9c3bb568a995846d09f793620c5a366))
* Replace relative paths to noir-protocol-circuits ([7caa288](https://github.com/AztecProtocol/aztec-packages/commit/7caa28892086b9f97d417d0694e3cad228fd5788))
* Replace relative paths to noir-protocol-circuits ([8a299e9](https://github.com/AztecProtocol/aztec-packages/commit/8a299e99783775d70ba8871f44057a03daaf4917))
* Replace relative paths to noir-protocol-circuits ([acf1188](https://github.com/AztecProtocol/aztec-packages/commit/acf1188fb1fc5ea4d53d57a6c0a362ad55cd707e))
* Replace relative paths to noir-protocol-circuits ([094b511](https://github.com/AztecProtocol/aztec-packages/commit/094b511e6a696e5c2a7687147ca21007801237de))
* Replace relative paths to noir-protocol-circuits ([8e07176](https://github.com/AztecProtocol/aztec-packages/commit/8e0717654ec7f75fe2ea8577457359ec2a102b58))
* Replace relative paths to noir-protocol-circuits ([52b6934](https://github.com/AztecProtocol/aztec-packages/commit/52b69341129a23e300292e3f5e207cb512b05aa6))
* Replace relative paths to noir-protocol-circuits ([873dcea](https://github.com/AztecProtocol/aztec-packages/commit/873dcea15e4e802f99998e2ed113ebaa8bd834e6))
* Replace sibling path read with leaf read ([#6834](https://github.com/AztecProtocol/aztec-packages/issues/6834)) ([a20d845](https://github.com/AztecProtocol/aztec-packages/commit/a20d845d35715816ddc889fb9a75fb9fba4fc356))
* Run all test programs in brillig as well as ACIR (https://github.com/noir-lang/noir/pull/5128) ([a44b8c8](https://github.com/AztecProtocol/aztec-packages/commit/a44b8c81458eb789e54624e020b6c93d0e9963cc))
* Schnorr signature verification in noir (https://github.com/noir-lang/noir/pull/5188) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Small fixes for the tube flows ([#7014](https://github.com/AztecProtocol/aztec-packages/issues/7014)) ([838ceed](https://github.com/AztecProtocol/aztec-packages/commit/838ceed3b6ccf1bb7d89552a147db92c3514f0c1))
* Split log emission to encrypt and a log, remove address input ([#6987](https://github.com/AztecProtocol/aztec-packages/issues/6987)) ([ca0e084](https://github.com/AztecProtocol/aztec-packages/commit/ca0e0848563cfae72ebd7d4487a6e2812c2a405c))
* Start moving lints into a separate linting directory (https://github.com/noir-lang/noir/pull/5165) ([bf38cc2](https://github.com/AztecProtocol/aztec-packages/commit/bf38cc29821d96d801f56e70342426e1b12692e1))
* Terraform Updates ([#6887](https://github.com/AztecProtocol/aztec-packages/issues/6887)) ([33a3870](https://github.com/AztecProtocol/aztec-packages/commit/33a3870d06ae8bb5d08dbbd9f72a62e0811e5e7d))
* Transfer and transferfrom to save constrains for simpler cases ([#7013](https://github.com/AztecProtocol/aztec-packages/issues/7013)) ([612b972](https://github.com/AztecProtocol/aztec-packages/commit/612b9724a419224c72cd823c889ece4ae8f00ab0))
* TS avm proving test - add a pattern for assertion failure (timestamp example) ([#7005](https://github.com/AztecProtocol/aztec-packages/issues/7005)) ([cfef246](https://github.com/AztecProtocol/aztec-packages/commit/cfef24654492a1f3eef94db60937bd3a45f8ec3c))
* Update comment on transient nullification ([#7001](https://github.com/AztecProtocol/aztec-packages/issues/7001)) ([6c4e61c](https://github.com/AztecProtocol/aztec-packages/commit/6c4e61c19613560af8aedba03531958f8471bb62))
* Updated devnet terraform ([#6927](https://github.com/AztecProtocol/aztec-packages/issues/6927)) ([4692fb0](https://github.com/AztecProtocol/aztec-packages/commit/4692fb034f22bb62593d257777b7b545993c27ab))
* Updated l1 contracts in compose file ([#6942](https://github.com/AztecProtocol/aztec-packages/issues/6942)) ([15371ce](https://github.com/AztecProtocol/aztec-packages/commit/15371ceafb62627cd0bcb5ba65c854f07e09cb49))
* Updated sha for devnet image in compose file ([#6939](https://github.com/AztecProtocol/aztec-packages/issues/6939)) ([83dd231](https://github.com/AztecProtocol/aztec-packages/commit/83dd231d7c7bc561829296cb3f252fb9ab50528f))


### Documentation

* Add account tags ([#7011](https://github.com/AztecProtocol/aztec-packages/issues/7011)) ([8580467](https://github.com/AztecProtocol/aztec-packages/commit/8580467354fe32cda87c956ea40caa4d0f058a04))
* **avm:** Comments in pil file related to range checks of addresses ([#6837](https://github.com/AztecProtocol/aztec-packages/issues/6837)) ([66f1c87](https://github.com/AztecProtocol/aztec-packages/commit/66f1c876578b05838698377f2ede12b52671e4ca))
* Aztec macros ([#6935](https://github.com/AztecProtocol/aztec-packages/issues/6935)) ([57078d4](https://github.com/AztecProtocol/aztec-packages/commit/57078d4aea54c4beaf66f10db2f0052d4577e46a))
* Clarify comment on collapse ([#7038](https://github.com/AztecProtocol/aztec-packages/issues/7038)) ([6237ddd](https://github.com/AztecProtocol/aztec-packages/commit/6237ddd9052fc98a26de07f11bd494843dcc07ee))
* Update HOW_WE_WRITE_DOCS.md ([#6850](https://github.com/AztecProtocol/aztec-packages/issues/6850)) ([d4dfdaf](https://github.com/AztecProtocol/aztec-packages/commit/d4dfdaf9ab03eeafa3d54be178fc72c59ac51b95))

## [0.42.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.41.0...aztec-packages-v0.42.0) (2024-06-04)


### ⚠ BREAKING CHANGES

* introduce UnconstrainedContext ([#6752](https://github.com/AztecProtocol/aztec-packages/issues/6752))
* integrate AVM proving ([#6775](https://github.com/AztecProtocol/aztec-packages/issues/6775))
* constrain note_getter filter ([#6703](https://github.com/AztecProtocol/aztec-packages/issues/6703))
* **stdlib:** eddsa function using turbofish (https://github.com/noir-lang/noir/pull/5050)
* migrate public to avm simulator ([#6448](https://github.com/AztecProtocol/aztec-packages/issues/6448))

### Features

* ACIR integration tests in Bb ([#6607](https://github.com/AztecProtocol/aztec-packages/issues/6607)) ([ca89670](https://github.com/AztecProtocol/aztec-packages/commit/ca896707dd5c3da077fa8797e348aad7a6f05637))
* Activate return_data in ACIR opcodes (https://github.com/noir-lang/noir/pull/5080) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Add `as_witness` builtin function in order to constrain a witness to be equal to a variable  (https://github.com/noir-lang/noir/pull/4641) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* Add cli to published  image ([#6758](https://github.com/AztecProtocol/aztec-packages/issues/6758)) ([1e45400](https://github.com/AztecProtocol/aztec-packages/commit/1e45400e8338ec1a83bbfe5c0af255a15a8db7a7))
* Add code-workspace and update build dirs ([#6723](https://github.com/AztecProtocol/aztec-packages/issues/6723)) ([c373d15](https://github.com/AztecProtocol/aztec-packages/commit/c373d15fc49f8f4c91c5f6a4ef424b268f8e2e44))
* Add estimateGas to aztec-js ([#6701](https://github.com/AztecProtocol/aztec-packages/issues/6701)) ([cf603df](https://github.com/AztecProtocol/aztec-packages/commit/cf603df4b9a8ef21279af3fc6f3acc6a9f9af12c))
* Add flag to not mask address in siloing enc log ([#6668](https://github.com/AztecProtocol/aztec-packages/issues/6668)) ([73708ee](https://github.com/AztecProtocol/aztec-packages/commit/73708ee4e4870b4189c229732fb56bbe6dab0928))
* Add goblin recursive verifier to ClientIVC recursive verifier ([#6811](https://github.com/AztecProtocol/aztec-packages/issues/6811)) ([bd795c7](https://github.com/AztecProtocol/aztec-packages/commit/bd795c70874662674ebc580c1496d3ccec93e93a))
* Add intrinsic to get if running inside an unconstrained context (https://github.com/noir-lang/noir/pull/5098) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Add native rust implementation of schnorr signature verification (https://github.com/noir-lang/noir/pull/5053) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* **avm executor:** Kernel outputs & execution hints in executor  ([#6769](https://github.com/AztecProtocol/aztec-packages/issues/6769)) ([6ab7360](https://github.com/AztecProtocol/aztec-packages/commit/6ab73606d3424aa5f286a63e5a91d57963132126))
* Avm unconstrained external call ([#6846](https://github.com/AztecProtocol/aztec-packages/issues/6846)) ([5a65ffc](https://github.com/AztecProtocol/aztec-packages/commit/5a65ffca0d7514a609fae71ecff4744027a4ff51))
* **avm:** Add storage address kernel opcode ([#6863](https://github.com/AztecProtocol/aztec-packages/issues/6863)) ([19aba3e](https://github.com/AztecProtocol/aztec-packages/commit/19aba3e10391b0031e1dedd47eb556ef44a95282))
* **avm:** Add temporary sha256 execution ([#6604](https://github.com/AztecProtocol/aztec-packages/issues/6604)) ([34088b4](https://github.com/AztecProtocol/aztec-packages/commit/34088b48acdfc40df3144509a3effd348167ee58))
* **avm:** Avm keccak permutation ([#6596](https://github.com/AztecProtocol/aztec-packages/issues/6596)) ([c0917e4](https://github.com/AztecProtocol/aztec-packages/commit/c0917e4a4febe895d911e0bfff7ddd2a4d965fc6))
* **avm:** Handle debuglog ([#6630](https://github.com/AztecProtocol/aztec-packages/issues/6630)) ([eba345b](https://github.com/AztecProtocol/aztec-packages/commit/eba345ba253fc5ff7a6a2cd5bc7ec4f69ada8b56))
* **avm:** In vm static gas accounting ([#6542](https://github.com/AztecProtocol/aztec-packages/issues/6542)) ([6b88ae0](https://github.com/AztecProtocol/aztec-packages/commit/6b88ae0a4b8aa1b738f930a8af564ac43be76f5b))
* **avm:** Internal call stack dedicated memory ([#6503](https://github.com/AztecProtocol/aztec-packages/issues/6503)) ([d3c3d4a](https://github.com/AztecProtocol/aztec-packages/commit/d3c3d4a565568329ca55f24e7f5e8d3e293e1366)), closes [#6245](https://github.com/AztecProtocol/aztec-packages/issues/6245)
* **avm:** JUMPI opcode in AVM circuit ([#6800](https://github.com/AztecProtocol/aztec-packages/issues/6800)) ([64d4ba9](https://github.com/AztecProtocol/aztec-packages/commit/64d4ba9e57dabf7786dfb6369afbe17545a0bc2b)), closes [#6795](https://github.com/AztecProtocol/aztec-packages/issues/6795)
* **avm:** Kernel output opcodes ([#6416](https://github.com/AztecProtocol/aztec-packages/issues/6416)) ([0281b8f](https://github.com/AztecProtocol/aztec-packages/commit/0281b8f91f3e8178b9f3c2413d8833c5129fb5c4))
* **avm:** Pedersen ops ([#6765](https://github.com/AztecProtocol/aztec-packages/issues/6765)) ([7b3a72c](https://github.com/AztecProtocol/aztec-packages/commit/7b3a72c8c9fcf8e2dabbf6d0bee658188450bbe4))
* **avm:** Plumb execution hints from TS to AVM prover ([#6806](https://github.com/AztecProtocol/aztec-packages/issues/6806)) ([f3234f1](https://github.com/AztecProtocol/aztec-packages/commit/f3234f1bf037b68237bd6a8b2bce8b016bb170d0))
* **avm:** Poseidon2 gadget ([#6504](https://github.com/AztecProtocol/aztec-packages/issues/6504)) ([9e8cba1](https://github.com/AztecProtocol/aztec-packages/commit/9e8cba1f585834f2960cd821c0fbde3c893c6daf))
* **avm:** Sha256_compression ([#6452](https://github.com/AztecProtocol/aztec-packages/issues/6452)) ([5596bb3](https://github.com/AztecProtocol/aztec-packages/commit/5596bb3a19fce545b6276231d938464948f08f79))
* Batch simulate ([#6599](https://github.com/AztecProtocol/aztec-packages/issues/6599)) ([8d54ac1](https://github.com/AztecProtocol/aztec-packages/commit/8d54ac1efb3657bdea751cb1f5a0162a28da5c78))
* Bench uploading ([#5787](https://github.com/AztecProtocol/aztec-packages/issues/5787)) ([bd64ceb](https://github.com/AztecProtocol/aztec-packages/commit/bd64cebcbb2d66a328cce2aebf9ea5b8fb1f8793))
* Biggroup handles points at infinity ([#6391](https://github.com/AztecProtocol/aztec-packages/issues/6391)) ([bd72db5](https://github.com/AztecProtocol/aztec-packages/commit/bd72db5c02245b758b34f867734af4c19045fe33))
* Bump prover concurrency ([#6814](https://github.com/AztecProtocol/aztec-packages/issues/6814)) ([a543675](https://github.com/AztecProtocol/aztec-packages/commit/a543675a7d8a45683c8b77d197a7e249e428d94d))
* Changing finite field arithmetic in wasm to 29 bits for multiplications (second try, disabled AVM build in wasm) ([#6027](https://github.com/AztecProtocol/aztec-packages/issues/6027)) ([c3fa366](https://github.com/AztecProtocol/aztec-packages/commit/c3fa36616e398603d8ec995fc50488d72099e007))
* Claim and pay fee on the same tx ([#6579](https://github.com/AztecProtocol/aztec-packages/issues/6579)) ([4c09894](https://github.com/AztecProtocol/aztec-packages/commit/4c09894b6a0e003ac4caccfa25a5a76d076d1eea)), closes [#6562](https://github.com/AztecProtocol/aztec-packages/issues/6562)
* ClientIvc recursive verifier ([#6721](https://github.com/AztecProtocol/aztec-packages/issues/6721)) ([ceec7e2](https://github.com/AztecProtocol/aztec-packages/commit/ceec7e2c4e56f16cd5e6c2eb70804656f7d606f1))
* Complete ECCVM recursive verifier ([#6720](https://github.com/AztecProtocol/aztec-packages/issues/6720)) ([a98d30b](https://github.com/AztecProtocol/aztec-packages/commit/a98d30b9c678b87c17cba21653fc5107705be45a))
* Consider block parameters in variable liveness (https://github.com/noir-lang/noir/pull/5097) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Constrain note_getter filter ([#6703](https://github.com/AztecProtocol/aztec-packages/issues/6703)) ([545da36](https://github.com/AztecProtocol/aztec-packages/commit/545da3652fa2025ae6350c5a83f1dab14a830c63))
* Cycle scalar &lt;&gt; bigfield interactions ([#6744](https://github.com/AztecProtocol/aztec-packages/issues/6744)) ([6e363ec](https://github.com/AztecProtocol/aztec-packages/commit/6e363ec51164af8118d8676cdf153a3699909314))
* Devnet docker compose ([#6761](https://github.com/AztecProtocol/aztec-packages/issues/6761)) ([c62dfee](https://github.com/AztecProtocol/aztec-packages/commit/c62dfeecfb81a0eb82794c31ff6aa30f0366d5c9))
* Do not build L2 blocks over 1mb ([#6829](https://github.com/AztecProtocol/aztec-packages/issues/6829)) ([ce23a15](https://github.com/AztecProtocol/aztec-packages/commit/ce23a15ad59b5d9e88f1737bf3dfd08b26fc7f15))
* **docs:** How to Test ([#6186](https://github.com/AztecProtocol/aztec-packages/issues/6186)) ([172e415](https://github.com/AztecProtocol/aztec-packages/commit/172e4150d0750825063ad76be0c73e27fe48f3a9))
* Emit note logs linked to note hash counters, header.is_transient -&gt; note_hash_counter ([#6728](https://github.com/AztecProtocol/aztec-packages/issues/6728)) ([dd1e85c](https://github.com/AztecProtocol/aztec-packages/commit/dd1e85c7d07285312e53b2c90c51a230cb781292))
* Enable honk_recursion through acir ([#6719](https://github.com/AztecProtocol/aztec-packages/issues/6719)) ([7ce4cbe](https://github.com/AztecProtocol/aztec-packages/commit/7ce4cbef78ac0da590fbbad184219038ffa5afd9))
* Fold acir programs ([#6563](https://github.com/AztecProtocol/aztec-packages/issues/6563)) ([f7d6541](https://github.com/AztecProtocol/aztec-packages/commit/f7d65416c741790ce5b5cda8cba08d869a659670))
* Folding acir programs ([#6685](https://github.com/AztecProtocol/aztec-packages/issues/6685)) ([8d1788d](https://github.com/AztecProtocol/aztec-packages/commit/8d1788de43c41929ce131c3dbd4687ce555e48bc))
* Generate vks and verifier contract in CI ([#6627](https://github.com/AztecProtocol/aztec-packages/issues/6627)) ([523905f](https://github.com/AztecProtocol/aztec-packages/commit/523905f246c0f88e91643d94245611e48bae683e))
* Get protocol contract addresses ([#6852](https://github.com/AztecProtocol/aztec-packages/issues/6852)) ([b540fdd](https://github.com/AztecProtocol/aztec-packages/commit/b540fdda0e7103d9aac79492db2653b8c50a237c))
* Goblin Recursive Verifier ([#6778](https://github.com/AztecProtocol/aztec-packages/issues/6778)) ([53d0d55](https://github.com/AztecProtocol/aztec-packages/commit/53d0d55594cc4a71894394455308472f90b434be))
* Historical access of key getters, fixing logic in contracts after rotation ([#6656](https://github.com/AztecProtocol/aztec-packages/issues/6656)) ([d9d0193](https://github.com/AztecProtocol/aztec-packages/commit/d9d019335a9b2a57c8764bdec0560339ca21c185))
* Implement turbofish operator (https://github.com/noir-lang/noir/pull/3542) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* Increasing MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, removing hack ([#6739](https://github.com/AztecProtocol/aztec-packages/issues/6739)) ([d5550c6](https://github.com/AztecProtocol/aztec-packages/commit/d5550c66f601ca9e7a6f3ffe3d552b0ce309f293))
* Inject fee payment update in base rollup ([#6403](https://github.com/AztecProtocol/aztec-packages/issues/6403)) ([4991188](https://github.com/AztecProtocol/aztec-packages/commit/4991188afcab7c5c707fea99797bd3f5222ab2dc))
* Instantiate ECCVM relations on bigfield ([#6647](https://github.com/AztecProtocol/aztec-packages/issues/6647)) ([0ee6ef9](https://github.com/AztecProtocol/aztec-packages/commit/0ee6ef9abae584a5d137ba007c44bdfde0a01f8a))
* Integrate AVM proving ([#6775](https://github.com/AztecProtocol/aztec-packages/issues/6775)) ([f1512a2](https://github.com/AztecProtocol/aztec-packages/commit/f1512a21f48c724f1c0e373bef20f5b7fdd0bbf7))
* Introduce initialize_or_replace ([#6519](https://github.com/AztecProtocol/aztec-packages/issues/6519)) ([b59cd4c](https://github.com/AztecProtocol/aztec-packages/commit/b59cd4cc70560bfbf381100a803dbfdb25712b53))
* Introduce UnconstrainedContext ([#6752](https://github.com/AztecProtocol/aztec-packages/issues/6752)) ([e00b251](https://github.com/AztecProtocol/aztec-packages/commit/e00b2514267dced99eb31e767a6885bfa11c2382))
* Make ACVM generic across fields (https://github.com/noir-lang/noir/pull/5114) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Meaningful outgoing ([#6560](https://github.com/AztecProtocol/aztec-packages/issues/6560)) ([3f93757](https://github.com/AztecProtocol/aztec-packages/commit/3f93757cec4bcf8114e2a123298d2f4f5f4c63c8)), closes [#6410](https://github.com/AztecProtocol/aztec-packages/issues/6410)
* Migrate public to avm simulator ([#6448](https://github.com/AztecProtocol/aztec-packages/issues/6448)) ([c45e8c2](https://github.com/AztecProtocol/aztec-packages/commit/c45e8c2ada91d4c6a31bda1f65ac6cca89ca2254))
* **Misc:** Multi wallet token simulator ([#6763](https://github.com/AztecProtocol/aztec-packages/issues/6763)) ([3d376f2](https://github.com/AztecProtocol/aztec-packages/commit/3d376f2c515967f01b72437aef8f16dc7ee782da))
* **nargo:** Hidden option to show contract artifact paths written by `nargo compile` ([#6131](https://github.com/AztecProtocol/aztec-packages/issues/6131)) ([d4377ee](https://github.com/AztecProtocol/aztec-packages/commit/d4377ee6edbc5b4ce8fa87ac4613c77d470c35b4))
* New test program for verifying honk ([#6781](https://github.com/AztecProtocol/aztec-packages/issues/6781)) ([1324d58](https://github.com/AztecProtocol/aztec-packages/commit/1324d5880e416586e47045d90efca55959773e18))
* Node verifies proofs ([#6735](https://github.com/AztecProtocol/aztec-packages/issues/6735)) ([3a215ed](https://github.com/AztecProtocol/aztec-packages/commit/3a215ed3e4f9015267d55b2a8572475dc5cc55d4))
* Pow method for bigfield ([#6725](https://github.com/AztecProtocol/aztec-packages/issues/6725)) ([e4feb80](https://github.com/AztecProtocol/aztec-packages/commit/e4feb8027548cd0a7da56c68ad1af0a0be16bdc5))
* Prepare circuit output for validation ([#6678](https://github.com/AztecProtocol/aztec-packages/issues/6678)) ([03511f5](https://github.com/AztecProtocol/aztec-packages/commit/03511f54a8141faf7fafb04a8057dbc38ab67e07))
* Prototype for using the databus with ACIR opcode ([#6366](https://github.com/AztecProtocol/aztec-packages/issues/6366)) ([9f746d9](https://github.com/AztecProtocol/aztec-packages/commit/9f746d9f0cd24cecdb86c0fb9af555bc2d1f0bf6))
* Prove padding tx for block building ([#6759](https://github.com/AztecProtocol/aztec-packages/issues/6759)) ([0921401](https://github.com/AztecProtocol/aztec-packages/commit/0921401fc06c69cfc0ac112f23f2b444decbf08d))
* Re-introduced Aztec CLI ([#6734](https://github.com/AztecProtocol/aztec-packages/issues/6734)) ([a120015](https://github.com/AztecProtocol/aztec-packages/commit/a1200153ad17e51e77508323831baaed9d395780))
* Recursion in public kernels and rollup circuits ([#6425](https://github.com/AztecProtocol/aztec-packages/issues/6425)) ([86fb999](https://github.com/AztecProtocol/aztec-packages/commit/86fb9992f0a41a68546b96f3505a225ec73f853d))
* Reduce stack size after optimizations ([#6698](https://github.com/AztecProtocol/aztec-packages/issues/6698)) ([3502ccd](https://github.com/AztecProtocol/aztec-packages/commit/3502ccdcf4764ac7519d8db4c1223f574867659b))
* Reenable native fee payment ([#6571](https://github.com/AztecProtocol/aztec-packages/issues/6571)) ([78f8cbe](https://github.com/AztecProtocol/aztec-packages/commit/78f8cbe6e7cd8f44f748b4c161163d56ea207199))
* Remove conditional compilation of `bn254_blackbox_solver` (https://github.com/noir-lang/noir/pull/5058) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Remove external blackbox solver from acir simulator ([#6586](https://github.com/AztecProtocol/aztec-packages/issues/6586)) ([9c54590](https://github.com/AztecProtocol/aztec-packages/commit/9c54590f2246554d142b494b8c7a03238096600d))
* Replace stdlib poseidon implementation with optimized version (https://github.com/noir-lang/noir/pull/5122) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* Representation of a grumpkin verifier commitment key inside a bn254 circuit ([#6593](https://github.com/AztecProtocol/aztec-packages/issues/6593)) ([1d84975](https://github.com/AztecProtocol/aztec-packages/commit/1d84975d1093e601b4c9ad9d68855b39898ef79a))
* Revertible teardown ([#6490](https://github.com/AztecProtocol/aztec-packages/issues/6490)) ([288231b](https://github.com/AztecProtocol/aztec-packages/commit/288231b2872483bb8dda230a60564d8632597500))
* Run kernel reset between iterations as needed ([#6554](https://github.com/AztecProtocol/aztec-packages/issues/6554)) ([d2ab01d](https://github.com/AztecProtocol/aztec-packages/commit/d2ab01dc908832f5ce9c69aaf8525da29a39ebaa))
* **sandbox:** Auto transpile public contract functions in sandbox ([#6140](https://github.com/AztecProtocol/aztec-packages/issues/6140)) ([9639f34](https://github.com/AztecProtocol/aztec-packages/commit/9639f340a661ed56f023951a736f7f89ebd97f10))
* Silo logs hashes in tail circuit ([#6536](https://github.com/AztecProtocol/aztec-packages/issues/6536)) ([0f45b77](https://github.com/AztecProtocol/aztec-packages/commit/0f45b77232e8f6ff7d7891755241587417494194))
* **stdlib:** Eddsa function using turbofish (https://github.com/noir-lang/noir/pull/5050) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Sumcheck part of ECCVM recursive verifier instantiated as an UltraCircuit ([#6413](https://github.com/AztecProtocol/aztec-packages/issues/6413)) ([afe84a2](https://github.com/AztecProtocol/aztec-packages/commit/afe84a201cb8462c0e9f538b4518085f68bbdab5))
* Support AVM in bb-prover-exec ([#6666](https://github.com/AztecProtocol/aztec-packages/issues/6666)) ([a64a921](https://github.com/AztecProtocol/aztec-packages/commit/a64a921f227451e9a0e1d00569a69a281ff0a8c7))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5070) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/5125) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Track per function sim/witgen time ([#6498](https://github.com/AztecProtocol/aztec-packages/issues/6498)) ([d49acaf](https://github.com/AztecProtocol/aztec-packages/commit/d49acaf56f75dbf7bc4b68326851e68327cf7274))
* Update honk recursion constraint ([#6545](https://github.com/AztecProtocol/aztec-packages/issues/6545)) ([6f86352](https://github.com/AztecProtocol/aztec-packages/commit/6f86352fafa7d22f9b0f64ec67199efe6346d82f))


### Bug Fixes

* Add cbind declarations for new methods to fix autogen ([#6622](https://github.com/AztecProtocol/aztec-packages/issues/6622)) ([2429cd8](https://github.com/AztecProtocol/aztec-packages/commit/2429cd87a980eca62d2ff4543e6887f5ee9dd600))
* Add missing initializers ([#6591](https://github.com/AztecProtocol/aztec-packages/issues/6591)) ([a575708](https://github.com/AztecProtocol/aztec-packages/commit/a575708cecc47a73bce82b67b3964e21bce77cdf))
* Allow end-setup from any private function ([#6692](https://github.com/AztecProtocol/aztec-packages/issues/6692)) ([1512017](https://github.com/AztecProtocol/aztec-packages/commit/1512017e597776db9f2ec5c51076695b66b0cc76))
* Apply self type from generic trait constraint before instantiating identifiers (https://github.com/noir-lang/noir/pull/5087) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Auto dereference trait methods in the elaborator (https://github.com/noir-lang/noir/pull/5124) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Bad merge ([#6767](https://github.com/AztecProtocol/aztec-packages/issues/6767)) ([c745b7b](https://github.com/AztecProtocol/aztec-packages/commit/c745b7be5a77feea54ce2cd761ff8a737d9df679))
* Build initial header from committed data ([#6826](https://github.com/AztecProtocol/aztec-packages/issues/6826)) ([1f760e7](https://github.com/AztecProtocol/aztec-packages/commit/1f760e7caba4dfe88920157714fa456609542314))
* Bump timeouts in fetch. enable logging. no retry ([#6831](https://github.com/AztecProtocol/aztec-packages/issues/6831)) ([dd0c810](https://github.com/AztecProtocol/aztec-packages/commit/dd0c8106919f37739af93e3c93159d9370508013))
* CCI jobs for avm-transpiler and aztec-nargo should require job noir as prereq ([#6743](https://github.com/AztecProtocol/aztec-packages/issues/6743)) ([76b0981](https://github.com/AztecProtocol/aztec-packages/commit/76b0981881666ede771ca5b514d406a22c739739))
* Ci retry ([#6619](https://github.com/AztecProtocol/aztec-packages/issues/6619)) ([25fd783](https://github.com/AztecProtocol/aztec-packages/commit/25fd783bf79ee0be9df903e3855f22f36d2b8ea2))
* **ci:** Disable firefox box tests for now ([#6677](https://github.com/AztecProtocol/aztec-packages/issues/6677)) ([eadcc6f](https://github.com/AztecProtocol/aztec-packages/commit/eadcc6f9f91058e7b0691a41745728311e44973b))
* **ci:** Only retry on actual fail ([#6625](https://github.com/AztecProtocol/aztec-packages/issues/6625)) ([8604f9b](https://github.com/AztecProtocol/aztec-packages/commit/8604f9bebeb864b1c3e84948a621b63f473c4d3f))
* **ci:** Remove disk-having instance type ([#6592](https://github.com/AztecProtocol/aztec-packages/issues/6592)) ([f912ba3](https://github.com/AztecProtocol/aztec-packages/commit/f912ba33f55c9c145fc65e4cbbdd3562d185d211))
* **ci:** Remove instance types with a disk ([#6587](https://github.com/AztecProtocol/aztec-packages/issues/6587)) ([c18de5b](https://github.com/AztecProtocol/aztec-packages/commit/c18de5b6924c2352ed987e9ee1977829654e26aa))
* **ci:** Rerun check ([#6637](https://github.com/AztecProtocol/aztec-packages/issues/6637)) ([5ba48fc](https://github.com/AztecProtocol/aztec-packages/commit/5ba48fcd58a1ba801dc69e15fac50c3a7b82817d))
* Cleaning up tx effects related comments ([#6790](https://github.com/AztecProtocol/aztec-packages/issues/6790)) ([fc61d92](https://github.com/AztecProtocol/aztec-packages/commit/fc61d925972ec7ef49b53dbb604ed23a8a07db44))
* Configurable ttl for test runners ([#6629](https://github.com/AztecProtocol/aztec-packages/issues/6629)) ([de50ddd](https://github.com/AztecProtocol/aztec-packages/commit/de50dddc054ae2cd2326e22bfdffcbdf9a5d5642))
* Constraining app_secret_keys_generators ([#6603](https://github.com/AztecProtocol/aztec-packages/issues/6603)) ([be2adc3](https://github.com/AztecProtocol/aztec-packages/commit/be2adc30233b5f2e5c31f8106264eb39e8407109))
* Disable redundant failing acir tests ([#6700](https://github.com/AztecProtocol/aztec-packages/issues/6700)) ([00eed94](https://github.com/AztecProtocol/aztec-packages/commit/00eed9458bdca47d4c4f3f69bba4ed1d19913601))
* Do not use kernel constants for computing tx fee ([#6635](https://github.com/AztecProtocol/aztec-packages/issues/6635)) ([8c1ecf0](https://github.com/AztecProtocol/aztec-packages/commit/8c1ecf0d7d2da378d945774ac3473d24a103a513))
* **docs:** Link source code snippets to appropriate version ([#6828](https://github.com/AztecProtocol/aztec-packages/issues/6828)) ([40ec691](https://github.com/AztecProtocol/aztec-packages/commit/40ec6919fae4a57eab03bdb4c12741f448ff5679))
* **docs:** Update links ([#6776](https://github.com/AztecProtocol/aztec-packages/issues/6776)) ([df86a55](https://github.com/AztecProtocol/aztec-packages/commit/df86a557b14d3043b102b8f930bb0a7c00d9d0e4))
* **docs:** Update token bridge tutorial ([#6809](https://github.com/AztecProtocol/aztec-packages/issues/6809)) ([2a3a098](https://github.com/AztecProtocol/aztec-packages/commit/2a3a0986ebef5522b046a924b8af988f35afde60))
* Don't always run CCI ([#6847](https://github.com/AztecProtocol/aztec-packages/issues/6847)) ([63919d2](https://github.com/AztecProtocol/aztec-packages/commit/63919d2c4ec9b3574347abec15c524b41e9145d7))
* Don't filter AVM changes for running e2e ([#6797](https://github.com/AztecProtocol/aztec-packages/issues/6797)) ([ca2fbf4](https://github.com/AztecProtocol/aztec-packages/commit/ca2fbf4e0bdb9a5395cfdd03bd94d480d2e171c2))
* Dont take down ensure-builder runs early ([#6526](https://github.com/AztecProtocol/aztec-packages/issues/6526)) ([2e8351d](https://github.com/AztecProtocol/aztec-packages/commit/2e8351d807f76858133af2b357cfb8c5736338cc))
* ECCVM correctly handles points at infinity and group operation edge cases ([#6388](https://github.com/AztecProtocol/aztec-packages/issues/6388)) ([a022220](https://github.com/AztecProtocol/aztec-packages/commit/a022220d1eaae3e8a0650899d4c05643c9bc9005))
* Estimate gas cost of contract deployment ([#6710](https://github.com/AztecProtocol/aztec-packages/issues/6710)) ([0f86674](https://github.com/AztecProtocol/aztec-packages/commit/0f86674bb99db37b39bedeff2169f6a08fc2c507))
* **experimental elaborator:** Avoid calling `add_generics` twice on trait methods (https://github.com/noir-lang/noir/pull/5108) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* **experimental elaborator:** Fix duplicate `resolve_type` on self type and don't leak a trait impl's generics (https://github.com/noir-lang/noir/pull/5102) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* **experimental elaborator:** Fix panic in the elaborator (https://github.com/noir-lang/noir/pull/5082) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* **experimental elaborator:** Only call `add_generics` once (https://github.com/noir-lang/noir/pull/5091) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* Fix boxes after npk changes ([#6609](https://github.com/AztecProtocol/aztec-packages/issues/6609)) ([d504c03](https://github.com/AztecProtocol/aztec-packages/commit/d504c035bd31ad98e003a3cc391ac196100a7dfb))
* **frontend:** Call trait method with mut self from generic definition (https://github.com/noir-lang/noir/pull/5041) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* **frontend:** Correctly monomorphize turbofish functions (https://github.com/noir-lang/noir/pull/5049) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Hotfix paths-filter change ([#6737](https://github.com/AztecProtocol/aztec-packages/issues/6737)) ([ded4e22](https://github.com/AztecProtocol/aztec-packages/commit/ded4e2238acf9541154fad1b4276db17645df51e))
* P2p dial ([#6695](https://github.com/AztecProtocol/aztec-packages/issues/6695)) ([8d6d42e](https://github.com/AztecProtocol/aztec-packages/commit/8d6d42e17eadb5601d958dd70d3da6f3f0dc8738))
* **p2p:** IP Query bool check ([#6833](https://github.com/AztecProtocol/aztec-packages/issues/6833)) ([4dc6938](https://github.com/AztecProtocol/aztec-packages/commit/4dc69386db251487883039e67144e515f64d1e4a))
* Proving-friendly bootstrap ([#6791](https://github.com/AztecProtocol/aztec-packages/issues/6791)) ([008cea1](https://github.com/AztecProtocol/aztec-packages/commit/008cea1274815e5dcb50359baa7b0f4dac797d40))
* Read node url ([#6793](https://github.com/AztecProtocol/aztec-packages/issues/6793)) ([7f4051d](https://github.com/AztecProtocol/aztec-packages/commit/7f4051df10d69c16d68fc45a62f14bd020af2dd7))
* Refreshing constants ([#6786](https://github.com/AztecProtocol/aztec-packages/issues/6786)) ([49a3b1d](https://github.com/AztecProtocol/aztec-packages/commit/49a3b1de6274644a1e9e0e9f91f0f657665919c3))
* Release from CCI for 0.42.0 ([#6825](https://github.com/AztecProtocol/aztec-packages/issues/6825)) ([f9d5626](https://github.com/AztecProtocol/aztec-packages/commit/f9d562623d2f0621c331679ea6d3a175df92fe34))
* Remove finalize from acir create circuit ([#6585](https://github.com/AztecProtocol/aztec-packages/issues/6585)) ([f45d20d](https://github.com/AztecProtocol/aztec-packages/commit/f45d20d9340d40efadebcc13b27dd8f1c43f0540))
* Restore boxes in ci workflow ([#6395](https://github.com/AztecProtocol/aztec-packages/issues/6395)) ([aab288b](https://github.com/AztecProtocol/aztec-packages/commit/aab288bdc5bb27e262ca655286b5fa18d7a79af8))
* Run more Bb CI ([#6812](https://github.com/AztecProtocol/aztec-packages/issues/6812)) ([67a8f8a](https://github.com/AztecProtocol/aztec-packages/commit/67a8f8a5e20e28aab2af82b8404a20af1e38906c))
* Run x86 pxe image ([#6784](https://github.com/AztecProtocol/aztec-packages/issues/6784)) ([389d888](https://github.com/AztecProtocol/aztec-packages/commit/389d888cffa5e71a5ab79b850d2715f50f33813f))
* Serialise Fr and Avm inputs ([#6843](https://github.com/AztecProtocol/aztec-packages/issues/6843)) ([2c38f52](https://github.com/AztecProtocol/aztec-packages/commit/2c38f52426078bff62c35e1ba56909a414557422))
* Shared mutable private getter fixes ([#6652](https://github.com/AztecProtocol/aztec-packages/issues/6652)) ([a28cd0a](https://github.com/AztecProtocol/aztec-packages/commit/a28cd0a1170c13a72ff772d8a0054db50ef17776))
* Show account info before proving tx ([#6854](https://github.com/AztecProtocol/aztec-packages/issues/6854)) ([3eb7a98](https://github.com/AztecProtocol/aztec-packages/commit/3eb7a98d52134588ddb216a0b09884013a770dc3))
* Specify timeout in avm_proving.test.ts ([#6689](https://github.com/AztecProtocol/aztec-packages/issues/6689)) ([fbe9fc1](https://github.com/AztecProtocol/aztec-packages/commit/fbe9fc125caef5aeb4db16139cb6ddba420f1a62))
* Tx receipt serialization to JSON ([#6711](https://github.com/AztecProtocol/aztec-packages/issues/6711)) ([1d785fd](https://github.com/AztecProtocol/aztec-packages/commit/1d785fd1087d7387fc29213ca3be50b2fc9c4725))
* UDP comms for AWS ([#6827](https://github.com/AztecProtocol/aztec-packages/issues/6827)) ([f4814d3](https://github.com/AztecProtocol/aztec-packages/commit/f4814d399a64c9a99fd9bc0114abac1328e58bd0))
* Use plain integer addresses for opcodes in DAP disassembly view (https://github.com/noir-lang/noir/pull/4941) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Use sh-friendly test ([#6792](https://github.com/AztecProtocol/aztec-packages/issues/6792)) ([b359639](https://github.com/AztecProtocol/aztec-packages/commit/b359639b637ddd543c7d617a3c6ce0825de7165d))
* Yarn project arm image ([#6782](https://github.com/AztecProtocol/aztec-packages/issues/6782)) ([f0ceca5](https://github.com/AztecProtocol/aztec-packages/commit/f0ceca51ea05b6e163b181f77fd07f15c25cae9e))


### Miscellaneous

* Acir tests in bb ([#6620](https://github.com/AztecProtocol/aztec-packages/issues/6620)) ([a4e001e](https://github.com/AztecProtocol/aztec-packages/commit/a4e001e44562bd0b9cd1f3e5c213f051927b712e))
* Add bench programs ([#6566](https://github.com/AztecProtocol/aztec-packages/issues/6566)) ([edb6db6](https://github.com/AztecProtocol/aztec-packages/commit/edb6db67f6e2b2b369b5d279f1996c7216dc9c69))
* Add e2e full proving tests with padding txs ([#6787](https://github.com/AztecProtocol/aztec-packages/issues/6787)) ([4441e21](https://github.com/AztecProtocol/aztec-packages/commit/4441e2121561e346429295723a6631af7d51e285))
* Add earthly prune workflow ([#6838](https://github.com/AztecProtocol/aztec-packages/issues/6838)) ([eb3d657](https://github.com/AztecProtocol/aztec-packages/commit/eb3d657cf36b1dc5065424f73aa78e008c9472ca))
* Add example for recursion on the CLI ([#6389](https://github.com/AztecProtocol/aztec-packages/issues/6389)) ([e704ff6](https://github.com/AztecProtocol/aztec-packages/commit/e704ff6f64d1a66d4c7f401cd2fa054e1d2c75e3))
* Add l1 to l2 msg read requests to public circuit public inputs ([#6762](https://github.com/AztecProtocol/aztec-packages/issues/6762)) ([69d90c4](https://github.com/AztecProtocol/aztec-packages/commit/69d90c409fc10020cd65e5078a3777545ecc1c85))
* Add linting to avm-transpiler CI ([#6732](https://github.com/AztecProtocol/aztec-packages/issues/6732)) ([57864f9](https://github.com/AztecProtocol/aztec-packages/commit/57864f987d03fa31d4976482c199232d22716127))
* Add note hash read requests to public circuit public inputs ([#6754](https://github.com/AztecProtocol/aztec-packages/issues/6754)) ([42e492e](https://github.com/AztecProtocol/aztec-packages/commit/42e492ee4225adb7120e6409ea06635487aa5073))
* Add retries to noir tests ([#6606](https://github.com/AztecProtocol/aztec-packages/issues/6606)) ([15b0ed5](https://github.com/AztecProtocol/aztec-packages/commit/15b0ed579053d76d80a61ed539c2e76af0b8fcb2))
* Add script to apply sync fixes ([#6731](https://github.com/AztecProtocol/aztec-packages/issues/6731)) ([68e1a08](https://github.com/AztecProtocol/aztec-packages/commit/68e1a0864791be84d2fe494e06af564e72582c86))
* Add serde traits to compressed string ([#6569](https://github.com/AztecProtocol/aztec-packages/issues/6569)) ([65ee122](https://github.com/AztecProtocol/aztec-packages/commit/65ee122a62055a621ad35f3fb7db97317500ea3a))
* Add simple `bb` installer script ([#6376](https://github.com/AztecProtocol/aztec-packages/issues/6376)) ([51bc682](https://github.com/AztecProtocol/aztec-packages/commit/51bc6823db025432b495b5b08a796942a87d1302))
* Automatically clear any unwanted directories in `test_programs` (https://github.com/noir-lang/noir/pull/5081) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* **avm:** AVM Minimial lookup table for testing ([#6641](https://github.com/AztecProtocol/aztec-packages/issues/6641)) ([79beef1](https://github.com/AztecProtocol/aztec-packages/commit/79beef1fdcd40bf0906275bd615441b05d7b9bae))
* **avm:** Better error msgs and some cpp nits ([#6796](https://github.com/AztecProtocol/aztec-packages/issues/6796)) ([f8a9452](https://github.com/AztecProtocol/aztec-packages/commit/f8a9452103c154062748e5ef3351c2cb8825c886))
* **avm:** Disable mem gas accounting ([#6862](https://github.com/AztecProtocol/aztec-packages/issues/6862)) ([12b1b0e](https://github.com/AztecProtocol/aztec-packages/commit/12b1b0eb2930b6d3010658d9fc9edf2523d09537))
* **avm:** Remove portal opcode from avm circuit ([#6706](https://github.com/AztecProtocol/aztec-packages/issues/6706)) ([a790d24](https://github.com/AztecProtocol/aztec-packages/commit/a790d248b1460ab7850ca17e3b5688d6fee83c7e))
* Avoid creating witness for simple multiplications (https://github.com/noir-lang/noir/pull/5100) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* **aztec-nr:** Cleanup context interfaces ([#6799](https://github.com/AztecProtocol/aztec-packages/issues/6799)) ([e37cbbd](https://github.com/AztecProtocol/aztec-packages/commit/e37cbbd8b9c76a973aa9e8662e55a0d10c4f3678))
* Bb changes dont run e2e ([#6794](https://github.com/AztecProtocol/aztec-packages/issues/6794)) ([a728858](https://github.com/AztecProtocol/aztec-packages/commit/a7288580a0d80eaf3ea3142e740bee3360efb446))
* **bb:** -30% compile time ([#6610](https://github.com/AztecProtocol/aztec-packages/issues/6610)) ([d6838a1](https://github.com/AztecProtocol/aztec-packages/commit/d6838a1fa27f57ab4ab60c286b703314c1103ab5))
* **bb:** Better AVM disable in WASM ([#6683](https://github.com/AztecProtocol/aztec-packages/issues/6683)) ([d3cfc4c](https://github.com/AztecProtocol/aztec-packages/commit/d3cfc4cb5eae3a65a394af48c75c3ba271939baa))
* **bb:** Reduce and smooth compile time ([#6688](https://github.com/AztecProtocol/aztec-packages/issues/6688)) ([1253330](https://github.com/AztecProtocol/aztec-packages/commit/125333019df7bde50e4269cd5f05190507d49c17))
* **bb:** Small compile improvements ([#6665](https://github.com/AztecProtocol/aztec-packages/issues/6665)) ([61f27b7](https://github.com/AztecProtocol/aztec-packages/commit/61f27b793258764536892058ad9a80751837739c))
* Bump boxes timeout ([#6636](https://github.com/AztecProtocol/aztec-packages/issues/6636)) ([7dfc369](https://github.com/AztecProtocol/aztec-packages/commit/7dfc369b5a5c1f9735ebe8bd8320b833065a8ef0))
* Checkout sync fix script from master ([#6745](https://github.com/AztecProtocol/aztec-packages/issues/6745)) ([696c03c](https://github.com/AztecProtocol/aztec-packages/commit/696c03cce0832ebcc766f1fe0cd5e1a142c01075))
* Chopping transient logs in ts ([#6708](https://github.com/AztecProtocol/aztec-packages/issues/6708)) ([b9a0d93](https://github.com/AztecProtocol/aztec-packages/commit/b9a0d9336432b2becb4bf26b804f7c49084307fe))
* **ci:** Allow updating lockfile in `sync-fixup.sh` ([#6746](https://github.com/AztecProtocol/aztec-packages/issues/6746)) ([2a16b91](https://github.com/AztecProtocol/aztec-packages/commit/2a16b91c29dc2b6ddaac296eb48b1516ad8060d2))
* **ci:** Always show build summary ([#6612](https://github.com/AztecProtocol/aztec-packages/issues/6612)) ([d606c21](https://github.com/AztecProtocol/aztec-packages/commit/d606c21a8c88e0141ed77d1410e0cee3f6c84cd4))
* **ci:** Automatically retry from failed once ([#6614](https://github.com/AztecProtocol/aztec-packages/issues/6614)) ([fd74b06](https://github.com/AztecProtocol/aztec-packages/commit/fd74b0634a31d6c0231abbbcd3478967d7fd905c))
* **ci:** Better runner recovery from docker image corruption ([#6687](https://github.com/AztecProtocol/aztec-packages/issues/6687)) ([28a20fc](https://github.com/AztecProtocol/aztec-packages/commit/28a20fcd529fcce27e714a4a7794f271819df749))
* **ci:** Delay heavy bb compilation ([#6613](https://github.com/AztecProtocol/aztec-packages/issues/6613)) ([acd07f8](https://github.com/AztecProtocol/aztec-packages/commit/acd07f89f09b35b6ae6a766315cccc1e84581f78))
* **ci:** Don't fail creating launch template ([#6595](https://github.com/AztecProtocol/aztec-packages/issues/6595)) ([2d10848](https://github.com/AztecProtocol/aztec-packages/commit/2d1084815e8d85515bc7bae9419988802c5a45b2))
* **ci:** Ensure ad-hoc apt doesnt time out ([#6755](https://github.com/AztecProtocol/aztec-packages/issues/6755)) ([12921d4](https://github.com/AztecProtocol/aztec-packages/commit/12921d4ed3b7ba7ff8e5909cc5da762b76e7ecaf))
* **ci:** Fix bad image download rescue script ([#6663](https://github.com/AztecProtocol/aztec-packages/issues/6663)) ([1c00cad](https://github.com/AztecProtocol/aztec-packages/commit/1c00cad03f09b4ab1c0d6d8286d4ee70826f888b))
* **ci:** Fix running out of disk ([#6597](https://github.com/AztecProtocol/aztec-packages/issues/6597)) ([f45eb94](https://github.com/AztecProtocol/aztec-packages/commit/f45eb94248ce0ef38344d1fc7e55998fdf31723c))
* **ci:** Merge-check fixes ([#6485](https://github.com/AztecProtocol/aztec-packages/issues/6485)) ([bd0ae42](https://github.com/AztecProtocol/aztec-packages/commit/bd0ae422e9f1b7e955c9b68b4a5238b248e128d4))
* **ci:** More spot retry ([#6617](https://github.com/AztecProtocol/aztec-packages/issues/6617)) ([672329f](https://github.com/AztecProtocol/aztec-packages/commit/672329f2a1649b25f9d3d1fbc0fdd9726e2e654c))
* **ci:** Paths filter fixup ([#6675](https://github.com/AztecProtocol/aztec-packages/issues/6675)) ([33c07a1](https://github.com/AztecProtocol/aztec-packages/commit/33c07a1e7ecbd4895998fbf39e01fce18ce637b7))
* **ci:** Rebuild less ([#6670](https://github.com/AztecProtocol/aztec-packages/issues/6670)) ([ffbf416](https://github.com/AztecProtocol/aztec-packages/commit/ffbf4160e77b2eb56c207da15cfab2921cce844e))
* **ci:** Recover from docker image corruption ([#6638](https://github.com/AztecProtocol/aztec-packages/issues/6638)) ([0750132](https://github.com/AztecProtocol/aztec-packages/commit/07501320e1f20c64b456666a098beeaf7f40fdbf))
* **ci:** Remove invalid input from boxes jobs ([#6733](https://github.com/AztecProtocol/aztec-packages/issues/6733)) ([0d894fa](https://github.com/AztecProtocol/aztec-packages/commit/0d894fa3204e56e892ada6b85b756ad36654c4bc))
* **ci:** Try a different ubuntu runner ([#6740](https://github.com/AztecProtocol/aztec-packages/issues/6740)) ([2e89ae6](https://github.com/AztecProtocol/aztec-packages/commit/2e89ae6240ddc5a2c9eed2aa262bcf4f66c4fe35))
* **ci:** Update graph bug advice ([#6621](https://github.com/AztecProtocol/aztec-packages/issues/6621)) ([131bbfa](https://github.com/AztecProtocol/aztec-packages/commit/131bbfa0b4a3737eb7f33c6406e02bb5875a647a))
* Clarify that ci-consistency is a warning ([#6567](https://github.com/AztecProtocol/aztec-packages/issues/6567)) ([7720ea9](https://github.com/AztecProtocol/aztec-packages/commit/7720ea9d99394fe2a4acf64c0832e82eff02699d))
* Cleanup after outgoing ([#6736](https://github.com/AztecProtocol/aztec-packages/issues/6736)) ([1741b1a](https://github.com/AztecProtocol/aztec-packages/commit/1741b1abe5984270d1402a7140520727215e9057)), closes [#6640](https://github.com/AztecProtocol/aztec-packages/issues/6640)
* Dedup public data writes in kernel ([#6749](https://github.com/AztecProtocol/aztec-packages/issues/6749)) ([e4d75e5](https://github.com/AztecProtocol/aztec-packages/commit/e4d75e5da50583c3aa009aed2985d000e551e599))
* Deduplicate `ReturnConstant` warning (https://github.com/noir-lang/noir/pull/5109) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Delete redundant CircleCI GCC job ([#6712](https://github.com/AztecProtocol/aztec-packages/issues/6712)) ([3e786de](https://github.com/AztecProtocol/aztec-packages/commit/3e786dea3a6ff1b76bf251d1fcd14b05a7204a85))
* Delete spike vm ([#6818](https://github.com/AztecProtocol/aztec-packages/issues/6818)) ([5633ee9](https://github.com/AztecProtocol/aztec-packages/commit/5633ee935f8255564526e72c9cbaf1f5f789e3a1))
* Devnet setup for p2p bootstrap node ([#6660](https://github.com/AztecProtocol/aztec-packages/issues/6660)) ([e0bb743](https://github.com/AztecProtocol/aztec-packages/commit/e0bb743e8f6db7c589f88265e1cf1ee93e3e0c88)), closes [#6513](https://github.com/AztecProtocol/aztec-packages/issues/6513)
* Do less in bench prover ([#6771](https://github.com/AztecProtocol/aztec-packages/issues/6771)) ([b75bbdc](https://github.com/AztecProtocol/aztec-packages/commit/b75bbdc358eee3acc6cd3a5beaeed3218785e520))
* **docs:** Add redirects ([#6581](https://github.com/AztecProtocol/aztec-packages/issues/6581)) ([432cec0](https://github.com/AztecProtocol/aztec-packages/commit/432cec020ed1bb73adcc1d6be8159609c22d328e))
* **docs:** Link to evmdiff for supported EVM chains (https://github.com/noir-lang/noir/pull/5107) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* **docs:** Rewriting absolute paths ([#6511](https://github.com/AztecProtocol/aztec-packages/issues/6511)) ([7aec973](https://github.com/AztecProtocol/aztec-packages/commit/7aec97359180e678dff5be5675a4dc80615da78c))
* **docs:** Zp's nits ([#6608](https://github.com/AztecProtocol/aztec-packages/issues/6608)) ([defd2ba](https://github.com/AztecProtocol/aztec-packages/commit/defd2ba1474dacff6b0bfdbadc31d963450ef9cd))
* Don't skip docs-preview ([#6602](https://github.com/AztecProtocol/aztec-packages/issues/6602)) ([e51726e](https://github.com/AztecProtocol/aztec-packages/commit/e51726e2caba4db3630e2f064ad43aa3ad0a56a1))
* Dont notify on arm failures for now ([#6680](https://github.com/AztecProtocol/aztec-packages/issues/6680)) ([09cfff4](https://github.com/AztecProtocol/aztec-packages/commit/09cfff4e23b746c875756b79820b1cb6043a2949))
* Evaluate expressions in constant gen ([#6813](https://github.com/AztecProtocol/aztec-packages/issues/6813)) ([c2a50f4](https://github.com/AztecProtocol/aztec-packages/commit/c2a50f4d83c7816d589ef08aad71929b612c0f84))
* **experimental:** Add types and traits to the elaborator (https://github.com/noir-lang/noir/pull/5066) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* **experimental:** Elaborate globals (https://github.com/noir-lang/noir/pull/5069) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* **experimental:** Elaborate impls & non-trait impls (https://github.com/noir-lang/noir/pull/5007) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* Fix 0.42 migration notes ([#6785](https://github.com/AztecProtocol/aztec-packages/issues/6785)) ([058856e](https://github.com/AztecProtocol/aztec-packages/commit/058856e37c7fe6c535951a56ab0bcca651bc38ed))
* Fix benchmark comparison with master ([#6598](https://github.com/AztecProtocol/aztec-packages/issues/6598)) ([a30bcc5](https://github.com/AztecProtocol/aztec-packages/commit/a30bcc5a52d2072734cc9a0d1727b32a0c053acb))
* Fix compilation error due to missing import ([#6815](https://github.com/AztecProtocol/aztec-packages/issues/6815)) ([98ba899](https://github.com/AztecProtocol/aztec-packages/commit/98ba89950ceed41a09758a2b4d0dbc4d995f3793))
* Fix unencrypted logs mismatch for AVM with a +4 ([#6580](https://github.com/AztecProtocol/aztec-packages/issues/6580)) ([da82b58](https://github.com/AztecProtocol/aztec-packages/commit/da82b58ff8a84fd12278b5b6bfea90e5521cf6fa))
* Goblin cleanup ([#6722](https://github.com/AztecProtocol/aztec-packages/issues/6722)) ([8e9ab3d](https://github.com/AztecProtocol/aztec-packages/commit/8e9ab3d84cc9430352512da13089afc43a212566))
* Historical apis available on header ([#6601](https://github.com/AztecProtocol/aztec-packages/issues/6601)) ([0b95eb3](https://github.com/AztecProtocol/aztec-packages/commit/0b95eb38cdb71eaec1f9ff077bbbbf0237b0b83a)), closes [#6589](https://github.com/AztecProtocol/aztec-packages/issues/6589)
* Improve aztec-nr testing infra ([#6611](https://github.com/AztecProtocol/aztec-packages/issues/6611)) ([c6f37a1](https://github.com/AztecProtocol/aztec-packages/commit/c6f37a1b92fbe84dc45e6ad4cc22b5ea9b2d02ab))
* Make public data update requests, note hashes, and unencrypted logs readonly in TS ([#6658](https://github.com/AztecProtocol/aztec-packages/issues/6658)) ([1230e56](https://github.com/AztecProtocol/aztec-packages/commit/1230e566698b62591a3bf8f35baa60c454f089e9))
* Move `is_native_field` up into `noirc_frontend` (https://github.com/noir-lang/noir/pull/5119) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* Move turbofish changes to the elaborator (https://github.com/noir-lang/noir/pull/5094) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Note processor test rewrite ([#6836](https://github.com/AztecProtocol/aztec-packages/issues/6836)) ([dbef5e4](https://github.com/AztecProtocol/aztec-packages/commit/dbef5e43d427f08d9bd3b214b370df89ad7c9688))
* Notify Noir team on changes to ACIR format ([#6582](https://github.com/AztecProtocol/aztec-packages/issues/6582)) ([b826310](https://github.com/AztecProtocol/aztec-packages/commit/b8263108395ec46c65f1f96a6165bb46a7b81cd7))
* Nuking broadcast param ([#6741](https://github.com/AztecProtocol/aztec-packages/issues/6741)) ([2d69253](https://github.com/AztecProtocol/aztec-packages/commit/2d692531dbbe56bcac3cf4c9120df70dd3780209))
* Optimizing inclusion proofs tests with historical apis ([#6616](https://github.com/AztecProtocol/aztec-packages/issues/6616)) ([d861364](https://github.com/AztecProtocol/aztec-packages/commit/d8613640745eaf44eb9f477cbe6c6669b594819f)), closes [#6615](https://github.com/AztecProtocol/aztec-packages/issues/6615)
* Perform dead instruction elimination through `std::as_witness` (https://github.com/noir-lang/noir/pull/5123) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Purge unconstrained + batch simulate improvements ([#6639](https://github.com/AztecProtocol/aztec-packages/issues/6639)) ([1945ed9](https://github.com/AztecProtocol/aztec-packages/commit/1945ed9dc72ef2243a09311fa6df62fd3484c374))
* Quick revert ([#6623](https://github.com/AztecProtocol/aztec-packages/issues/6623)) ([94aadbd](https://github.com/AztecProtocol/aztec-packages/commit/94aadbdb3bc5e36aca6b1c951e1d47a4d6364eda))
* Reactivate gates report (https://github.com/noir-lang/noir/pull/5084) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Redo typo PR by dufucun ([#6822](https://github.com/AztecProtocol/aztec-packages/issues/6822)) ([6bd94ff](https://github.com/AztecProtocol/aztec-packages/commit/6bd94ffc445234e31f18ce9b08e736f986e8fa5f))
* Reduce bench history from 10 to 5 blocks ([#6634](https://github.com/AztecProtocol/aztec-packages/issues/6634)) ([c40f8df](https://github.com/AztecProtocol/aztec-packages/commit/c40f8df1d70d37b0681cba3a00d46872f434ca5d))
* Reduce compilation by breaking up acir types.hpp ([#6816](https://github.com/AztecProtocol/aztec-packages/issues/6816)) ([d9f7da3](https://github.com/AztecProtocol/aztec-packages/commit/d9f7da39b02c41641fbf15a13882f39905b7a583))
* Reduce p2p bootnode machine requirements ([#6841](https://github.com/AztecProtocol/aztec-packages/issues/6841)) ([f0a5c49](https://github.com/AztecProtocol/aztec-packages/commit/f0a5c49d65dc602abd6bae41aea69a772205cb11))
* Refactor and reenable account init fees tests ([#6691](https://github.com/AztecProtocol/aztec-packages/issues/6691)) ([8bb658d](https://github.com/AztecProtocol/aztec-packages/commit/8bb658db1598e6ea36e3d9ee1a5b095f89a1e702))
* Release Noir(0.30.0) (https://github.com/noir-lang/noir/pull/4981) ([221e247](https://github.com/AztecProtocol/aztec-packages/commit/221e2479622aef8e70120dc0a9f91ffcbc99efba))
* Remove acir goblin flow ([#6724](https://github.com/AztecProtocol/aztec-packages/issues/6724)) ([f035231](https://github.com/AztecProtocol/aztec-packages/commit/f035231ca5cde4592be599a2e7b4ce706cdee27d))
* Remove ACIR public execution simulator tests ([#6805](https://github.com/AztecProtocol/aztec-packages/issues/6805)) ([e1f73e1](https://github.com/AztecProtocol/aztec-packages/commit/e1f73e1949479fa1f8ed6f50076e98ab59e65b20))
* Remove aes slice ([#6550](https://github.com/AztecProtocol/aztec-packages/issues/6550)) ([f44d567](https://github.com/AztecProtocol/aztec-packages/commit/f44d567ec39f3806a1d24b6bc526c78611381034))
* Remove CLI register-account ([#6853](https://github.com/AztecProtocol/aztec-packages/issues/6853)) ([5f00f17](https://github.com/AztecProtocol/aztec-packages/commit/5f00f179f17de868e1ff895cd6adfe7fd827d234))
* Remove duplicated code from LSP (https://github.com/noir-lang/noir/pull/5116) ([46c2ad0](https://github.com/AztecProtocol/aztec-packages/commit/46c2ad0b551a37e74118a789a1ea32a2daa1f849))
* Remove prover-pool ([#6727](https://github.com/AztecProtocol/aztec-packages/issues/6727)) ([5833f29](https://github.com/AztecProtocol/aztec-packages/commit/5833f29445dd757b9ab6b755d6cae09d11e9414a))
* Remove public_execution_context ([#6804](https://github.com/AztecProtocol/aztec-packages/issues/6804)) ([b78695a](https://github.com/AztecProtocol/aztec-packages/commit/b78695a334e3afb017593f8ec8da3f1568e9fb66))
* Remove unused closing tag from `indexed_merkle_tree.mdx` ([#6729](https://github.com/AztecProtocol/aztec-packages/issues/6729)) ([03fa925](https://github.com/AztecProtocol/aztec-packages/commit/03fa92512f2fc70d02547ce469429a1f381c09e9))
* Remove warnings field from `DebugArtifact` (https://github.com/noir-lang/noir/pull/5118) ([a37895c](https://github.com/AztecProtocol/aztec-packages/commit/a37895c984e35454eac654d7ce23199267340765))
* Replace relative paths to noir-protocol-circuits ([8c71587](https://github.com/AztecProtocol/aztec-packages/commit/8c71587583cd939be9e2f202c596be65302ee862))
* Replace relative paths to noir-protocol-circuits ([0b1dba8](https://github.com/AztecProtocol/aztec-packages/commit/0b1dba8ba311b44b82d32929194941d8211f94cc))
* Replace relative paths to noir-protocol-circuits ([ac365a0](https://github.com/AztecProtocol/aztec-packages/commit/ac365a0efd655f259df5c6b7d4f0ba513cfa608d))
* Replace relative paths to noir-protocol-circuits ([735eed4](https://github.com/AztecProtocol/aztec-packages/commit/735eed4e8160f3f8338ca022bfda9d2274568862))
* Replace relative paths to noir-protocol-circuits ([f18deec](https://github.com/AztecProtocol/aztec-packages/commit/f18deecc36be0296639ae87e28527903209e772b))
* Replace relative paths to noir-protocol-circuits ([8fbf439](https://github.com/AztecProtocol/aztec-packages/commit/8fbf43989d081df74eb1d000fdb358a0a25f48fc))
* Replace relative paths to noir-protocol-circuits ([2979aa9](https://github.com/AztecProtocol/aztec-packages/commit/2979aa9a665cf406530d0f6039cd7473cb75a973))
* Replace relative paths to noir-protocol-circuits ([72b9ddb](https://github.com/AztecProtocol/aztec-packages/commit/72b9ddb57065795d8a57b4b9cf83163ccaab90ea))
* Replace relative paths to noir-protocol-circuits ([66f213e](https://github.com/AztecProtocol/aztec-packages/commit/66f213e7eefaa84f75e98d6776dc3f3f47bdec21))
* Replace relative paths to noir-protocol-circuits ([681ef79](https://github.com/AztecProtocol/aztec-packages/commit/681ef790b2655c4a5501fe21cb474c425355afc9))
* Replace relative paths to noir-protocol-circuits ([7d19259](https://github.com/AztecProtocol/aztec-packages/commit/7d19259bef8dd8ddf7b0303c5102e3650027de42))
* Replace relative paths to noir-protocol-circuits ([2b8dc76](https://github.com/AztecProtocol/aztec-packages/commit/2b8dc76330fb19f948feaaf00e6752c2b2fcbc2c))
* Replace relative paths to noir-protocol-circuits ([5708181](https://github.com/AztecProtocol/aztec-packages/commit/5708181affb9e432f46b07ebdb4429d489eca47e))
* Sort public call requests in the circuits ([#6650](https://github.com/AztecProtocol/aztec-packages/issues/6650)) ([f67d6f3](https://github.com/AztecProtocol/aztec-packages/commit/f67d6f38b03225849e99c02d069073a504ce7c55))
* Stop building/publishing `acvm_backend.wasm` ([#6584](https://github.com/AztecProtocol/aztec-packages/issues/6584)) ([7a3a491](https://github.com/AztecProtocol/aztec-packages/commit/7a3a491c85092e0a6cd21948e0fe4e9c736be27a))
* Test cleanup ([#6649](https://github.com/AztecProtocol/aztec-packages/issues/6649)) ([a90370d](https://github.com/AztecProtocol/aztec-packages/commit/a90370d2afc6f05b214beab1a14e4da39fdc5150))
* Tool to analyze C++ compilation time ([#6823](https://github.com/AztecProtocol/aztec-packages/issues/6823)) ([101e966](https://github.com/AztecProtocol/aztec-packages/commit/101e9664f290ee601f2a8587ddf931d2e27e1d0f))
* Try to improve boxes caching ([#6664](https://github.com/AztecProtocol/aztec-packages/issues/6664)) ([b12aa40](https://github.com/AztecProtocol/aztec-packages/commit/b12aa40c56eb1d0109a1d53a74f70c798bde3571))
* Try to lower CI costs ([#6645](https://github.com/AztecProtocol/aztec-packages/issues/6645)) ([069a407](https://github.com/AztecProtocol/aztec-packages/commit/069a40715fd99395d6f7d5c7bdcaa02759498a4f))
* Ultra goblin --&gt; mega ([#6674](https://github.com/AztecProtocol/aztec-packages/issues/6674)) ([d272abd](https://github.com/AztecProtocol/aztec-packages/commit/d272abd1b332aae8d062b2340afd5cabf61e31d9))
* Update `nargo info` table to remove circuit size column ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Update docs to represent zksync supporting necessary precompiles (https://github.com/noir-lang/noir/pull/5071) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Updated image hash ([#6842](https://github.com/AztecProtocol/aztec-packages/issues/6842)) ([4133e06](https://github.com/AztecProtocol/aztec-packages/commit/4133e0646f2370a5c70b3a255e6fa2a49f787753))
* Use `bbup` to install `bb` (https://github.com/noir-lang/noir/pull/5073) ([17f6e1d](https://github.com/AztecProtocol/aztec-packages/commit/17f6e1de5363a7db85cb02d3ae0e6f35397a2a58))
* Use only 1 tx in e2e avm proving ([#6832](https://github.com/AztecProtocol/aztec-packages/issues/6832)) ([7f1c302](https://github.com/AztecProtocol/aztec-packages/commit/7f1c302e28f22c47c091b805ad9e1ad5a7dc6b8e))
* Workaround earthly graph bug ([#6789](https://github.com/AztecProtocol/aztec-packages/issues/6789)) ([10e822a](https://github.com/AztecProtocol/aztec-packages/commit/10e822a195d2ce10e27178ad0de0e10d22f5daec))


### Documentation

* Fix minor formatting ([#6704](https://github.com/AztecProtocol/aztec-packages/issues/6704)) ([9e05f28](https://github.com/AztecProtocol/aztec-packages/commit/9e05f2893a28b119c938a43d482b7b2a93af2ae4))
* Update docusaurus.config.js - pretty diffs ([#6594](https://github.com/AztecProtocol/aztec-packages/issues/6594)) ([27e3c90](https://github.com/AztecProtocol/aztec-packages/commit/27e3c900f30e465b54d2a38818d25e971424dd6e))

## [0.41.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.40.1...aztec-packages-v0.41.0) (2024-05-21)


### ⚠ BREAKING CHANGES

* compile-time incorrect exec environment errors ([#6442](https://github.com/AztecProtocol/aztec-packages/issues/6442))
* add is_infinite to curve addition opcode ([#6384](https://github.com/AztecProtocol/aztec-packages/issues/6384))
* remove backend interactions from `nargo` ([#6320](https://github.com/AztecProtocol/aztec-packages/issues/6320))

### Features

* `npk_m_hash` in all notes + key rotation test ([#6405](https://github.com/AztecProtocol/aztec-packages/issues/6405)) ([74e98d4](https://github.com/AztecProtocol/aztec-packages/commit/74e98d413b4135a1c84da9cef08ddb547a9c0bd5))
* Add encrypted log outgoing body ([#6334](https://github.com/AztecProtocol/aztec-packages/issues/6334)) ([fa9f442](https://github.com/AztecProtocol/aztec-packages/commit/fa9f442b8fcb73f5606cc2bb4da7bc2eb01b142e))
* Add first version of kernel reset circuit ([#6393](https://github.com/AztecProtocol/aztec-packages/issues/6393)) ([ed6df8e](https://github.com/AztecProtocol/aztec-packages/commit/ed6df8e4f8f8242c01df2cc763833a45a144f57e))
* Add is_infinite to curve addition opcode ([#6384](https://github.com/AztecProtocol/aztec-packages/issues/6384)) ([75d81c5](https://github.com/AztecProtocol/aztec-packages/commit/75d81c5fccf52d270239261bab79dd1fde41c19a))
* Add native rust implementations of pedersen functions (https://github.com/noir-lang/noir/pull/4871) ([8bbbbb6](https://github.com/AztecProtocol/aztec-packages/commit/8bbbbb645e7f12b535b1dc683b5ab5aea0b73e5b))
* Add nullifying key to Token Note ([#6130](https://github.com/AztecProtocol/aztec-packages/issues/6130)) ([95c6b4a](https://github.com/AztecProtocol/aztec-packages/commit/95c6b4af7db62162935b123277cfae51c23d30d3))
* Adding autogenerated variants for the reset circuit ([#6508](https://github.com/AztecProtocol/aztec-packages/issues/6508)) ([8e8d2dd](https://github.com/AztecProtocol/aztec-packages/commit/8e8d2ddd5eefed4f5ca7248b882dfc3d8287d1bb))
* **avm-simulator:** Cap gas for external calls ([#6479](https://github.com/AztecProtocol/aztec-packages/issues/6479)) ([c8771ba](https://github.com/AztecProtocol/aztec-packages/commit/c8771ba7c75fc2395ed0b12a117a2d3eb5ab6983))
* **avm:** Gzip avm bytecode ([#6475](https://github.com/AztecProtocol/aztec-packages/issues/6475)) ([29559bd](https://github.com/AztecProtocol/aztec-packages/commit/29559bd3ef28d7f208ebd7052fd85a8a4cd23436))
* **avm:** To_radix gadget ([#6368](https://github.com/AztecProtocol/aztec-packages/issues/6368)) ([89dd25f](https://github.com/AztecProtocol/aztec-packages/commit/89dd25f2b25f720def6cac003ce204e92de66c47))
* Benchmark private proving ([#6409](https://github.com/AztecProtocol/aztec-packages/issues/6409)) ([e9e5526](https://github.com/AztecProtocol/aztec-packages/commit/e9e5526178ef6b8228138d10acd9a1df5cdeb90c))
* Compile-time incorrect exec environment errors ([#6442](https://github.com/AztecProtocol/aztec-packages/issues/6442)) ([0f75efd](https://github.com/AztecProtocol/aztec-packages/commit/0f75efd5050bc4044d8c80dd6bd2ecd9fffb511b))
* Do not return databus returndata, keep it private. (https://github.com/noir-lang/noir/pull/5023) ([26f2197](https://github.com/AztecProtocol/aztec-packages/commit/26f2197b01331577bb499234050cc33a71c47f05))
* **docs:** Authwit how tos ([#6220](https://github.com/AztecProtocol/aztec-packages/issues/6220)) ([78f13d9](https://github.com/AztecProtocol/aztec-packages/commit/78f13d9e0070502a980003c7ded3738fd353dd5a))
* **docs:** Key rotation / owner -&gt; nullifier key docs ([#6538](https://github.com/AztecProtocol/aztec-packages/issues/6538)) ([2453ba8](https://github.com/AztecProtocol/aztec-packages/commit/2453ba8f2ba193df829ec2d4937d8ae3770373d3))
* Full encryption and decryption of log in ts ([#6348](https://github.com/AztecProtocol/aztec-packages/issues/6348)) ([0ac83dc](https://github.com/AztecProtocol/aztec-packages/commit/0ac83dc8e65b87652a4bc3f4f931bfc23c7f41aa))
* Generic key validation request ([#6474](https://github.com/AztecProtocol/aztec-packages/issues/6474)) ([948ec38](https://github.com/AztecProtocol/aztec-packages/commit/948ec383b30c4f467b6da6591fa518ce793fc54d))
* Improved ClientIvc ([#6429](https://github.com/AztecProtocol/aztec-packages/issues/6429)) ([f360b3f](https://github.com/AztecProtocol/aztec-packages/commit/f360b3fd30b9dd1e80e5f1a3d42c325c0f54f8ed))
* Laying out a new recursion constraint for honk ([#6489](https://github.com/AztecProtocol/aztec-packages/issues/6489)) ([af9fea4](https://github.com/AztecProtocol/aztec-packages/commit/af9fea4bbafe1a41b09d9351a34a896db2c8ab7d))
* New docs structure ([#6195](https://github.com/AztecProtocol/aztec-packages/issues/6195)) ([9cca814](https://github.com/AztecProtocol/aztec-packages/commit/9cca8146db4c5eb4a505b5909f3b078f83916a71))
* Pay out arbitrary fee to coinbase on L1 ([#6436](https://github.com/AztecProtocol/aztec-packages/issues/6436)) ([1b99de8](https://github.com/AztecProtocol/aztec-packages/commit/1b99de81e58a97fb47604d2e94582e6f227f98dd))
* Remove total logs len from pre tail kernels + add to L1 ([#6466](https://github.com/AztecProtocol/aztec-packages/issues/6466)) ([66a2d43](https://github.com/AztecProtocol/aztec-packages/commit/66a2d43432607ec43eaac5b0ee7ac69f44d18d92))
* Run benchmarks for ACIR proving ([#6155](https://github.com/AztecProtocol/aztec-packages/issues/6155)) ([ebf6fc2](https://github.com/AztecProtocol/aztec-packages/commit/ebf6fc2313c82b97d9ccd8c36caee42fb7a1c901))
* Squash transient note logs ([#6268](https://github.com/AztecProtocol/aztec-packages/issues/6268)) ([4574877](https://github.com/AztecProtocol/aztec-packages/commit/457487795c6bce1db336b2ba80060ad016dd1265))
* Sum transaction fees and pay on l1 ([#6522](https://github.com/AztecProtocol/aztec-packages/issues/6522)) ([bf441da](https://github.com/AztecProtocol/aztec-packages/commit/bf441da243405744caa9d5422e1b8a2676efba8b))
* Translator recursive verifier ([#6327](https://github.com/AztecProtocol/aztec-packages/issues/6327)) ([9321aef](https://github.com/AztecProtocol/aztec-packages/commit/9321aef1a49eb33ea388838ba7b0c00dddd9c898))
* Update the encrypted note log format ([#6411](https://github.com/AztecProtocol/aztec-packages/issues/6411)) ([e5cc9dc](https://github.com/AztecProtocol/aztec-packages/commit/e5cc9dccb6c36159ad90068d41786c8715af66da))
* Validate counters ([#6365](https://github.com/AztecProtocol/aztec-packages/issues/6365)) ([1f28b3a](https://github.com/AztecProtocol/aztec-packages/commit/1f28b3a622e603f47f88b20361abef559952a5af))
* View functions with static context enforcing ([#6338](https://github.com/AztecProtocol/aztec-packages/issues/6338)) ([22ad5a5](https://github.com/AztecProtocol/aztec-packages/commit/22ad5a5728afce5dcf32c8e6d8025691081e0de1))
* Vk_as_fields, proof_as_fields flows for honk ([#6406](https://github.com/AztecProtocol/aztec-packages/issues/6406)) ([a6100ad](https://github.com/AztecProtocol/aztec-packages/commit/a6100ad3d5126321d457b5c336ab4a3521ff1fb2))


### Bug Fixes

* Arm ci ([#6480](https://github.com/AztecProtocol/aztec-packages/issues/6480)) ([237952e](https://github.com/AztecProtocol/aztec-packages/commit/237952e9fe5ea46585580c168421b6cdcdbf64e5))
* Asset struct serialization does not match Noir internal serialization ([#6494](https://github.com/AztecProtocol/aztec-packages/issues/6494)) ([9e6a4c3](https://github.com/AztecProtocol/aztec-packages/commit/9e6a4c3f37f7ebc3e91ca124dc6d643f5a16ecf7))
* **avm-simulator:** Actually wrap oracles ([#6449](https://github.com/AztecProtocol/aztec-packages/issues/6449)) ([8685acc](https://github.com/AztecProtocol/aztec-packages/commit/8685acc7df4c61cedde4c336f8523ead340ef5e2))
* **avm-simulator:** Nested calls should preserve static context ([#6414](https://github.com/AztecProtocol/aztec-packages/issues/6414)) ([44d7916](https://github.com/AztecProtocol/aztec-packages/commit/44d79163a4ded1f24463a7cee8306b303f98d266))
* **avm-simulator:** Pending storage and nullifiers should be accessible in grandchild nested calls ([#6428](https://github.com/AztecProtocol/aztec-packages/issues/6428)) ([84d2e1f](https://github.com/AztecProtocol/aztec-packages/commit/84d2e1faf9a0bbee670cdf13992b21d9e58871b3))
* Buggy e2e key registry test setup ([#6496](https://github.com/AztecProtocol/aztec-packages/issues/6496)) ([52d85d1](https://github.com/AztecProtocol/aztec-packages/commit/52d85d12269e4a58300c25653a0c9485ae3a6572))
* **ci:** ARM ([#6521](https://github.com/AztecProtocol/aztec-packages/issues/6521)) ([d1095f6](https://github.com/AztecProtocol/aztec-packages/commit/d1095f60bbd05d35748dc9b0188ad0c5f87390f5))
* **ci:** Arm concurrency ([#6564](https://github.com/AztecProtocol/aztec-packages/issues/6564)) ([a265da0](https://github.com/AztecProtocol/aztec-packages/commit/a265da0a0e4cb5666c91812dc725f5620ad9b740))
* Disable buggy ClientIVC tests ([#6546](https://github.com/AztecProtocol/aztec-packages/issues/6546)) ([b61dea3](https://github.com/AztecProtocol/aztec-packages/commit/b61dea36947a203457b6f9fe0943f3d28e8aab01))
* Disk attach edge case ([#6430](https://github.com/AztecProtocol/aztec-packages/issues/6430)) ([2366ad3](https://github.com/AztecProtocol/aztec-packages/commit/2366ad39b3e351e3b1b75b798db4d09cb1c26afd))
* **docs:** Clarify content on portals ([#6431](https://github.com/AztecProtocol/aztec-packages/issues/6431)) ([869df4d](https://github.com/AztecProtocol/aztec-packages/commit/869df4d217ccb944ec66adf4aefc2e61173d9f69))
* Don't start multiple runners during RequestLimitExceeded ([#6444](https://github.com/AztecProtocol/aztec-packages/issues/6444)) ([7c4c822](https://github.com/AztecProtocol/aztec-packages/commit/7c4c8226cb2f642c1e4d7a3a6add9b9065454986))
* Dont start multiple builders ([#6437](https://github.com/AztecProtocol/aztec-packages/issues/6437)) ([d67ab1c](https://github.com/AztecProtocol/aztec-packages/commit/d67ab1cb00002af55ff58404e8edb48990e65efb))
* Fix no predicates for brillig with intermediate functions (https://github.com/noir-lang/noir/pull/5015) ([26f2197](https://github.com/AztecProtocol/aztec-packages/commit/26f2197b01331577bb499234050cc33a71c47f05))
* Fixed several vulnerabilities in U128, added some tests (https://github.com/noir-lang/noir/pull/5024) ([26f2197](https://github.com/AztecProtocol/aztec-packages/commit/26f2197b01331577bb499234050cc33a71c47f05))
* Increase N_max in Zeromorph ([#6415](https://github.com/AztecProtocol/aztec-packages/issues/6415)) ([9e643b4](https://github.com/AztecProtocol/aztec-packages/commit/9e643b429b22a1b8905ede07ab2e9561f42a1a89))
* Quick fix of [#6405](https://github.com/AztecProtocol/aztec-packages/issues/6405) by removing context from value note utils ([#6509](https://github.com/AztecProtocol/aztec-packages/issues/6509)) ([3a4d828](https://github.com/AztecProtocol/aztec-packages/commit/3a4d82857326ceb099cbd2af307cc6836027dfd1))
* Removed plain from path in array args of contract interfaces ([#6497](https://github.com/AztecProtocol/aztec-packages/issues/6497)) ([2b37729](https://github.com/AztecProtocol/aztec-packages/commit/2b37729e07fc16b560a60ddc0713cafba3aa5704))
* Runs-on inconsistency and simplify concurrency keys ([#6433](https://github.com/AztecProtocol/aztec-packages/issues/6433)) ([80674d9](https://github.com/AztecProtocol/aztec-packages/commit/80674d9fa20f53dea857ab6c5bc79c6d13c1aadb))
* Spot retry fixup ([#6476](https://github.com/AztecProtocol/aztec-packages/issues/6476)) ([784d784](https://github.com/AztecProtocol/aztec-packages/commit/784d78404ac9145902c75dbe9898f872174350f4))


### Miscellaneous

* Add benchmarks for pedersen and schnorr verification (https://github.com/noir-lang/noir/pull/5056) ([8bbbbb6](https://github.com/AztecProtocol/aztec-packages/commit/8bbbbb645e7f12b535b1dc683b5ab5aea0b73e5b))
* Add c++ tests for generator derivation ([#6528](https://github.com/AztecProtocol/aztec-packages/issues/6528)) ([72931bd](https://github.com/AztecProtocol/aztec-packages/commit/72931bdb8202c34042cdfb8cee2ef44b75939879))
* Add script to print lines of code (https://github.com/noir-lang/noir/pull/4991) ([26f2197](https://github.com/AztecProtocol/aztec-packages/commit/26f2197b01331577bb499234050cc33a71c47f05))
* Add some docs on syncing noir ([#6340](https://github.com/AztecProtocol/aztec-packages/issues/6340)) ([bb68fcd](https://github.com/AztecProtocol/aztec-packages/commit/bb68fcd7cbfc2a2fa213da0005dc3a45bc1d6482))
* Anvil kill wrapper now supports mac ([#6520](https://github.com/AztecProtocol/aztec-packages/issues/6520)) ([2a5d975](https://github.com/AztecProtocol/aztec-packages/commit/2a5d975a09dcdd5adfcb8fc14af711ad3c40c022))
* **avm:** Wrap oracles with unconstrained fns ([#6421](https://github.com/AztecProtocol/aztec-packages/issues/6421)) ([3e7e094](https://github.com/AztecProtocol/aztec-packages/commit/3e7e094ff3ba60253c457ba081d1d4c4cc192296))
* Bump earthly ([#6419](https://github.com/AztecProtocol/aztec-packages/issues/6419)) ([3d78751](https://github.com/AztecProtocol/aztec-packages/commit/3d787515faeae084b54042ae338d6070d34d6d2c))
* Bump maximum nullifier read requests (necessary for e2e tests in AVM) ([#6462](https://github.com/AztecProtocol/aztec-packages/issues/6462)) ([26eac62](https://github.com/AztecProtocol/aztec-packages/commit/26eac620b22e3e4b19491884fe46ea3950ff5802))
* Bump maximum nullifier read requests (necessary for e2e tests in AVM) ([#6495](https://github.com/AztecProtocol/aztec-packages/issues/6495)) ([90d8092](https://github.com/AztecProtocol/aztec-packages/commit/90d80926cb6f8f7ae3c5f791e0386f4f313c7d90))
* Change some error messages for avm switch ([#6447](https://github.com/AztecProtocol/aztec-packages/issues/6447)) ([74d6519](https://github.com/AztecProtocol/aztec-packages/commit/74d6519d7a98a019db5e46d5c188c7479fb51430))
* **ci:** Better retry defaults ([#6472](https://github.com/AztecProtocol/aztec-packages/issues/6472)) ([b23f1fd](https://github.com/AztecProtocol/aztec-packages/commit/b23f1fdb0ccf2d5493f03cd5227aca7cb117bdbe))
* **ci:** Consistency as external check ([#6460](https://github.com/AztecProtocol/aztec-packages/issues/6460)) ([6793a75](https://github.com/AztecProtocol/aztec-packages/commit/6793a75ab1f4da67ad35114b55adecfb2fe90d9a))
* **ci:** Dont detach ebs ([#6441](https://github.com/AztecProtocol/aztec-packages/issues/6441)) ([f933fc0](https://github.com/AztecProtocol/aztec-packages/commit/f933fc0b722024251abc05dcaad35f41ffe25c60))
* **ci:** Fix on-demand starting ([#6434](https://github.com/AztecProtocol/aztec-packages/issues/6434)) ([c3efb9c](https://github.com/AztecProtocol/aztec-packages/commit/c3efb9cc1d8569d74c537bdf26851108b2d0ef0d))
* **ci:** Increase timeouts ([#6426](https://github.com/AztecProtocol/aztec-packages/issues/6426)) ([44986fe](https://github.com/AztecProtocol/aztec-packages/commit/44986feda565e0b4177ea69fd5440df281941471))
* **ci:** Only run circleci on master ([#6525](https://github.com/AztecProtocol/aztec-packages/issues/6525)) ([c75fbd4](https://github.com/AztecProtocol/aztec-packages/commit/c75fbd44c4b564e703ed9e33e948368eadc0867a))
* **ci:** Push l1-contracts tests off of critical path ([#6400](https://github.com/AztecProtocol/aztec-packages/issues/6400)) ([ce0ae6d](https://github.com/AztecProtocol/aztec-packages/commit/ce0ae6d8248d4adf9815db9339f0b86593b86fd2))
* **ci:** Reenable arm build ([#6455](https://github.com/AztecProtocol/aztec-packages/issues/6455)) ([2862767](https://github.com/AztecProtocol/aztec-packages/commit/2862767cb2d44d561ac136f3503e629a93775e65))
* **ci:** Require setup in merge-check ([#6454](https://github.com/AztecProtocol/aztec-packages/issues/6454)) ([ad73061](https://github.com/AztecProtocol/aztec-packages/commit/ad73061b191c20e5e4881928539f2642eeacc9fa))
* **ci:** Spot capacity and reaping ([#6561](https://github.com/AztecProtocol/aztec-packages/issues/6561)) ([8c639b5](https://github.com/AztecProtocol/aztec-packages/commit/8c639b509251f2919d9b83096d16a2d8fb5ce7f3))
* Clean up kernel types by removing is_static from function_data ([#6557](https://github.com/AztecProtocol/aztec-packages/issues/6557)) ([83ba29f](https://github.com/AztecProtocol/aztec-packages/commit/83ba29fa05528f055b5faa7ec4777019328144b2))
* Cleanup the encrypted log incoming body ([#6325](https://github.com/AztecProtocol/aztec-packages/issues/6325)) ([e88c209](https://github.com/AztecProtocol/aztec-packages/commit/e88c209965f862a1478422980d0a1a9a3df46295))
* Copy subset of constants to cpp ([#6544](https://github.com/AztecProtocol/aztec-packages/issues/6544)) ([21dc72a](https://github.com/AztecProtocol/aztec-packages/commit/21dc72aaf29ada2c1a12682d3763370c76eff524))
* Do not rebuild yarn-projects on bench-comment ([#6396](https://github.com/AztecProtocol/aztec-packages/issues/6396)) ([797115b](https://github.com/AztecProtocol/aztec-packages/commit/797115b82ae595be16d8ad887a0c310e7b53afa4))
* **docs:** Adding analytics ([#6350](https://github.com/AztecProtocol/aztec-packages/issues/6350)) ([6417cd9](https://github.com/AztecProtocol/aztec-packages/commit/6417cd905ab032c6724b6c07082e81f67d5f750f))
* **docs:** Restructure improvs ([#6502](https://github.com/AztecProtocol/aztec-packages/issues/6502)) ([c3b573e](https://github.com/AztecProtocol/aztec-packages/commit/c3b573e82e3fdc1feca6ce861951d2dd93e4f9b3))
* Fix linter issues in AVM ([#6057](https://github.com/AztecProtocol/aztec-packages/issues/6057)) ([c2e72b1](https://github.com/AztecProtocol/aztec-packages/commit/c2e72b1b8dbf7b9eb414d0db17dbbf7acc8a3b54))
* Fix logs upload to S3 ([#6401](https://github.com/AztecProtocol/aztec-packages/issues/6401)) ([9df0602](https://github.com/AztecProtocol/aztec-packages/commit/9df06021e58c40b879f630544ab8887d4e546d55))
* Fix migration notes ([#6458](https://github.com/AztecProtocol/aztec-packages/issues/6458)) ([bee85a9](https://github.com/AztecProtocol/aztec-packages/commit/bee85a952bca1ca3aa6e08ce440f933cfbe94307))
* Fix migration notes ([#6551](https://github.com/AztecProtocol/aztec-packages/issues/6551)) ([89bc350](https://github.com/AztecProtocol/aztec-packages/commit/89bc350575076c8a6a7d25a6f687884b76803aa8))
* Fix notes 0.41.0 ([#6461](https://github.com/AztecProtocol/aztec-packages/issues/6461)) ([04b0ec5](https://github.com/AztecProtocol/aztec-packages/commit/04b0ec563b656de363cf78b55a6eed4783bfbb52))
* Fix poor performance and long compile times in value_note.derement() ([#6523](https://github.com/AztecProtocol/aztec-packages/issues/6523)) ([002b4aa](https://github.com/AztecProtocol/aztec-packages/commit/002b4aa556041aa1a12f0fd09bb5ad0b07f04daa))
* Fix tester image copy pattern ([#6438](https://github.com/AztecProtocol/aztec-packages/issues/6438)) ([b892eae](https://github.com/AztecProtocol/aztec-packages/commit/b892eae79997438fa5351b77766900b1afae5823))
* Get_nullifier_keys cleanup ([#6451](https://github.com/AztecProtocol/aztec-packages/issues/6451)) ([8a71fd5](https://github.com/AztecProtocol/aztec-packages/commit/8a71fd5a1a8d7a59302ac5671536d5b505b8cf23))
* Lower max public bytecode to 20k ([#6477](https://github.com/AztecProtocol/aztec-packages/issues/6477)) ([ce192f0](https://github.com/AztecProtocol/aztec-packages/commit/ce192f0804d1d00ecf800198a4a5fda5a364a502))
* Move `UPLOAD_LOGS` into root earthfile ([#6424](https://github.com/AztecProtocol/aztec-packages/issues/6424)) ([d723da9](https://github.com/AztecProtocol/aztec-packages/commit/d723da9ff49e47908a78ac7eedc4cae025861316))
* Nuking `KeyStore` and `KeyPair` interfaces ([#6553](https://github.com/AztecProtocol/aztec-packages/issues/6553)) ([23e0518](https://github.com/AztecProtocol/aztec-packages/commit/23e0518fc46eb7308f93e65df7080278c2d732cf))
* Parameterise cycle_group by `Builder` rather than `Composer` ([#6565](https://github.com/AztecProtocol/aztec-packages/issues/6565)) ([ea36bf9](https://github.com/AztecProtocol/aztec-packages/commit/ea36bf9bbd5e22ba4c566b08a4c8410e46175c70))
* Prefetch noir deps in earthly for caching ([#6556](https://github.com/AztecProtocol/aztec-packages/issues/6556)) ([8ee9060](https://github.com/AztecProtocol/aztec-packages/commit/8ee9060858a9ee9d9b1741b7ba550bfaadd5e6d4))
* Private call validation ([#6510](https://github.com/AztecProtocol/aztec-packages/issues/6510)) ([07dc072](https://github.com/AztecProtocol/aztec-packages/commit/07dc0726501bc78d691e1d2360dda84d1a93b9c5))
* Purge secret and open keywords ([#6501](https://github.com/AztecProtocol/aztec-packages/issues/6501)) ([f9c74c4](https://github.com/AztecProtocol/aztec-packages/commit/f9c74c4195739ea10af77dfc307d2c32ee13dfd8)), closes [#5538](https://github.com/AztecProtocol/aztec-packages/issues/5538)
* Recommend Noir and rust plugin ([#6558](https://github.com/AztecProtocol/aztec-packages/issues/6558)) ([298561f](https://github.com/AztecProtocol/aztec-packages/commit/298561fde56c843962dab7733247bc037e34d841))
* Refactor key rotate and address comments from 6405 ([#6450](https://github.com/AztecProtocol/aztec-packages/issues/6450)) ([6f3dab8](https://github.com/AztecProtocol/aztec-packages/commit/6f3dab87f1ccae2afd5da4dfae7ef7f4ee4797ce))
* Remove acvmInfo from bb.js CLI ([#6507](https://github.com/AztecProtocol/aztec-packages/issues/6507)) ([e298c76](https://github.com/AztecProtocol/aztec-packages/commit/e298c766d17029a9dbfce694b48327c5e76dfddb))
* Remove backend interactions from `nargo` ([#6320](https://github.com/AztecProtocol/aztec-packages/issues/6320)) ([7a31896](https://github.com/AztecProtocol/aztec-packages/commit/7a318964f67dc844f15efe3faa40b33f4a4fad47))
* Replace relative paths to noir-protocol-circuits ([94ab877](https://github.com/AztecProtocol/aztec-packages/commit/94ab87786b2b83af40033e93b8fe1edb68b5e4f9))
* Replace relative paths to noir-protocol-circuits ([9f04bfe](https://github.com/AztecProtocol/aztec-packages/commit/9f04bfea51a0c4fe980bbbcde5867089e8f5d8a5))
* Replace relative paths to noir-protocol-circuits ([67f29e5](https://github.com/AztecProtocol/aztec-packages/commit/67f29e5ca14d4502cd4f1e04f1526597a0e89f58))
* Replace relative paths to noir-protocol-circuits ([a7a4b86](https://github.com/AztecProtocol/aztec-packages/commit/a7a4b86ce2501c1995219ae8136f9e94fcae8a0d))
* Replace relative paths to noir-protocol-circuits ([c6f61a4](https://github.com/AztecProtocol/aztec-packages/commit/c6f61a47c5d4dc0213d02b9389131f28c9644869))
* Share decider with ultra_prover ([#5467](https://github.com/AztecProtocol/aztec-packages/issues/5467)) ([b3b7376](https://github.com/AztecProtocol/aztec-packages/commit/b3b7376161f353a273bf26d42e435667b41cc5e2))
* Switch over to constructing gates report in bash ([#6491](https://github.com/AztecProtocol/aztec-packages/issues/6491)) ([1fa5963](https://github.com/AztecProtocol/aztec-packages/commit/1fa59637a0829208d382d1dded36df33f4d61582))
* **tests:** Change error messages in preparation for AVM ([#6422](https://github.com/AztecProtocol/aztec-packages/issues/6422)) ([6616dc6](https://github.com/AztecProtocol/aztec-packages/commit/6616dc6ef382d605c0f94585bfee86c8469dc1e3))


### Documentation

* Sumcheck documentation ([#5841](https://github.com/AztecProtocol/aztec-packages/issues/5841)) ([116eef0](https://github.com/AztecProtocol/aztec-packages/commit/116eef06be3991fa03482425780715e6f78791ea))
* Updating key docs in concepts section ([#6387](https://github.com/AztecProtocol/aztec-packages/issues/6387)) ([921a7f4](https://github.com/AztecProtocol/aztec-packages/commit/921a7f4b9dce5fdc7e2f1978d6aba81908d38ede))

## [0.40.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.40.0...aztec-packages-v0.40.1) (2024-05-14)


### Bug Fixes

* **avm-simulator:** Unencrypted logs hash mismatch in kernel ([#6399](https://github.com/AztecProtocol/aztec-packages/issues/6399)) ([35645e4](https://github.com/AztecProtocol/aztec-packages/commit/35645e4c8b06ad78bb3078559a2d68d4dff70a73))
* **avm-transpiler:** Fix cast to u1 ([#6402](https://github.com/AztecProtocol/aztec-packages/issues/6402)) ([16ab1f7](https://github.com/AztecProtocol/aztec-packages/commit/16ab1f70752a4b167667a338d8c3e215a6554002))
* **unconstrained:** Add missing debugLog oracle ([#6397](https://github.com/AztecProtocol/aztec-packages/issues/6397)) ([87eb8ab](https://github.com/AztecProtocol/aztec-packages/commit/87eb8abf325385ced21be9c6e8029eea0a699570))

## [0.40.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.39.0...aztec-packages-v0.40.0) (2024-05-14)


### ⚠ BREAKING CHANGES

* debug logs for all ([#6392](https://github.com/AztecProtocol/aztec-packages/issues/6392))

### Features

* Add wasmtime ([#6314](https://github.com/AztecProtocol/aztec-packages/issues/6314)) ([ea6ccdd](https://github.com/AztecProtocol/aztec-packages/commit/ea6ccddff253a984f9b71200dca5c0117855abed))
* Debug logs for all ([#6392](https://github.com/AztecProtocol/aztec-packages/issues/6392)) ([10afa13](https://github.com/AztecProtocol/aztec-packages/commit/10afa13dfc85b02ace4c38e1fb347539d8041c21))
* Demonstrating use of nsk_app to check nullification ([#6362](https://github.com/AztecProtocol/aztec-packages/issues/6362)) ([ddf4461](https://github.com/AztecProtocol/aztec-packages/commit/ddf4461a4c24e16cbac3e8cd49099fa70d5b5075))
* Use gas estimation in aztecjs contract function interactions ([#6260](https://github.com/AztecProtocol/aztec-packages/issues/6260)) ([18192ac](https://github.com/AztecProtocol/aztec-packages/commit/18192ac66c82114524a232f2d9fc6dd6ed5ccffb))


### Miscellaneous

* Add more serialisation traits to protocol circuits ([#6385](https://github.com/AztecProtocol/aztec-packages/issues/6385)) ([97d5422](https://github.com/AztecProtocol/aztec-packages/commit/97d54220791a6069ffde0c53ca0f304e1624ae4e))
* **ci:** Bump timeout of prover-client-test ([#6394](https://github.com/AztecProtocol/aztec-packages/issues/6394)) ([d05cd07](https://github.com/AztecProtocol/aztec-packages/commit/d05cd07e534528b5cca4eac2adb052b3ac6b023f))
* Reenable bench summary ([#6211](https://github.com/AztecProtocol/aztec-packages/issues/6211)) ([713b243](https://github.com/AztecProtocol/aztec-packages/commit/713b24351dac51b3f0109d9783c9154d6290140b))
* **token-contract-tests:** Change intrinsic assertion messages ([#6386](https://github.com/AztecProtocol/aztec-packages/issues/6386)) ([aca81ae](https://github.com/AztecProtocol/aztec-packages/commit/aca81ae5bfd49a647c49f327ebd0328e5d6ed6a9))

## [0.39.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.38.0...aztec-packages-v0.39.0) (2024-05-14)


### ⚠ BREAKING CHANGES

* switch `bb` over to read ACIR from nargo artifacts ([#6283](https://github.com/AztecProtocol/aztec-packages/issues/6283))
* shared mutable configurable delays ([#6104](https://github.com/AztecProtocol/aztec-packages/issues/6104))
* specify databus arrays for BB ([#6239](https://github.com/AztecProtocol/aztec-packages/issues/6239))

### Features

* Add `Not` trait to stdlib (https://github.com/noir-lang/noir/pull/4999) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Add support for u16/i16 (https://github.com/noir-lang/noir/pull/4985) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Avm support for public input columns ([#5700](https://github.com/AztecProtocol/aztec-packages/issues/5700)) ([8cf9168](https://github.com/AztecProtocol/aztec-packages/commit/8cf9168c61d8f2bdee5cc29763df6c888422a0bc))
* **avm-simulator:** Add to_radix_le instruction ([#6308](https://github.com/AztecProtocol/aztec-packages/issues/6308)) ([6374a32](https://github.com/AztecProtocol/aztec-packages/commit/6374a328859eefed0346a3c12b3500dd960e0884))
* **avm-simulator:** Error stack tracking and enriching in AVM to match ACVM/ACIR-SIM ([#6289](https://github.com/AztecProtocol/aztec-packages/issues/6289)) ([5c1f895](https://github.com/AztecProtocol/aztec-packages/commit/5c1f8959ee55856d66acd80a8a90ed18da52efaa))
* **aztec-nr:** Add 'with_gas()' function to avm call interface ([#6256](https://github.com/AztecProtocol/aztec-packages/issues/6256)) ([0aedd23](https://github.com/AztecProtocol/aztec-packages/commit/0aedd23067154e7de4819583251a188e860acd85))
* **aztec-nr:** Add enqueue functions to AvmCallInterface ([#6264](https://github.com/AztecProtocol/aztec-packages/issues/6264)) ([1c74387](https://github.com/AztecProtocol/aztec-packages/commit/1c74387e56b49102043fc6701735325a891e6c65))
* Build-images as earthly. ([#6194](https://github.com/AztecProtocol/aztec-packages/issues/6194)) ([67fedf1](https://github.com/AztecProtocol/aztec-packages/commit/67fedf1a4a93aed9c1ee1e14a21f4b098dde995e))
* Div opcode ([#6053](https://github.com/AztecProtocol/aztec-packages/issues/6053)) ([8e111f8](https://github.com/AztecProtocol/aztec-packages/commit/8e111f8bab5a0348fe8c7185f89e979541f91a67))
* Encrypted log body ([#6251](https://github.com/AztecProtocol/aztec-packages/issues/6251)) ([ba618d5](https://github.com/AztecProtocol/aztec-packages/commit/ba618d5aa715f5f45988bb5aae4638d4091a6786))
* Enforce note hash read requests to read within own contract ([#6310](https://github.com/AztecProtocol/aztec-packages/issues/6310)) ([bd10595](https://github.com/AztecProtocol/aztec-packages/commit/bd10595a5275ac2c2da06bf4f839e4f86ec36c81))
* Expose `set_as_fee_payer` and test it in e2e ([#6380](https://github.com/AztecProtocol/aztec-packages/issues/6380)) ([a8274f3](https://github.com/AztecProtocol/aztec-packages/commit/a8274f30337d7776d454379fa917d4b21f63845e))
* Implement `ops` traits on `u16`/`i16` (https://github.com/noir-lang/noir/pull/4996) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Increase default expression width to 4 (https://github.com/noir-lang/noir/pull/4995) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Move abi demonomorphizer to noir_codegen and use noir_codegen in protocol types ([#6302](https://github.com/AztecProtocol/aztec-packages/issues/6302)) ([690e500](https://github.com/AztecProtocol/aztec-packages/commit/690e5001e732295aa93eaf90726ba68106011d7f))
* Move to_radix to a blackbox ([#6294](https://github.com/AztecProtocol/aztec-packages/issues/6294)) ([ac27376](https://github.com/AztecProtocol/aztec-packages/commit/ac27376b9a0cdf0624a02d36c64ec25886b44b4a))
* **p2p:** GossibSub ([#6170](https://github.com/AztecProtocol/aztec-packages/issues/6170)) ([98d32f1](https://github.com/AztecProtocol/aztec-packages/commit/98d32f112971e6cc96896ddd2c95500f61ba3e8d)), closes [#5055](https://github.com/AztecProtocol/aztec-packages/issues/5055)
* Plumb fee payer ([#6286](https://github.com/AztecProtocol/aztec-packages/issues/6286)) ([1f8fd1c](https://github.com/AztecProtocol/aztec-packages/commit/1f8fd1c4c215bccc0a6f43f4be8a5afa8abb0af8))
* Private Kernel Recursion ([#6278](https://github.com/AztecProtocol/aztec-packages/issues/6278)) ([eae5822](https://github.com/AztecProtocol/aztec-packages/commit/eae5822cfcf47d03739e09911c183ba9f4ced18b))
* Proper padding in ts AES and constrained AES in body and header computations ([#6269](https://github.com/AztecProtocol/aztec-packages/issues/6269)) ([ef9cdde](https://github.com/AztecProtocol/aztec-packages/commit/ef9cdde09d6cdd8a5deb0217fea1e828477f0c03))
* PublicKeys struct ([#6333](https://github.com/AztecProtocol/aztec-packages/issues/6333)) ([2633cfc](https://github.com/AztecProtocol/aztec-packages/commit/2633cfccef8513151f80e5d43b2baf58e3c244e9))
* Re-enabling authwit constraint ([#6323](https://github.com/AztecProtocol/aztec-packages/issues/6323)) ([aa06d55](https://github.com/AztecProtocol/aztec-packages/commit/aa06d55df8366d34669857e2b78c677f957ae6c2)), closes [#5830](https://github.com/AztecProtocol/aztec-packages/issues/5830)
* Remove query to backend to get expression width (https://github.com/noir-lang/noir/pull/4975) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Replacing mentions to aztec-starter with codespace methods ([#6177](https://github.com/AztecProtocol/aztec-packages/issues/6177)) ([63e8788](https://github.com/AztecProtocol/aztec-packages/commit/63e87881b2dbf0dc5f3359297854f0eab32efb0e))
* Return gas usage per phase from node tx simulation ([#6255](https://github.com/AztecProtocol/aztec-packages/issues/6255)) ([fb58dfc](https://github.com/AztecProtocol/aztec-packages/commit/fb58dfcb935735ed3dba6f60ba98fb9a62577a69))
* Shared mutable configurable delays ([#6104](https://github.com/AztecProtocol/aztec-packages/issues/6104)) ([c191a40](https://github.com/AztecProtocol/aztec-packages/commit/c191a40bebf5910d4001f3fac61bb7235f805104))
* Small translator optimisations ([#6354](https://github.com/AztecProtocol/aztec-packages/issues/6354)) ([ba6c42e](https://github.com/AztecProtocol/aztec-packages/commit/ba6c42e24bbb0b3876699c979b36638b15560764))
* Specify databus arrays for BB ([#6239](https://github.com/AztecProtocol/aztec-packages/issues/6239)) ([01d9f24](https://github.com/AztecProtocol/aztec-packages/commit/01d9f24d2f089f7ce6e522e31e77c1e70177d8ef))
* Structured trace in client ivc ([#6132](https://github.com/AztecProtocol/aztec-packages/issues/6132)) ([92c1478](https://github.com/AztecProtocol/aztec-packages/commit/92c14780a7cdec87173d1ec9a22675ca13bf1ae7))
* Switch `bb` over to read ACIR from nargo artifacts ([#6283](https://github.com/AztecProtocol/aztec-packages/issues/6283)) ([78adcc0](https://github.com/AztecProtocol/aztec-packages/commit/78adcc0f6bd74d7ead6de58099dda1a3f88eefb0))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4993) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* ToRadix BB + avm transpiler support ([#6330](https://github.com/AztecProtocol/aztec-packages/issues/6330)) ([c3c602f](https://github.com/AztecProtocol/aztec-packages/commit/c3c602f75ce2224489dfd2490ee7e991aca9d48f))
* **vm:** Reading kernel state opcodes ([#5739](https://github.com/AztecProtocol/aztec-packages/issues/5739)) ([3250a8a](https://github.com/AztecProtocol/aztec-packages/commit/3250a8a217646fd369f491100c644f73a8fe99e4))


### Bug Fixes

* `CombinedConstantData` not registered for serialization ([#6292](https://github.com/AztecProtocol/aztec-packages/issues/6292)) ([89ab8ee](https://github.com/AztecProtocol/aztec-packages/commit/89ab8eeab35dfeae36efbb1ae159c6600f40e059))
* **avm-context:** Enqueueing of public from private ([#6299](https://github.com/AztecProtocol/aztec-packages/issues/6299)) ([bd2ccf0](https://github.com/AztecProtocol/aztec-packages/commit/bd2ccf0bd58f66bed0846617ac2a737f4a619262))
* **avm-simulator:** Always set revertReason when reverting ([#6297](https://github.com/AztecProtocol/aztec-packages/issues/6297)) ([cc59981](https://github.com/AztecProtocol/aztec-packages/commit/cc59981a8f69375c4ca92999a12a955e0d385ada))
* **avm-simulator:** Correctly create call stack in shallow assertions ([#6274](https://github.com/AztecProtocol/aztec-packages/issues/6274)) ([f6045fd](https://github.com/AztecProtocol/aztec-packages/commit/f6045fdb9dd44edf4025aaaa12c5be2e1fc3d9fb))
* **avm-simulator:** Fix env getters ([#6357](https://github.com/AztecProtocol/aztec-packages/issues/6357)) ([485fe40](https://github.com/AztecProtocol/aztec-packages/commit/485fe40aecb1af1ece834417797b85e557ff9ae5))
* **avm-simulator:** Fix message sender ([#6331](https://github.com/AztecProtocol/aztec-packages/issues/6331)) ([f7e2d26](https://github.com/AztecProtocol/aztec-packages/commit/f7e2d260799b44219c6b85b2542626e5691383da))
* **avm-simulator:** Fix test expectation ([#6293](https://github.com/AztecProtocol/aztec-packages/issues/6293)) ([f51acfa](https://github.com/AztecProtocol/aztec-packages/commit/f51acfaade686ffab0bde7d91c97a13280b9e2c6))
* **avm-simulator:** Rethrow nested assertions ([#6275](https://github.com/AztecProtocol/aztec-packages/issues/6275)) ([cd05b91](https://github.com/AztecProtocol/aztec-packages/commit/cd05b91a1c70af9dca54cd2c717745022388614e))
* **avm-transpiler:** Patch debug infos with modified PCs ([#6371](https://github.com/AztecProtocol/aztec-packages/issues/6371)) ([c36f0fa](https://github.com/AztecProtocol/aztec-packages/commit/c36f0fa993b42fcdccf44b4370f300798d475883))
* Check for public args in aztec functions ([#6355](https://github.com/AztecProtocol/aztec-packages/issues/6355)) ([219efd6](https://github.com/AztecProtocol/aztec-packages/commit/219efd605981f9ca643461f91667afa0352dd906))
* **ci:** Bench list ([#6282](https://github.com/AztecProtocol/aztec-packages/issues/6282)) ([2652576](https://github.com/AztecProtocol/aztec-packages/commit/26525764396ccfb2176e47a1016d194244b374f9))
* **circuits.js:** Fix nullifier non existent hints ([#6346](https://github.com/AztecProtocol/aztec-packages/issues/6346)) ([297779a](https://github.com/AztecProtocol/aztec-packages/commit/297779ae64af33687256f1ced11b1aee3fd29946))
* **ci:** Stop mass serialization ([#6290](https://github.com/AztecProtocol/aztec-packages/issues/6290)) ([60104e9](https://github.com/AztecProtocol/aztec-packages/commit/60104e9ff00ab5b39ee94310816f1e1098af6f53))
* Defer overflow checks for unsigned integers to acir-gen (https://github.com/noir-lang/noir/pull/4832) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* Enable client proof tests ([#6249](https://github.com/AztecProtocol/aztec-packages/issues/6249)) ([6d3a800](https://github.com/AztecProtocol/aztec-packages/commit/6d3a800b9088764d162a061dd4c4f6e13f5fedc5))
* Ignore no_predicates in brillig functions (https://github.com/noir-lang/noir/pull/5012) ([3cda21a](https://github.com/AztecProtocol/aztec-packages/commit/3cda21a9e2ff598232fe0119a235e98463ec718b))
* Noir_js import ([#6381](https://github.com/AztecProtocol/aztec-packages/issues/6381)) ([e9c7e5f](https://github.com/AztecProtocol/aztec-packages/commit/e9c7e5f7a400efc746a734ab38bae6ab3d80cdad))
* Pw/update merge check ([#6201](https://github.com/AztecProtocol/aztec-packages/issues/6201)) ([856657f](https://github.com/AztecProtocol/aztec-packages/commit/856657fbd1f82b7526b3ff0214e3e6758db214e3))
* Run noir browser tests in series ([#6232](https://github.com/AztecProtocol/aztec-packages/issues/6232)) ([e092514](https://github.com/AztecProtocol/aztec-packages/commit/e09251498ee085586e8b3dee465a073628d497bf))
* Temporarily revert to_radix blackbox ([#6304](https://github.com/AztecProtocol/aztec-packages/issues/6304)) ([044d0fe](https://github.com/AztecProtocol/aztec-packages/commit/044d0fef3bbecf673c579bd63d2640dc81b35ba3))


### Miscellaneous

* `CompleteAddress` cleanup ([#6300](https://github.com/AztecProtocol/aztec-packages/issues/6300)) ([9c30759](https://github.com/AztecProtocol/aztec-packages/commit/9c30759ad9d45bc14f487b602837228392fab44f)), closes [#5834](https://github.com/AztecProtocol/aztec-packages/issues/5834)
* Adding name shadowing tests template program (https://github.com/noir-lang/noir/pull/4799) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* **avm-context:** Implement Empty ([#6303](https://github.com/AztecProtocol/aztec-packages/issues/6303)) ([27534ac](https://github.com/AztecProtocol/aztec-packages/commit/27534aca901c74e2754e5c27d62ad686756e90d1))
* **avm-simulator:** Add U128 overflow tests to AVM simulator ([#6281](https://github.com/AztecProtocol/aztec-packages/issues/6281)) ([5514143](https://github.com/AztecProtocol/aztec-packages/commit/5514143aab1db195aa466752e1e476d95a993a08))
* Bump public bytecode size to 40000 in prep for AVM migration ([#6266](https://github.com/AztecProtocol/aztec-packages/issues/6266)) ([2b61123](https://github.com/AztecProtocol/aztec-packages/commit/2b6112318551c9e72f78970706ed8a572147cfc9))
* Bump timeout for after-hook for data store test ([#6364](https://github.com/AztecProtocol/aztec-packages/issues/6364)) ([18eca39](https://github.com/AztecProtocol/aztec-packages/commit/18eca39b35057f7943155a2696b3f7c05ec27266))
* **ci:** Fix master, better spot copy times ([#6374](https://github.com/AztecProtocol/aztec-packages/issues/6374)) ([fee7649](https://github.com/AztecProtocol/aztec-packages/commit/fee764922b1dee6062f9dd7f9776dee6186740b7))
* **ci:** Hotfix runner checks ([#6373](https://github.com/AztecProtocol/aztec-packages/issues/6373)) ([d5fd668](https://github.com/AztecProtocol/aztec-packages/commit/d5fd668e555062da7c844528a0680bb67a0575c8))
* **ci:** Reuse ssh connections ([#6382](https://github.com/AztecProtocol/aztec-packages/issues/6382)) ([5f6c31e](https://github.com/AztecProtocol/aztec-packages/commit/5f6c31ee622e30223f7b852e96d07d9b95969c47))
* **ci:** Revert inline cache push for now ([#6318](https://github.com/AztecProtocol/aztec-packages/issues/6318)) ([4c9bfb0](https://github.com/AztecProtocol/aztec-packages/commit/4c9bfb040c667da1e5ebff06ed55864a8a7094ed))
* **ci:** Run clippy on benchmarks (https://github.com/noir-lang/noir/pull/4988) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* **ci:** Run e2e on isolated spots ([#6287](https://github.com/AztecProtocol/aztec-packages/issues/6287)) ([e7d2dd6](https://github.com/AztecProtocol/aztec-packages/commit/e7d2dd617209f2603af81d9f5e89dd62bbb13fa1))
* **ci:** Spot health fix, earthly workarounds ([#6379](https://github.com/AztecProtocol/aztec-packages/issues/6379)) ([da7573c](https://github.com/AztecProtocol/aztec-packages/commit/da7573cd33b77e0169645252874bfcbbd2ea1c29))
* **ci:** Stability after spot changes ([#6367](https://github.com/AztecProtocol/aztec-packages/issues/6367)) ([7ad4179](https://github.com/AztecProtocol/aztec-packages/commit/7ad41790fa4aa698a0cf43735e91d7a76dbe2674))
* **ci:** Use on-demand runners ([#6311](https://github.com/AztecProtocol/aztec-packages/issues/6311)) ([dba835d](https://github.com/AztecProtocol/aztec-packages/commit/dba835d1a1c6214cf4a4c2a62e4bcee49bf83e10))
* Deploying accounts after key registry ([#6322](https://github.com/AztecProtocol/aztec-packages/issues/6322)) ([84878d1](https://github.com/AztecProtocol/aztec-packages/commit/84878d17fa7c0fb4ffa4a20628741f96869d851d))
* Disable `gates_report.yml` (https://github.com/noir-lang/noir/pull/4997) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* **docs:** Update contract deployments page ([#6319](https://github.com/AztecProtocol/aztec-packages/issues/6319)) ([2e331b5](https://github.com/AztecProtocol/aztec-packages/commit/2e331b53bae16121c41b2348a035aa9737d4a64f))
* **dsl:** Update backend gateCount command to query a Program in a single request ([#6228](https://github.com/AztecProtocol/aztec-packages/issues/6228)) ([8079f60](https://github.com/AztecProtocol/aztec-packages/commit/8079f601a23219ddd96f01064d0c31c6e8109471))
* Enforce formatting of noir code ([#6271](https://github.com/AztecProtocol/aztec-packages/issues/6271)) ([356f7bb](https://github.com/AztecProtocol/aztec-packages/commit/356f7bb88576cc88fb82e1868706f90aac65fd0a))
* **experimental:** Add compiler option to enable the Elaborator (https://github.com/noir-lang/noir/pull/5003) ([3cda21a](https://github.com/AztecProtocol/aztec-packages/commit/3cda21a9e2ff598232fe0119a235e98463ec718b))
* **experimental:** Add Elaborator pass (https://github.com/noir-lang/noir/pull/4992) ([3cda21a](https://github.com/AztecProtocol/aztec-packages/commit/3cda21a9e2ff598232fe0119a235e98463ec718b))
* Make coinbase and fee_recipient inaccessible ([#6375](https://github.com/AztecProtocol/aztec-packages/issues/6375)) ([ded28b7](https://github.com/AztecProtocol/aztec-packages/commit/ded28b799a14a7305ceacdc5075f02ea4f26522f))
* Make MSM builder more explicit ([#6110](https://github.com/AztecProtocol/aztec-packages/issues/6110)) ([40306b6](https://github.com/AztecProtocol/aztec-packages/commit/40306b6d5ea01bf191288b0a3bca6fdbeae9912f))
* Pw/refactor bb prover ([#6349](https://github.com/AztecProtocol/aztec-packages/issues/6349)) ([8eb0398](https://github.com/AztecProtocol/aztec-packages/commit/8eb039821d187449fe4ae702e1714937b0796c28))
* Remove `bb info` command ([#6276](https://github.com/AztecProtocol/aztec-packages/issues/6276)) ([f0a1c89](https://github.com/AztecProtocol/aztec-packages/commit/f0a1c89a064c1e170db4751be46874f089dd1385))
* Replace relative paths to noir-protocol-circuits ([fd40d99](https://github.com/AztecProtocol/aztec-packages/commit/fd40d99f07c64483efb1067652aa1aa3d456555f))
* Replace relative paths to noir-protocol-circuits ([53dbcb5](https://github.com/AztecProtocol/aztec-packages/commit/53dbcb5679ccccd95d6133c7d57494d651be7de9))
* Replace relative paths to noir-protocol-circuits ([48e07c3](https://github.com/AztecProtocol/aztec-packages/commit/48e07c39ffa41f8dcab37f216180a26423cf3a87))
* Replace relative paths to noir-protocol-circuits ([6532725](https://github.com/AztecProtocol/aztec-packages/commit/65327254f4e95ca41634d9d86c206cfc777668bf))
* Replace relative paths to noir-protocol-circuits ([8330f70](https://github.com/AztecProtocol/aztec-packages/commit/8330f70b6813d70f8a98d2d120185cf7420624f5))
* Replace relative paths to noir-protocol-circuits ([484741a](https://github.com/AztecProtocol/aztec-packages/commit/484741aa23186652ec31271175bcd1d1d9ab3026))
* Replacing old pub key oracle with get_ivpk_m ([#6219](https://github.com/AztecProtocol/aztec-packages/issues/6219)) ([9acc9ec](https://github.com/AztecProtocol/aztec-packages/commit/9acc9ec0065cee2a1a0bfd9f649377ccc6afe7fe))
* Siloing in tails ([#6167](https://github.com/AztecProtocol/aztec-packages/issues/6167)) ([c20dd50](https://github.com/AztecProtocol/aztec-packages/commit/c20dd501f2eff024034f4d6f267f9489d58d6f9d))
* Simplify nargo CLI to read from artifacts ([#6279](https://github.com/AztecProtocol/aztec-packages/issues/6279)) ([b2c019b](https://github.com/AztecProtocol/aztec-packages/commit/b2c019b6b11c3aaa98d8bbb79b77b42a5f87f0d0))
* Skip formatting informattable comments ([#6288](https://github.com/AztecProtocol/aztec-packages/issues/6288)) ([95b499b](https://github.com/AztecProtocol/aztec-packages/commit/95b499bead8b05afcb4cac8c7a12832ce7c7bfcd))
* Split `ops` into `arith` and `bit` modules (https://github.com/noir-lang/noir/pull/4989) ([11cde44](https://github.com/AztecProtocol/aztec-packages/commit/11cde4434060807e4ee5fcb39268c6e8dbcc4a45))
* **test-contracts:** Prepare e2e_token_contract+ error msgs for AVM migration ([#6307](https://github.com/AztecProtocol/aztec-packages/issues/6307)) ([0c20f44](https://github.com/AztecProtocol/aztec-packages/commit/0c20f44f10b6436cafab690a9d6d5a888b37b4ee))
* Update cspell for abi demonomorphizer ([#6258](https://github.com/AztecProtocol/aztec-packages/issues/6258)) ([ce2d43c](https://github.com/AztecProtocol/aztec-packages/commit/ce2d43c8793755ff54ce363d94e420afac3ef657))
* Update serialisation ([#6378](https://github.com/AztecProtocol/aztec-packages/issues/6378)) ([527129d](https://github.com/AztecProtocol/aztec-packages/commit/527129d6f9e624716642a78b0744c3f99ed8e1a1))
* Validating private call data ([#6316](https://github.com/AztecProtocol/aztec-packages/issues/6316)) ([84b9fcd](https://github.com/AztecProtocol/aztec-packages/commit/84b9fcdc6ef011b593ccae09cbb17b9f38b389ef))


### Documentation

* Call types ([#5472](https://github.com/AztecProtocol/aztec-packages/issues/5472)) ([1ca0d28](https://github.com/AztecProtocol/aztec-packages/commit/1ca0d28d4931e7461bcb00ef77d412b9ade02630))
* Re-add and update accounts docs ([#6345](https://github.com/AztecProtocol/aztec-packages/issues/6345)) ([4926d15](https://github.com/AztecProtocol/aztec-packages/commit/4926d15b779778bc18f36fd9d2d2e207db9ee45a))
* Updated protocol specs ([#6341](https://github.com/AztecProtocol/aztec-packages/issues/6341)) ([a0f82db](https://github.com/AztecProtocol/aztec-packages/commit/a0f82dbbfc2edba6e9c2a15fe6c9ec3d9220da23))

## [0.38.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.37.0...aztec-packages-v0.38.0) (2024-05-07)


### ⚠ BREAKING CHANGES

* AES blackbox ([#6016](https://github.com/AztecProtocol/aztec-packages/issues/6016))

### Features

* `multi_scalar_mul` blackbox func ([#6097](https://github.com/AztecProtocol/aztec-packages/issues/6097)) ([f6b1ba6](https://github.com/AztecProtocol/aztec-packages/commit/f6b1ba60daf37a5a6466ca1e5ee7be70354af485))
* Add `Neg` trait to stdlib (https://github.com/noir-lang/noir/pull/4983) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* Add ciphertext computation for log header ([#6175](https://github.com/AztecProtocol/aztec-packages/issues/6175)) ([3e05534](https://github.com/AztecProtocol/aztec-packages/commit/3e0553456535cd32743f7cf33e51ffd8a36ff75d))
* Add proving retries ([#6145](https://github.com/AztecProtocol/aztec-packages/issues/6145)) ([39ab99c](https://github.com/AztecProtocol/aztec-packages/commit/39ab99c3d0c819094b7eb39edd22c81322ca4627))
* Add public teardown to circuit structs ([#6191](https://github.com/AztecProtocol/aztec-packages/issues/6191)) ([03e1b93](https://github.com/AztecProtocol/aztec-packages/commit/03e1b937db09dc64ac73960285849c4dd88e1f01))
* AES blackbox ([#6016](https://github.com/AztecProtocol/aztec-packages/issues/6016)) ([e4b97a8](https://github.com/AztecProtocol/aztec-packages/commit/e4b97a8cd7574a828c2a54b4a93b5ced79df6abf))
* Always including debug data in a function artifact ([#6223](https://github.com/AztecProtocol/aztec-packages/issues/6223)) ([5d6d22c](https://github.com/AztecProtocol/aztec-packages/commit/5d6d22ca416c6471428b56a55968e859334caa6a))
* **avm-simulator:** Consider previous pending nullifiers across enqueued calls ([#6188](https://github.com/AztecProtocol/aztec-packages/issues/6188)) ([4676431](https://github.com/AztecProtocol/aztec-packages/commit/4676431ecf18003c6648e914effb1c3087108f0f))
* **avm-simulator:** Make storage work across enqueued calls ([#6181](https://github.com/AztecProtocol/aztec-packages/issues/6181)) ([8e218a2](https://github.com/AztecProtocol/aztec-packages/commit/8e218a22c1f85e7b0de4afc4219a860e6bbab7fb))
* **avm:** Add TransactionFee opcode to simulator ([#6210](https://github.com/AztecProtocol/aztec-packages/issues/6210)) ([fcac844](https://github.com/AztecProtocol/aztec-packages/commit/fcac84451f657bb4a70c496538b443dda5bc961e))
* Complex outputs from acir call (https://github.com/noir-lang/noir/pull/4952) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Expose set_public_teardown_function in private context ([#6199](https://github.com/AztecProtocol/aztec-packages/issues/6199)) ([4d8b51c](https://github.com/AztecProtocol/aztec-packages/commit/4d8b51caf477ff83390ec6b40f11b0768e57903f))
* Handle empty response foreign calls without an external resolver (https://github.com/noir-lang/noir/pull/4959) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Hash logs inside circuit ([#5934](https://github.com/AztecProtocol/aztec-packages/issues/5934)) ([6b99527](https://github.com/AztecProtocol/aztec-packages/commit/6b99527881345d7aa0dc90cfc61832432d817587))
* Honk flows exposed through wasm ([#6096](https://github.com/AztecProtocol/aztec-packages/issues/6096)) ([c9b3206](https://github.com/AztecProtocol/aztec-packages/commit/c9b32061b2849442516ff0395b69d9a230191234))
* Implement `From` array trait for `BoundedVec` (https://github.com/noir-lang/noir/pull/4927) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* Include transaction fee in txreceipt ([#6139](https://github.com/AztecProtocol/aztec-packages/issues/6139)) ([6785512](https://github.com/AztecProtocol/aztec-packages/commit/6785512fff9dfec77bec5ce1580880c7ae21dce8))
* Making keys getters complete ([#6171](https://github.com/AztecProtocol/aztec-packages/issues/6171)) ([e85dde9](https://github.com/AztecProtocol/aztec-packages/commit/e85dde9743c4e2e6c2f0dfd7bf487a2b4234d2b5))
* Move noir-tests to earthly ([#6185](https://github.com/AztecProtocol/aztec-packages/issues/6185)) ([4daea40](https://github.com/AztecProtocol/aztec-packages/commit/4daea40fc8d994f25321ee6359ad37321ccd99dd))
* Note hash read requests fixes and refactoring ([#6125](https://github.com/AztecProtocol/aztec-packages/issues/6125)) ([9d03f34](https://github.com/AztecProtocol/aztec-packages/commit/9d03f34ca023c954832889ee8eef65aca60f1b1b))
* Optimize array sets in if conditions (alternate version) (https://github.com/noir-lang/noir/pull/4716) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Osxcross ([#6099](https://github.com/AztecProtocol/aztec-packages/issues/6099)) ([6cc924d](https://github.com/AztecProtocol/aztec-packages/commit/6cc924dc44a36d9ef2aeda05ea69a120898fc272))
* Parsing non-string assertion payloads in noir js ([#6079](https://github.com/AztecProtocol/aztec-packages/issues/6079)) ([fbd78fd](https://github.com/AztecProtocol/aztec-packages/commit/fbd78fdc53071f3548971dfb4832a440512f4687))
* Proving benchmark ([#6051](https://github.com/AztecProtocol/aztec-packages/issues/6051)) ([644bd85](https://github.com/AztecProtocol/aztec-packages/commit/644bd8525f6de8b71d6cc299baf3fda94b68abbb))
* Proving the private kernels and app circuits ([#6112](https://github.com/AztecProtocol/aztec-packages/issues/6112)) ([4a43fab](https://github.com/AztecProtocol/aztec-packages/commit/4a43fab043d9974a80c259703ebe2e0027e8ae57))
* Publish transaction_fee ([#6126](https://github.com/AztecProtocol/aztec-packages/issues/6126)) ([6f3a036](https://github.com/AztecProtocol/aztec-packages/commit/6f3a036585da589e04eb35b823ed2aaa7135bae5))
* Recursive folding verifier and decider as ultra circuits and circuit simulator ([#6150](https://github.com/AztecProtocol/aztec-packages/issues/6150)) ([acc8641](https://github.com/AztecProtocol/aztec-packages/commit/acc86416668ccfd6425ee3af4a898f2e8513168b))
* Reproducible ClientIVC proofs ([#6227](https://github.com/AztecProtocol/aztec-packages/issues/6227)) ([c145757](https://github.com/AztecProtocol/aztec-packages/commit/c145757a13ba4ff881c4bb05c4caaee7351053b3))
* Run noir-packages-test in Earthly ([#6174](https://github.com/AztecProtocol/aztec-packages/issues/6174)) ([58e40c9](https://github.com/AztecProtocol/aztec-packages/commit/58e40c9125e6d7b30abf7a4cbb170bbfc15e2037))
* Set aztec private functions to be recursive ([#6192](https://github.com/AztecProtocol/aztec-packages/issues/6192)) ([22625f8](https://github.com/AztecProtocol/aztec-packages/commit/22625f845f22703dc0d6e661fa36a0f67e6c719e))
* Use actual tx fee in gas token when charging fee ([#6166](https://github.com/AztecProtocol/aztec-packages/issues/6166)) ([8418eac](https://github.com/AztecProtocol/aztec-packages/commit/8418eac301fc9761cc29efd901ca5f719c3dfa09))


### Bug Fixes

* **abstract-phase-manager:** Get available gas from latest kernel output ([#6102](https://github.com/AztecProtocol/aztec-packages/issues/6102)) ([0fa509b](https://github.com/AztecProtocol/aztec-packages/commit/0fa509b68da7a8ab1b5865d17a7cf4cb197eb8b3))
* Aztec-run not exposing port for builder ([#6241](https://github.com/AztecProtocol/aztec-packages/issues/6241)) ([a80c091](https://github.com/AztecProtocol/aztec-packages/commit/a80c0911c629852d72bbff48b22af3b178b191b2))
* Boxes use base image ([#6120](https://github.com/AztecProtocol/aztec-packages/issues/6120)) ([ef2589a](https://github.com/AztecProtocol/aztec-packages/commit/ef2589a41f72981e5245f294695c5da8d4f04d0e))
* Compute the correct slice length when coercing from a literal array of complex types (https://github.com/noir-lang/noir/pull/4986) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* Correct circuit size estimation for UltraHonk ([#6164](https://github.com/AztecProtocol/aztec-packages/issues/6164)) ([ed84fe3](https://github.com/AztecProtocol/aztec-packages/commit/ed84fe3bcc29c69b1e9d9caafd2c2c2134a67dce))
* Docs release ci setup ([#6159](https://github.com/AztecProtocol/aztec-packages/issues/6159)) ([6d5cfe6](https://github.com/AztecProtocol/aztec-packages/commit/6d5cfe65dadf56b3f9094a2662b32792dd1a9520))
* **docs:** Fix broken link in tree implementations page ([#6143](https://github.com/AztecProtocol/aztec-packages/issues/6143)) ([b39f1db](https://github.com/AztecProtocol/aztec-packages/commit/b39f1db91942096eb1768a37ba9ecfb94d4e1313))
* **docs:** Update sandbox reference ([#6094](https://github.com/AztecProtocol/aztec-packages/issues/6094)) ([0641085](https://github.com/AztecProtocol/aztec-packages/commit/06410858fd1b6d0d8a1c225a08b8c6628ad9ddcc))
* Increase default number of proving agents ([#6146](https://github.com/AztecProtocol/aztec-packages/issues/6146)) ([5ade36e](https://github.com/AztecProtocol/aztec-packages/commit/5ade36e63ad9d521efe62e889836de5e891e6d0b))
* Install aztec-builder ([#6149](https://github.com/AztecProtocol/aztec-packages/issues/6149)) ([0497dcf](https://github.com/AztecProtocol/aztec-packages/commit/0497dcf4876b9e7bd7e7459f8d49a6167fd57323))
* Move remove_if_else pass after second inlining  (https://github.com/noir-lang/noir/pull/4976) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* **public-kernel:** Only validate start-gas for execution requests ([#6100](https://github.com/AztecProtocol/aztec-packages/issues/6100)) ([3ec9303](https://github.com/AztecProtocol/aztec-packages/commit/3ec9303c4fe25eb8bf5b81e58dcf989acc8ac7e6))
* Registering PublicDataWitness in JsonRpcServer ([#6243](https://github.com/AztecProtocol/aztec-packages/issues/6243)) ([e8c4455](https://github.com/AztecProtocol/aztec-packages/commit/e8c4455339ac0b4c7444aba7ff1308c10af4d139))
* Scope netlify to yarn bin ([#6162](https://github.com/AztecProtocol/aztec-packages/issues/6162)) ([be8e3c0](https://github.com/AztecProtocol/aztec-packages/commit/be8e3c00837f7b823b74dfad7ef0875265ae35fe))
* Set index and value to 0 for array_get with predicate (https://github.com/noir-lang/noir/pull/4971) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* Set up the ci runner for doc deployment ([#6160](https://github.com/AztecProtocol/aztec-packages/issues/6160)) ([e295900](https://github.com/AztecProtocol/aztec-packages/commit/e2959004c132f87b876e7b08ed3b2c3eb99622bf))
* Sporadic failure of GoblinRecursionTests.Vanilla ([#6218](https://github.com/AztecProtocol/aztec-packages/issues/6218)) ([f4ecea5](https://github.com/AztecProtocol/aztec-packages/commit/f4ecea5a83bcc88fd11698ac5c8e174c2461a74b))
* Use annotated type when checking declaration (https://github.com/noir-lang/noir/pull/4966) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Use pushed build images. ([#6154](https://github.com/AztecProtocol/aztec-packages/issues/6154)) ([426f7a7](https://github.com/AztecProtocol/aztec-packages/commit/426f7a7c0911512058d5d5d49a3ed9f2ab5ed4e0))
* Use random id for proving jobs ([#6084](https://github.com/AztecProtocol/aztec-packages/issues/6084)) ([0e0fc58](https://github.com/AztecProtocol/aztec-packages/commit/0e0fc585b9329371e5f89accf10ff1b7a08749c0))
* Various aztec-builder issues ([#6233](https://github.com/AztecProtocol/aztec-packages/issues/6233)) ([9a644ba](https://github.com/AztecProtocol/aztec-packages/commit/9a644baeae7c46250ced9942ce30f3f8694efe7f))


### Miscellaneous

* Add avm team as codeowners for public context ([#6247](https://github.com/AztecProtocol/aztec-packages/issues/6247)) ([c571ff0](https://github.com/AztecProtocol/aztec-packages/commit/c571ff0545d54819dd5b386e1bbd932dbe603819))
* **avm-simulator:** Avm's nested calls now stay internal and properly track PublicExecutionResult ([#6165](https://github.com/AztecProtocol/aztec-packages/issues/6165)) ([9fd4f39](https://github.com/AztecProtocol/aztec-packages/commit/9fd4f39e48793262d8d84e4ac0990c80072dcca3))
* **avm-simulator:** Make shifts take u8 ([#5905](https://github.com/AztecProtocol/aztec-packages/issues/5905)) ([4719ff1](https://github.com/AztecProtocol/aztec-packages/commit/4719ff19e71e27965a3ccf75b7356a27389ee766))
* **avm-simulator:** Track recursive public execution result in avm-simulator for integration with old kernel ([#6106](https://github.com/AztecProtocol/aztec-packages/issues/6106)) ([df3bcc6](https://github.com/AztecProtocol/aztec-packages/commit/df3bcc6315ba6ded3a352f7374888504ecc48eb9))
* **aztec-macros:** Avm function return types are auto tagged as `pub` ([#6250](https://github.com/AztecProtocol/aztec-packages/issues/6250)) ([0e828f3](https://github.com/AztecProtocol/aztec-packages/commit/0e828f3914078850b9a8e1e928c886c59cfab64e))
* **aztec-nr:** Create a 'with_selector' version of `emit_unencrypted_log` in avm context ([#6248](https://github.com/AztecProtocol/aztec-packages/issues/6248)) ([fda6442](https://github.com/AztecProtocol/aztec-packages/commit/fda64425ed673e2f4f4f7edc231b7a563ec5b0cc))
* Bump bb.js timeouts ([#6196](https://github.com/AztecProtocol/aztec-packages/issues/6196)) ([acab3de](https://github.com/AztecProtocol/aztec-packages/commit/acab3de86aae9ce5078795ba1ed0626d0c018565))
* Check root parity is only enqueued once its deps are ready ([#6015](https://github.com/AztecProtocol/aztec-packages/issues/6015)) ([c1120d1](https://github.com/AztecProtocol/aztec-packages/commit/c1120d16a68550934ab6744f8759b41f3dcdf4eb))
* **ci:** Fix restarts with fresh spot, acir test fixes, non-mandatory benches ([#6226](https://github.com/AztecProtocol/aztec-packages/issues/6226)) ([adb7f37](https://github.com/AztecProtocol/aztec-packages/commit/adb7f37a4ad01acf1ef197189a1e78323cae8f0b))
* **ci:** Force earthly prune if corrupted cache ([#6152](https://github.com/AztecProtocol/aztec-packages/issues/6152)) ([3910314](https://github.com/AztecProtocol/aztec-packages/commit/39103141a56f7f71fffb2d4164f0c4f432704a81))
* **ci:** Improve dependency structure ([#6200](https://github.com/AztecProtocol/aztec-packages/issues/6200)) ([3abc862](https://github.com/AztecProtocol/aztec-packages/commit/3abc862f77b883382e6f03ec66c5fd93efef9989))
* **ci:** Migrate `protocol-circuits-gate-diff` to earthly ([#6204](https://github.com/AztecProtocol/aztec-packages/issues/6204)) ([4b43295](https://github.com/AztecProtocol/aztec-packages/commit/4b432951a9fe46ca1b0e0d38ebafe523bebf04eb))
* **ci:** More stable spot request ([#6212](https://github.com/AztecProtocol/aztec-packages/issues/6212)) ([00156b5](https://github.com/AztecProtocol/aztec-packages/commit/00156b566dbc2973ddc8a61550000e980f9c3454))
* **ci:** Optimize e2e build ([#6202](https://github.com/AztecProtocol/aztec-packages/issues/6202)) ([4614059](https://github.com/AztecProtocol/aztec-packages/commit/4614059c9667d4b42063d47a2b4cc5b24d54db9b))
* **ci:** Rollback earthly prune ([#6208](https://github.com/AztecProtocol/aztec-packages/issues/6208)) ([3ccc6ac](https://github.com/AztecProtocol/aztec-packages/commit/3ccc6acae834f9add0548c0ca044e65a2e13b08b))
* **ci:** Try to make base image more stable ([#6144](https://github.com/AztecProtocol/aztec-packages/issues/6144)) ([979a22d](https://github.com/AztecProtocol/aztec-packages/commit/979a22d5668f5b46c350f2355b60da8bd59e2cda))
* Debug log oracle calls return nothing ([#6209](https://github.com/AztecProtocol/aztec-packages/issues/6209)) ([151d3a3](https://github.com/AztecProtocol/aztec-packages/commit/151d3a3feaad5cf59041eac1b47f2bc31d1dbcf2))
* **docs:** Fix some typos in specs of private kernel initial ([#6224](https://github.com/AztecProtocol/aztec-packages/issues/6224)) ([ead54c4](https://github.com/AztecProtocol/aztec-packages/commit/ead54c479ce221f6eed2b31fe37db82e615897ea))
* E2e workaround ([#6158](https://github.com/AztecProtocol/aztec-packages/issues/6158)) ([7794d78](https://github.com/AztecProtocol/aztec-packages/commit/7794d788cb9675dbb4714f850e3a39d6dd3ce990))
* Migrate acir tests to earthly ([#6142](https://github.com/AztecProtocol/aztec-packages/issues/6142)) ([18c8ea8](https://github.com/AztecProtocol/aztec-packages/commit/18c8ea8eb5f9fd1cb51c116d6d1976c774d51bc1))
* Misc AVM migration prep changes ([#6253](https://github.com/AztecProtocol/aztec-packages/issues/6253)) ([fe19404](https://github.com/AztecProtocol/aztec-packages/commit/fe194043b6a7b7256b39b1db786b4df754b14890))
* Nuking `GrumpkinScalar` ([#6240](https://github.com/AztecProtocol/aztec-packages/issues/6240)) ([d2df10d](https://github.com/AztecProtocol/aztec-packages/commit/d2df10d78036f6fb4e0dae5c7287e4523bd8b47d))
* Release Noir(0.29.0) (https://github.com/noir-lang/noir/pull/4905) ([02d3d17](https://github.com/AztecProtocol/aztec-packages/commit/02d3d177e86683aa77680127c3e6738bc22fdc02))
* Rename instruction checks for side effects (https://github.com/noir-lang/noir/pull/4945) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Replace relative paths to noir-protocol-circuits ([cf543a6](https://github.com/AztecProtocol/aztec-packages/commit/cf543a6ea944e49e9fff71e52620718385456428))
* Replace relative paths to noir-protocol-circuits ([53cf7bb](https://github.com/AztecProtocol/aztec-packages/commit/53cf7bbc008fc1dae4c295901153d6751bf9eacd))
* Replace relative paths to noir-protocol-circuits ([ca29cea](https://github.com/AztecProtocol/aztec-packages/commit/ca29cea33adda120adc90b3a32163625271af319))
* Replace relative paths to noir-protocol-circuits ([08e538b](https://github.com/AztecProtocol/aztec-packages/commit/08e538b3ef0805270c498b3d65443378cf720985))
* Speedup static_call test ([#6157](https://github.com/AztecProtocol/aztec-packages/issues/6157)) ([abe8875](https://github.com/AztecProtocol/aztec-packages/commit/abe8875fe40703419fcf12653a21d734e8028b4d))
* Switch Noir JS to use execute program instead of circuit (https://github.com/noir-lang/noir/pull/4965) ([3ed41a0](https://github.com/AztecProtocol/aztec-packages/commit/3ed41a08c1fef80a6b8eecf4618dcc9be891e4c0))
* Use correct call type ([#6064](https://github.com/AztecProtocol/aztec-packages/issues/6064)) ([b3ae289](https://github.com/AztecProtocol/aztec-packages/commit/b3ae289748954229aac7ae2e1fe72483ede79a52))


### Documentation

* Add GlobalVariables to CombinedConstantData ([#6071](https://github.com/AztecProtocol/aztec-packages/issues/6071)) ([cf026d2](https://github.com/AztecProtocol/aztec-packages/commit/cf026d2c5928ce081bfac1e0d85260075b06f418))
* Update fees kernel tracking docs ([#6151](https://github.com/AztecProtocol/aztec-packages/issues/6151)) ([7d80428](https://github.com/AztecProtocol/aztec-packages/commit/7d804287889164873c5fdec452a9af0144bbe183))

## [0.37.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.36.0...aztec-packages-v0.37.0) (2024-05-02)


### ⚠ BREAKING CHANGES

* use `distinct` return value witnesses by default (https://github.com/noir-lang/noir/pull/4951)
* Bit shift is restricted to u8 right operand (https://github.com/noir-lang/noir/pull/4907)

### Features

* Abort ongoing proving jobs ([#6049](https://github.com/AztecProtocol/aztec-packages/issues/6049)) ([0aa352d](https://github.com/AztecProtocol/aztec-packages/commit/0aa352d7df32c01eb1dec65f137149e7c7351266))
* Add aztecprotocol/aztec-builder ([#6116](https://github.com/AztecProtocol/aztec-packages/issues/6116)) ([30899d0](https://github.com/AztecProtocol/aztec-packages/commit/30899d0346c632e1c3cfe2ae14357700d635986c))
* Add de-sugaring for `impl Trait` in function parameters (https://github.com/noir-lang/noir/pull/4919) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* Aztec nr lib constraining nullifier key is fresh ([#5939](https://github.com/AztecProtocol/aztec-packages/issues/5939)) ([f95de6b](https://github.com/AztecProtocol/aztec-packages/commit/f95de6b498d34e138cd55f88340917c6881eec6b))
* Bit shift is restricted to u8 right operand (https://github.com/noir-lang/noir/pull/4907) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* Count Bb lines weighted by complexity ([#6090](https://github.com/AztecProtocol/aztec-packages/issues/6090)) ([705177f](https://github.com/AztecProtocol/aztec-packages/commit/705177f2caf4c11f31a22a1ea01b8d0fcd1b4158))
* Devbox ([#5772](https://github.com/AztecProtocol/aztec-packages/issues/5772)) ([72321f9](https://github.com/AztecProtocol/aztec-packages/commit/72321f9d3af27f85c92564754d444ac3df1fcad2))
* Enforce gas limits from private kernels ([#6105](https://github.com/AztecProtocol/aztec-packages/issues/6105)) ([4395855](https://github.com/AztecProtocol/aztec-packages/commit/43958554c0d0887e5962580830abc950b86fcff2))
* **experimental:** `comptime` globals (https://github.com/noir-lang/noir/pull/4918) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* Handle `no_predicates` attribute (https://github.com/noir-lang/noir/pull/4942) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Migrate boxes to GA and Earthly ([#6076](https://github.com/AztecProtocol/aztec-packages/issues/6076)) ([4a49f9d](https://github.com/AztecProtocol/aztec-packages/commit/4a49f9dd573fee22d52996c0d9139d30c6bab1e3))
* Pippenger benchmarks compatible with wasmtime ([#6095](https://github.com/AztecProtocol/aztec-packages/issues/6095)) ([5297b5b](https://github.com/AztecProtocol/aztec-packages/commit/5297b5bb2de63003fdb97b9ad75e06c485327b1c))
* Private da gas metering ([#6103](https://github.com/AztecProtocol/aztec-packages/issues/6103)) ([1a8f372](https://github.com/AztecProtocol/aztec-packages/commit/1a8f372b326eacbccdc2e7e56f87821f658a75f8))
* Prover metrics ([#6050](https://github.com/AztecProtocol/aztec-packages/issues/6050)) ([5b133f2](https://github.com/AztecProtocol/aztec-packages/commit/5b133f2eafd633b5f99a588f12d7e70ee28d6496))
* Use `distinct` return value witnesses by default (https://github.com/noir-lang/noir/pull/4951) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))


### Bug Fixes

* Ban self-referential structs (https://github.com/noir-lang/noir/pull/4883) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* **ci:** Build-key hotfix ([#6123](https://github.com/AztecProtocol/aztec-packages/issues/6123)) ([5791004](https://github.com/AztecProtocol/aztec-packages/commit/57910041fdd05b2e76489b86d60282411e0c7dde))
* **ci:** Ssh'ing into instances ([#6136](https://github.com/AztecProtocol/aztec-packages/issues/6136)) ([af3192d](https://github.com/AztecProtocol/aztec-packages/commit/af3192dce60c3907898a839ea8db47262da5b78e))
* Discard ref counts during unrolling (https://github.com/noir-lang/noir/pull/4923) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* **docs:** Add codegen to `aztec-builder` command ([#6098](https://github.com/AztecProtocol/aztec-packages/issues/6098)) ([4839ed9](https://github.com/AztecProtocol/aztec-packages/commit/4839ed99aa3fd6a4a2329612a97a86f4d02eae2d))
* Ensure where clauses propagated to trait default definitions (https://github.com/noir-lang/noir/pull/4894) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Require for all foldable functions to use distinct return  (https://github.com/noir-lang/noir/pull/4949) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))


### Miscellaneous

* Add regression test for [#3051](https://github.com/AztecProtocol/aztec-packages/issues/3051) (https://github.com/noir-lang/noir/pull/4815) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))
* Add test for recursing a foldable function (https://github.com/noir-lang/noir/pull/4948) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Adding devcontainer with create aztec app ([#5960](https://github.com/AztecProtocol/aztec-packages/issues/5960)) ([ae5cb21](https://github.com/AztecProtocol/aztec-packages/commit/ae5cb2116c141afedfb31b0a7a5c99674288204e))
* Build docs in earthly ([#6038](https://github.com/AztecProtocol/aztec-packages/issues/6038)) ([784d542](https://github.com/AztecProtocol/aztec-packages/commit/784d54258ca3bce47f030790afa3ba1e4d5e0ad9))
* Bump bench-tx-size timeout ([#6109](https://github.com/AztecProtocol/aztec-packages/issues/6109)) ([aa3eefa](https://github.com/AztecProtocol/aztec-packages/commit/aa3eefac5e7b21fad06a975b0b0a523ac7af7b3d))
* **ci:** Fix spot runner build key ([#6119](https://github.com/AztecProtocol/aztec-packages/issues/6119)) ([f332bc9](https://github.com/AztecProtocol/aztec-packages/commit/f332bc95e624e6600b01b4bde67accf9d21add1e))
* **ci:** Hotfix ([#6124](https://github.com/AztecProtocol/aztec-packages/issues/6124)) ([f60dfcd](https://github.com/AztecProtocol/aztec-packages/commit/f60dfcd71e58d0a49e29c7efc7fc2f67670a306b))
* **ci:** Run benchmarks on Earthly ([#6089](https://github.com/AztecProtocol/aztec-packages/issues/6089)) ([c985c73](https://github.com/AztecProtocol/aztec-packages/commit/c985c73c0da3212076d14c5f6e9f83d450c88278))
* **ci:** Turn off ARM build for now ([#6135](https://github.com/AztecProtocol/aztec-packages/issues/6135)) ([853913f](https://github.com/AztecProtocol/aztec-packages/commit/853913fd771034fcd959b5dd155a49437eb8989a))
* Disable bench-summary for now ([67485f1](https://github.com/AztecProtocol/aztec-packages/commit/67485f1fe649d145c1ad2c4bcbf98e2ee319f321))
* Disable doc builds ([#6107](https://github.com/AztecProtocol/aztec-packages/issues/6107)) ([7933f0f](https://github.com/AztecProtocol/aztec-packages/commit/7933f0f32c293e4c9f1f4437fbb4c82cf63b35bf))
* **docs:** Adding matomo tracking (https://github.com/noir-lang/noir/pull/4898) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Ebs attach robustness ([#6108](https://github.com/AztecProtocol/aztec-packages/issues/6108)) ([c702688](https://github.com/AztecProtocol/aztec-packages/commit/c702688435d66436d0ee0d66c32aba99392d719d))
* Fix typo in `ResolverError::AbiAttributeOutsideContract` (https://github.com/noir-lang/noir/pull/4933) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Migrate the prover client test to earthly ([#6118](https://github.com/AztecProtocol/aztec-packages/issues/6118)) ([a59a6c0](https://github.com/AztecProtocol/aztec-packages/commit/a59a6c07a15d401e1fbcb4ffc08d79cf4fa6d2fc))
* Redo typo PR by stayweek ([#6080](https://github.com/AztecProtocol/aztec-packages/issues/6080)) ([0869452](https://github.com/AztecProtocol/aztec-packages/commit/086945223602f5d909fc79b56420046c0808f35c))
* Redo typo PR by vitalmotif ([#6081](https://github.com/AztecProtocol/aztec-packages/issues/6081)) ([1a89d1a](https://github.com/AztecProtocol/aztec-packages/commit/1a89d1a8bfa05fbac9904adc63b57ba94896c8d6))
* Refactor nested contract test for speed ([#6117](https://github.com/AztecProtocol/aztec-packages/issues/6117)) ([b346a2f](https://github.com/AztecProtocol/aztec-packages/commit/b346a2f71f8e2c6658abdbf986b3351d3947995a))
* Remove unnecessary `pub(super)` in interpreter (https://github.com/noir-lang/noir/pull/4939) ([4dc5efb](https://github.com/AztecProtocol/aztec-packages/commit/4dc5efb2299b488e0c7f3a79493350f812bb25ce))
* Replace relative paths to noir-protocol-circuits ([47592a2](https://github.com/AztecProtocol/aztec-packages/commit/47592a2663077e5b689a69cfac91571129dbce79))
* Replace relative paths to noir-protocol-circuits ([f0d95f5](https://github.com/AztecProtocol/aztec-packages/commit/f0d95f5cfc5654120192d48dddc4803d21dfbbc8))
* Update error conversion traits to act on references (https://github.com/noir-lang/noir/pull/4936) ([f060fa6](https://github.com/AztecProtocol/aztec-packages/commit/f060fa6e8b5bb504680a8d26793ebc202f196020))


### Documentation

* Tweaks for release ([#6129](https://github.com/AztecProtocol/aztec-packages/issues/6129)) ([77b45b9](https://github.com/AztecProtocol/aztec-packages/commit/77b45b9abd5d615a1e394e1c256faee53dfd4790))
* Update @aztec/builder readme ([#6115](https://github.com/AztecProtocol/aztec-packages/issues/6115)) ([248761e](https://github.com/AztecProtocol/aztec-packages/commit/248761e75651d1d8b766ee676459f39c8a36b03e))
* Yellow paper updates for the parity circuits ([#6048](https://github.com/AztecProtocol/aztec-packages/issues/6048)) ([cfe1b05](https://github.com/AztecProtocol/aztec-packages/commit/cfe1b05b46be55072ada81e54a6eda7ef6fff23f))

## [0.36.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.35.1...aztec-packages-v0.36.0) (2024-04-30)


### ⚠ BREAKING CHANGES

* remove `Opcode::Brillig` from ACIR ([#5995](https://github.com/AztecProtocol/aztec-packages/issues/5995))
* delete field note ([#5959](https://github.com/AztecProtocol/aztec-packages/issues/5959))
* remove slow updates tree ([#5954](https://github.com/AztecProtocol/aztec-packages/issues/5954))
* Add `as_array` and remove `_slice` variants of hash functions (https://github.com/noir-lang/noir/pull/4675)
* reserve keyword `super` (https://github.com/noir-lang/noir/pull/4836)
* **aztec-nr:** unencrypted logs go behind context ([#5871](https://github.com/AztecProtocol/aztec-packages/issues/5871))
* move fixtures to @aztec/circuits.js/testing/fixtures ([#5826](https://github.com/AztecProtocol/aztec-packages/issues/5826))
* contract interfaces and better function calls ([#5687](https://github.com/AztecProtocol/aztec-packages/issues/5687))
* change backend width to 4 ([#5374](https://github.com/AztecProtocol/aztec-packages/issues/5374))

### Features

* `variable_base_scalar_mul` blackbox func ([#6039](https://github.com/AztecProtocol/aztec-packages/issues/6039)) ([81142fe](https://github.com/AztecProtocol/aztec-packages/commit/81142fe799338e6ed73b30eeac4468c1345f6fab))
* **acir_gen:** Brillig stdlib (https://github.com/noir-lang/noir/pull/4848) ([8f73f18](https://github.com/AztecProtocol/aztec-packages/commit/8f73f18f3c07de0fd5e247ade5a48109c37c1bc5))
* Add `#[inline(tag)]` attribute and codegen (https://github.com/noir-lang/noir/pull/4913) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Add `min` and `max` functions to the stdlib (https://github.com/noir-lang/noir/pull/4839) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Add `NARGO_FOREIGN_CALL_TIMEOUT` environment variable (https://github.com/noir-lang/noir/pull/4780) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Add comptime Interpreter (https://github.com/noir-lang/noir/pull/4821) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Add key registry to deployment (e2e & sandbox) ([#5875](https://github.com/AztecProtocol/aztec-packages/issues/5875)) ([0881cd3](https://github.com/AztecProtocol/aztec-packages/commit/0881cd3083af70271bceda695d0c8ad21212c172)), closes [#5611](https://github.com/AztecProtocol/aztec-packages/issues/5611)
* Add promiseWithResolvers ([#5808](https://github.com/AztecProtocol/aztec-packages/issues/5808)) ([afeef17](https://github.com/AztecProtocol/aztec-packages/commit/afeef17e14054f8ee95a6244c1b165435fddaa50))
* Add proving queue ([#5754](https://github.com/AztecProtocol/aztec-packages/issues/5754)) ([a0a9668](https://github.com/AztecProtocol/aztec-packages/commit/a0a9668d933907a89f21077fd700b6d2f44e6c74))
* Add side effect counter to logs ([#5718](https://github.com/AztecProtocol/aztec-packages/issues/5718)) ([d7486a6](https://github.com/AztecProtocol/aztec-packages/commit/d7486a6b0b26b5264a1b02c1134d82abfb497aa0))
* Add the storage layout to the contract artifact ([#5952](https://github.com/AztecProtocol/aztec-packages/issues/5952)) ([88ee0af](https://github.com/AztecProtocol/aztec-packages/commit/88ee0af9987063d63afb49c4f61ab5ae5f7c1b73))
* Add TimeoutError ([#5751](https://github.com/AztecProtocol/aztec-packages/issues/5751)) ([741fdf1](https://github.com/AztecProtocol/aztec-packages/commit/741fdf16e7f0b3f116c724505afa8e604bc51bf1))
* Add variable size sha256 (https://github.com/noir-lang/noir/pull/4920) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Add variable size sha256 (https://github.com/noir-lang/noir/pull/4920) ([078aa61](https://github.com/AztecProtocol/aztec-packages/commit/078aa61b06557aba74ac9cce557ee6bd05040feb))
* AES oracle ([#5996](https://github.com/AztecProtocol/aztec-packages/issues/5996)) ([8e0a563](https://github.com/AztecProtocol/aztec-packages/commit/8e0a56306ba45ea1eaaa25ee47d84b7334e0bbe3)), closes [#5895](https://github.com/AztecProtocol/aztec-packages/issues/5895)
* AES oracle padding ([#6013](https://github.com/AztecProtocol/aztec-packages/issues/6013)) ([4b563cd](https://github.com/AztecProtocol/aztec-packages/commit/4b563cd79f16f513a05c1595a4f2673cdaa7600a))
* Allow numeric generics to non inlined ACIR functions (https://github.com/noir-lang/noir/pull/4834) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Avm mem trace validation ([#6025](https://github.com/AztecProtocol/aztec-packages/issues/6025)) ([3a3afb5](https://github.com/AztecProtocol/aztec-packages/commit/3a3afb57ab8b6b3f11b7a7799d557436638c8cd3)), closes [#5950](https://github.com/AztecProtocol/aztec-packages/issues/5950)
* **avm:** Avm circuit FDIV opcode ([#5958](https://github.com/AztecProtocol/aztec-packages/issues/5958)) ([fed5b6d](https://github.com/AztecProtocol/aztec-packages/commit/fed5b6dd1ee310fc90404a3e5ec9eb02ad7dbc10)), closes [#5953](https://github.com/AztecProtocol/aztec-packages/issues/5953)
* **avm:** CAST opcode implementation ([#5477](https://github.com/AztecProtocol/aztec-packages/issues/5477)) ([a821bcc](https://github.com/AztecProtocol/aztec-packages/commit/a821bccef7b1894140f0495510d7c6b4eefde821)), closes [#5466](https://github.com/AztecProtocol/aztec-packages/issues/5466)
* **avm:** Negative tests ([#5919](https://github.com/AztecProtocol/aztec-packages/issues/5919)) ([8a5ece7](https://github.com/AztecProtocol/aztec-packages/commit/8a5ece7548a86d099ac6a166f04882624b8d95fd))
* **avm:** Shift relations ([#5716](https://github.com/AztecProtocol/aztec-packages/issues/5716)) ([a516637](https://github.com/AztecProtocol/aztec-packages/commit/a51663707b96914b0a300440611748ce44fbe933))
* Avoiding redundant computation in PG ([#5844](https://github.com/AztecProtocol/aztec-packages/issues/5844)) ([9f57733](https://github.com/AztecProtocol/aztec-packages/commit/9f5773353aa0261fa07a81704bcadcee513d42c5))
* Brillig pointer codegen and execution ([#5737](https://github.com/AztecProtocol/aztec-packages/issues/5737)) ([a7b9d20](https://github.com/AztecProtocol/aztec-packages/commit/a7b9d20a962c33d8585502fd00739138c6d79aca))
* Bump lmdb ([#5783](https://github.com/AztecProtocol/aztec-packages/issues/5783)) ([f7d5cf2](https://github.com/AztecProtocol/aztec-packages/commit/f7d5cf2c683ee7840885ac176b9e838b4e3ab6e2))
* Change backend width to 4 ([#5374](https://github.com/AztecProtocol/aztec-packages/issues/5374)) ([3f24fc2](https://github.com/AztecProtocol/aztec-packages/commit/3f24fc2cdb56eff6da6e47062d2a2a3dc0fa4bd2))
* Circuit simulator for Ultra and GoblinUltra verifiers ([#1195](https://github.com/AztecProtocol/aztec-packages/issues/1195)) ([0032a3a](https://github.com/AztecProtocol/aztec-packages/commit/0032a3a55dea5e4c9051dbc36607288f8ca1be4a))
* Computing sym key for incoming ciphertext ([#6020](https://github.com/AztecProtocol/aztec-packages/issues/6020)) ([1904fa8](https://github.com/AztecProtocol/aztec-packages/commit/1904fa864ff8c546d4d849436c6ca7a7606fb3d2))
* Configure prover as separate process ([#5973](https://github.com/AztecProtocol/aztec-packages/issues/5973)) ([c0dd7b2](https://github.com/AztecProtocol/aztec-packages/commit/c0dd7b21779b99f1b9d3ed43623d3de25a332699))
* Contract interfaces and better function calls ([#5687](https://github.com/AztecProtocol/aztec-packages/issues/5687)) ([274f7d9](https://github.com/AztecProtocol/aztec-packages/commit/274f7d935230ce21d062644f6ec5f7cd0f58ae62))
* Decoded return values ([#5762](https://github.com/AztecProtocol/aztec-packages/issues/5762)) ([03e693a](https://github.com/AztecProtocol/aztec-packages/commit/03e693a0db52a0c0b02c403f9ded2e28f3c7ced2))
* Delete field note ([#5959](https://github.com/AztecProtocol/aztec-packages/issues/5959)) ([ae18396](https://github.com/AztecProtocol/aztec-packages/commit/ae183960a96d14d1eac2876bc070ed09f75b8f25))
* **docs:** Nuke CLI from docs ([#5936](https://github.com/AztecProtocol/aztec-packages/issues/5936)) ([9af68d8](https://github.com/AztecProtocol/aztec-packages/commit/9af68d8bd59a84e014567b429e9c9b4aed7fee74))
* Dynamic assertion payloads v2 ([#5949](https://github.com/AztecProtocol/aztec-packages/issues/5949)) ([405bdf6](https://github.com/AztecProtocol/aztec-packages/commit/405bdf6a297b81e0c3fda303cf2b1480eaea69f1))
* **experimental:** Add `comptime` keyword (https://github.com/noir-lang/noir/pull/4840) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Handle `BrilligCall` opcodes in the debugger (https://github.com/noir-lang/noir/pull/4897) ([3b91791](https://github.com/AztecProtocol/aztec-packages/commit/3b9179118369137880277f1444f0e3f94b3f5e79))
* Implement `Eq` trait on `BoundedVec` (https://github.com/noir-lang/noir/pull/4830) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Implement canonical key registry 5609 and implement shared mutable getter from another contract 5689  ([#5723](https://github.com/AztecProtocol/aztec-packages/issues/5723)) ([15b569f](https://github.com/AztecProtocol/aztec-packages/commit/15b569f24e55d374bfb5a54c8771118653e5e77c))
* Implement recursive verification in the parity circuits ([#6006](https://github.com/AztecProtocol/aztec-packages/issues/6006)) ([a5b6dac](https://github.com/AztecProtocol/aztec-packages/commit/a5b6dacd5512d7a035655845381b2c720b1e550a))
* Keshas skipping plus conditions for grand prod relations ([#5766](https://github.com/AztecProtocol/aztec-packages/issues/5766)) ([d8fcfb5](https://github.com/AztecProtocol/aztec-packages/commit/d8fcfb590f788b911111010e20458797d76f5779))
* Lalrpop lexer prototype (https://github.com/noir-lang/noir/pull/4656) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Make public l1tol2 message consumption take leafIndex ([#5805](https://github.com/AztecProtocol/aztec-packages/issues/5805)) ([302e3bb](https://github.com/AztecProtocol/aztec-packages/commit/302e3bbb2d7a7d54f362026edb314f3d3596b6d6))
* More robust spot shutdown + CI commandline ([#5825](https://github.com/AztecProtocol/aztec-packages/issues/5825)) ([12064f9](https://github.com/AztecProtocol/aztec-packages/commit/12064f95abb3125933eb55996abb978c4aeaad53))
* Naive structured execution trace ([#5853](https://github.com/AztecProtocol/aztec-packages/issues/5853)) ([23aab17](https://github.com/AztecProtocol/aztec-packages/commit/23aab171b17d0dfb840621a74266496ac270b3e8))
* **nargo:** Handle call stacks for multiple Acir calls (https://github.com/noir-lang/noir/pull/4711) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Narrow ABI encoding errors down to target problem argument/field (https://github.com/noir-lang/noir/pull/4798) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* **p2p:** DiscV5 Peer Discovery ([#5652](https://github.com/AztecProtocol/aztec-packages/issues/5652)) ([0e81642](https://github.com/AztecProtocol/aztec-packages/commit/0e8164239b6a1180fd292e37faf1a0e64aa9cff4))
* Prove the public kernel circuits ([#5778](https://github.com/AztecProtocol/aztec-packages/issues/5778)) ([f9a843a](https://github.com/AztecProtocol/aztec-packages/commit/f9a843a00ff41ef39b958ae7f5a24bdbc1b1add2))
* Prove then verify flow for honk ([#5957](https://github.com/AztecProtocol/aztec-packages/issues/5957)) ([099346e](https://github.com/AztecProtocol/aztec-packages/commit/099346ebbab9428f57bfffdc03e8bede5c2e2bed))
* Re-introducing update command ([#5946](https://github.com/AztecProtocol/aztec-packages/issues/5946)) ([13153d0](https://github.com/AztecProtocol/aztec-packages/commit/13153d02c8b0eb9cae1b7c0436fe1a1ddb49734f))
* Remove slow updates tree ([#5954](https://github.com/AztecProtocol/aztec-packages/issues/5954)) ([52a1631](https://github.com/AztecProtocol/aztec-packages/commit/52a1631b59297ce062eda14a10e99e552f7fa706))
* Reserve keyword `super` (https://github.com/noir-lang/noir/pull/4836) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Serialize public kernel private inputs ([#5971](https://github.com/AztecProtocol/aztec-packages/issues/5971)) ([0c712b9](https://github.com/AztecProtocol/aztec-packages/commit/0c712b9c0f69bad0da3910add5adba40622d3cea))
* Simplify `BoundedVec::eq` (https://github.com/noir-lang/noir/pull/4838) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Squashing transient note hashes and nullifiers ([#6059](https://github.com/AztecProtocol/aztec-packages/issues/6059)) ([2b8b2c3](https://github.com/AztecProtocol/aztec-packages/commit/2b8b2c3bcbed425027f343bd2b90fd6380e2ddc4))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4792) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4833) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4902) ([3b91791](https://github.com/AztecProtocol/aztec-packages/commit/3b9179118369137880277f1444f0e3f94b3f5e79))
* Validate globals in public kernel ([#6031](https://github.com/AztecProtocol/aztec-packages/issues/6031)) ([82b17c8](https://github.com/AztecProtocol/aztec-packages/commit/82b17c8f0e9207db803fd3b824e63bac25ea69f6))
* Verify public data reads ([#5701](https://github.com/AztecProtocol/aztec-packages/issues/5701)) ([323f59f](https://github.com/AztecProtocol/aztec-packages/commit/323f59f55bcd64e32725d1ed5aab72d5b9dbe31d))
* Wire gas from public execution to kernels ([#5941](https://github.com/AztecProtocol/aztec-packages/issues/5941)) ([6894fc7](https://github.com/AztecProtocol/aztec-packages/commit/6894fc759cc4cd4e77d297fe6164cd39478ece4a))


### Bug Fixes

* `test_native.sh` not running all noir tests ([#6075](https://github.com/AztecProtocol/aztec-packages/issues/6075)) ([cc7676e](https://github.com/AztecProtocol/aztec-packages/commit/cc7676e87a7002f14b1b77b7c13f88f71355ec5b))
* Args + selector in deploy.nr ([#5948](https://github.com/AztecProtocol/aztec-packages/issues/5948)) ([100744f](https://github.com/AztecProtocol/aztec-packages/commit/100744f89b676a03990c2d29aa0b48da77be5d8d))
* **avm-simulator:** L1TOL2MESSAGEEXISTS opcode ([#5807](https://github.com/AztecProtocol/aztec-packages/issues/5807)) ([71b60f3](https://github.com/AztecProtocol/aztec-packages/commit/71b60f32c3b3781dda1c79bb6a926050bad7bb55))
* **avm:** Comments and assert ([#5956](https://github.com/AztecProtocol/aztec-packages/issues/5956)) ([ae50219](https://github.com/AztecProtocol/aztec-packages/commit/ae502199b84999418d461ed5d0d6fca0c60494c5))
* **avm:** Do not scale CALLDATACOPY base cost with size ([#5879](https://github.com/AztecProtocol/aztec-packages/issues/5879)) ([99e12b1](https://github.com/AztecProtocol/aztec-packages/commit/99e12b1abd7e66e871b41572a54cee63b5300d96))
* Bigint corruption in lmdb ([#6002](https://github.com/AztecProtocol/aztec-packages/issues/6002)) ([703e0c1](https://github.com/AztecProtocol/aztec-packages/commit/703e0c1e2c2a5703410ff5fd4c1a135131254a53))
* Calculate tx fee using current constants in public kernel ([#6066](https://github.com/AztecProtocol/aztec-packages/issues/6066)) ([c359d79](https://github.com/AztecProtocol/aztec-packages/commit/c359d796e72c215edf1af06c54d9287ee87df425))
* Catch panics from EC point creation (e.g. the point is at infinity) (https://github.com/noir-lang/noir/pull/4790) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Check if a runner is available + safer refcount for spot life ([#5793](https://github.com/AztecProtocol/aztec-packages/issues/5793)) ([67077a1](https://github.com/AztecProtocol/aztec-packages/commit/67077a11250cb28dbef890009668341524b85d8b))
* **ci:** Always run merge-check ([#6065](https://github.com/AztecProtocol/aztec-packages/issues/6065)) ([b43b84f](https://github.com/AztecProtocol/aztec-packages/commit/b43b84f5993e6950c5f081c3c77e9549dc7fddbe))
* **ci:** Deploy_npm script ([#5817](https://github.com/AztecProtocol/aztec-packages/issues/5817)) ([df1c3c4](https://github.com/AztecProtocol/aztec-packages/commit/df1c3c4c706a44847b25a66d27544eedc508cf62))
* **ci:** Merge check fails ungracefully on spot issues ([#5887](https://github.com/AztecProtocol/aztec-packages/issues/5887)) ([3683f0b](https://github.com/AztecProtocol/aztec-packages/commit/3683f0bb034ea59258c587d70d0517ee2ed00b91))
* **ci:** Race condition when making spot in multiple PRs ([#5798](https://github.com/AztecProtocol/aztec-packages/issues/5798)) ([18e75b8](https://github.com/AztecProtocol/aztec-packages/commit/18e75b85bcd6eec53cee3a5da854a8d27e3f186e))
* Deploy L1 contracts before starting node ([#5969](https://github.com/AztecProtocol/aztec-packages/issues/5969)) ([1908139](https://github.com/AztecProtocol/aztec-packages/commit/190813911c5e4fc7533525478ceca4162170fa6b))
* **docs:** Fix admonition in contract class protocol spec ([#6017](https://github.com/AztecProtocol/aztec-packages/issues/6017)) ([12bfc15](https://github.com/AztecProtocol/aztec-packages/commit/12bfc15923ee4b7b57e50ac714953cb8129e7d5d))
* **docs:** Fix formatting in protocol specs ([#5882](https://github.com/AztecProtocol/aztec-packages/issues/5882)) ([07fc143](https://github.com/AztecProtocol/aztec-packages/commit/07fc1434ac780f8a35533775e26ef2bd9e190816))
* **docs:** Tutorial fixes ([#5600](https://github.com/AztecProtocol/aztec-packages/issues/5600)) ([6421467](https://github.com/AztecProtocol/aztec-packages/commit/642146705857cf34eb0f9feab665977fb2d8fb02))
* Don't refcount spot ([#5812](https://github.com/AztecProtocol/aztec-packages/issues/5812)) ([98e8da0](https://github.com/AztecProtocol/aztec-packages/commit/98e8da094dbac1c06f800f82bd89181a6b9039b5))
* Don't reuse brillig with slice arguments ([#5800](https://github.com/AztecProtocol/aztec-packages/issues/5800)) ([be9f24c](https://github.com/AztecProtocol/aztec-packages/commit/be9f24c16484b26a1eb88bcf35b785553160995d))
* **experimental:** Skip over comptime functions in scan pass (https://github.com/noir-lang/noir/pull/4893) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Fix and reenable fees e2e tests ([#5877](https://github.com/AztecProtocol/aztec-packages/issues/5877)) ([165e62f](https://github.com/AztecProtocol/aztec-packages/commit/165e62f38239f25cc6595bb43f435e9f4673fd83))
* Fix curve parameters for bigints (https://github.com/noir-lang/noir/pull/4900) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Fix panic when returning a zeroed unit value (https://github.com/noir-lang/noir/pull/4797) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Fix relation skipping for sumcheck ([#6092](https://github.com/AztecProtocol/aztec-packages/issues/6092)) ([1449c33](https://github.com/AztecProtocol/aztec-packages/commit/1449c338ca79f8d72b71484546aa46ddebb21779))
* Hotfix stopped instance terminate ([#6037](https://github.com/AztecProtocol/aztec-packages/issues/6037)) ([005c71c](https://github.com/AztecProtocol/aztec-packages/commit/005c71cff4b592f89833c5556d827e55b7678b7b))
* Issue 4682 and add solver for unconstrained bigintegers (https://github.com/noir-lang/noir/pull/4729) ([beab8c9](https://github.com/AztecProtocol/aztec-packages/commit/beab8c93857536e07fa37994213fc664a5864013))
* Make discv5 test deterministic ([#5968](https://github.com/AztecProtocol/aztec-packages/issues/5968)) ([41749a5](https://github.com/AztecProtocol/aztec-packages/commit/41749a5148b9b5360659e06155cd09d8d7f2a78e))
* MemoryFifo return null when empty and timeout 0 ([#5753](https://github.com/AztecProtocol/aztec-packages/issues/5753)) ([27129e6](https://github.com/AztecProtocol/aztec-packages/commit/27129e6d483e787abea5084c029e560d5d4b5b0e))
* Merge-check ([#5873](https://github.com/AztecProtocol/aztec-packages/issues/5873)) ([c999dae](https://github.com/AztecProtocol/aztec-packages/commit/c999dae76580ea63486aaa17edb774736dc32b89))
* Move fixtures to @aztec/circuits.js/testing/fixtures ([#5826](https://github.com/AztecProtocol/aztec-packages/issues/5826)) ([fb7a617](https://github.com/AztecProtocol/aztec-packages/commit/fb7a6175b185725e607d28a8930a15d88f84e117))
* Nested array equality (https://github.com/noir-lang/noir/pull/4903) ([3b91791](https://github.com/AztecProtocol/aztec-packages/commit/3b9179118369137880277f1444f0e3f94b3f5e79))
* Proper field inversion for bigints (https://github.com/noir-lang/noir/pull/4802) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Refuse to start sequencer without a prover ([#6000](https://github.com/AztecProtocol/aztec-packages/issues/6000)) ([b30d0b6](https://github.com/AztecProtocol/aztec-packages/commit/b30d0b6481b0f0b2241f1fcc9ec9bc0f82308ce9))
* Remove tx.origin ([#5765](https://github.com/AztecProtocol/aztec-packages/issues/5765)) ([c8784d7](https://github.com/AztecProtocol/aztec-packages/commit/c8784d7994937bfae9d23f5d17eb914bae92d8dc)), closes [#5756](https://github.com/AztecProtocol/aztec-packages/issues/5756)
* Reset the noir-gates-diff report on master (https://github.com/noir-lang/noir/pull/4878) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Revert "feat: Sync from noir" ([#6034](https://github.com/AztecProtocol/aztec-packages/issues/6034)) ([6383a09](https://github.com/AztecProtocol/aztec-packages/commit/6383a09ce5d9ab581af5d458bdcb65f92d9011fb))
* **Revert:** "refactor: purge unconstrained functions where possible" ([#5911](https://github.com/AztecProtocol/aztec-packages/issues/5911)) ([c36246b](https://github.com/AztecProtocol/aztec-packages/commit/c36246bb692bf9a3d8e338bbc26a3ce801f0e389))
* Set gas settings in bench ([#5796](https://github.com/AztecProtocol/aztec-packages/issues/5796)) ([86d8176](https://github.com/AztecProtocol/aztec-packages/commit/86d8176279fdc07d3b0eed91f226985e2b15f54e))
* Start spot label ([#5810](https://github.com/AztecProtocol/aztec-packages/issues/5810)) ([96da333](https://github.com/AztecProtocol/aztec-packages/commit/96da3334af2b4b1815f9d3eb9839840c0b76d5bc))
* Start-spot.yml ([#5824](https://github.com/AztecProtocol/aztec-packages/issues/5824)) ([3cf9c2c](https://github.com/AztecProtocol/aztec-packages/commit/3cf9c2c908b361437050e97fcdf67359727eff8b))
* Temporarily exclude bytecode from class id computation ([#5857](https://github.com/AztecProtocol/aztec-packages/issues/5857)) ([55ff125](https://github.com/AztecProtocol/aztec-packages/commit/55ff1251c2c1c02ecbbaadfa38764c5847fee910))
* Update noir-gates-diff commit to use master reference report (https://github.com/noir-lang/noir/pull/4891) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Use correct gates diff commit now that master has been reset ([#6004](https://github.com/AztecProtocol/aztec-packages/issues/6004)) ([d8e5af4](https://github.com/AztecProtocol/aztec-packages/commit/d8e5af4eb023f68140d8cebd39d1d15b4683a4a3))


### Miscellaneous

* `create_fixed_base_constraint` cleanup ([#6047](https://github.com/AztecProtocol/aztec-packages/issues/6047)) ([e1d6526](https://github.com/AztecProtocol/aztec-packages/commit/e1d6526b34f03458f258c0f0fa6967b5f20035f4))
* `TransparentNote` cleanup ([#5904](https://github.com/AztecProtocol/aztec-packages/issues/5904)) ([febf00f](https://github.com/AztecProtocol/aztec-packages/commit/febf00fb841407d54f42634560146568c383f80a))
* Add `as_array` and remove `_slice` variants of hash functions (https://github.com/noir-lang/noir/pull/4675) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Add benchmarks for serializing a dummy program (https://github.com/noir-lang/noir/pull/4813) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Add error conversion from `InterpreterError` (https://github.com/noir-lang/noir/pull/4896) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Add error conversion from `InterpreterError` (https://github.com/noir-lang/noir/pull/4896) ([078aa61](https://github.com/AztecProtocol/aztec-packages/commit/078aa61b06557aba74ac9cce557ee6bd05040feb))
* Add Hir -&gt; Ast conversion (https://github.com/noir-lang/noir/pull/4788) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Add target for individual e2e tests ([#6009](https://github.com/AztecProtocol/aztec-packages/issues/6009)) ([e2842a6](https://github.com/AztecProtocol/aztec-packages/commit/e2842a6e02bd42792c8ee8ca03316e5aa4902f5b))
* Adding devcontainer with create aztec app ([#5752](https://github.com/AztecProtocol/aztec-packages/issues/5752)) ([c72f34e](https://github.com/AztecProtocol/aztec-packages/commit/c72f34ef0ff582269db245643909e02b66a4d37a))
* Adding devcontainer with create aztec app ([#5849](https://github.com/AztecProtocol/aztec-packages/issues/5849)) ([eb1cfef](https://github.com/AztecProtocol/aztec-packages/commit/eb1cfefc4ff11802a97127f10ab30fb5487335fd))
* Allow expressions in constant generation ([#5839](https://github.com/AztecProtocol/aztec-packages/issues/5839)) ([cb1e25b](https://github.com/AztecProtocol/aztec-packages/commit/cb1e25b8c6f203d8a7e4beb2f027d72bee981695))
* **avm-simulator:** Remove AvmContext::raw_* external calls ([#5869](https://github.com/AztecProtocol/aztec-packages/issues/5869)) ([0c9d0b4](https://github.com/AztecProtocol/aztec-packages/commit/0c9d0b4e611472e0e8718449f6d8f2451e0391a0))
* **avm:** More test cleanup ([#5771](https://github.com/AztecProtocol/aztec-packages/issues/5771)) ([23d0070](https://github.com/AztecProtocol/aztec-packages/commit/23d0070095bf7d32cfdcf97e7aea348753bb7492))
* **avm:** Negative unit tests for AVM CAST opcode ([#5907](https://github.com/AztecProtocol/aztec-packages/issues/5907)) ([4465e3b](https://github.com/AztecProtocol/aztec-packages/commit/4465e3be870963ea435d9a0cd063397020442f0b)), closes [#5908](https://github.com/AztecProtocol/aztec-packages/issues/5908)
* **avm:** Re-enable proof in some unit tests ([#6056](https://github.com/AztecProtocol/aztec-packages/issues/6056)) ([0ebee28](https://github.com/AztecProtocol/aztec-packages/commit/0ebee28b14042417956a02a3247af68f4f13dcf5)), closes [#6019](https://github.com/AztecProtocol/aztec-packages/issues/6019)
* **aztec-nr:** Unencrypted logs go behind context ([#5871](https://github.com/AztecProtocol/aztec-packages/issues/5871)) ([6a5ad7c](https://github.com/AztecProtocol/aztec-packages/commit/6a5ad7ccfe7fc17237a5a6237493c1fbb6af6d53))
* Bump `rustls` to v0.21.11 (https://github.com/noir-lang/noir/pull/4895) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Bump MSRV to `1.74.1` (https://github.com/noir-lang/noir/pull/4873) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Bump public call depth ([#5845](https://github.com/AztecProtocol/aztec-packages/issues/5845)) ([b61502f](https://github.com/AztecProtocol/aztec-packages/commit/b61502f372e5b09d8bff138fdb74e3175b5186ff))
* Bundle spot runner + target more spot types ([#6012](https://github.com/AztecProtocol/aztec-packages/issues/6012)) ([d51c8b8](https://github.com/AztecProtocol/aztec-packages/commit/d51c8b8698187b4a69aadf1ce47f1565d71d2827))
* Check working copy is clean before extract-repo ([#5851](https://github.com/AztecProtocol/aztec-packages/issues/5851)) ([8ff9767](https://github.com/AztecProtocol/aztec-packages/commit/8ff9767c213d172ee1568aeedaa7265ead7b5466))
* **ci:** Address start-runner edge-cases ([#5888](https://github.com/AztecProtocol/aztec-packages/issues/5888)) ([564b893](https://github.com/AztecProtocol/aztec-packages/commit/564b893486375e88945bdeb63364bca374f376fb))
* **ci:** Back to on-demand ([#5998](https://github.com/AztecProtocol/aztec-packages/issues/5998)) ([f2f15f0](https://github.com/AztecProtocol/aztec-packages/commit/f2f15f0808c7b03a3ef90afae4b71015cfe1b9fd))
* **ci:** Better docker prune ([#5889](https://github.com/AztecProtocol/aztec-packages/issues/5889)) ([b5a8e02](https://github.com/AztecProtocol/aztec-packages/commit/b5a8e02edf44bacc3415e478b75b182c5b352ca2))
* **ci:** Don't interleave docker prunes ([#5914](https://github.com/AztecProtocol/aztec-packages/issues/5914)) ([2b51fee](https://github.com/AztecProtocol/aztec-packages/commit/2b51fee7dee663ee4a8cc54b5a5412d862d04862))
* **ci:** Don't use redirected earthly ([#5909](https://github.com/AztecProtocol/aztec-packages/issues/5909)) ([2e55713](https://github.com/AztecProtocol/aztec-packages/commit/2e557130ace2f6db555fa27eab80ccfc18f0d88e))
* **ci:** Fix alerts on msrv issues (https://github.com/noir-lang/noir/pull/4816) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* **ci:** Fix concurrency key ([#5962](https://github.com/AztecProtocol/aztec-packages/issues/5962)) ([7eb164f](https://github.com/AztecProtocol/aztec-packages/commit/7eb164f28a65426482557cc5dfcb31b9e7c23ab9))
* **ci:** Hotfix runners not starting ([067e460](https://github.com/AztecProtocol/aztec-packages/commit/067e4607019c17dad7c3861734c4bee0e849fbad))
* **ci:** Make syncing out to Noir manual ([#5997](https://github.com/AztecProtocol/aztec-packages/issues/5997)) ([1801db8](https://github.com/AztecProtocol/aztec-packages/commit/1801db88640b0e012fa32650bf8074587709ef83))
* **ci:** Move yarn-project-test to new CI ([#5850](https://github.com/AztecProtocol/aztec-packages/issues/5850)) ([d8254ef](https://github.com/AztecProtocol/aztec-packages/commit/d8254efe958898d28dbe25474b9eb21cebb4ed2c))
* **ci:** Notify internal Slack channel when CI breaks on master ([#5788](https://github.com/AztecProtocol/aztec-packages/issues/5788)) ([70b3f3f](https://github.com/AztecProtocol/aztec-packages/commit/70b3f3f1aebbb626014d54e121e841938407bdaf))
* **ci:** Notify on ARM failures ([#5847](https://github.com/AztecProtocol/aztec-packages/issues/5847)) ([bdb59cb](https://github.com/AztecProtocol/aztec-packages/commit/bdb59cb4dc2c691798be3d1ef46d422f8fcd930d))
* **ci:** Prevent haywire logs ([#5966](https://github.com/AztecProtocol/aztec-packages/issues/5966)) ([b12f609](https://github.com/AztecProtocol/aztec-packages/commit/b12f60994fdd54cb4d8e18e444c207e319f9d6a6))
* **ci:** Reenable deploy tests ([#6011](https://github.com/AztecProtocol/aztec-packages/issues/6011)) ([087a624](https://github.com/AztecProtocol/aztec-packages/commit/087a624689ca34de4ac6dca759cf5e644a163b37))
* **ci:** Reenable spot ([348b34f](https://github.com/AztecProtocol/aztec-packages/commit/348b34f868e98c1e6dc388b164b0df6ee131ae6c))
* **ci:** Remove devnet deployments (for now) ([#5912](https://github.com/AztecProtocol/aztec-packages/issues/5912)) ([d9c1ee9](https://github.com/AztecProtocol/aztec-packages/commit/d9c1ee938ea8ff94639f29e457bd5e04ab2b9e8a))
* **ci:** Use on-demand for now ([#5933](https://github.com/AztecProtocol/aztec-packages/issues/5933)) ([f77636f](https://github.com/AztecProtocol/aztec-packages/commit/f77636f686d5416c9e2f893ed40730a08b48a5ee))
* Clean up and clarify some translator flavor logic ([#5965](https://github.com/AztecProtocol/aztec-packages/issues/5965)) ([242b364](https://github.com/AztecProtocol/aztec-packages/commit/242b364aacdf662cd6dab6254562ab5f61a58731))
* Clean up stopped instances ([#6030](https://github.com/AztecProtocol/aztec-packages/issues/6030)) ([1318bd5](https://github.com/AztecProtocol/aztec-packages/commit/1318bd5493e65ac8f478d74bc1537dea2facd575))
* **debugger:** Docs (https://github.com/noir-lang/noir/pull/4145) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Delete dead code (https://github.com/noir-lang/noir/pull/4906) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Delete dead code (https://github.com/noir-lang/noir/pull/4906) ([078aa61](https://github.com/AztecProtocol/aztec-packages/commit/078aa61b06557aba74ac9cce557ee6bd05040feb))
* Delete flake.lock (https://github.com/noir-lang/noir/pull/4855) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Delete flake.lock (https://github.com/noir-lang/noir/pull/4855) ([078aa61](https://github.com/AztecProtocol/aztec-packages/commit/078aa61b06557aba74ac9cce557ee6bd05040feb))
* Delete unnecessary Prover.toml file (https://github.com/noir-lang/noir/pull/4829) ([beab8c9](https://github.com/AztecProtocol/aztec-packages/commit/beab8c93857536e07fa37994213fc664a5864013))
* Delete unused brillig methods (https://github.com/noir-lang/noir/pull/4887) ([8f73f18](https://github.com/AztecProtocol/aztec-packages/commit/8f73f18f3c07de0fd5e247ade5a48109c37c1bc5))
* Do not aggregate note decryption time for benchmarks ([#6032](https://github.com/AztecProtocol/aztec-packages/issues/6032)) ([658a880](https://github.com/AztecProtocol/aztec-packages/commit/658a880fe40273e16cb65bbc18ede7740895baf4))
* Do not bootstrap cache if working copy is dirty ([#6033](https://github.com/AztecProtocol/aztec-packages/issues/6033)) ([3671932](https://github.com/AztecProtocol/aztec-packages/commit/367193253670a1d61ffa440d94dad4b9d068e72f))
* **docs:** Fix migration notes ([#6083](https://github.com/AztecProtocol/aztec-packages/issues/6083)) ([e1f3e32](https://github.com/AztecProtocol/aztec-packages/commit/e1f3e320f15003282ca5b5ea707471cfcd1b6354))
* **docs:** Fix wrong Nargo.toml workspace examples (https://github.com/noir-lang/noir/pull/4822) ([beab8c9](https://github.com/AztecProtocol/aztec-packages/commit/beab8c93857536e07fa37994213fc664a5864013))
* **docs:** Remove 'yellow paper' reference from protocol specs ([#5872](https://github.com/AztecProtocol/aztec-packages/issues/5872)) ([b348ec1](https://github.com/AztecProtocol/aztec-packages/commit/b348ec149b7df0d4620a79d501834a6590078160))
* **docs:** Remove link to play.noir-lang.org (https://github.com/noir-lang/noir/pull/4872) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* **experimental:** Add scan pass and `into_expression` for comptime interpreter (https://github.com/noir-lang/noir/pull/4884) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* **experimental:** Improve variable not defined error message in comptime interpreter (https://github.com/noir-lang/noir/pull/4889) ([2e64428](https://github.com/AztecProtocol/aztec-packages/commit/2e64428af9525bd8c390931061505f7b48d729a4))
* Extend SharedMutable tests ([#6005](https://github.com/AztecProtocol/aztec-packages/issues/6005)) ([4cee8e0](https://github.com/AztecProtocol/aztec-packages/commit/4cee8e0644780e527395da452a831055ec41a4c7))
* Fix alerts on rust msrv (https://github.com/noir-lang/noir/pull/4817) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Fix and reenable e2e account init fees test ([#5878](https://github.com/AztecProtocol/aztec-packages/issues/5878)) ([cec8191](https://github.com/AztecProtocol/aztec-packages/commit/cec819178635b41c3b310431afa435bea207d925))
* Fix formatting and serialization that fails CI run ([#5942](https://github.com/AztecProtocol/aztec-packages/issues/5942)) ([da67f18](https://github.com/AztecProtocol/aztec-packages/commit/da67f181b09439e2e2e04209ed3d84c21c7cc6bf))
* Fix typo in error message ([#5139](https://github.com/AztecProtocol/aztec-packages/issues/5139)) ([b194f83](https://github.com/AztecProtocol/aztec-packages/commit/b194f83188f0e874a1f4c67a512370d3efcf883b))
* Flag account init test as flakey ([ea030e5](https://github.com/AztecProtocol/aztec-packages/commit/ea030e534b965d154b00ececd5974606dd85f217))
* Flag two failing e2e tests as flakey until we fix them ([901ae87](https://github.com/AztecProtocol/aztec-packages/commit/901ae87795ba39420258e5d70b92221f11f7d20e))
* Improve `compute_note_hash_and_nullifier` autogeneration and `NoteProcessor` warnings ([#5838](https://github.com/AztecProtocol/aztec-packages/issues/5838)) ([566f25c](https://github.com/AztecProtocol/aztec-packages/commit/566f25c25744501ce1ae31243820ef549d9b1f30))
* Improved naming in `TxExecutionRequest` ([#6014](https://github.com/AztecProtocol/aztec-packages/issues/6014)) ([f2364d4](https://github.com/AztecProtocol/aztec-packages/commit/f2364d40f850414029ed967eb05c48b5be2ffff6))
* Integrate new key store ([#5731](https://github.com/AztecProtocol/aztec-packages/issues/5731)) ([ab9fe78](https://github.com/AztecProtocol/aztec-packages/commit/ab9fe780e8a9fc3187a02b37ddbefa609d3bff8f)), closes [#5720](https://github.com/AztecProtocol/aztec-packages/issues/5720)
* Introducing re-export for poseidon2 ([#5898](https://github.com/AztecProtocol/aztec-packages/issues/5898)) ([03a87b8](https://github.com/AztecProtocol/aztec-packages/commit/03a87b8d97b72f8144ef83b679eed564048d4683)), closes [#5863](https://github.com/AztecProtocol/aztec-packages/issues/5863)
* Lift run-e2e to yarn-project earthfile ([#6018](https://github.com/AztecProtocol/aztec-packages/issues/6018)) ([b7900b8](https://github.com/AztecProtocol/aztec-packages/commit/b7900b88a66bfd9d75b92ed05a4236dda41b2013))
* Migrate blacklist token to use shared mutable ([#5885](https://github.com/AztecProtocol/aztec-packages/issues/5885)) ([26c1eec](https://github.com/AztecProtocol/aztec-packages/commit/26c1eecc76613c7c7883031691672ba36fb16152))
* More explicit `self` parameter in `Into` trait (https://github.com/noir-lang/noir/pull/4867) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* No implicit overrides ([#5792](https://github.com/AztecProtocol/aztec-packages/issues/5792)) ([0fafaef](https://github.com/AztecProtocol/aztec-packages/commit/0fafaef8eb92ba261c1aefe1daab2539caab0bea))
* Nuking CLI ([#5865](https://github.com/AztecProtocol/aztec-packages/issues/5865)) ([c48c913](https://github.com/AztecProtocol/aztec-packages/commit/c48c91349dd592520ee33d4c45bc2d3913883541)), closes [#5894](https://github.com/AztecProtocol/aztec-packages/issues/5894)
* Nuking unused keys.nr ([#5910](https://github.com/AztecProtocol/aztec-packages/issues/5910)) ([1d3af93](https://github.com/AztecProtocol/aztec-packages/commit/1d3af93f26d7b09389debe4b7046ae3359ff1893))
* Optimize poseidon2 implementation (https://github.com/noir-lang/noir/pull/4807) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Patch jest to not use JSON serialization in message passing ([#5883](https://github.com/AztecProtocol/aztec-packages/issues/5883)) ([1c24c8e](https://github.com/AztecProtocol/aztec-packages/commit/1c24c8e53f190c7b1ac3b4d8896abb4ab6b5712b))
* Prepare ScheduledValueChange for mutable delays. ([#6085](https://github.com/AztecProtocol/aztec-packages/issues/6085)) ([cfa850b](https://github.com/AztecProtocol/aztec-packages/commit/cfa850bbbad9ff54a7efd9fd7a045a6d3f158ebf))
* ProvingKey has ProverPolynomials ([#5940](https://github.com/AztecProtocol/aztec-packages/issues/5940)) ([0a64279](https://github.com/AztecProtocol/aztec-packages/commit/0a64279ba1b2b3bb6627c675b8a0b116be17f579))
* Purge unconstrained functions where possible ([#5819](https://github.com/AztecProtocol/aztec-packages/issues/5819)) ([ce84161](https://github.com/AztecProtocol/aztec-packages/commit/ce8416174f360a4a00cc70c20c8f2d99354aec2e)), closes [#5451](https://github.com/AztecProtocol/aztec-packages/issues/5451)
* Purging portal addresses ([#5842](https://github.com/AztecProtocol/aztec-packages/issues/5842)) ([4faccad](https://github.com/AztecProtocol/aztec-packages/commit/4faccad569e39228b0f3fbf741fc95e3a189e276))
* Redo typo PR by dockercui ([#5930](https://github.com/AztecProtocol/aztec-packages/issues/5930)) ([b23e42f](https://github.com/AztecProtocol/aztec-packages/commit/b23e42f5f897936bb9607ba94e57f31723d9984b))
* Redo typo PR by satyambnsal ([#5929](https://github.com/AztecProtocol/aztec-packages/issues/5929)) ([d28b1cb](https://github.com/AztecProtocol/aztec-packages/commit/d28b1cbc0364c1d760187ffa7263c147e9295dd4))
* Redo typo PR by socialsister ([#5931](https://github.com/AztecProtocol/aztec-packages/issues/5931)) ([e817f78](https://github.com/AztecProtocol/aztec-packages/commit/e817f78158e895807151f6b451cb506cab9c2510))
* Redo typo script ([#5926](https://github.com/AztecProtocol/aztec-packages/issues/5926)) ([41fa87e](https://github.com/AztecProtocol/aztec-packages/commit/41fa87e1216eeb6ff774eb1925797f9ae721c70b))
* Reenable account init fees e2e ([#5938](https://github.com/AztecProtocol/aztec-packages/issues/5938)) ([49c45c3](https://github.com/AztecProtocol/aztec-packages/commit/49c45c38f01e5a2034f81506089640d93c87744d))
* Refactor e2e tests to use the new simulate fn ([#5854](https://github.com/AztecProtocol/aztec-packages/issues/5854)) ([e7d2aff](https://github.com/AztecProtocol/aztec-packages/commit/e7d2aff3a1922dc685bc859901dffdb83933dff2))
* Refactor public cross chain tests for speed ([#6082](https://github.com/AztecProtocol/aztec-packages/issues/6082)) ([6065a6c](https://github.com/AztecProtocol/aztec-packages/commit/6065a6c4157a2d356964f4c5476425da55e09728))
* Refactor recursive verifier tests ([#6063](https://github.com/AztecProtocol/aztec-packages/issues/6063)) ([94a2d61](https://github.com/AztecProtocol/aztec-packages/commit/94a2d61d10d8e21d0080b7ea3a8b283f8dd0162f))
* Refactor token blacklist test for speed ([#6054](https://github.com/AztecProtocol/aztec-packages/issues/6054)) ([ab36d7e](https://github.com/AztecProtocol/aztec-packages/commit/ab36d7e42ccd6403c5b6967c4e2b319ab7b85d37))
* Release Noir(0.28.0) (https://github.com/noir-lang/noir/pull/4776) ([3b91791](https://github.com/AztecProtocol/aztec-packages/commit/3b9179118369137880277f1444f0e3f94b3f5e79))
* Remove `Opcode::Brillig` from ACIR ([#5995](https://github.com/AztecProtocol/aztec-packages/issues/5995)) ([ffd5f46](https://github.com/AztecProtocol/aztec-packages/commit/ffd5f460fce8b1f12265730f97c8cfcd3a4774ca))
* Remove `SecondaryAttribute::Event` (https://github.com/noir-lang/noir/pull/4868) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Remove empty yarn.lock ([#5835](https://github.com/AztecProtocol/aztec-packages/issues/5835)) ([c3dd039](https://github.com/AztecProtocol/aztec-packages/commit/c3dd039e5d2a779cc9bda1c0ac46306563914578))
* Remove get_portal_address oracle ([#5816](https://github.com/AztecProtocol/aztec-packages/issues/5816)) ([67c2823](https://github.com/AztecProtocol/aztec-packages/commit/67c2823e3bb302d4d7a28bca03de468c92336680))
* Remove initialisation of logger in `acvm_js` tests (https://github.com/noir-lang/noir/pull/4850) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Remove l1 gas ([#6069](https://github.com/AztecProtocol/aztec-packages/issues/6069)) ([0e3705f](https://github.com/AztecProtocol/aztec-packages/commit/0e3705f2591c1da36778c316d8b7ab914f5d6757))
* Remove private kernel snapshot test ([#5829](https://github.com/AztecProtocol/aztec-packages/issues/5829)) ([9434784](https://github.com/AztecProtocol/aztec-packages/commit/9434784b12f5e5402e93596110ee2e131317b251))
* Remove pub wildcard import of ast into `noirc_frontend` root (https://github.com/noir-lang/noir/pull/4862) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Remove unnecessary casts in `BoundedVec` (https://github.com/noir-lang/noir/pull/4831) ([beab8c9](https://github.com/AztecProtocol/aztec-packages/commit/beab8c93857536e07fa37994213fc664a5864013))
* Rename 'global' to 'function' in the monomorphization pass (https://github.com/noir-lang/noir/pull/4774) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Rename capture to end_setup ([#6008](https://github.com/AztecProtocol/aztec-packages/issues/6008)) ([61e61ab](https://github.com/AztecProtocol/aztec-packages/commit/61e61aba60ec02d12141ef396c4e827c800d57bf))
* Renaming `noir-compiler` as `builder` ([#5951](https://github.com/AztecProtocol/aztec-packages/issues/5951)) ([185e57d](https://github.com/AztecProtocol/aztec-packages/commit/185e57d51e8bbf6194628ce62db3dd44f11634a4))
* Reorganize gas fields in structs ([#5828](https://github.com/AztecProtocol/aztec-packages/issues/5828)) ([e26d342](https://github.com/AztecProtocol/aztec-packages/commit/e26d342c5646425da31d95c50ce94025e5c6d053))
* Replace queue with facade over CircuitProver ([#5972](https://github.com/AztecProtocol/aztec-packages/issues/5972)) ([dafb3ed](https://github.com/AztecProtocol/aztec-packages/commit/dafb3edc799b2adaf285ffe57b41630040c68449))
* Replace relative paths to noir-protocol-circuits ([b723534](https://github.com/AztecProtocol/aztec-packages/commit/b723534db2fcbd3399aca722354df7c45ee8a84f))
* Replace relative paths to noir-protocol-circuits ([20057b2](https://github.com/AztecProtocol/aztec-packages/commit/20057b25bbf9c6b007fe3595eca7a2cff872aa52))
* Replace relative paths to noir-protocol-circuits ([543ff13](https://github.com/AztecProtocol/aztec-packages/commit/543ff131c32cd005de2e83fe2af59b132c5896de))
* Replace relative paths to noir-protocol-circuits ([d0622cf](https://github.com/AztecProtocol/aztec-packages/commit/d0622cffa2dfffdf8bd96cc34627a78aeb8a72e5))
* Replace relative paths to noir-protocol-circuits ([41d6e81](https://github.com/AztecProtocol/aztec-packages/commit/41d6e81426090c5b8c50787123bac826a732204d))
* Replace relative paths to noir-protocol-circuits ([c0c8e3f](https://github.com/AztecProtocol/aztec-packages/commit/c0c8e3f880076d20cca96a3c92a1484abdcc66a0))
* Replace relative paths to noir-protocol-circuits ([8b33a58](https://github.com/AztecProtocol/aztec-packages/commit/8b33a58e095815d5b131ab3fbd668c4e88680e13))
* Replace relative paths to noir-protocol-circuits ([ce4a010](https://github.com/AztecProtocol/aztec-packages/commit/ce4a010c7c075cb68bed91e0123d4fcecc7c6938))
* Replace use of PublicContext with interface ([#5840](https://github.com/AztecProtocol/aztec-packages/issues/5840)) ([834067f](https://github.com/AztecProtocol/aztec-packages/commit/834067f12b07a36b9348a368b83d61d789c5c22b))
* Reset noir-gates-diff report on master  ([#6003](https://github.com/AztecProtocol/aztec-packages/issues/6003)) ([7f01f7d](https://github.com/AztecProtocol/aztec-packages/commit/7f01f7d16230fe011a3f52db9e477a958796b202))
* Revert "Check working copy is clean before extract-repo ([#5851](https://github.com/AztecProtocol/aztec-packages/issues/5851))" ([ec21fb8](https://github.com/AztecProtocol/aztec-packages/commit/ec21fb8251c34d535fd0c5e08f354cfa22c25320))
* **revert:** "chore(ci): don't use redirected earthly" ([#6062](https://github.com/AztecProtocol/aztec-packages/issues/6062)) ([26cba9e](https://github.com/AztecProtocol/aztec-packages/commit/26cba9e4ef9b63bf100e451b66cfe3ea62ab416c))
* Rework workspace structure for utils crates (https://github.com/noir-lang/noir/pull/4886) ([e615a83](https://github.com/AztecProtocol/aztec-packages/commit/e615a831a12b78644b798e12395d970bf5601948))
* Rework workspace structure for utils crates (https://github.com/noir-lang/noir/pull/4886) ([078aa61](https://github.com/AztecProtocol/aztec-packages/commit/078aa61b06557aba74ac9cce557ee6bd05040feb))
* Run clippy (https://github.com/noir-lang/noir/pull/4810) ([84c930a](https://github.com/AztecProtocol/aztec-packages/commit/84c930a912ca9ed0d9c0ce2436309a4e9a840bcb))
* Run flakey e2e tests on CI but allow failure ([#5937](https://github.com/AztecProtocol/aztec-packages/issues/5937)) ([a074251](https://github.com/AztecProtocol/aztec-packages/commit/a07425184d08d647588e3778221740e724b1b052))
* Run noir projects tests in earthly ([#6024](https://github.com/AztecProtocol/aztec-packages/issues/6024)) ([e950433](https://github.com/AztecProtocol/aztec-packages/commit/e9504333dcb25c3f9bd1344743a0e12e7719ab2e))
* Simplify computation of pow for each sumcheck round ([#5903](https://github.com/AztecProtocol/aztec-packages/issues/5903)) ([74a9d5d](https://github.com/AztecProtocol/aztec-packages/commit/74a9d5d6736a4376e40e501765974b9686ca738e))
* Temporarily skip failing gas tests ([#5874](https://github.com/AztecProtocol/aztec-packages/issues/5874)) ([ad55af0](https://github.com/AztecProtocol/aztec-packages/commit/ad55af0d44b3c818d5e42fe75bb72fa95e88c309))
* Update noir README (https://github.com/noir-lang/noir/pull/4856) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Update NoirJS tutorial (https://github.com/noir-lang/noir/pull/4861) ([cea5107](https://github.com/AztecProtocol/aztec-packages/commit/cea51073be4ecc65a3c8d36cfe107df8390b853f))
* Use BrilligCall for unconstrained main and update AVM transpiler ([#5797](https://github.com/AztecProtocol/aztec-packages/issues/5797)) ([3fb94c0](https://github.com/AztecProtocol/aztec-packages/commit/3fb94c0cd5ffba20a99b97c0088ae5ef357c205d))
* Use new mock.get_last_params() for public storage writes ([#5823](https://github.com/AztecProtocol/aztec-packages/issues/5823)) ([6b0f919](https://github.com/AztecProtocol/aztec-packages/commit/6b0f919d83209a83e5d1900942a160424b30fe22))
* Using poseidon2 when computing a nullifier ([#5906](https://github.com/AztecProtocol/aztec-packages/issues/5906)) ([3a10e5e](https://github.com/AztecProtocol/aztec-packages/commit/3a10e5e75b8053dfea13a4901873d42ca01ca7c2)), closes [#5832](https://github.com/AztecProtocol/aztec-packages/issues/5832) [#1205](https://github.com/AztecProtocol/aztec-packages/issues/1205)
* Validate instance deployer address every time we request it ([#5848](https://github.com/AztecProtocol/aztec-packages/issues/5848)) ([2422891](https://github.com/AztecProtocol/aztec-packages/commit/2422891fa021cfb4c83b91849ff1f22baa93a4b9))
* Workaround earthly flake ([#5811](https://github.com/AztecProtocol/aztec-packages/issues/5811)) ([dd3a521](https://github.com/AztecProtocol/aztec-packages/commit/dd3a521b59b950871645306179d23a3f332ef6f3))
* Yarn build:dev don't clear terminal ([#5970](https://github.com/AztecProtocol/aztec-packages/issues/5970)) ([b3fdb3b](https://github.com/AztecProtocol/aztec-packages/commit/b3fdb3b59e887974b89db0eb209e16b0630b1360))


### Documentation

* Addition around Nargo.toml search ([#5943](https://github.com/AztecProtocol/aztec-packages/issues/5943)) ([d1350da](https://github.com/AztecProtocol/aztec-packages/commit/d1350da9e3d78fa53ccd5663219f70c67df4c66d))
* Aztec smart contract tutorial - crowdfunding ([#5786](https://github.com/AztecProtocol/aztec-packages/issues/5786)) ([91cc0a4](https://github.com/AztecProtocol/aztec-packages/commit/91cc0a424031b9b8346cc9182f303d1468b1179b))
* Gas and accounting ([#5855](https://github.com/AztecProtocol/aztec-packages/issues/5855)) ([d0b3f06](https://github.com/AztecProtocol/aztec-packages/commit/d0b3f06ff29d5e5ac99097cb1ab2906190eec5c3))
* Migration notes for GasOpts in public calls ([#5822](https://github.com/AztecProtocol/aztec-packages/issues/5822)) ([edeea3d](https://github.com/AztecProtocol/aztec-packages/commit/edeea3dfe425b83b36c981dde3ce169e33aaece9))
* Remove mentions of slow updates ([#5884](https://github.com/AztecProtocol/aztec-packages/issues/5884)) ([029d1e5](https://github.com/AztecProtocol/aztec-packages/commit/029d1e5d4ff679f73dce72779cb316a1d8c7eda8))
* Shared state ([#5963](https://github.com/AztecProtocol/aztec-packages/issues/5963)) ([86c106f](https://github.com/AztecProtocol/aztec-packages/commit/86c106f122b3fe0daa5853f7824bb68abadf70d0))
* Update emit_event.md ([#5964](https://github.com/AztecProtocol/aztec-packages/issues/5964)) ([616a8f3](https://github.com/AztecProtocol/aztec-packages/commit/616a8f328f893ab563b1d90c5c627572cf838968))
* Update info around VERSION ([#5891](https://github.com/AztecProtocol/aztec-packages/issues/5891)) ([e1eb98e](https://github.com/AztecProtocol/aztec-packages/commit/e1eb98e85e6ef6ca87f502036426457c8c2a7efc))

## [0.35.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.35.0...aztec-packages-v0.35.1) (2024-04-16)


### Bug Fixes

* Disable bench-tx-size until fixed ([#5789](https://github.com/AztecProtocol/aztec-packages/issues/5789)) ([9a85c20](https://github.com/AztecProtocol/aztec-packages/commit/9a85c205f3a539494dc57fc7abb7c7700ad3a3df))

## [0.35.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.34.0...aztec-packages-v0.35.0) (2024-04-16)


### ⚠ BREAKING CHANGES

* Use fixed size arrays in black box functions where sizes are known ([#5620](https://github.com/AztecProtocol/aztec-packages/issues/5620))
* trap with revert data ([#5732](https://github.com/AztecProtocol/aztec-packages/issues/5732))
* **acir:** BrilligCall opcode  ([#5709](https://github.com/AztecProtocol/aztec-packages/issues/5709))
* rename request_max_block_number ([#5675](https://github.com/AztecProtocol/aztec-packages/issues/5675))
* pay fee for account init ([#5601](https://github.com/AztecProtocol/aztec-packages/issues/5601))

### Features

* **acir:** BrilligCall opcode  ([#5709](https://github.com/AztecProtocol/aztec-packages/issues/5709)) ([f06f64c](https://github.com/AztecProtocol/aztec-packages/commit/f06f64c451333d36eb05951202d69ee85cca8851))
* Add serialisation methods ([#5749](https://github.com/AztecProtocol/aztec-packages/issues/5749)) ([20d290c](https://github.com/AztecProtocol/aztec-packages/commit/20d290c38fe4589b4bdafc6c529d6fd903d596e4))
* App siloing in new key store ([#5721](https://github.com/AztecProtocol/aztec-packages/issues/5721)) ([ae37d32](https://github.com/AztecProtocol/aztec-packages/commit/ae37d32ce58417eaa5345f2b77f92f1dfe6709d1)), closes [#5635](https://github.com/AztecProtocol/aztec-packages/issues/5635)
* **avm-simulator:** Plumb noir assertion messages ([#5774](https://github.com/AztecProtocol/aztec-packages/issues/5774)) ([2cf11ac](https://github.com/AztecProtocol/aztec-packages/commit/2cf11ac76805b8d648a4b32d7cd6446eb31b9a35))
* **avm:** CMOV opcode ([#5575](https://github.com/AztecProtocol/aztec-packages/issues/5575)) ([19dbe46](https://github.com/AztecProtocol/aztec-packages/commit/19dbe46bce95221bf2e68c9361618998a6bdc64f)), closes [#5557](https://github.com/AztecProtocol/aztec-packages/issues/5557)
* **avm:** Enable contract testing with bb binary ([#5584](https://github.com/AztecProtocol/aztec-packages/issues/5584)) ([d007d79](https://github.com/AztecProtocol/aztec-packages/commit/d007d79c7014261d9c663e28c948600d92e85759))
* **avm:** Enable range check on the ALU registers ([#5696](https://github.com/AztecProtocol/aztec-packages/issues/5696)) ([202fc1b](https://github.com/AztecProtocol/aztec-packages/commit/202fc1b750e83f91c32b128b981db1c5c92ef3f2))
* **avm:** Keccak as blackbox function ([#5722](https://github.com/AztecProtocol/aztec-packages/issues/5722)) ([6ea677a](https://github.com/AztecProtocol/aztec-packages/commit/6ea677aa97e6a597f5c6b580e655143ffeddbccf))
* **avm:** Poseidon2_permutation as black box ([#5707](https://github.com/AztecProtocol/aztec-packages/issues/5707)) ([5526b36](https://github.com/AztecProtocol/aztec-packages/commit/5526b36721a57b143ff8d0f381f94be275ccf59c))
* **avm:** Sha256 as blackbox function ([#5727](https://github.com/AztecProtocol/aztec-packages/issues/5727)) ([cac9cba](https://github.com/AztecProtocol/aztec-packages/commit/cac9cba8974a4923bce9e8f4627e2654bfab4f81))
* **avm:** Take sizeOffset in CALL ([#5763](https://github.com/AztecProtocol/aztec-packages/issues/5763)) ([95eadd6](https://github.com/AztecProtocol/aztec-packages/commit/95eadd67a72a286f07876f80586e6d57605d0af5))
* Brillig heterogeneous memory cells ([#5608](https://github.com/AztecProtocol/aztec-packages/issues/5608)) ([3287aa2](https://github.com/AztecProtocol/aztec-packages/commit/3287aa29c1e85dd89d5ae9f73bffa9406cc47d08))
* Change public nullifiers api ([#5660](https://github.com/AztecProtocol/aztec-packages/issues/5660)) ([986e7f9](https://github.com/AztecProtocol/aztec-packages/commit/986e7f924e9af6461e3e88de29f22cf2a8f45c4e))
* Changing finite field arithmetic in wasm to 29 bits for multiplications ([#5435](https://github.com/AztecProtocol/aztec-packages/issues/5435)) ([b2d9b9d](https://github.com/AztecProtocol/aztec-packages/commit/b2d9b9d5f1764b159e081b3cc9806ee83fdf341f))
* **ci:** Turn on new CI as mandatory ([#5761](https://github.com/AztecProtocol/aztec-packages/issues/5761)) ([bebed32](https://github.com/AztecProtocol/aztec-packages/commit/bebed32272e0974de21b5c7d21344d3cf1597a24))
* **docs:** Merge yellow paper into docs protocol specs section ([#5668](https://github.com/AztecProtocol/aztec-packages/issues/5668)) ([66dc509](https://github.com/AztecProtocol/aztec-packages/commit/66dc5091b2aff53580e1a313e1001369ffc87e6b))
* E2e token contract can run in 2m with snapshots and test separation. ([#5526](https://github.com/AztecProtocol/aztec-packages/issues/5526)) ([b0037dd](https://github.com/AztecProtocol/aztec-packages/commit/b0037dd051bd0312abe79d686cd93231cfe63d56))
* Export poseidon2_permutation and add to foundation/crypto ([#5706](https://github.com/AztecProtocol/aztec-packages/issues/5706)) ([6b91e27](https://github.com/AztecProtocol/aztec-packages/commit/6b91e2776de8fd5b1f489b5cfeee83c7e0996c2e))
* Get last mock oracles params (https://github.com/noir-lang/noir/pull/4789) ([825c455](https://github.com/AztecProtocol/aztec-packages/commit/825c455a62faeae5d148ce4f914efacb8f4c50fd))
* Impl of missing functionality in new key store ([#5750](https://github.com/AztecProtocol/aztec-packages/issues/5750)) ([af49a29](https://github.com/AztecProtocol/aztec-packages/commit/af49a290722c3430ad26a458cfb489b8b4ea7604))
* LT/LTE for AVM ([#5559](https://github.com/AztecProtocol/aztec-packages/issues/5559)) ([350abeb](https://github.com/AztecProtocol/aztec-packages/commit/350abeb4c88d7e7878abc32e9263c558633f0df9))
* New key store ([#5653](https://github.com/AztecProtocol/aztec-packages/issues/5653)) ([3e44a58](https://github.com/AztecProtocol/aztec-packages/commit/3e44a580a3769d7e65124294500ccedab9cdfce4)), closes [#5607](https://github.com/AztecProtocol/aztec-packages/issues/5607)
* Pay fee for account init ([#5601](https://github.com/AztecProtocol/aztec-packages/issues/5601)) ([aca804f](https://github.com/AztecProtocol/aztec-packages/commit/aca804f96ca9e74b6b553449333e195c0639b151))
* Poseidon separator ([#5717](https://github.com/AztecProtocol/aztec-packages/issues/5717)) ([d5256d2](https://github.com/AztecProtocol/aztec-packages/commit/d5256d29f3bc7d2094ba760c5a528dd61475bb40))
* Proving the rollup circuits ([#5599](https://github.com/AztecProtocol/aztec-packages/issues/5599)) ([145cbcd](https://github.com/AztecProtocol/aztec-packages/commit/145cbcda61fd73f4e135348b31c59c774cfae965))
* Public Kernel proving orchestration ([#5748](https://github.com/AztecProtocol/aztec-packages/issues/5748)) ([2ae0ee5](https://github.com/AztecProtocol/aztec-packages/commit/2ae0ee537b37c444f59b8255bd856f4da2ef818f))
* Rename request_max_block_number ([#5675](https://github.com/AztecProtocol/aztec-packages/issues/5675)) ([c695fcd](https://github.com/AztecProtocol/aztec-packages/commit/c695fcd91041a4ba9fd1d38b170993612c5e2ad1))
* Separate nullfier_inclusion checks for private/public/avm ([#5657](https://github.com/AztecProtocol/aztec-packages/issues/5657)) ([e4d2df6](https://github.com/AztecProtocol/aztec-packages/commit/e4d2df6b0b5592fe847e3c47020455ac8003d84d))
* Sequencer validates setup/teardown function selectors ([#5649](https://github.com/AztecProtocol/aztec-packages/issues/5649)) ([8f8ad56](https://github.com/AztecProtocol/aztec-packages/commit/8f8ad56471617d611317b96f04cf13f2edc2b493)), closes [#5401](https://github.com/AztecProtocol/aztec-packages/issues/5401)
* Shared mutable storage ([#5490](https://github.com/AztecProtocol/aztec-packages/issues/5490)) ([c4e41a9](https://github.com/AztecProtocol/aztec-packages/commit/c4e41a9809e0a6de87a95c78ba3b71aff81da64a))
* **simulator:** Fetch return values at circuit execution ([#5642](https://github.com/AztecProtocol/aztec-packages/issues/5642)) ([413a4e0](https://github.com/AztecProtocol/aztec-packages/commit/413a4e0e4e22c0dcf86a2d3de6b75c98ae1d67d4))
* Split `backend_barretenburg` into prover and verifier classes (https://github.com/noir-lang/noir/pull/4769) ([825c455](https://github.com/AztecProtocol/aztec-packages/commit/825c455a62faeae5d148ce4f914efacb8f4c50fd))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4764) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Sync from aztec-packages (https://github.com/noir-lang/noir/pull/4787) ([825c455](https://github.com/AztecProtocol/aztec-packages/commit/825c455a62faeae5d148ce4f914efacb8f4c50fd))
* Trap with revert data ([#5732](https://github.com/AztecProtocol/aztec-packages/issues/5732)) ([f849575](https://github.com/AztecProtocol/aztec-packages/commit/f84957584ff76c22c069903f7648735a0be91d7f))
* Unroll loops iteratively (https://github.com/noir-lang/noir/pull/4779) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Update circuits structs with gas info ([#5677](https://github.com/AztecProtocol/aztec-packages/issues/5677)) ([3db6dd1](https://github.com/AztecProtocol/aztec-packages/commit/3db6dd1a213d5f6de8a5e79511ecce9f072b3d41))
* Use fixed size arrays in black box functions where sizes are known ([#5620](https://github.com/AztecProtocol/aztec-packages/issues/5620)) ([f50b180](https://github.com/AztecProtocol/aztec-packages/commit/f50b180379ac90d782aba3472708f8cef122c25b))
* Variable length returns ([#5633](https://github.com/AztecProtocol/aztec-packages/issues/5633)) ([b4a6f17](https://github.com/AztecProtocol/aztec-packages/commit/b4a6f174e2b88f5e4fed128752fc0c30d919084d))
* Wire AVM gas used to public kernel ([#5740](https://github.com/AztecProtocol/aztec-packages/issues/5740)) ([4f55d10](https://github.com/AztecProtocol/aztec-packages/commit/4f55d1023b07df6bf4fc83b6ecb0024426585d0a))


### Bug Fixes

* "feat: Changing finite field arithmetic in wasm to 29 bits for multiplications" ([#5779](https://github.com/AztecProtocol/aztec-packages/issues/5779)) ([bcfee97](https://github.com/AztecProtocol/aztec-packages/commit/bcfee97da99c654f8641ab8099bbbe5d58e2a5c7))
* Anvil start retry in case something bad. Fix colors. ([#5673](https://github.com/AztecProtocol/aztec-packages/issues/5673)) ([0b6b6f6](https://github.com/AztecProtocol/aztec-packages/commit/0b6b6f64ab5984cc8dbaeb1b80136ce1c8a1e3b7))
* ArrayGet and Set are not pure (https://github.com/noir-lang/noir/pull/4783) ([825c455](https://github.com/AztecProtocol/aztec-packages/commit/825c455a62faeae5d148ce4f914efacb8f4c50fd))
* Avoid get row in databus ([#5742](https://github.com/AztecProtocol/aztec-packages/issues/5742)) ([d67b6c8](https://github.com/AztecProtocol/aztec-packages/commit/d67b6c8bb703d856c0d95d4d47cc6de93467e9ab))
* Avoid huge unrolling in hash_args ([#5703](https://github.com/AztecProtocol/aztec-packages/issues/5703)) ([10d9ad9](https://github.com/AztecProtocol/aztec-packages/commit/10d9ad99200a5897417ff5669763ead4e38d87fa))
* **ci,noir-projects:** Bring apt-get higher in cache ([#5775](https://github.com/AztecProtocol/aztec-packages/issues/5775)) ([d37cbb9](https://github.com/AztecProtocol/aztec-packages/commit/d37cbb95cfbb773242bda568f8a7a0b066edc437))
* **ci:** 192 core spot runner ([#5767](https://github.com/AztecProtocol/aztec-packages/issues/5767)) ([37daac6](https://github.com/AztecProtocol/aztec-packages/commit/37daac6a4547d70d06714e1cd727eddbe297cf64))
* **ci:** Bigger cache disk, cache+prune docker images, disable ClientIvcTests.Full ([#5729](https://github.com/AztecProtocol/aztec-packages/issues/5729)) ([5dcbd75](https://github.com/AztecProtocol/aztec-packages/commit/5dcbd75c0795640d48592efbd750cd22b5e5ddd5))
* **ci:** Builder types ([#5711](https://github.com/AztecProtocol/aztec-packages/issues/5711)) ([b16f169](https://github.com/AztecProtocol/aztec-packages/commit/b16f16967ac3d99fc6a23c92dec7e7dd8535adf7))
* **ci:** Cache size not honoured ([#5738](https://github.com/AztecProtocol/aztec-packages/issues/5738)) ([d4ff340](https://github.com/AztecProtocol/aztec-packages/commit/d4ff340745456df31f1554588c3c41d9fa2f1fa6))
* **ci:** Don't fail if can't prune ([d9bb2c7](https://github.com/AztecProtocol/aztec-packages/commit/d9bb2c7fc561a7b1c7dfb516a5afc107cc8bdbbd))
* **ci:** Error in spot ([#5745](https://github.com/AztecProtocol/aztec-packages/issues/5745)) ([4d754aa](https://github.com/AztecProtocol/aztec-packages/commit/4d754aa9a76a9673ea50819e4d9a7e2e04944829))
* **ci:** Fix arm e2e references, spot shutdown ([#5741](https://github.com/AztecProtocol/aztec-packages/issues/5741)) ([1c4667c](https://github.com/AztecProtocol/aztec-packages/commit/1c4667cc58a2eda323ae51f85cfc4223d3eb143d))
* **ci:** Hotfix arm ([1ddb1c7](https://github.com/AztecProtocol/aztec-packages/commit/1ddb1c7136225d874d9d135d354ff601695cc590))
* **ci:** Hotfix just one ARM task ([10f27ae](https://github.com/AztecProtocol/aztec-packages/commit/10f27ae0aa856d1eebede71169a93a23313fe617))
* **ci:** Speculative deploy fix ([9a9eab6](https://github.com/AztecProtocol/aztec-packages/commit/9a9eab6b6b4e087ef1c15172c974353f90815b8b))
* **ci:** Wait for mainnet fork deployment ([#5735](https://github.com/AztecProtocol/aztec-packages/issues/5735)) ([8f3794d](https://github.com/AztecProtocol/aztec-packages/commit/8f3794dad170dba616c505d85bc1731fb6177ce0))
* **ci:** Wait_for_fork env var ([#5780](https://github.com/AztecProtocol/aztec-packages/issues/5780)) ([d85267b](https://github.com/AztecProtocol/aztec-packages/commit/d85267b7f1c34bf93087f3a48690ddfd8a9cf05d))
* Correct ICE panic messages in brillig `convert_black_box_call` (https://github.com/noir-lang/noir/pull/4761) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Disable flakey vanilla recursion test ([#5672](https://github.com/AztecProtocol/aztec-packages/issues/5672)) ([f84f7b6](https://github.com/AztecProtocol/aztec-packages/commit/f84f7b68f6c8072480127a065def1c4453e55877))
* Don't run e2e tests against wrong anvil ([#5686](https://github.com/AztecProtocol/aztec-packages/issues/5686)) ([9ff45f6](https://github.com/AztecProtocol/aztec-packages/commit/9ff45f69af562db4cec69c6231659476667f1cf9))
* Dont error in bench summary ([#5693](https://github.com/AztecProtocol/aztec-packages/issues/5693)) ([470b0f3](https://github.com/AztecProtocol/aztec-packages/commit/470b0f36138f34fcff1da869f7fd54f2d50c1480))
* E2e getStack, disable failing e2e ([#5768](https://github.com/AztecProtocol/aztec-packages/issues/5768)) ([e5f3ece](https://github.com/AztecProtocol/aztec-packages/commit/e5f3ece131b16aeede897f0a9bb3ecc23cb4d9dc))
* GA concurrency ([#5713](https://github.com/AztecProtocol/aztec-packages/issues/5713)) ([eac2585](https://github.com/AztecProtocol/aztec-packages/commit/eac25853cc8ca14254010076106b6a9555ef5d17))
* Generate_aztecnr_reference.js not getting generics or multi-line params ([#5679](https://github.com/AztecProtocol/aztec-packages/issues/5679)) ([a22bc3d](https://github.com/AztecProtocol/aztec-packages/commit/a22bc3d4004b050449569bf66fe7431ccd04a16c))
* Hotfix submodule cache ([92b92b3](https://github.com/AztecProtocol/aztec-packages/commit/92b92b32c46f7c4d332a372c0e1039040c762eff))
* Hotfix underspec'd machine ([#5710](https://github.com/AztecProtocol/aztec-packages/issues/5710)) ([059e38e](https://github.com/AztecProtocol/aztec-packages/commit/059e38e3d319b84259bb1bf36fca6ed71425eb98))
* **hotfix:** CI ignore git safe.directory checks ([#5659](https://github.com/AztecProtocol/aztec-packages/issues/5659)) ([9fc3fe3](https://github.com/AztecProtocol/aztec-packages/commit/9fc3fe3e5aa74e593eabb2fd1f358476f039e595))
* Less earthly cache ([#5690](https://github.com/AztecProtocol/aztec-packages/issues/5690)) ([8190dc7](https://github.com/AztecProtocol/aztec-packages/commit/8190dc7826d480f44107456984f7f192358ba8da))
* Make earthly more parallel ([#5747](https://github.com/AztecProtocol/aztec-packages/issues/5747)) ([9734455](https://github.com/AztecProtocol/aztec-packages/commit/9734455acd0d6e0cba44477f889ec8165e7f3003))
* Primary_message typo in errors.rs ([#5646](https://github.com/AztecProtocol/aztec-packages/issues/5646)) ([1dfbe7b](https://github.com/AztecProtocol/aztec-packages/commit/1dfbe7bc3bf3c455d8fb6c8b5fe6a96c1edf7af9))
* Pull noir ([#5699](https://github.com/AztecProtocol/aztec-packages/issues/5699)) ([bf35464](https://github.com/AztecProtocol/aztec-packages/commit/bf3546444272f36a3e3905246c49f2e5f65cff6e))
* REDO dont error in bench summary ([#5695](https://github.com/AztecProtocol/aztec-packages/issues/5695)) ([8c1a7b9](https://github.com/AztecProtocol/aztec-packages/commit/8c1a7b976bcc63af63ddcc8cf4f89b338f17bdf8))
* Running e2e tests as part of build, requires forcing ip4 (not ip6) when connecting to anvil ([#5744](https://github.com/AztecProtocol/aztec-packages/issues/5744)) ([66fc89f](https://github.com/AztecProtocol/aztec-packages/commit/66fc89f417d0582865acdb04a75258a4a449c8b6))
* Simplify ECCVM prover constructor and add a TODO ([#5681](https://github.com/AztecProtocol/aztec-packages/issues/5681)) ([8c151ea](https://github.com/AztecProtocol/aztec-packages/commit/8c151eab1492dda901a1d6b691c9ca68960fd9e6))
* Spot refcount ([#5746](https://github.com/AztecProtocol/aztec-packages/issues/5746)) ([9e18444](https://github.com/AztecProtocol/aztec-packages/commit/9e184448f218ab06247d8cbbadf755ad384bd40a))
* Take a deep copy of circuit inputs for proving ([#5777](https://github.com/AztecProtocol/aztec-packages/issues/5777)) ([785591e](https://github.com/AztecProtocol/aztec-packages/commit/785591e0527beef7adf0f7a791618afff9df3705))
* Temporarily disable the bench tests ([#5755](https://github.com/AztecProtocol/aztec-packages/issues/5755)) ([1d52ac5](https://github.com/AztecProtocol/aztec-packages/commit/1d52ac5c15524648094c23cf67ea0cfb921fe186))
* Update commit for noir-gates-diff (https://github.com/noir-lang/noir/pull/4773) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Use entrypoint instead of pay_init_fee ([#5623](https://github.com/AztecProtocol/aztec-packages/issues/5623)) ([62ac765](https://github.com/AztecProtocol/aztec-packages/commit/62ac765d192d499b7c8a8b1b4583c17fb95d00ac))
* Watch less files. ([#5651](https://github.com/AztecProtocol/aztec-packages/issues/5651)) ([57a1d69](https://github.com/AztecProtocol/aztec-packages/commit/57a1d69f3fcade6f4536d007baad28e878242d59))


### Miscellaneous

* Add missing aztec-address tests ([#5674](https://github.com/AztecProtocol/aztec-packages/issues/5674)) ([58aefba](https://github.com/AztecProtocol/aztec-packages/commit/58aefbad0144b41619fa36a3714fa2a440945c74))
* **avm:** Add a boolean to toggle proving in AVM unit tests ([#5667](https://github.com/AztecProtocol/aztec-packages/issues/5667)) ([ec122c9](https://github.com/AztecProtocol/aztec-packages/commit/ec122c9b9c1c63c72158c6956d8fdb2398faf96b)), closes [#5663](https://github.com/AztecProtocol/aztec-packages/issues/5663)
* **avm:** Hashing tests cleanup ([#5733](https://github.com/AztecProtocol/aztec-packages/issues/5733)) ([53d0102](https://github.com/AztecProtocol/aztec-packages/commit/53d010232600b736ac0f5116a3b7804f2d412dd0))
* **avm:** Range checks negative tests ([#5770](https://github.com/AztecProtocol/aztec-packages/issues/5770)) ([2907142](https://github.com/AztecProtocol/aztec-packages/commit/29071423ba65774039e4e5c1f7ca67123a18f738))
* **avm:** Split the negative test on range check for high 16-bit registers ([#5785](https://github.com/AztecProtocol/aztec-packages/issues/5785)) ([8ebbe57](https://github.com/AztecProtocol/aztec-packages/commit/8ebbe57953e35f81ca010cdd686fb43ddf0f20b2))
* **avm:** Split up AVM test contract as it was growing too large ([#5702](https://github.com/AztecProtocol/aztec-packages/issues/5702)) ([5b8e812](https://github.com/AztecProtocol/aztec-packages/commit/5b8e812802457b429d173a5c4793ffa04faec1fa))
* **aztec-nr:** Minor public interface changes ([#5776](https://github.com/AztecProtocol/aztec-packages/issues/5776)) ([91b8110](https://github.com/AztecProtocol/aztec-packages/commit/91b8110ab44979fe37fcc57824bdee845295ccb7))
* **ci:** Break e2e-deploy into multiple test suites ([#5704](https://github.com/AztecProtocol/aztec-packages/issues/5704)) ([2522294](https://github.com/AztecProtocol/aztec-packages/commit/2522294d96e064b4531e05ae1b21eb4fa8f90125))
* **ci:** Earthly in spot with persistent cache ([#5644](https://github.com/AztecProtocol/aztec-packages/issues/5644)) ([a39c2f6](https://github.com/AztecProtocol/aztec-packages/commit/a39c2f64565665260dbf8640478691dd5d47cee5))
* **ci:** Hotfix AMI's, workflow to stop personal spot runners ([#5712](https://github.com/AztecProtocol/aztec-packages/issues/5712)) ([5f18139](https://github.com/AztecProtocol/aztec-packages/commit/5f1813947e05cc56553c8606a254f26d39ca6093))
* **ci:** Only run ARM on master ([#5705](https://github.com/AztecProtocol/aztec-packages/issues/5705)) ([f77c142](https://github.com/AztecProtocol/aztec-packages/commit/f77c142b1634442434aa4631317186523d456a69))
* **ci:** Use 128 cores for x86 and add timeouts ([#5665](https://github.com/AztecProtocol/aztec-packages/issues/5665)) ([0c5dc0a](https://github.com/AztecProtocol/aztec-packages/commit/0c5dc0a8d90c52c46f9802ec5fb93561d0551b6a))
* Compute_note_hash_and_nullifier - improve error message ([#5671](https://github.com/AztecProtocol/aztec-packages/issues/5671)) ([8942d69](https://github.com/AztecProtocol/aztec-packages/commit/8942d69ab59f9ca7b33b72c91fb960c02afd66b0))
* Create placeholder version of 0.26.0 docs (https://github.com/noir-lang/noir/pull/4782) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* **doc:** Fix broken docs links (https://github.com/noir-lang/noir/pull/4606) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* **docs:** Fix link in the Data Types page (https://github.com/noir-lang/noir/pull/4527) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Don't strip bb wasm ([#5743](https://github.com/AztecProtocol/aztec-packages/issues/5743)) ([d4cb410](https://github.com/AztecProtocol/aztec-packages/commit/d4cb4108900f1fb6307de17be9ee3516d6023609))
* Fix master after merge issue related to validate_trace renaming ([#5676](https://github.com/AztecProtocol/aztec-packages/issues/5676)) ([44e0d8a](https://github.com/AztecProtocol/aztec-packages/commit/44e0d8abd2104a9969d5750736e77fe9a8d4d621))
* Fix max-block-number and auth e2e tests ([#5694](https://github.com/AztecProtocol/aztec-packages/issues/5694)) ([f1bf314](https://github.com/AztecProtocol/aztec-packages/commit/f1bf31431df68655fa5364c0504c2323ad660c22))
* Op queue ([#5648](https://github.com/AztecProtocol/aztec-packages/issues/5648)) ([822c7e6](https://github.com/AztecProtocol/aztec-packages/commit/822c7e63e91cb30219a79513c05d84ee4f03d8fe))
* **public:** Remove getNullifierMembershipWitness ([#5715](https://github.com/AztecProtocol/aztec-packages/issues/5715)) ([3be402c](https://github.com/AztecProtocol/aztec-packages/commit/3be402cca4fc1451a2b4c7560c346843eb8439cd))
* Re-enable e2e fees tests ([#5784](https://github.com/AztecProtocol/aztec-packages/issues/5784)) ([102e8b8](https://github.com/AztecProtocol/aztec-packages/commit/102e8b89e91c324f945ab94357508c25eba4fa92))
* Release Noir(0.27.0) (https://github.com/noir-lang/noir/pull/4632) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Remove the old Value struct from the oracle docs (https://github.com/noir-lang/noir/pull/4738) ([1eb288e](https://github.com/AztecProtocol/aztec-packages/commit/1eb288e9adc8b803e75ed80317c3e0979d0dfdee))
* Replace relative paths to noir-protocol-circuits ([fb2b298](https://github.com/AztecProtocol/aztec-packages/commit/fb2b298066bbf0fa15e5a39fa7e48cb288e50182))
* Replace relative paths to noir-protocol-circuits ([e20920d](https://github.com/AztecProtocol/aztec-packages/commit/e20920d3b2f2acf4682eecc94a04e46fc50a8767))
* Replace relative paths to noir-protocol-circuits ([6351dc5](https://github.com/AztecProtocol/aztec-packages/commit/6351dc52fe63200293aaf1749330a8f572356163))
* Replace relative paths to noir-protocol-circuits ([fee13bf](https://github.com/AztecProtocol/aztec-packages/commit/fee13bf5869a03a84a05bb3166d0f5493c7056fd))
* Replacing unsafe::zeroed() ([#5685](https://github.com/AztecProtocol/aztec-packages/issues/5685)) ([ea3884e](https://github.com/AztecProtocol/aztec-packages/commit/ea3884ec49e8a37d66b212551e1b78ab47a93e37))
* Small logging changes ([#5654](https://github.com/AztecProtocol/aztec-packages/issues/5654)) ([25cc70d](https://github.com/AztecProtocol/aztec-packages/commit/25cc70de04342f89665883012b86f1c54c916c44))
* Temporarily skip failing e2e fees test ([a3ac5ff](https://github.com/AztecProtocol/aztec-packages/commit/a3ac5ff251270f7a14d56e9b7846655dce716a44))
* Testing that nargo fmt is idempotent (https://github.com/noir-lang/noir/pull/4765) ([825c455](https://github.com/AztecProtocol/aztec-packages/commit/825c455a62faeae5d148ce4f914efacb8f4c50fd))
* TS hash wrappers cleanup ([#5691](https://github.com/AztecProtocol/aztec-packages/issues/5691)) ([7f8b09f](https://github.com/AztecProtocol/aztec-packages/commit/7f8b09fca6370b140870041a49692383a4db6551))
* Turn ENABLE_GAS where it is needed ([#5730](https://github.com/AztecProtocol/aztec-packages/issues/5730)) ([30a2edd](https://github.com/AztecProtocol/aztec-packages/commit/30a2edd91cc46196c13fe421cdea2e03f30052fc))
* Update noir gates diff ([#5658](https://github.com/AztecProtocol/aztec-packages/issues/5658)) ([9816c1a](https://github.com/AztecProtocol/aztec-packages/commit/9816c1adead8d3455c091664744f064bb0433ee7))
* We can run 35 of our e2e tests just using jest. ([#5643](https://github.com/AztecProtocol/aztec-packages/issues/5643)) ([4fcaeae](https://github.com/AztecProtocol/aztec-packages/commit/4fcaeaea01acc42f87e6c6a5b0d229deaee2a063))


### Documentation

* Fix yp typo control-flow.md ([#5638](https://github.com/AztecProtocol/aztec-packages/issues/5638)) ([363d227](https://github.com/AztecProtocol/aztec-packages/commit/363d2275592f190d57501a25014bfe37ccad5e30))

## [0.34.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.33.0...aztec-packages-v0.34.0) (2024-04-10)


### ⚠ BREAKING CHANGES

* remove fixed-length keccak256 ([#5617](https://github.com/AztecProtocol/aztec-packages/issues/5617))

### Features

* **acvm_js:** Execute program  (https://github.com/noir-lang/noir/pull/4694) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Add `remove_enable_side_effects` SSA pass (https://github.com/noir-lang/noir/pull/4224) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Allow slices to brillig entry points (https://github.com/noir-lang/noir/pull/4713) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **avm:** Basic AVM-ACVM interoperability ([#5595](https://github.com/AztecProtocol/aztec-packages/issues/5595)) ([d872445](https://github.com/AztecProtocol/aztec-packages/commit/d87244599363164816fd7a51f8a3c254e15d9caa))
* **avm:** Make authwit work with avm ([#5594](https://github.com/AztecProtocol/aztec-packages/issues/5594)) ([b02d1e1](https://github.com/AztecProtocol/aztec-packages/commit/b02d1e11419093f2b12a85161d7ff8f0d8e89282))
* **docs:** Documenting noir codegen (https://github.com/noir-lang/noir/pull/4454) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Generalize protogalaxy to multiple instances ([#5510](https://github.com/AztecProtocol/aztec-packages/issues/5510)) ([f038b70](https://github.com/AztecProtocol/aztec-packages/commit/f038b704c638604b26fe8752792c878cf897327f))
* Improve nargo check cli with --override flag and feedback for existing files (https://github.com/noir-lang/noir/pull/4575) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Improve optimisations on range constraints (https://github.com/noir-lang/noir/pull/4690) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Improve SSA type-awareness in EQ and MUL instructions (https://github.com/noir-lang/noir/pull/4691) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **nargo:** Multiple circuits info for binary programs (https://github.com/noir-lang/noir/pull/4719) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Stdlib databus ([#5598](https://github.com/AztecProtocol/aztec-packages/issues/5598)) ([633a711](https://github.com/AztecProtocol/aztec-packages/commit/633a711f1b4aecd20131075fa1066c3f24b8d78c))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable (https://github.com/noir-lang/noir/pull/4708) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Do not retry RPC requests on 4xx errors ([#5634](https://github.com/AztecProtocol/aztec-packages/issues/5634)) ([5af2b95](https://github.com/AztecProtocol/aztec-packages/commit/5af2b95a477966d4068741a60a25f8da433abecf))
* Field comparisons (https://github.com/noir-lang/noir/pull/4704) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Last use analysis & make it an SSA pass (https://github.com/noir-lang/noir/pull/4686) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **ssa:** Do not use get_value_max_num_bits when we want pure type information (https://github.com/noir-lang/noir/pull/4700) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Unknown slice lengths coming from as_slice (https://github.com/noir-lang/noir/pull/4725) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Use empty artifact in test ([#5640](https://github.com/AztecProtocol/aztec-packages/issues/5640)) ([1d18a5e](https://github.com/AztecProtocol/aztec-packages/commit/1d18a5ecf5779887e20249f2ce8cab6476de73ad))


### Miscellaneous

* Check for references to private functions during path resolution (https://github.com/noir-lang/noir/pull/4622) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **ci:** Fix cutting new versions of the docs (https://github.com/noir-lang/noir/pull/4737) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **ci:** Replace `yarn build:js:only` script (https://github.com/noir-lang/noir/pull/4735) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* **ci:** Stop updating version list before cutting new docs version (https://github.com/noir-lang/noir/pull/4726) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Disable earthly cloud ([#5639](https://github.com/AztecProtocol/aztec-packages/issues/5639)) ([47e9c25](https://github.com/AztecProtocol/aztec-packages/commit/47e9c25ac8b8b4d34915ce7dad50218d397cc64a))
* Fix clippy errors (https://github.com/noir-lang/noir/pull/4684) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Reduce log verbosity in local e2e tests ([#5622](https://github.com/AztecProtocol/aztec-packages/issues/5622)) ([c496a10](https://github.com/AztecProtocol/aztec-packages/commit/c496a105eac3b78e53b7d42d4a64e88e3a4759a5))
* Remove `FunctionInput::dummy` (https://github.com/noir-lang/noir/pull/4723) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Remove conditional compilation around `acvm_js` package (https://github.com/noir-lang/noir/pull/4702) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Remove docker CI flow (https://github.com/noir-lang/noir/pull/4724) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Remove fixed-length keccak256 ([#5617](https://github.com/AztecProtocol/aztec-packages/issues/5617)) ([40480b3](https://github.com/AztecProtocol/aztec-packages/commit/40480b3b73cc2c7c296a7361a42ad3511e3b2a16))
* Remove last traces of nix (https://github.com/noir-lang/noir/pull/4679) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Remove unused env vars from `Cross.toml` (https://github.com/noir-lang/noir/pull/4717) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Replace relative paths to noir-protocol-circuits ([bc214c5](https://github.com/AztecProtocol/aztec-packages/commit/bc214c5bb36fc3c61b0ea6f63f66839652c369c9))
* Simplify how `acvm_backend.wasm` is embedded (https://github.com/noir-lang/noir/pull/4703) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Simplify how blns is loaded into tests (https://github.com/noir-lang/noir/pull/4705) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Update condition for clearing warning comment on release PRs (https://github.com/noir-lang/noir/pull/4739) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Update from vulnerable version of h2 (https://github.com/noir-lang/noir/pull/4714) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Update JS publish workflow to upload build artifacts correctly. (https://github.com/noir-lang/noir/pull/4734) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))
* Use is_entry_point helper on RuntimeType (https://github.com/noir-lang/noir/pull/4678) ([ff28080](https://github.com/AztecProtocol/aztec-packages/commit/ff28080bcfb946177010960722925973ee19646b))

## [0.33.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.32.1...aztec-packages-v0.33.0) (2024-04-09)


### ⚠ BREAKING CHANGES

* **acir:** Add predicate to call opcode ([#5616](https://github.com/AztecProtocol/aztec-packages/issues/5616))
* contract_abi-exports ([#5386](https://github.com/AztecProtocol/aztec-packages/issues/5386))
* **avm:** rollback FunctionAbi isTranspiled changes ([#5561](https://github.com/AztecProtocol/aztec-packages/issues/5561))

### Features

* /foundry is canoncial build of foundry. e2e tests can start own anvil. ([#5522](https://github.com/AztecProtocol/aztec-packages/issues/5522)) ([510daa0](https://github.com/AztecProtocol/aztec-packages/commit/510daa06610b30483f9e80befd0b31908e7d63c6))
* `add` and `sub` methods of `EasyPrivateUint` throw when called in public ([#5581](https://github.com/AztecProtocol/aztec-packages/issues/5581)) ([29f337d](https://github.com/AztecProtocol/aztec-packages/commit/29f337d809ff376900c4cdb9b120d2e68bf4ce68))
* **acir:** Add predicate to call opcode ([#5616](https://github.com/AztecProtocol/aztec-packages/issues/5616)) ([e8cec0a](https://github.com/AztecProtocol/aztec-packages/commit/e8cec0a81da29a1b4df8bc6c70b04e37902f7609))
* **acvm_js:** Execute program  (https://github.com/noir-lang/noir/pull/4694) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Add return values to aztec fns ([#5389](https://github.com/AztecProtocol/aztec-packages/issues/5389)) ([7b88bac](https://github.com/AztecProtocol/aztec-packages/commit/7b88bacb734ba263ced43e79d9d579a791418e6f))
* Allow slices to brillig entry points (https://github.com/noir-lang/noir/pull/4713) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Avm logup ([#5577](https://github.com/AztecProtocol/aztec-packages/issues/5577)) ([7e4e9b9](https://github.com/AztecProtocol/aztec-packages/commit/7e4e9b9830c3beb6396ccd3110dda09d3f4e2d59))
* **avm:** Add index to pedersen opcode ([#5486](https://github.com/AztecProtocol/aztec-packages/issues/5486)) ([e1d7d11](https://github.com/AztecProtocol/aztec-packages/commit/e1d7d1157ec4566efdb5569262f42796265b3e79))
* **avm:** Contract instance opcode ([#5487](https://github.com/AztecProtocol/aztec-packages/issues/5487)) ([ceacba6](https://github.com/AztecProtocol/aztec-packages/commit/ceacba6cbb6d070ec3e5e42673b9ab11dab79566))
* **avm:** Gas usage for nested calls ([#5495](https://github.com/AztecProtocol/aztec-packages/issues/5495)) ([11699c8](https://github.com/AztecProtocol/aztec-packages/commit/11699c82eed49a464a7f766111beb1b4d4edfcd6))
* **avm:** Indirect memory for set opcode ([#5546](https://github.com/AztecProtocol/aztec-packages/issues/5546)) ([e0e7200](https://github.com/AztecProtocol/aztec-packages/commit/e0e7200819d30170d3d84d42540015d24d7cb1e8)), closes [#5542](https://github.com/AztecProtocol/aztec-packages/issues/5542)
* **avm:** Integrate AVM with initializers ([#5469](https://github.com/AztecProtocol/aztec-packages/issues/5469)) ([59799f2](https://github.com/AztecProtocol/aztec-packages/commit/59799f273addec01eb0cdea365fe72bcbc8d9493))
* **avm:** Set gas allowance in public calls ([#5567](https://github.com/AztecProtocol/aztec-packages/issues/5567)) ([ee23415](https://github.com/AztecProtocol/aztec-packages/commit/ee234153655642ad062d84b12f420866fc3208e7))
* **avm:** Track gas from memory accesses explicitly ([#5563](https://github.com/AztecProtocol/aztec-packages/issues/5563)) ([18c9128](https://github.com/AztecProtocol/aztec-packages/commit/18c9128f57c1a0e67682d658b45d601ec6c856e1)), closes [#5514](https://github.com/AztecProtocol/aztec-packages/issues/5514)
* Contract_abi-exports ([#5386](https://github.com/AztecProtocol/aztec-packages/issues/5386)) ([745d522](https://github.com/AztecProtocol/aztec-packages/commit/745d5229db86b2188f52ab7ccc8f568aef8f5797))
* DataBus notion with calldata/return data ([#5504](https://github.com/AztecProtocol/aztec-packages/issues/5504)) ([95a1d8a](https://github.com/AztecProtocol/aztec-packages/commit/95a1d8ac45e0db8ec21ba1cb2e3f47bb68909b71))
* DebugLog(...) in noir-protocol-circuits ([#5568](https://github.com/AztecProtocol/aztec-packages/issues/5568)) ([a07bb92](https://github.com/AztecProtocol/aztec-packages/commit/a07bb92284b5bb2c80f19f6079465fcec870232b))
* **docs:** Documenting noir codegen (https://github.com/noir-lang/noir/pull/4454) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Improve nargo check cli with --override flag and feedback for existing files (https://github.com/noir-lang/noir/pull/4575) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Improve optimisations on range constraints (https://github.com/noir-lang/noir/pull/4690) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Improve SSA type-awareness in EQ and MUL instructions (https://github.com/noir-lang/noir/pull/4691) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Improve the proving orchestration lifecycle ([#5535](https://github.com/AztecProtocol/aztec-packages/issues/5535)) ([4e4f843](https://github.com/AztecProtocol/aztec-packages/commit/4e4f8434b49adb2cb5b44d33dcc15c2b2b43040b))
* Jest fast transpile. no more ts-jest. ([#5530](https://github.com/AztecProtocol/aztec-packages/issues/5530)) ([1912802](https://github.com/AztecProtocol/aztec-packages/commit/19128024292a91d0f947f397ab1b0dc2cd7ef7aa))
* Logging deployed contract address to help debug e2e account test ([#5571](https://github.com/AztecProtocol/aztec-packages/issues/5571)) ([1907473](https://github.com/AztecProtocol/aztec-packages/commit/190747348b940914bee161808b21091aad99824e))
* Only export values from accumulated data ([#5604](https://github.com/AztecProtocol/aztec-packages/issues/5604)) ([a974ec8](https://github.com/AztecProtocol/aztec-packages/commit/a974ec82d01595bd4245fa8ac44e25f4b57cf397))
* Optimise relations ([#5552](https://github.com/AztecProtocol/aztec-packages/issues/5552)) ([a581e80](https://github.com/AztecProtocol/aztec-packages/commit/a581e80dedfd0398d4f8a831b4e0031e8460dff7))
* Optimize auxiliary relations slightly ([#5517](https://github.com/AztecProtocol/aztec-packages/issues/5517)) ([30be431](https://github.com/AztecProtocol/aztec-packages/commit/30be43186980672a271fc568344f0341055c7b57))
* Public inputs refactor ([#5500](https://github.com/AztecProtocol/aztec-packages/issues/5500)) ([6b9a538](https://github.com/AztecProtocol/aztec-packages/commit/6b9a538c4b1d52893deae2ec2b5fab2ffbb64528))
* Restore hashing args via slice for performance ([#5539](https://github.com/AztecProtocol/aztec-packages/issues/5539)) ([eb3acdf](https://github.com/AztecProtocol/aztec-packages/commit/eb3acdf16d58968ac67ffad4c2d5efb10fa31d26))
* **SimulateTx:** Simulate constrained transaction execution with return values ([#5432](https://github.com/AztecProtocol/aztec-packages/issues/5432)) ([0249737](https://github.com/AztecProtocol/aztec-packages/commit/0249737e8b925406e9278b80fc7adc0f6ab5468d))


### Bug Fixes

* **acvm:** Mark outputs of Opcode::Call solvable (https://github.com/noir-lang/noir/pull/4708) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* **avm:** Nullifier handling ([#5488](https://github.com/AztecProtocol/aztec-packages/issues/5488)) ([bc8211d](https://github.com/AztecProtocol/aztec-packages/commit/bc8211dfd9979b8152a6bcc486ec2c157e070f8b))
* **ci:** Cache submodules in GA ([#5531](https://github.com/AztecProtocol/aztec-packages/issues/5531)) ([75f2cc6](https://github.com/AztecProtocol/aztec-packages/commit/75f2cc636a9cc8a2aae9b686e35d61c27130e277))
* **ci:** Install fixed foundry version in CI ([#5582](https://github.com/AztecProtocol/aztec-packages/issues/5582)) ([46fdb37](https://github.com/AztecProtocol/aztec-packages/commit/46fdb371d2b59fa94d3d49df680516c605d55170))
* Dependabot update ([#5547](https://github.com/AztecProtocol/aztec-packages/issues/5547)) ([f7e6cc8](https://github.com/AztecProtocol/aztec-packages/commit/f7e6cc86a4bc9c6fd572910314471a27f40a296e))
* E2e earthly status ([#5564](https://github.com/AztecProtocol/aztec-packages/issues/5564)) ([a5076ca](https://github.com/AztecProtocol/aztec-packages/commit/a5076cac80d0e9b914ec7f198141574bb2095322))
* Field comparisons (https://github.com/noir-lang/noir/pull/4704) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Invalid fork terraform ([#5585](https://github.com/AztecProtocol/aztec-packages/issues/5585)) ([826353b](https://github.com/AztecProtocol/aztec-packages/commit/826353b4f7456a2280df0b5515f9ae2a74002e76))
* Last use analysis & make it an SSA pass (https://github.com/noir-lang/noir/pull/4686) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Mainnet fork redeploys ([#5573](https://github.com/AztecProtocol/aztec-packages/issues/5573)) ([88e8b6d](https://github.com/AztecProtocol/aztec-packages/commit/88e8b6de05a70b2dc54202434ba0e3194de5f026))
* Remove EFS lifecycle rule ([#5587](https://github.com/AztecProtocol/aztec-packages/issues/5587)) ([eb66fc6](https://github.com/AztecProtocol/aztec-packages/commit/eb66fc6a6f2012a5a1c2d443051f3f60161ab814))
* **ssa:** Do not use get_value_max_num_bits when we want pure type information (https://github.com/noir-lang/noir/pull/4700) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Taint fork file storage ([#5560](https://github.com/AztecProtocol/aztec-packages/issues/5560)) ([f144f3b](https://github.com/AztecProtocol/aztec-packages/commit/f144f3bf3fcaa74e8ee3a2425ef7fdf473e252d5))
* Unknown slice lengths coming from as_slice (https://github.com/noir-lang/noir/pull/4725) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Update CLI & terraforms with new contract addresses ([#5553](https://github.com/AztecProtocol/aztec-packages/issues/5553)) ([eb73d20](https://github.com/AztecProtocol/aztec-packages/commit/eb73d20e825f3e18acfb68a2b0b7d3501f39e52d))


### Miscellaneous

* **avm:** Rollback FunctionAbi isTranspiled changes ([#5561](https://github.com/AztecProtocol/aztec-packages/issues/5561)) ([150932a](https://github.com/AztecProtocol/aztec-packages/commit/150932a323fd3170484ea951e0b7c7ea37524e7a))
* Check for references to private functions during path resolution (https://github.com/noir-lang/noir/pull/4622) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* **docs:** Add file to prove ownership for google search console ([#5554](https://github.com/AztecProtocol/aztec-packages/issues/5554)) ([dfa3998](https://github.com/AztecProtocol/aztec-packages/commit/dfa399835bab8582c61a119aa7e7415d8527f457))
* **docs:** Fix indexed-merkle-tree docs images ([#4674](https://github.com/AztecProtocol/aztec-packages/issues/4674)) ([8fc29d5](https://github.com/AztecProtocol/aztec-packages/commit/8fc29d5b245c7257dbfcda931cd3505db93542d0))
* **docs:** Random updates ([#5281](https://github.com/AztecProtocol/aztec-packages/issues/5281)) ([b8c9273](https://github.com/AztecProtocol/aztec-packages/commit/b8c927379327bf94cba022a598b2c96704e3dd08))
* ECCVM flavor depends on builder ([#5323](https://github.com/AztecProtocol/aztec-packages/issues/5323)) ([a594683](https://github.com/AztecProtocol/aztec-packages/commit/a5946836eb52f8d836a05de31725d1e0f741a6db))
* Fix clippy errors (https://github.com/noir-lang/noir/pull/4684) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Get rid of ECCVM composer ([#5562](https://github.com/AztecProtocol/aztec-packages/issues/5562)) ([43ed901](https://github.com/AztecProtocol/aztec-packages/commit/43ed901838dc2f8d59c665bd667ceec81d31bbdc))
* Move e2e-avm-initializer test to e2e-avm-simulator ([#5570](https://github.com/AztecProtocol/aztec-packages/issues/5570)) ([d827705](https://github.com/AztecProtocol/aztec-packages/commit/d827705f28b61ab83a4a7b21eb87c01590671e68))
* Nuking accounts from e2e setup ([#5574](https://github.com/AztecProtocol/aztec-packages/issues/5574)) ([be6f843](https://github.com/AztecProtocol/aztec-packages/commit/be6f8432c9beaf8c6ee30e65fd5354576870eeff)), closes [#5307](https://github.com/AztecProtocol/aztec-packages/issues/5307)
* Nuking L2BlockContext ([#5569](https://github.com/AztecProtocol/aztec-packages/issues/5569)) ([1299190](https://github.com/AztecProtocol/aztec-packages/commit/12991908cb17329a35b47610c4b5c27f34b92771))
* Pad when needed and not sooner ([#5482](https://github.com/AztecProtocol/aztec-packages/issues/5482)) ([e928c33](https://github.com/AztecProtocol/aztec-packages/commit/e928c3332b4cb93dcea3858668c7bbf5c17db4fb)), closes [#5357](https://github.com/AztecProtocol/aztec-packages/issues/5357)
* Remove conditional compilation around `acvm_js` package (https://github.com/noir-lang/noir/pull/4702) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Remove debug log from watch.sh ([a9a349d](https://github.com/AztecProtocol/aztec-packages/commit/a9a349db3902970a2b484b5e1509931423ac5775))
* Remove last traces of nix (https://github.com/noir-lang/noir/pull/4679) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Remove unused boolean return values from stores ([#5470](https://github.com/AztecProtocol/aztec-packages/issues/5470)) ([07794ee](https://github.com/AztecProtocol/aztec-packages/commit/07794ee11ddd98dd824fba9c858267a502f4671e))
* Remove unused env vars from `Cross.toml` (https://github.com/noir-lang/noir/pull/4717) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Replace relative paths to noir-protocol-circuits ([51a1983](https://github.com/AztecProtocol/aztec-packages/commit/51a198384fd2745fd3088f1269eb3fb556ec20f9))
* Replace relative paths to noir-protocol-circuits ([a716270](https://github.com/AztecProtocol/aztec-packages/commit/a7162709f0449ff6093bbdd93ad1856ed80cdba3))
* Replace relative paths to noir-protocol-circuits ([6827014](https://github.com/AztecProtocol/aztec-packages/commit/6827014e29665c23a3108c3c78f0726b00fc6739))
* Replace relative paths to noir-protocol-circuits ([356caf7](https://github.com/AztecProtocol/aztec-packages/commit/356caf731445e5222573e01bd7d9611b4b318306))
* Simplify how `acvm_backend.wasm` is embedded (https://github.com/noir-lang/noir/pull/4703) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Simplify how blns is loaded into tests (https://github.com/noir-lang/noir/pull/4705) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))
* Update snapshot ([#5626](https://github.com/AztecProtocol/aztec-packages/issues/5626)) ([fb66426](https://github.com/AztecProtocol/aztec-packages/commit/fb664260b2df8d28c3fb03407220d4dd9a0e28fc))
* Use is_entry_point helper on RuntimeType (https://github.com/noir-lang/noir/pull/4678) ([8b30b95](https://github.com/AztecProtocol/aztec-packages/commit/8b30b95fd2c1767ab5969bc47d15e157cd9a3f72))


### Documentation

* Minor fixes in keys ([#5550](https://github.com/AztecProtocol/aztec-packages/issues/5550)) ([ea48ad3](https://github.com/AztecProtocol/aztec-packages/commit/ea48ad3ec59cff40afb526c5e4df8b624b087b06))
* **spec:** Hashing and keys ([#5478](https://github.com/AztecProtocol/aztec-packages/issues/5478)) ([820ac8c](https://github.com/AztecProtocol/aztec-packages/commit/820ac8cb413b114826231cd3c940102730cf985a))
* Update quickstart.md to use Docker daemon ([#5576](https://github.com/AztecProtocol/aztec-packages/issues/5576)) ([42b9827](https://github.com/AztecProtocol/aztec-packages/commit/42b98274176e033faf1ee8c55ce4d248d4d2f08c))

## [0.32.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.32.0...aztec-packages-v0.32.1) (2024-04-02)


### Features

* **acir_gen:** Fold attribute at compile-time and initial non inlined ACIR ([#5341](https://github.com/AztecProtocol/aztec-packages/issues/5341)) ([a979150](https://github.com/AztecProtocol/aztec-packages/commit/a979150271da3da9143aed92626523227b1721a7))
* **acvm:** Execute multiple circuits  ([#5380](https://github.com/AztecProtocol/aztec-packages/issues/5380)) ([bb71920](https://github.com/AztecProtocol/aztec-packages/commit/bb719200034e3bc6db09fb56538dadca4203abf4))
* Dont double check num bits in brillig vm ([#5489](https://github.com/AztecProtocol/aztec-packages/issues/5489)) ([a18288d](https://github.com/AztecProtocol/aztec-packages/commit/a18288d9b8f3057b9e79362d922da656dacf22a9))
* Earthly split runners, structure reverts ([#5524](https://github.com/AztecProtocol/aztec-packages/issues/5524)) ([fcb8787](https://github.com/AztecProtocol/aztec-packages/commit/fcb8787f4623eccbc6189f9399d444a4cb863684))
* Parallel gtest ([#5498](https://github.com/AztecProtocol/aztec-packages/issues/5498)) ([349ea59](https://github.com/AztecProtocol/aztec-packages/commit/349ea59e58c7209358e9e1680e42775fd7d39d01))


### Bug Fixes

* **ci:** Turn on earthly for everyone ([#5423](https://github.com/AztecProtocol/aztec-packages/issues/5423)) ([bea3fcb](https://github.com/AztecProtocol/aztec-packages/commit/bea3fcbde91d08f13cb7c2ceeff8be33b3edcdfd))
* Cpp cache and add other e2e ([#5512](https://github.com/AztecProtocol/aztec-packages/issues/5512)) ([4118bcd](https://github.com/AztecProtocol/aztec-packages/commit/4118bcd278524b3ba72f8f656285beb1c284f8f2))
* Require noir-packages-test to finish ([#5505](https://github.com/AztecProtocol/aztec-packages/issues/5505)) ([191f0df](https://github.com/AztecProtocol/aztec-packages/commit/191f0df3245a90626c7e10bc0f618b783afe5bbd))
* Univariate evals not set in ECCVM prover ([#5529](https://github.com/AztecProtocol/aztec-packages/issues/5529)) ([f9a2b7c](https://github.com/AztecProtocol/aztec-packages/commit/f9a2b7c927a35efae1d45ab47eab5d8495862bcd))


### Miscellaneous

* Add goblin ops in add_gates_to_ensure_all_polys_are_non_zero  ([#5468](https://github.com/AztecProtocol/aztec-packages/issues/5468)) ([b9041e4](https://github.com/AztecProtocol/aztec-packages/commit/b9041e4dea9dba035481d8656886f1c70c671fac))
* **avm:** Add 15 additional 16-bit registers in ALU trace of AVM circuit ([#5503](https://github.com/AztecProtocol/aztec-packages/issues/5503)) ([8725c39](https://github.com/AztecProtocol/aztec-packages/commit/8725c393ef7efead6e6e19c341decaef56f6d035))
* **avm:** Migrate memory data structure in AVM circuit to unordered map ([#5506](https://github.com/AztecProtocol/aztec-packages/issues/5506)) ([ccd09aa](https://github.com/AztecProtocol/aztec-packages/commit/ccd09aae6b80f263b5d40c76adf98c29b3b50093))
* Build contracts and protocol circuits sequentially if not enough ram ([#5499](https://github.com/AztecProtocol/aztec-packages/issues/5499)) ([ea072b6](https://github.com/AztecProtocol/aztec-packages/commit/ea072b6de8af914efd72d1c3cb41dcadafb155a2))
* Bye bye shared ptrs for ultra/goblin ultra proving_keys :) ([#5407](https://github.com/AztecProtocol/aztec-packages/issues/5407)) ([b94d0db](https://github.com/AztecProtocol/aztec-packages/commit/b94d0db920f5194d3ebb9697cce6b1c9d194596b))
* Clean up compute_next_accumulator ([#5516](https://github.com/AztecProtocol/aztec-packages/issues/5516)) ([f9be2f2](https://github.com/AztecProtocol/aztec-packages/commit/f9be2f2f708cef5b375facbfd1dfb19710c5ab65))
* Explicit type imports ([#5519](https://github.com/AztecProtocol/aztec-packages/issues/5519)) ([2a217de](https://github.com/AztecProtocol/aztec-packages/commit/2a217de4da2031a9f3913a657a4b39201f4483bf))
* Improve caching in noir Earthfile ([#5513](https://github.com/AztecProtocol/aztec-packages/issues/5513)) ([5d1fb44](https://github.com/AztecProtocol/aztec-packages/commit/5d1fb44f58e5d403412c3dfa5e71df84684be5b0))
* Inject fetcher instead of using global ([#5502](https://github.com/AztecProtocol/aztec-packages/issues/5502)) ([a066544](https://github.com/AztecProtocol/aztec-packages/commit/a066544cda095adda0c1dc7918c64ecad8656b91))
* Make get notes return all notes at beginning of array [#4991](https://github.com/AztecProtocol/aztec-packages/issues/4991) ([#5321](https://github.com/AztecProtocol/aztec-packages/issues/5321)) ([5c5b627](https://github.com/AztecProtocol/aztec-packages/commit/5c5b6270e4cc9c09c7f208f6523227e51d4acc20))
* Move alphas generation to oink ([#5515](https://github.com/AztecProtocol/aztec-packages/issues/5515)) ([3b964f3](https://github.com/AztecProtocol/aztec-packages/commit/3b964f39fd4a1128f8db534ec00577a8833344a8))
* Replace relative paths to noir-protocol-circuits ([a689e4e](https://github.com/AztecProtocol/aztec-packages/commit/a689e4e91e65473af204d89956cedeebac2fc615))
* Replace relative paths to noir-protocol-circuits ([db1bab5](https://github.com/AztecProtocol/aztec-packages/commit/db1bab55259d301740d553ad9f18aeb5c64d1604))
* Replace relative paths to noir-protocol-circuits ([b2ab64b](https://github.com/AztecProtocol/aztec-packages/commit/b2ab64b253723c18f4320216b9ade8406427754c))
* Replace relative paths to noir-protocol-circuits ([1f468db](https://github.com/AztecProtocol/aztec-packages/commit/1f468db910906f25f54eaa8a208b8dfd97de7d98))
* Run nargo format for noir-projects ([#5483](https://github.com/AztecProtocol/aztec-packages/issues/5483)) ([277168f](https://github.com/AztecProtocol/aztec-packages/commit/277168f030c544074e8658850151d4d97d2e0ce1))

## [0.32.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.31.0...aztec-packages-v0.32.0) (2024-03-27)


### ⚠ BREAKING CHANGES

* Brillig typed memory ([#5395](https://github.com/AztecProtocol/aztec-packages/issues/5395))

### Features

* Add specific error for attempting `string[x] = ".."` (https://github.com/noir-lang/noir/pull/4611) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* **avm:** Dynamic gas costs for arithmetic, calldatacopy, and set ([#5473](https://github.com/AztecProtocol/aztec-packages/issues/5473)) ([bbd33fb](https://github.com/AztecProtocol/aztec-packages/commit/bbd33fb7628f0987bc8fa3bbd9c5166d2c4e2187))
* **avm:** EQ opcode output u8 and execution ([#5402](https://github.com/AztecProtocol/aztec-packages/issues/5402)) ([3450e24](https://github.com/AztecProtocol/aztec-packages/commit/3450e24fd025296ebe9cc2c7025f0e4fe811f997)), closes [#5290](https://github.com/AztecProtocol/aztec-packages/issues/5290)
* Benchmark tx size with fee ([#5414](https://github.com/AztecProtocol/aztec-packages/issues/5414)) ([543f8a2](https://github.com/AztecProtocol/aztec-packages/commit/543f8a232cf323b26d689a6632277701ee9fcff9)), closes [#5403](https://github.com/AztecProtocol/aztec-packages/issues/5403)
* Brillig typed memory ([#5395](https://github.com/AztecProtocol/aztec-packages/issues/5395)) ([16b0bdd](https://github.com/AztecProtocol/aztec-packages/commit/16b0bdd7fbca6ce296906dc9d3affa308571cbfe))
* Sequencer checks list of allowed FPCs ([#5310](https://github.com/AztecProtocol/aztec-packages/issues/5310)) ([adf20dc](https://github.com/AztecProtocol/aztec-packages/commit/adf20dc4974707255daffdaf3526dc48dc035873)), closes [#5000](https://github.com/AztecProtocol/aztec-packages/issues/5000)


### Bug Fixes

* `l1-contracts/bootstrap.sh` ([#5479](https://github.com/AztecProtocol/aztec-packages/issues/5479)) ([f7d1d70](https://github.com/AztecProtocol/aztec-packages/commit/f7d1d700ac84264c73c8d15e27b57ac130eebe7d))
* Add FPC to allowlist ([#5464](https://github.com/AztecProtocol/aztec-packages/issues/5464)) ([424960f](https://github.com/AztecProtocol/aztec-packages/commit/424960f4934c4f359ceef537d02ced98993b2283))
* **ci:** Fix earthly ctest ([#5424](https://github.com/AztecProtocol/aztec-packages/issues/5424)) ([9cac8a4](https://github.com/AztecProtocol/aztec-packages/commit/9cac8a43778ef7ab2cf62852bc427a7f6ed2391b))
* Docs example e2e test ([#5456](https://github.com/AztecProtocol/aztec-packages/issues/5456)) ([ae5126a](https://github.com/AztecProtocol/aztec-packages/commit/ae5126abaef0baea6ccaa058d29e42d3a7677a3b))
* Serial bb builds for mac ([#5462](https://github.com/AztecProtocol/aztec-packages/issues/5462)) ([4317819](https://github.com/AztecProtocol/aztec-packages/commit/43178199bf9e9e1e6131917e9da30118d4bbc8ab))
* Slice coercions (https://github.com/noir-lang/noir/pull/4640) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* **ssa:** Fix slice intrinsic handling in the capacity tracker  (https://github.com/noir-lang/noir/pull/4643) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* **ssa:** Use accurate type during SSA AsSlice simplficiation (https://github.com/noir-lang/noir/pull/4610) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))


### Miscellaneous

* Append-only merkle tree generics ([#5355](https://github.com/AztecProtocol/aztec-packages/issues/5355)) ([ef7bf79](https://github.com/AztecProtocol/aztec-packages/commit/ef7bf79bdb810389d27b0b1d280e6d563cef49aa))
* **avm:** Deterministic codegen from pil and some renaming ([#5476](https://github.com/AztecProtocol/aztec-packages/issues/5476)) ([ba834a4](https://github.com/AztecProtocol/aztec-packages/commit/ba834a445dbc23c715bba45bfd77b236361f5e24))
* **avm:** Test cleanup and update yp to allow for zero gas ([#5459](https://github.com/AztecProtocol/aztec-packages/issues/5459)) ([1829741](https://github.com/AztecProtocol/aztec-packages/commit/1829741598f4334f25a85871c984b2e009c893f1))
* **avm:** Unify noir macros flow ([#5461](https://github.com/AztecProtocol/aztec-packages/issues/5461)) ([54aee58](https://github.com/AztecProtocol/aztec-packages/commit/54aee58952b2433ccad83f1b5fc3088957b10fbb))
* **ci:** Add missing dependency to circleci config ([#5437](https://github.com/AztecProtocol/aztec-packages/issues/5437)) ([753cb78](https://github.com/AztecProtocol/aztec-packages/commit/753cb78e2f5caa30ba3f1f716e411bd89c761787))
* **ci:** Add warning sticky comment (https://github.com/noir-lang/noir/pull/4647) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Convert `BlockExpression` into a standard struct (https://github.com/noir-lang/noir/pull/4623) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Delete `R1CSTransformer` (https://github.com/noir-lang/noir/pull/4649) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Fallback to building barretenberg targets sequentially when RAM constrained ([#5426](https://github.com/AztecProtocol/aztec-packages/issues/5426)) ([29588e0](https://github.com/AztecProtocol/aztec-packages/commit/29588e05ea6ceb865c402260662742bcf053a6f1))
* Fix acvm crates reporting errors as JS packages (https://github.com/noir-lang/noir/pull/4637) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Fix versioning of `bn254_blackbox_solver` crate (https://github.com/noir-lang/noir/pull/4638) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* **github:** Improve PR template "document later" checkbox description (https://github.com/noir-lang/noir/pull/4625) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Introduce selectors to enable range checks of 8-bit and 16-bit sizes ([#5465](https://github.com/AztecProtocol/aztec-packages/issues/5465)) ([ef44674](https://github.com/AztecProtocol/aztec-packages/commit/ef4467476785a8df99f88bc21d64d0189a742136))
* Leveraging `Bufferable` in `pedersenHash(...)` and `sha256ToField(...)` ([#5444](https://github.com/AztecProtocol/aztec-packages/issues/5444)) ([0e0748c](https://github.com/AztecProtocol/aztec-packages/commit/0e0748cacac5b0e04e76b7241a07be372daaf32d))
* Release Noir(0.26.0) (https://github.com/noir-lang/noir/pull/4526) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Renaming rand() as unsafe_rand() ([#5457](https://github.com/AztecProtocol/aztec-packages/issues/5457)) ([19ad2bb](https://github.com/AztecProtocol/aztec-packages/commit/19ad2bbef65d8d5b3442d7aff5d2116d01fa915b)), closes [#3746](https://github.com/AztecProtocol/aztec-packages/issues/3746)
* Replace relative paths to noir-protocol-circuits ([d332ad1](https://github.com/AztecProtocol/aztec-packages/commit/d332ad16ecfe3cf7ccb7c111aa4d32fe9a054e1c))
* Timestamp as u64 instead of a Field ([#5453](https://github.com/AztecProtocol/aztec-packages/issues/5453)) ([d80dbbf](https://github.com/AztecProtocol/aztec-packages/commit/d80dbbf26b2fab00666451cb63ce92aa6fb58da7)), closes [#5446](https://github.com/AztecProtocol/aztec-packages/issues/5446)
* Typed encrypted and unencrypted L2 log containers ([#5422](https://github.com/AztecProtocol/aztec-packages/issues/5422)) ([a4d4ee8](https://github.com/AztecProtocol/aztec-packages/commit/a4d4ee8dc927cf7f8b09013c72a5b4cc4bf86075))
* Update docs with function names to match version 0.25.0 specifications (https://github.com/noir-lang/noir/pull/4466) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))
* Update integers.md to note support for Fields using `from_integer` (https://github.com/noir-lang/noir/pull/4536) ([27bd8d3](https://github.com/AztecProtocol/aztec-packages/commit/27bd8d318df486f6d30a01212f9d7894cafcec74))


### Documentation

* **yp:** Spec how bytecode is encoded in class registerer ([#5471](https://github.com/AztecProtocol/aztec-packages/issues/5471)) ([e3bced2](https://github.com/AztecProtocol/aztec-packages/commit/e3bced205c6f9adc237010ffbc0a1200ff11c7d2))

## [0.31.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.30.1...aztec-packages-v0.31.0) (2024-03-26)


### ⚠ BREAKING CHANGES

* **avm:** per function avm run ([#5421](https://github.com/AztecProtocol/aztec-packages/issues/5421))
* rename storage inclusion proof to historical storage read ([#5379](https://github.com/AztecProtocol/aztec-packages/issues/5379))
* plug-in new outbox and update examples to use api to fetch inclusion proofs #4769 ([#5292](https://github.com/AztecProtocol/aztec-packages/issues/5292))
* Mark transactions as reverted on L1 ([#5226](https://github.com/AztecProtocol/aztec-packages/issues/5226))

### Features

* Add batched signerless contract calls ([#5313](https://github.com/AztecProtocol/aztec-packages/issues/5313)) ([be60eb3](https://github.com/AztecProtocol/aztec-packages/commit/be60eb3afbf65cb9c2dec2e912e398caffb2ebd0))
* Add specific error for attempting `string[x] = ".."` (https://github.com/noir-lang/noir/pull/4611) ([13a12d5](https://github.com/AztecProtocol/aztec-packages/commit/13a12d5255e788be94d575c726da141e652f14e3))
* **AuthWit:** Chain_id and version in hash ([#5331](https://github.com/AztecProtocol/aztec-packages/issues/5331)) ([5235c95](https://github.com/AztecProtocol/aztec-packages/commit/5235c952e4d26d605ece3955fb50449cd378f615))
* **Authwit:** Lookup the validity of authwits ([#5316](https://github.com/AztecProtocol/aztec-packages/issues/5316)) ([7c24870](https://github.com/AztecProtocol/aztec-packages/commit/7c24870a0a446b8725ca4e8ac46b1004bea6c8f3))
* Avm lookup and/or/xor ([#5338](https://github.com/AztecProtocol/aztec-packages/issues/5338)) ([489bc2c](https://github.com/AztecProtocol/aztec-packages/commit/489bc2cbe9758064924462e65b5ec676f1a0d0c4))
* **avm:** Add AvmContextInputs ([#5396](https://github.com/AztecProtocol/aztec-packages/issues/5396)) ([12e2844](https://github.com/AztecProtocol/aztec-packages/commit/12e2844f9af433beb1a586640b08ce284ad91095))
* **avm:** Per function avm run ([#5421](https://github.com/AztecProtocol/aztec-packages/issues/5421)) ([f024751](https://github.com/AztecProtocol/aztec-packages/commit/f024751b944ff2fe4e870bb56046816dd8f343fe))
* **avm:** Track gas usage in AVM simulator ([#5438](https://github.com/AztecProtocol/aztec-packages/issues/5438)) ([4884d83](https://github.com/AztecProtocol/aztec-packages/commit/4884d833eaf6d8f2e79c87c72ce9a58c99524a69))
* Capture broadcasted functions in node ([#5353](https://github.com/AztecProtocol/aztec-packages/issues/5353)) ([bc05db2](https://github.com/AztecProtocol/aztec-packages/commit/bc05db26c864c9a9dae43f149814e082cdcfd7df))
* Dynamic proving ([#5346](https://github.com/AztecProtocol/aztec-packages/issues/5346)) ([6a7ccca](https://github.com/AztecProtocol/aztec-packages/commit/6a7ccca5dfa4a3354555f8b04b014da6ef72549a))
* Earthly bb tests + arm + satellites ([#5268](https://github.com/AztecProtocol/aztec-packages/issues/5268)) ([eca12b3](https://github.com/AztecProtocol/aztec-packages/commit/eca12b3a173f9ef1880e3b703ab778beb036a23b))
* Fix awkward snippet indention in docs ([#5367](https://github.com/AztecProtocol/aztec-packages/issues/5367)) ([c55d3da](https://github.com/AztecProtocol/aztec-packages/commit/c55d3daffd6dbf8c5c950cc8699dec13b7acca32))
* Fold proving key polys instead of prover polys ([#5436](https://github.com/AztecProtocol/aztec-packages/issues/5436)) ([239ebfb](https://github.com/AztecProtocol/aztec-packages/commit/239ebfb5cadee7b38fdc1e0f44d8b54533e44eb2))
* Implement serdes for u64 [#4990](https://github.com/AztecProtocol/aztec-packages/issues/4990) ([#5411](https://github.com/AztecProtocol/aztec-packages/issues/5411)) ([5a6bcef](https://github.com/AztecProtocol/aztec-packages/commit/5a6bcef8ecbeb1f3d89788fb5edab2b4fd88c8d0))
* Introduce max_block_number ([#5251](https://github.com/AztecProtocol/aztec-packages/issues/5251)) ([6573173](https://github.com/AztecProtocol/aztec-packages/commit/65731734559a8e937a0fdadc3d72d9672dc71308))
* Less earthly runners + e2e GA runners, bb bench ([#5356](https://github.com/AztecProtocol/aztec-packages/issues/5356)) ([2136a66](https://github.com/AztecProtocol/aztec-packages/commit/2136a66cc1fa2249b3ef47b787cfa1de9576dc38))
* Mark transactions as reverted on L1 ([#5226](https://github.com/AztecProtocol/aztec-packages/issues/5226)) ([40ecc02](https://github.com/AztecProtocol/aztec-packages/commit/40ecc02c4c307512bb31990d40d2c042ae10bebc))
* Plug-in new outbox and update examples to use api to fetch inclusion proofs [#4769](https://github.com/AztecProtocol/aztec-packages/issues/4769) ([#5292](https://github.com/AztecProtocol/aztec-packages/issues/5292)) ([fec1008](https://github.com/AztecProtocol/aztec-packages/commit/fec10081246f2005ef727bdb32ed79c67a1ebf9c))
* Read_calldata ([#5409](https://github.com/AztecProtocol/aztec-packages/issues/5409)) ([034fbf0](https://github.com/AztecProtocol/aztec-packages/commit/034fbf01e957a0e9f33a6a3b078c8acd33b8f3d8))
* Remove NUM_FIELDS_PER_SHA256 ([#5392](https://github.com/AztecProtocol/aztec-packages/issues/5392)) ([86a181b](https://github.com/AztecProtocol/aztec-packages/commit/86a181b821c62806275e5d33d357ecd3dd11918e))
* Rename storage inclusion proof to historical storage read ([#5379](https://github.com/AztecProtocol/aztec-packages/issues/5379)) ([b6e7216](https://github.com/AztecProtocol/aztec-packages/commit/b6e721672406bee0718d8b112b2ac0015fb81883))
* Returning non-nullified messages only ([#5390](https://github.com/AztecProtocol/aztec-packages/issues/5390)) ([4c671be](https://github.com/AztecProtocol/aztec-packages/commit/4c671be32fdf5d0f6f673e581cf035d00eaf0725))
* Simplified bb Honk interface ([#5319](https://github.com/AztecProtocol/aztec-packages/issues/5319)) ([a2d138f](https://github.com/AztecProtocol/aztec-packages/commit/a2d138fa8c0ecf90bea843d38d2d693d6a38b2cc))
* Simplify offsets and sizing using new block structure ([#5404](https://github.com/AztecProtocol/aztec-packages/issues/5404)) ([efa0842](https://github.com/AztecProtocol/aztec-packages/commit/efa08429f98933ed06bac4049921b0c08a5070f6))
* Throw by default when awaiting a tx that reverted ([#5431](https://github.com/AztecProtocol/aztec-packages/issues/5431)) ([c9113ec](https://github.com/AztecProtocol/aztec-packages/commit/c9113ec31fe905ce7ca7d448df5c90d418acb74c))
* Truncate SHA hashes inside circuits ([#5160](https://github.com/AztecProtocol/aztec-packages/issues/5160)) ([9dc0d2a](https://github.com/AztecProtocol/aztec-packages/commit/9dc0d2a718346ae43f97e4b525bfce0d250b47aa))
* Unified CircuitChecker interface ([#5343](https://github.com/AztecProtocol/aztec-packages/issues/5343)) ([13cef1f](https://github.com/AztecProtocol/aztec-packages/commit/13cef1f7c4f50a1a1941a92f070daf975c2f25f5))
* ZeroMorph working with IPA and integration with ECCVM ([#5246](https://github.com/AztecProtocol/aztec-packages/issues/5246)) ([c4dce94](https://github.com/AztecProtocol/aztec-packages/commit/c4dce948eba0daac3f6ba7812bd2e0d2d61fab24))


### Bug Fixes

* Addressing flakiness of `uniswap_trade_on_l1_from_l2.test.ts` ([#5443](https://github.com/AztecProtocol/aztec-packages/issues/5443)) ([2db9cad](https://github.com/AztecProtocol/aztec-packages/commit/2db9cad40c4d69b0c3f6f2b9baef713024a33f06))
* **avm-simulator:** Hashing opcodes indirection ([#5376](https://github.com/AztecProtocol/aztec-packages/issues/5376)) ([a4b1ebc](https://github.com/AztecProtocol/aztec-packages/commit/a4b1ebca3547936d61c41faee22a25b6e1eb625a))
* Broadcasting unconstrained function with empty sibling ([#5429](https://github.com/AztecProtocol/aztec-packages/issues/5429)) ([933145e](https://github.com/AztecProtocol/aztec-packages/commit/933145e894e1081976b04dd6a9838c6805e9e899))
* **ci:** Disable uniswap test in earthly build ([#5344](https://github.com/AztecProtocol/aztec-packages/issues/5344)) ([0d69162](https://github.com/AztecProtocol/aztec-packages/commit/0d6916205c98cb0e8e96de23f012d19632556509))
* **cli:** Support initializers not named constructor in cli ([#5397](https://github.com/AztecProtocol/aztec-packages/issues/5397)) ([85f14c5](https://github.com/AztecProtocol/aztec-packages/commit/85f14c5dc84c46910b8de498472959fa561d593c))
* Copy and deploy complete contents of l1-contracts ([#5447](https://github.com/AztecProtocol/aztec-packages/issues/5447)) ([501c5e9](https://github.com/AztecProtocol/aztec-packages/commit/501c5e95c4c536ea061ad6da5b9d8cd3ec322e5c))
* Don't cancel protocol-circuits-gate-diff in master ([#5441](https://github.com/AztecProtocol/aztec-packages/issues/5441)) ([6894a78](https://github.com/AztecProtocol/aztec-packages/commit/6894a781273e3973cda3f470941753835aa1d216))
* E2e_static_calls.test.ts bad merge ([#5405](https://github.com/AztecProtocol/aztec-packages/issues/5405)) ([4c56536](https://github.com/AztecProtocol/aztec-packages/commit/4c5653674a4d675842c9ead21d986efbac6376a8))
* Generate noir interface for constructors ([#5352](https://github.com/AztecProtocol/aztec-packages/issues/5352)) ([8434d2f](https://github.com/AztecProtocol/aztec-packages/commit/8434d2f3ab06eb64a0360eb362a0c23e29efcfe2))
* Limit earthly to few users ([#5375](https://github.com/AztecProtocol/aztec-packages/issues/5375)) ([71e8ab4](https://github.com/AztecProtocol/aztec-packages/commit/71e8ab4e96899c5e40bd496afff0848ef28f3336))
* Login to dockerhub before 'docker compose' ([#5440](https://github.com/AztecProtocol/aztec-packages/issues/5440)) ([4f7696b](https://github.com/AztecProtocol/aztec-packages/commit/4f7696b85f307e37020732fdad293906158a6d90))
* Revert cbind breakage ([#5348](https://github.com/AztecProtocol/aztec-packages/issues/5348)) ([c237193](https://github.com/AztecProtocol/aztec-packages/commit/c2371936d90fc58d643ae0a870c7ad60fa65adf5))
* **ssa:** Use accurate type during SSA AsSlice simplficiation (https://github.com/noir-lang/noir/pull/4610) ([13a12d5](https://github.com/AztecProtocol/aztec-packages/commit/13a12d5255e788be94d575c726da141e652f14e3))
* Track class registered count in tx stats ([#5417](https://github.com/AztecProtocol/aztec-packages/issues/5417)) ([ff8eafc](https://github.com/AztecProtocol/aztec-packages/commit/ff8eafc8575517c1fcccae9948115c57981478b0))
* Watch command should not spawn more than one tsc watch ([#5391](https://github.com/AztecProtocol/aztec-packages/issues/5391)) ([25caf4d](https://github.com/AztecProtocol/aztec-packages/commit/25caf4d8050cc7446595fcb159f140e7fbee6767))


### Miscellaneous

* Always use serialize function to get hash preimage in noir circuits or when comparing structs etc [#3595](https://github.com/AztecProtocol/aztec-packages/issues/3595) ([#5439](https://github.com/AztecProtocol/aztec-packages/issues/5439)) ([22e0f0d](https://github.com/AztecProtocol/aztec-packages/commit/22e0f0d502685481e8b807f524e6310fd3705d29))
* **aztec-nr:** Unify contexts behind interfaces ([#5294](https://github.com/AztecProtocol/aztec-packages/issues/5294)) ([36e0f59](https://github.com/AztecProtocol/aztec-packages/commit/36e0f59b6784b64940111541f70089b8444d01c5))
* **bb:** Removed powers of eta in lookup and auxiliary relations ([#4695](https://github.com/AztecProtocol/aztec-packages/issues/4695)) ([f4e62ae](https://github.com/AztecProtocol/aztec-packages/commit/f4e62ae5bcc7a0ef7baccc61e6e3e959196c891a))
* CamelCase in noir-projects -&gt; snake_case ([#5381](https://github.com/AztecProtocol/aztec-packages/issues/5381)) ([eea711f](https://github.com/AztecProtocol/aztec-packages/commit/eea711f974f3bbe5d170d6a0dc84943ee30505be))
* **ci:** Create a dedicated job for the AVM unit tests ([#5369](https://github.com/AztecProtocol/aztec-packages/issues/5369)) ([59ca2ac](https://github.com/AztecProtocol/aztec-packages/commit/59ca2ac213d9e5c8ec0d0e8890bae7cd4731c5ac)), closes [#5366](https://github.com/AztecProtocol/aztec-packages/issues/5366)
* Clean out prover instance and remove instance from oink ([#5314](https://github.com/AztecProtocol/aztec-packages/issues/5314)) ([a83368c](https://github.com/AztecProtocol/aztec-packages/commit/a83368c8da55fde6ea4a1135fbab47a5b5298e28))
* Cleaning up messaging types ([#5442](https://github.com/AztecProtocol/aztec-packages/issues/5442)) ([dfffe5d](https://github.com/AztecProtocol/aztec-packages/commit/dfffe5d879df3131769e7e8709a69ed0a6c63b2e)), closes [#5420](https://github.com/AztecProtocol/aztec-packages/issues/5420)
* Compute registerer address on the fly ([#5394](https://github.com/AztecProtocol/aztec-packages/issues/5394)) ([5d669b9](https://github.com/AztecProtocol/aztec-packages/commit/5d669b93d1210262afe246976904f1e86d6ff518))
* Delete slither output from version control ([#5393](https://github.com/AztecProtocol/aztec-packages/issues/5393)) ([41107e3](https://github.com/AztecProtocol/aztec-packages/commit/41107e34fde34e85c118d9a797348bd463402c4b))
* Fix migration notes ([#5452](https://github.com/AztecProtocol/aztec-packages/issues/5452)) ([8c4e576](https://github.com/AztecProtocol/aztec-packages/commit/8c4e5760cf9477c30261f9362d806bcbb5369191))
* **github:** Improve PR template "document later" checkbox description (https://github.com/noir-lang/noir/pull/4625) ([13a12d5](https://github.com/AztecProtocol/aztec-packages/commit/13a12d5255e788be94d575c726da141e652f14e3))
* Make get_notes fail if returning no notes [#4988](https://github.com/AztecProtocol/aztec-packages/issues/4988) ([#5320](https://github.com/AztecProtocol/aztec-packages/issues/5320)) ([be86ed3](https://github.com/AztecProtocol/aztec-packages/commit/be86ed3a6a2fa1c35ec7613da1a18bbd2327b18e))
* Meld flavor and and circuit builder modules ([#5406](https://github.com/AztecProtocol/aztec-packages/issues/5406)) ([f0d9d1b](https://github.com/AztecProtocol/aztec-packages/commit/f0d9d1ba7340d294426c05d36ef36831ca3e7705))
* Messaging naming fixes ([#5383](https://github.com/AztecProtocol/aztec-packages/issues/5383)) ([0226102](https://github.com/AztecProtocol/aztec-packages/commit/0226102c8161b02c60405b51439e0712f044c921))
* Moving public inputs back to instance ([#5315](https://github.com/AztecProtocol/aztec-packages/issues/5315)) ([9cbe368](https://github.com/AztecProtocol/aztec-packages/commit/9cbe368f8804d7d0dc49db3d555fbe1e2d3dd016))
* Name change: gen perm sort to delta range constraint ([#5378](https://github.com/AztecProtocol/aztec-packages/issues/5378)) ([841855f](https://github.com/AztecProtocol/aztec-packages/commit/841855fc069b89a5937e63194452f1a3cfd76f5c))
* Nuking l1 to l2 messages from block body ([#5272](https://github.com/AztecProtocol/aztec-packages/issues/5272)) ([ee176d2](https://github.com/AztecProtocol/aztec-packages/commit/ee176d23de19e8f87df1dbcce2f97614a2cf89bf)), closes [#5072](https://github.com/AztecProtocol/aztec-packages/issues/5072)
* Reduce size of revert code from Field to u8 ([#5309](https://github.com/AztecProtocol/aztec-packages/issues/5309)) ([1868e25](https://github.com/AztecProtocol/aztec-packages/commit/1868e256ba8c82fd44a6c0401e498c9de38309cd))
* Remove mocking function in `EccOpQueue` again ([#5413](https://github.com/AztecProtocol/aztec-packages/issues/5413)) ([6fb4a75](https://github.com/AztecProtocol/aztec-packages/commit/6fb4a755bcac78803bd2c709ca661c4ab0ca5b9e))
* Remove snapshots from protocol-contracts ([#5342](https://github.com/AztecProtocol/aztec-packages/issues/5342)) ([31ca344](https://github.com/AztecProtocol/aztec-packages/commit/31ca34482415f8bcae110438fcf76151e542041a))
* Remove unused FunctionLeafPreimage struct ([#5354](https://github.com/AztecProtocol/aztec-packages/issues/5354)) ([dc51c2b](https://github.com/AztecProtocol/aztec-packages/commit/dc51c2bf05decdcc47e7c5f2d0794a46b62652bb))
* Rename reverted to revertCode ([#5301](https://github.com/AztecProtocol/aztec-packages/issues/5301)) ([950a96d](https://github.com/AztecProtocol/aztec-packages/commit/950a96d0443951876289342d2d572aafbee54fed))
* Replace relative paths to noir-protocol-circuits ([262ae02](https://github.com/AztecProtocol/aztec-packages/commit/262ae027cefff853702facbfe1ea35047423b44c))
* Replace relative paths to noir-protocol-circuits ([91a60db](https://github.com/AztecProtocol/aztec-packages/commit/91a60db75533ccfad0c4ea75af716d05b50bd4fb))
* Replace relative paths to noir-protocol-circuits ([9fc9fbd](https://github.com/AztecProtocol/aztec-packages/commit/9fc9fbda5fb71da769f5ac0f78da4cadc2a13888))
* Replace relative paths to noir-protocol-circuits ([9939e99](https://github.com/AztecProtocol/aztec-packages/commit/9939e99b375073abb1853f8287c614143f593ec7))
* Replace relative paths to noir-protocol-circuits ([0b24aae](https://github.com/AztecProtocol/aztec-packages/commit/0b24aaed99a20e96cd8453aa11bd789f1e0e6cf1))
* Replace relative paths to noir-protocol-circuits ([c4d89d5](https://github.com/AztecProtocol/aztec-packages/commit/c4d89d5d177318fa6a2ec44fe89949ec452a9354))
* Reverting accidental changes ([#5371](https://github.com/AztecProtocol/aztec-packages/issues/5371)) ([c1484ce](https://github.com/AztecProtocol/aztec-packages/commit/c1484cefaaaaa26eee7dce72f47601362e7f54f9))
* Skip foundry install if possible ([#5398](https://github.com/AztecProtocol/aztec-packages/issues/5398)) ([060fa1e](https://github.com/AztecProtocol/aztec-packages/commit/060fa1e149d88ea1f20d4a474bd1c1d69a8f1518))
* Skip slither in docker ([#5384](https://github.com/AztecProtocol/aztec-packages/issues/5384)) ([8a76068](https://github.com/AztecProtocol/aztec-packages/commit/8a7606875fbb7d3eb15b6d8eaa7e297e1a8838ea))
* Update docs with function names to match version 0.25.0 specifications (https://github.com/noir-lang/noir/pull/4466) ([13a12d5](https://github.com/AztecProtocol/aztec-packages/commit/13a12d5255e788be94d575c726da141e652f14e3))
* Update integers.md to note support for Fields using `from_integer` (https://github.com/noir-lang/noir/pull/4536) ([13a12d5](https://github.com/AztecProtocol/aztec-packages/commit/13a12d5255e788be94d575c726da141e652f14e3))
* Update min compiler version of contracts ([#5305](https://github.com/AztecProtocol/aztec-packages/issues/5305)) ([dcf6bb3](https://github.com/AztecProtocol/aztec-packages/commit/dcf6bb332d950a7c26a626874ad6d799f70be787))
* Use random tmp directory and cleanup afterwards ([#5368](https://github.com/AztecProtocol/aztec-packages/issues/5368)) ([5c0e15d](https://github.com/AztecProtocol/aztec-packages/commit/5c0e15d69c3fefe6294ba654b827e2a89df2dc16))


### Documentation

* Update versions-updating.md ([#5358](https://github.com/AztecProtocol/aztec-packages/issues/5358)) ([0f09b63](https://github.com/AztecProtocol/aztec-packages/commit/0f09b63dc40969e9c5ac810faad1422abb40f586))

## [0.30.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.30.0...aztec-packages-v0.30.1) (2024-03-20)


### Features

* Add CMOV instruction to brillig and brillig gen ([#5308](https://github.com/AztecProtocol/aztec-packages/issues/5308)) ([208abbb](https://github.com/AztecProtocol/aztec-packages/commit/208abbb63af4c9a3f25d723fe1c49e82aa461061))
* **avm:** Indirect memory support for arithmetic/bitwise opcodes ([#5328](https://github.com/AztecProtocol/aztec-packages/issues/5328)) ([d5ffa17](https://github.com/AztecProtocol/aztec-packages/commit/d5ffa17f19d2887ddc98c3c90d323c5351de6570)), closes [#5273](https://github.com/AztecProtocol/aztec-packages/issues/5273)
* **avm:** Indirect memory support for MOV ([#5257](https://github.com/AztecProtocol/aztec-packages/issues/5257)) ([10ef970](https://github.com/AztecProtocol/aztec-packages/commit/10ef9702c43d36afd334a78df26fe0301c2ac001)), closes [#5205](https://github.com/AztecProtocol/aztec-packages/issues/5205)
* Merge SMT Terms in one class ([#5254](https://github.com/AztecProtocol/aztec-packages/issues/5254)) ([f5c9b0f](https://github.com/AztecProtocol/aztec-packages/commit/f5c9b0fdd095070f48ba38600b9bf53354b731f7))
* Sorted execution trace ([#5252](https://github.com/AztecProtocol/aztec-packages/issues/5252)) ([a216759](https://github.com/AztecProtocol/aztec-packages/commit/a216759d47b8a7c0b6d68c8cf8cfffab76f7e02d))


### Bug Fixes

* Fix recursion tests and reinstate in CI ([#5300](https://github.com/AztecProtocol/aztec-packages/issues/5300)) ([96c6f21](https://github.com/AztecProtocol/aztec-packages/commit/96c6f21b7f01be61af61ecc1a54ae7d6e23fd5af))
* Skip uniswap l1 tests ([#5334](https://github.com/AztecProtocol/aztec-packages/issues/5334)) ([7a56941](https://github.com/AztecProtocol/aztec-packages/commit/7a56941c94a8850aa4688c6446c52f67d2327562))
* Update smt_verification README.md ([#5332](https://github.com/AztecProtocol/aztec-packages/issues/5332)) ([46b15e3](https://github.com/AztecProtocol/aztec-packages/commit/46b15e3d7c851f8f6312fe76c1ad675d564694ab))


### Miscellaneous

* Avm team as generated codeowners ([#5325](https://github.com/AztecProtocol/aztec-packages/issues/5325)) ([06d2786](https://github.com/AztecProtocol/aztec-packages/commit/06d2786b3afa22bc3ce15d42d716b6ad3b6c4d86))
* No Translator composer ([#5202](https://github.com/AztecProtocol/aztec-packages/issues/5202)) ([c8897ca](https://github.com/AztecProtocol/aztec-packages/commit/c8897ca7e551d988df0e23c7b4e9587569685052))
* Remove toy vm files ([#5326](https://github.com/AztecProtocol/aztec-packages/issues/5326)) ([d940356](https://github.com/AztecProtocol/aztec-packages/commit/d940356ca5584b7328d9d398529ee23b21a1748d))
* Replace relative paths to noir-protocol-circuits ([ea2ac09](https://github.com/AztecProtocol/aztec-packages/commit/ea2ac095522c0ac7a6001fe6c78837554dcf251d))

## [0.30.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.29.0...aztec-packages-v0.30.0) (2024-03-19)


### ⚠ BREAKING CHANGES

* **acir:** Program and witness stack structure ([#5149](https://github.com/AztecProtocol/aztec-packages/issues/5149))
* automatic NoteInterface and NoteGetterOptions auto select ([#4508](https://github.com/AztecProtocol/aztec-packages/issues/4508))

### Features

* **acir:** Program and witness stack structure ([#5149](https://github.com/AztecProtocol/aztec-packages/issues/5149)) ([ccc5016](https://github.com/AztecProtocol/aztec-packages/commit/ccc5016eaeedbfb3f6be6763979e30e12485188b))
* Allow registering contract classes in PXE ([#5291](https://github.com/AztecProtocol/aztec-packages/issues/5291)) ([b811207](https://github.com/AztecProtocol/aztec-packages/commit/b811207bad691f519b31a6391967b9215a9e17d3)), closes [#4055](https://github.com/AztecProtocol/aztec-packages/issues/4055)
* Automatic NoteInterface and NoteGetterOptions auto select ([#4508](https://github.com/AztecProtocol/aztec-packages/issues/4508)) ([b2df979](https://github.com/AztecProtocol/aztec-packages/commit/b2df97907cb63446e9336e87f40f9dbd7a710845))
* ECCVM witness generation optimisation ([#5211](https://github.com/AztecProtocol/aztec-packages/issues/5211)) ([85ac726](https://github.com/AztecProtocol/aztec-packages/commit/85ac72604e443ae2d50edfd9ef74b745d4d5d169))
* Ensure claimer is owner of the note in claim contract ([#5135](https://github.com/AztecProtocol/aztec-packages/issues/5135)) ([a80519d](https://github.com/AztecProtocol/aztec-packages/commit/a80519d3514785cba74c64dc0f044f75b60adf40))
* Sequencer checks fee balance ([#5267](https://github.com/AztecProtocol/aztec-packages/issues/5267)) ([09b2b7c](https://github.com/AztecProtocol/aztec-packages/commit/09b2b7c19e023f541c44f79f78f7ee0f40f0c0ae))
* Verify registered artifact matches instance class id ([#5297](https://github.com/AztecProtocol/aztec-packages/issues/5297)) ([dd56a0e](https://github.com/AztecProtocol/aztec-packages/commit/dd56a0e7cc2fb09262f071c9f8c74d6e117b190e))


### Bug Fixes

* **bb:** Cvc5 linking ([#5302](https://github.com/AztecProtocol/aztec-packages/issues/5302)) ([5e9cf41](https://github.com/AztecProtocol/aztec-packages/commit/5e9cf418e14eee8b5a694d792c034a5745e2d25b))
* Don't run earthly arm for now ([#5289](https://github.com/AztecProtocol/aztec-packages/issues/5289)) ([e65e210](https://github.com/AztecProtocol/aztec-packages/commit/e65e2101c0ade6c1916135c0989cf8f95e0d3160))
* Set denominator to 1 during verification of dsl/big-field division ([#5188](https://github.com/AztecProtocol/aztec-packages/issues/5188)) ([253d002](https://github.com/AztecProtocol/aztec-packages/commit/253d0022aa051fe1ac6a53a88f67d084cfa98516))
* Update aztec-nr sync job ([#5299](https://github.com/AztecProtocol/aztec-packages/issues/5299)) ([ce22020](https://github.com/AztecProtocol/aztec-packages/commit/ce22020725966cf15341a9769fbe4a5280b8d706))


### Miscellaneous

* Add gas portal to l1 contract addresses ([#5265](https://github.com/AztecProtocol/aztec-packages/issues/5265)) ([640c89a](https://github.com/AztecProtocol/aztec-packages/commit/640c89a04d7b780795d40e239be3b3db73a16923)), closes [#5022](https://github.com/AztecProtocol/aztec-packages/issues/5022)
* Add note to pack arguments ([#5304](https://github.com/AztecProtocol/aztec-packages/issues/5304)) ([832de86](https://github.com/AztecProtocol/aztec-packages/commit/832de8638aa8eb111b9299e798f66aaf81eaf490))
* **avm-simulator:** Be explicit about wrapping arithmetic ([#5287](https://github.com/AztecProtocol/aztec-packages/issues/5287)) ([1b2cf58](https://github.com/AztecProtocol/aztec-packages/commit/1b2cf58a85bb4a29a47b3fdf0cdc19deea3f9a9c))
* **docs:** Update migration notes ([#5311](https://github.com/AztecProtocol/aztec-packages/issues/5311)) ([b47abcf](https://github.com/AztecProtocol/aztec-packages/commit/b47abcf8311561c83c49d431a66c7bd725ff95f9))
* Extract tx validation to separate class ([#5266](https://github.com/AztecProtocol/aztec-packages/issues/5266)) ([ba9bc4c](https://github.com/AztecProtocol/aztec-packages/commit/ba9bc4cddea559a3de7da174dc5d79406239f835))
* Fix yml for gate diff workflow ([#5293](https://github.com/AztecProtocol/aztec-packages/issues/5293)) ([edb8c67](https://github.com/AztecProtocol/aztec-packages/commit/edb8c6790c2ab9ab4439283fdb86d3ab8ba94ae4))
* L1 l2 messages cleanup ([#5270](https://github.com/AztecProtocol/aztec-packages/issues/5270)) ([30908eb](https://github.com/AztecProtocol/aztec-packages/commit/30908eb01c7de9d508eeb6404ba73316b19fab79)), closes [#5264](https://github.com/AztecProtocol/aztec-packages/issues/5264)
* Removing L1 block number from L2 block ([#5285](https://github.com/AztecProtocol/aztec-packages/issues/5285)) ([57596d7](https://github.com/AztecProtocol/aztec-packages/commit/57596d7897508958f5ec7dbc2dcd38a4839c02f6)), closes [#5274](https://github.com/AztecProtocol/aztec-packages/issues/5274)
* Replace relative paths to noir-protocol-circuits ([0962814](https://github.com/AztecProtocol/aztec-packages/commit/0962814ec4a4c623ed1aef4126bc379c8112358e))
* Share verifier rounds ([#4849](https://github.com/AztecProtocol/aztec-packages/issues/4849)) ([1139308](https://github.com/AztecProtocol/aztec-packages/commit/1139308d6d90ade1868278915901f86b08daedda))


### Documentation

* Remove broadcast-all methods from class registerer ([#5298](https://github.com/AztecProtocol/aztec-packages/issues/5298)) ([21ccb4b](https://github.com/AztecProtocol/aztec-packages/commit/21ccb4b3b3aa2e76bd9d849e1b7d59790ae33815)), closes [#4462](https://github.com/AztecProtocol/aztec-packages/issues/4462)
* Verification key includes proving system identifier ([#5295](https://github.com/AztecProtocol/aztec-packages/issues/5295)) ([6e218d4](https://github.com/AztecProtocol/aztec-packages/commit/6e218d41a3bd80852f541603043491c6c04e301c))

## [0.29.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.28.1...aztec-packages-v0.29.0) (2024-03-18)


### ⚠ BREAKING CHANGES

* Acir call opcode ([#4773](https://github.com/AztecProtocol/aztec-packages/issues/4773))

### Features

* Acir call opcode ([#4773](https://github.com/AztecProtocol/aztec-packages/issues/4773)) ([0b15db2](https://github.com/AztecProtocol/aztec-packages/commit/0b15db2bea70696597911e82b60f0def595c1150))
* Add as_slice builtin function, add execution test (https://github.com/noir-lang/noir/pull/4523) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Add more impls on Option (https://github.com/noir-lang/noir/pull/4549) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Add RelWithAssert build ([#4997](https://github.com/AztecProtocol/aztec-packages/issues/4997)) ([4f337c7](https://github.com/AztecProtocol/aztec-packages/commit/4f337c7c09539dcc4b11ef44d6728f9ed5248417))
* Allow usage of noir `#[test]` syntax in stdlib (https://github.com/noir-lang/noir/pull/4553) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* **AuthWit:** Simplify create authwit syntax ([#5132](https://github.com/AztecProtocol/aztec-packages/issues/5132)) ([d0a5b19](https://github.com/AztecProtocol/aztec-packages/commit/d0a5b1912cf795519557a2f2659080f21be73d52))
* **avm:** Brillig CONST of size &gt; u128 ([#5217](https://github.com/AztecProtocol/aztec-packages/issues/5217)) ([2e63479](https://github.com/AztecProtocol/aztec-packages/commit/2e634796d5d0f77242c6196cab05e9d386d03705))
* **avm:** Mov opcode with direct memory ([#5204](https://github.com/AztecProtocol/aztec-packages/issues/5204)) ([08f9038](https://github.com/AztecProtocol/aztec-packages/commit/08f903817f93028551f69b42ff02f0c3c10e8737)), closes [#5159](https://github.com/AztecProtocol/aztec-packages/issues/5159)
* Brillig IR refactor ([#5233](https://github.com/AztecProtocol/aztec-packages/issues/5233)) ([9a73348](https://github.com/AztecProtocol/aztec-packages/commit/9a7334877f5e109c6d2695a9119414c0643f480e))
* Check initializer msg.sender matches deployer from address preimage ([#5222](https://github.com/AztecProtocol/aztec-packages/issues/5222)) ([438d16f](https://github.com/AztecProtocol/aztec-packages/commit/438d16f71db4cbac8a8fd06e2d6db4c3209633aa))
* Extended IPA tests and fuzzing ([#5140](https://github.com/AztecProtocol/aztec-packages/issues/5140)) ([0ae5ace](https://github.com/AztecProtocol/aztec-packages/commit/0ae5ace4874676eb3739c556702bf39d1c799e8e))
* Initial Earthly CI ([#5069](https://github.com/AztecProtocol/aztec-packages/issues/5069)) ([8e75fe5](https://github.com/AztecProtocol/aztec-packages/commit/8e75fe5c47250e860a4eae4dbf0973c503221720))
* New Outbox Contract [#4768](https://github.com/AztecProtocol/aztec-packages/issues/4768) ([#5090](https://github.com/AztecProtocol/aztec-packages/issues/5090)) ([6421a3d](https://github.com/AztecProtocol/aztec-packages/commit/6421a3dc4713650183e0e7fbabd8b037b35a0f9f))
* Remove curly braces with fmt  (https://github.com/noir-lang/noir/pull/4529) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Remove curly braces with fmt  (https://github.com/noir-lang/noir/pull/4529) ([d8b8456](https://github.com/AztecProtocol/aztec-packages/commit/d8b8456860f2a7331d253af90f0326511651c2c4))
* Remove unnecessary `mulmod`s from verifier contract ([#5269](https://github.com/AztecProtocol/aztec-packages/issues/5269)) ([20d9c0c](https://github.com/AztecProtocol/aztec-packages/commit/20d9c0c6c3591975b9195810a334d4708e45690d))
* Signed integer division and modulus in brillig gen ([#5279](https://github.com/AztecProtocol/aztec-packages/issues/5279)) ([82f8cf5](https://github.com/AztecProtocol/aztec-packages/commit/82f8cf5eba9deacdab43ad4ef95dbf27dd1c11c7))
* Use deployer in address computation ([#5201](https://github.com/AztecProtocol/aztec-packages/issues/5201)) ([258ff4a](https://github.com/AztecProtocol/aztec-packages/commit/258ff4a00208be8695e2e59aecc14d6a92eaac1c))


### Bug Fixes

* **avm-transpiler:** RETURN is direct ([#5277](https://github.com/AztecProtocol/aztec-packages/issues/5277)) ([f90b2cf](https://github.com/AztecProtocol/aztec-packages/commit/f90b2cf6737f254405f48dbf7341d10d055edce3))
* **bb:** Mac build ([#5253](https://github.com/AztecProtocol/aztec-packages/issues/5253)) ([ae021c0](https://github.com/AztecProtocol/aztec-packages/commit/ae021c04ebdba07f94f1f5deeb2a142aedb78c1f))
* CVC5 api update ([#5203](https://github.com/AztecProtocol/aztec-packages/issues/5203)) ([9cc32cb](https://github.com/AztecProtocol/aztec-packages/commit/9cc32cb5e4aaf03ea3457a8fcf3b38c1e39d3d04))
* Evaluate operators in globals in types (https://github.com/noir-lang/noir/pull/4537) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Evaluate operators in globals in types (https://github.com/noir-lang/noir/pull/4537) ([d8b8456](https://github.com/AztecProtocol/aztec-packages/commit/d8b8456860f2a7331d253af90f0326511651c2c4))
* Make `nargo` the default binary for cargo run (https://github.com/noir-lang/noir/pull/4554) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Make `nargo` the default binary for cargo run (https://github.com/noir-lang/noir/pull/4554) ([d8b8456](https://github.com/AztecProtocol/aztec-packages/commit/d8b8456860f2a7331d253af90f0326511651c2c4))
* Revert "fix: noir mirror merge strat" ([#5250](https://github.com/AztecProtocol/aztec-packages/issues/5250)) ([7e8e8e5](https://github.com/AztecProtocol/aztec-packages/commit/7e8e8e522817ab4452ba609a935216d505c8bd31))
* Validation requests ([#5236](https://github.com/AztecProtocol/aztec-packages/issues/5236)) ([25ce33b](https://github.com/AztecProtocol/aztec-packages/commit/25ce33bfe1edbf314a99febde7f677db3a4113ad))


### Miscellaneous

* Add avm team to codeowners for public context ([#5288](https://github.com/AztecProtocol/aztec-packages/issues/5288)) ([e146076](https://github.com/AztecProtocol/aztec-packages/commit/e14607661d4c1b70cb59cabb36b685121e28728c))
* Add more `Hash` impls to stdlib (https://github.com/noir-lang/noir/pull/4470) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Add more `Hash` impls to stdlib (https://github.com/noir-lang/noir/pull/4470) ([d8b8456](https://github.com/AztecProtocol/aztec-packages/commit/d8b8456860f2a7331d253af90f0326511651c2c4))
* Add quick explanatory comment to outbox suggested by [@benesjan](https://github.com/benesjan) ([#5247](https://github.com/AztecProtocol/aztec-packages/issues/5247)) ([56e8451](https://github.com/AztecProtocol/aztec-packages/commit/56e8451b20cb7d2329932977857c10fc65af8efa))
* **avm-simulator:** Update e2e test ([#5283](https://github.com/AztecProtocol/aztec-packages/issues/5283)) ([e9beeca](https://github.com/AztecProtocol/aztec-packages/commit/e9beeca769bbc9748fbf341e0c0b7d12b9db9faa))
* **avm-transpiler:** Return u8 in comparison ops ([#5280](https://github.com/AztecProtocol/aztec-packages/issues/5280)) ([1a5eb69](https://github.com/AztecProtocol/aztec-packages/commit/1a5eb6923adb2f469021715182c1c5443e2d415c))
* **avm-transpiler:** Transpiler cleanup ([#5218](https://github.com/AztecProtocol/aztec-packages/issues/5218)) ([199e918](https://github.com/AztecProtocol/aztec-packages/commit/199e91855fec096149f4ca1b1e664e618a7319ab))
* Delete ContractDao ([#5256](https://github.com/AztecProtocol/aztec-packages/issues/5256)) ([544e278](https://github.com/AztecProtocol/aztec-packages/commit/544e27879738b1914d600ca70ced4d8e6d3cb545))
* Delete ContractData ([#5258](https://github.com/AztecProtocol/aztec-packages/issues/5258)) ([e516f9b](https://github.com/AztecProtocol/aztec-packages/commit/e516f9b94d1fbdc126a9d0d7d79c571d61914980))
* Delete ExtendedContractData struct ([#5248](https://github.com/AztecProtocol/aztec-packages/issues/5248)) ([8ae0c13](https://github.com/AztecProtocol/aztec-packages/commit/8ae0c13ceaf8a1f3db09d0e61f0a3781c8926ca6))
* Delete isInternal and isConstructor fields from FunctionData ([#5232](https://github.com/AztecProtocol/aztec-packages/issues/5232)) ([dea3f87](https://github.com/AztecProtocol/aztec-packages/commit/dea3f8705c1688cf1ef465dc7e72470d649a0de3))
* Delete unused contract tree ts code ([#5229](https://github.com/AztecProtocol/aztec-packages/issues/5229)) ([b48dd23](https://github.com/AztecProtocol/aztec-packages/commit/b48dd230a68971e80a2afacf78b8530724a20877))
* Delete unused hash functions ([#5231](https://github.com/AztecProtocol/aztec-packages/issues/5231)) ([fed70a1](https://github.com/AztecProtocol/aztec-packages/commit/fed70a127cc91453a81ea6019bc32f6db7e4f9a4))
* Fix docker test workflows (https://github.com/noir-lang/noir/pull/4566) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Fixing some broken links (https://github.com/noir-lang/noir/pull/4556) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Making docs build before cutting versions (https://github.com/noir-lang/noir/pull/4568) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Old inbox purge ([#5206](https://github.com/AztecProtocol/aztec-packages/issues/5206)) ([a26d968](https://github.com/AztecProtocol/aztec-packages/commit/a26d96851a2d9de5e5063052be42152898b0d83d))
* Removing redundant receipts check ([#5271](https://github.com/AztecProtocol/aztec-packages/issues/5271)) ([5ab07fb](https://github.com/AztecProtocol/aztec-packages/commit/5ab07fb8b395b6edbda6167845c7ea864e9395a3))
* Separate tests for execution failures from compilation failures (https://github.com/noir-lang/noir/pull/4559) ([86e1a86](https://github.com/AztecProtocol/aztec-packages/commit/86e1a86461bff5263567af33f20756ce560e22ca))
* Separate tests for execution failures from compilation failures (https://github.com/noir-lang/noir/pull/4559) ([d8b8456](https://github.com/AztecProtocol/aztec-packages/commit/d8b8456860f2a7331d253af90f0326511651c2c4))
* Template Zeromorph by PCS ([#5215](https://github.com/AztecProtocol/aztec-packages/issues/5215)) ([03feab2](https://github.com/AztecProtocol/aztec-packages/commit/03feab2f155f312ba63980a94d3cc4141916ad4d))
* Use inotifywait to run generate in yarn-project ([#5168](https://github.com/AztecProtocol/aztec-packages/issues/5168)) ([137c13e](https://github.com/AztecProtocol/aztec-packages/commit/137c13e3dc33b98d2b641afcf30ca991e9f6071f))


### Documentation

* **yp:** Remove contract tree and deploy data from circuits and state ([#5260](https://github.com/AztecProtocol/aztec-packages/issues/5260)) ([acffa7b](https://github.com/AztecProtocol/aztec-packages/commit/acffa7b4d41496dea0d16fd94ab98a1a977d14a8))

## [0.28.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.28.0...aztec-packages-v0.28.1) (2024-03-14)


### Bug Fixes

* Docs links URL missing a letter ([#5223](https://github.com/AztecProtocol/aztec-packages/issues/5223)) ([c015a3f](https://github.com/AztecProtocol/aztec-packages/commit/c015a3fc09ab495baa88b19ad2250b894834069a))
* **docs:** Update other constructor refs in docs to use initializer ([#5227](https://github.com/AztecProtocol/aztec-packages/issues/5227)) ([f68ff28](https://github.com/AztecProtocol/aztec-packages/commit/f68ff289084f047d9a50f7d82ec2ff8c5c839a7d))


### Miscellaneous

* **docs:** Add note on new initializer ([#5224](https://github.com/AztecProtocol/aztec-packages/issues/5224)) ([79c6e99](https://github.com/AztecProtocol/aztec-packages/commit/79c6e9970fce2f8a297961ecd4daf02026a4b89b))

## [0.28.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.27.2...aztec-packages-v0.28.0) (2024-03-14)


### ⚠ BREAKING CHANGES

* Support contracts with no constructor ([#5175](https://github.com/AztecProtocol/aztec-packages/issues/5175))

### Features

* **avm-simulator:** Euclidean and field div ([#5181](https://github.com/AztecProtocol/aztec-packages/issues/5181)) ([037a38f](https://github.com/AztecProtocol/aztec-packages/commit/037a38f498ee7f9d9c530a4b3b236e9c377b377d))
* Isolate Plonk dependencies ([#5068](https://github.com/AztecProtocol/aztec-packages/issues/5068)) ([5cbbd7d](https://github.com/AztecProtocol/aztec-packages/commit/5cbbd7da89488f6f662f96d0a3532921534755b4))
* New brillig field operations and refactor of binary operations ([#5208](https://github.com/AztecProtocol/aztec-packages/issues/5208)) ([eb69504](https://github.com/AztecProtocol/aztec-packages/commit/eb6950462b1ab2a0c8f50722791c7b0b9f1daf83))
* Parallelize linearly dependent contribution in PG ([#4742](https://github.com/AztecProtocol/aztec-packages/issues/4742)) ([d1799ae](https://github.com/AztecProtocol/aztec-packages/commit/d1799aeccb328582fabed25811e756bf0453216c))
* Parity circuits ([#5082](https://github.com/AztecProtocol/aztec-packages/issues/5082)) ([335c46e](https://github.com/AztecProtocol/aztec-packages/commit/335c46e7b7eddc0396190e6dae7eb2255e3caa9e))
* Support contracts with no constructor ([#5175](https://github.com/AztecProtocol/aztec-packages/issues/5175)) ([df7fa32](https://github.com/AztecProtocol/aztec-packages/commit/df7fa32f34e790231e091c38a4a6e84be5407763))
* Track side effects in public ([#5129](https://github.com/AztecProtocol/aztec-packages/issues/5129)) ([d666f6f](https://github.com/AztecProtocol/aztec-packages/commit/d666f6f1a0a67fd95694bb5f42b3e7af19a0abea)), closes [#5185](https://github.com/AztecProtocol/aztec-packages/issues/5185)
* Update SMT Circuit class and add gate relaxation functionality ([#5176](https://github.com/AztecProtocol/aztec-packages/issues/5176)) ([5948996](https://github.com/AztecProtocol/aztec-packages/commit/5948996c0bab8ee99c4686352b8475da38604f28))


### Bug Fixes

* **avm-transpiler:** FDIV and U128 test case ([#5200](https://github.com/AztecProtocol/aztec-packages/issues/5200)) ([6977e81](https://github.com/AztecProtocol/aztec-packages/commit/6977e8166b5c27685458a6e04e840b45a77d4765))
* Barretenberg-acir-tests-bb.js thru version bump ([#5216](https://github.com/AztecProtocol/aztec-packages/issues/5216)) ([9298f93](https://github.com/AztecProtocol/aztec-packages/commit/9298f932b2d22aa5a4c87dab90d5e72614f222da))
* Do not release docs on every commit to master ([#5214](https://github.com/AztecProtocol/aztec-packages/issues/5214)) ([c34a299](https://github.com/AztecProtocol/aztec-packages/commit/c34a299e354847e3e4e253b41921814e86b38645))
* Fail transaction if we revert in setup or teardown ([#5093](https://github.com/AztecProtocol/aztec-packages/issues/5093)) ([db9a960](https://github.com/AztecProtocol/aztec-packages/commit/db9a960a99db663a328b261e08917ce5f1dd4e69))
* Intermittent invert 0 in Goblin ([#5189](https://github.com/AztecProtocol/aztec-packages/issues/5189)) ([6c70624](https://github.com/AztecProtocol/aztec-packages/commit/6c7062443ae23cc75ac06b7ac1492d12f803d0e5))
* Point docs links to current tag if available ([#5219](https://github.com/AztecProtocol/aztec-packages/issues/5219)) ([0e9c7c7](https://github.com/AztecProtocol/aztec-packages/commit/0e9c7c757ed5501d01bb20a57f22e857cf50b93d))
* Remove embedded srs ([#5173](https://github.com/AztecProtocol/aztec-packages/issues/5173)) ([cfd673d](https://github.com/AztecProtocol/aztec-packages/commit/cfd673d6224e95a7b09eaa51e1f6535b277b2827))
* Split setup/teardown functions when there's no public app logic ([#5156](https://github.com/AztecProtocol/aztec-packages/issues/5156)) ([2ee13b3](https://github.com/AztecProtocol/aztec-packages/commit/2ee13b3e9d17d4715ec72c738cf74e75e3c1581f))
* Validate EthAddress size in aztec-nr ([#5198](https://github.com/AztecProtocol/aztec-packages/issues/5198)) ([201c5e1](https://github.com/AztecProtocol/aztec-packages/commit/201c5e1cf94b448f4f75f460a9838b526903f3ce))


### Miscellaneous

* Add dependency instructions to bberg README ([#5187](https://github.com/AztecProtocol/aztec-packages/issues/5187)) ([850febc](https://github.com/AztecProtocol/aztec-packages/commit/850febc31400b0f5ca2064d91833a847adc5df31))
* **avm-simulator:** Make sure we support Map storage ([#5207](https://github.com/AztecProtocol/aztec-packages/issues/5207)) ([08835f9](https://github.com/AztecProtocol/aztec-packages/commit/08835f99e11c479cb498b411b15a16305695039f))
* **avm-simulator:** Restructure contract storage tests ([#5194](https://github.com/AztecProtocol/aztec-packages/issues/5194)) ([fcdd1cc](https://github.com/AztecProtocol/aztec-packages/commit/fcdd1cc260c1faf14eb5fe719d5c7f5306699b1e))
* **docs:** Add details to getting started contract deployment ([#5220](https://github.com/AztecProtocol/aztec-packages/issues/5220)) ([5c267ae](https://github.com/AztecProtocol/aztec-packages/commit/5c267ae50561c36eb02b84e5f8f7043b929e906c))
* Moving wit comms and witness and comm labels from instance to oink ([#5199](https://github.com/AztecProtocol/aztec-packages/issues/5199)) ([19eb7f9](https://github.com/AztecProtocol/aztec-packages/commit/19eb7f9bd48f1f5fb8d9e9a2e172c8f0c2c9445b))
* Oink ([#5210](https://github.com/AztecProtocol/aztec-packages/issues/5210)) ([321f149](https://github.com/AztecProtocol/aztec-packages/commit/321f149dd720f2e74d3b4118bf75c910b466d0ed))
* Pull noir ([#5193](https://github.com/AztecProtocol/aztec-packages/issues/5193)) ([aa90f6e](https://github.com/AztecProtocol/aztec-packages/commit/aa90f6ed7bfae06bdf6990816d154bbd24993689))
* Trying to fix intermitent ci failure for boxes ([#5182](https://github.com/AztecProtocol/aztec-packages/issues/5182)) ([f988cb8](https://github.com/AztecProtocol/aztec-packages/commit/f988cb85a35fbc16690c81071d8153bd76c51185))


### Documentation

* **yellow-paper:** Add pseudocode for verifying broadcasted functions in contract deployment ([#4431](https://github.com/AztecProtocol/aztec-packages/issues/4431)) ([8bdb921](https://github.com/AztecProtocol/aztec-packages/commit/8bdb9213ff2560a83aadd7cc4af062e08e98bd22))

## [0.27.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.27.1...aztec-packages-v0.27.2) (2024-03-13)


### Features

* Check initialization arguments in constructors ([#5144](https://github.com/AztecProtocol/aztec-packages/issues/5144)) ([d003bd6](https://github.com/AztecProtocol/aztec-packages/commit/d003bd62c1b7ba063f3a3a8f58c698f534bc7148))
* Multithreaded prover folding ([#5147](https://github.com/AztecProtocol/aztec-packages/issues/5147)) ([94922fc](https://github.com/AztecProtocol/aztec-packages/commit/94922fc24e728100b456ed5f0203974964fd9f83))
* Run tests in parallel in `nargo test`  (https://github.com/noir-lang/noir/pull/4484) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Track stack frames and their variables in the debugger (https://github.com/noir-lang/noir/pull/4188) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))


### Bug Fixes

* **acir_gen:** More granular element sizes array check (https://github.com/noir-lang/noir/pull/4528) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Add `follow_bindings` to follow `Type::Alias` links (https://github.com/noir-lang/noir/pull/4521) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Allow type aliases in main (https://github.com/noir-lang/noir/pull/4505) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Constant gen ([#5172](https://github.com/AztecProtocol/aztec-packages/issues/5172)) ([394a0e0](https://github.com/AztecProtocol/aztec-packages/commit/394a0e06928946c1c9eea1bdfec39269cb2d601a))
* **docs:** Update quickstart.md ([#5021](https://github.com/AztecProtocol/aztec-packages/issues/5021)) ([be9f8a1](https://github.com/AztecProtocol/aztec-packages/commit/be9f8a15c2c0b006e8d2d469cf9aa56b4346d52d))
* Dynamic assert messages in brillig (https://github.com/noir-lang/noir/pull/4531) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Fix brillig slowdown when assigning arrays in loops (https://github.com/noir-lang/noir/pull/4472) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Fix deployments ([#5183](https://github.com/AztecProtocol/aztec-packages/issues/5183)) ([596253b](https://github.com/AztecProtocol/aztec-packages/commit/596253b7c3317dbde5ebc826992b2654d5c5a83a))
* Force src impl for == on slices (https://github.com/noir-lang/noir/pull/4507) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Handling of gh deps in noir_wasm (https://github.com/noir-lang/noir/pull/4499) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Intermittent invert 0 in Goblin ([#5174](https://github.com/AztecProtocol/aztec-packages/issues/5174)) ([3e68b49](https://github.com/AztecProtocol/aztec-packages/commit/3e68b49f717aa643eb616976f6cc7ed0ac07686d))
* Iterative flattening pass (https://github.com/noir-lang/noir/pull/4492) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Noir mirror merge strat ([#5166](https://github.com/AztecProtocol/aztec-packages/issues/5166)) ([74fa8d6](https://github.com/AztecProtocol/aztec-packages/commit/74fa8d6de2da4509a1679f9b1c76c6f22df16139))
* **ssa:** Handle mergers of slices returned from calls (https://github.com/noir-lang/noir/pull/4496) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))


### Miscellaneous

* Add `ModuleDeclaration` struct (https://github.com/noir-lang/noir/pull/4512) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Add HashMap docs (https://github.com/noir-lang/noir/pull/4457) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Add regression test for issue 4449 (https://github.com/noir-lang/noir/pull/4503) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Better output in ci_deploy_contracts.sh ([#5171](https://github.com/AztecProtocol/aztec-packages/issues/5171)) ([8d73f8a](https://github.com/AztecProtocol/aztec-packages/commit/8d73f8aac3608e699cbf8face3f37f707f108d33))
* Bump bb to 0.26.3 (https://github.com/noir-lang/noir/pull/4488) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* **ci:** Fix JS publishing workflow checking out inconsistent commits (https://github.com/noir-lang/noir/pull/4493) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Custom hash for eddsa (https://github.com/noir-lang/noir/pull/4440) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Deterministic mode ([#5155](https://github.com/AztecProtocol/aztec-packages/issues/5155)) ([e68b56a](https://github.com/AztecProtocol/aztec-packages/commit/e68b56aa2beaaa1b8b58e0920ac531c6abe05668))
* Document big integers (https://github.com/noir-lang/noir/pull/4487) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Generalise `FunctionVisibility` to `ItemVisibility` (https://github.com/noir-lang/noir/pull/4495) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Interaction for a mock first circuit handled inside the `EccOpQueue` ([#4854](https://github.com/AztecProtocol/aztec-packages/issues/4854)) ([d9cbdc8](https://github.com/AztecProtocol/aztec-packages/commit/d9cbdc888d467ade8add5c3c03a1759dddbb398a))
* Move `check_method_signatures` to type checking phase (https://github.com/noir-lang/noir/pull/4516) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Move templated code for assert_message into the stdlib (https://github.com/noir-lang/noir/pull/4475) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Organize the `blackbox_solver` crate (https://github.com/noir-lang/noir/pull/4519) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Pass `import_directive` by reference (https://github.com/noir-lang/noir/pull/4511) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Pass macro processors by reference (https://github.com/noir-lang/noir/pull/4501) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Pull out separate function for compiling and running a test ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Release Noir(0.25.0) (https://github.com/noir-lang/noir/pull/4352) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Update cargo deny config (https://github.com/noir-lang/noir/pull/4486) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))
* Update various dependencies (https://github.com/noir-lang/noir/pull/4513) ([58e15ed](https://github.com/AztecProtocol/aztec-packages/commit/58e15edf7fd3d32267b0aed883fc84f6cee327c9))

## [0.27.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.27.0...aztec-packages-v0.27.1) (2024-03-12)


### Features

* Further ClientIVC breakdown ([#5146](https://github.com/AztecProtocol/aztec-packages/issues/5146)) ([c8e1cb8](https://github.com/AztecProtocol/aztec-packages/commit/c8e1cb8c6bc07bda2cf4aec3b5d2b2120bfafd01))
* Nullifier non membership ([#5152](https://github.com/AztecProtocol/aztec-packages/issues/5152)) ([426bd6d](https://github.com/AztecProtocol/aztec-packages/commit/426bd6d2490d59126a31de91cd783a76b0dfdc84))


### Bug Fixes

* Increase the json limit for RPC requests ([#5161](https://github.com/AztecProtocol/aztec-packages/issues/5161)) ([419958c](https://github.com/AztecProtocol/aztec-packages/commit/419958c7c9abc40a5d83739a74e1ea0797b4f474))
* Move timers for ClientIVC breakdown ([#5145](https://github.com/AztecProtocol/aztec-packages/issues/5145)) ([5457edb](https://github.com/AztecProtocol/aztec-packages/commit/5457edb3ddd29df96906f98fb05469a26a644654))


### Miscellaneous

* **boxes:** Adding clone contract option ([#4980](https://github.com/AztecProtocol/aztec-packages/issues/4980)) ([a427aa5](https://github.com/AztecProtocol/aztec-packages/commit/a427aa533216187a23b0697a2d91a9d89fb1e0eb))
* Share code between provers ([#4655](https://github.com/AztecProtocol/aztec-packages/issues/4655)) ([ef10d65](https://github.com/AztecProtocol/aztec-packages/commit/ef10d6576aa9e89eece5a40669c425ae7987ee8a))

## [0.27.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.6...aztec-packages-v0.27.0) (2024-03-12)


### ⚠ BREAKING CHANGES

* Remove open keyword from Noir ([#4967](https://github.com/AztecProtocol/aztec-packages/issues/4967))

### Features

* Add api for inclusion proof of outgoing message in block [#4562](https://github.com/AztecProtocol/aztec-packages/issues/4562) ([#4899](https://github.com/AztecProtocol/aztec-packages/issues/4899)) ([26d2643](https://github.com/AztecProtocol/aztec-packages/commit/26d26437022567e2d54052f21b1c937259f26c94))
* **avm-simulator:** External calls + integration ([#5051](https://github.com/AztecProtocol/aztec-packages/issues/5051)) ([dde33f4](https://github.com/AztecProtocol/aztec-packages/commit/dde33f498b0432e5c4adce84191d3517176077dd))
* **avm-simulator:** External static calls + integration ([#5089](https://github.com/AztecProtocol/aztec-packages/issues/5089)) ([428d950](https://github.com/AztecProtocol/aztec-packages/commit/428d950ec1f2dc7b129b61380d7d1426a7b7441d))
* **avm:** Equivalence check between Main trace and Mem trace ([#5032](https://github.com/AztecProtocol/aztec-packages/issues/5032)) ([7f216eb](https://github.com/AztecProtocol/aztec-packages/commit/7f216eb064fc95791de1286c7695e89575e02b40)), closes [#4955](https://github.com/AztecProtocol/aztec-packages/issues/4955)
* **avm:** Fix some Brillig problems ([#5091](https://github.com/AztecProtocol/aztec-packages/issues/5091)) ([07dd821](https://github.com/AztecProtocol/aztec-packages/commit/07dd8215dffd2c3c6d22e0f430f5072b4ff7c763))
* Initial integration avm prover ([#4878](https://github.com/AztecProtocol/aztec-packages/issues/4878)) ([2e2554e](https://github.com/AztecProtocol/aztec-packages/commit/2e2554e6a055ff7124e18d1566371d5d108c5d5d))
* Noir pull action ([#5062](https://github.com/AztecProtocol/aztec-packages/issues/5062)) ([b2d7d14](https://github.com/AztecProtocol/aztec-packages/commit/b2d7d14996722c50c769dfcd9f7b0c324b2e3a7e))
* Restore contract inclusion proofs ([#5141](https://github.com/AztecProtocol/aztec-packages/issues/5141)) ([a39cd61](https://github.com/AztecProtocol/aztec-packages/commit/a39cd6192022cd14b824d159b4262c10669b7de3))
* Update the core of SMT Circuit class ([#5096](https://github.com/AztecProtocol/aztec-packages/issues/5096)) ([1519d3b](https://github.com/AztecProtocol/aztec-packages/commit/1519d3b07664f471a43d3f6bbb3dbe2d387289fc))
* Updating archiver with new inbox ([#5025](https://github.com/AztecProtocol/aztec-packages/issues/5025)) ([f6d17c9](https://github.com/AztecProtocol/aztec-packages/commit/f6d17c972d2cf9c5aa468c8cf954431b42240f87)), closes [#4828](https://github.com/AztecProtocol/aztec-packages/issues/4828)


### Bug Fixes

* Duplicate factory code temporarily to unblock ([#5099](https://github.com/AztecProtocol/aztec-packages/issues/5099)) ([8b10600](https://github.com/AztecProtocol/aztec-packages/commit/8b1060013e35a3b4e73d75b18bb2a8c16985e662))
* Remove hard coded canonical gas address ([#5106](https://github.com/AztecProtocol/aztec-packages/issues/5106)) ([dc2fd9e](https://github.com/AztecProtocol/aztec-packages/commit/dc2fd9e584d987bdc5d2d7a117b76cb50a20b969))


### Miscellaneous

* **avm-simulator:** Enable compressed strings unencrypted log test ([#5083](https://github.com/AztecProtocol/aztec-packages/issues/5083)) ([8f7519b](https://github.com/AztecProtocol/aztec-packages/commit/8f7519bdacd3c8b3a91d4361e4648688ec5d47bc))
* **avm-simulator:** Formatting and fixes ([#5092](https://github.com/AztecProtocol/aztec-packages/issues/5092)) ([b3fa084](https://github.com/AztecProtocol/aztec-packages/commit/b3fa08469658bd7220863e514d8e4b069d40a00f))
* **AVM:** Negative unit tests for inter table relations ([#5143](https://github.com/AztecProtocol/aztec-packages/issues/5143)) ([a74dccb](https://github.com/AztecProtocol/aztec-packages/commit/a74dccbdef0939b77978ddec3875b1afc2d0b530)), closes [#5033](https://github.com/AztecProtocol/aztec-packages/issues/5033)
* Aztec-macros refactor ([#5127](https://github.com/AztecProtocol/aztec-packages/issues/5127)) ([2195441](https://github.com/AztecProtocol/aztec-packages/commit/2195441afde4d6e78ad0c6027d0a7dbc8671817d))
* **ci:** Fail on clippy warnings in noir ([#5101](https://github.com/AztecProtocol/aztec-packages/issues/5101)) ([54af648](https://github.com/AztecProtocol/aztec-packages/commit/54af648b5928b200cd40c8d90a21c155bc2e43bd))
* Extract bb binary in bs fast ([#5128](https://github.com/AztecProtocol/aztec-packages/issues/5128)) ([9ca41ef](https://github.com/AztecProtocol/aztec-packages/commit/9ca41ef6951566622ab9e68924958dbb66b160df))
* Increase bytecode size limit ([#5098](https://github.com/AztecProtocol/aztec-packages/issues/5098)) ([53b2381](https://github.com/AztecProtocol/aztec-packages/commit/53b238190a9d123c292c3079bb23ed2ecff824c8))
* Increase permitted bytecode size ([#5136](https://github.com/AztecProtocol/aztec-packages/issues/5136)) ([6865c34](https://github.com/AztecProtocol/aztec-packages/commit/6865c34fccfd74f83525c8d47b5c516d1696c432))
* Join-split example Part 2 ([#5016](https://github.com/AztecProtocol/aztec-packages/issues/5016)) ([0718320](https://github.com/AztecProtocol/aztec-packages/commit/07183200b136ec39087c2b35e5799686319d561b))
* Move alpine containers to ubuntu ([#5026](https://github.com/AztecProtocol/aztec-packages/issues/5026)) ([d483e67](https://github.com/AztecProtocol/aztec-packages/commit/d483e678e4b2558f74c3b79083cf2257d6eafe0c)), closes [#4708](https://github.com/AztecProtocol/aztec-packages/issues/4708)
* Nicer snapshots ([#5133](https://github.com/AztecProtocol/aztec-packages/issues/5133)) ([9a737eb](https://github.com/AztecProtocol/aztec-packages/commit/9a737eb9674a757ca3ac9c7a6607ed0f39304d52))
* Pin foundry ([#5151](https://github.com/AztecProtocol/aztec-packages/issues/5151)) ([69bd7dd](https://github.com/AztecProtocol/aztec-packages/commit/69bd7dd45af6b197b23c25dc883a1a5485955203))
* Remove old contract deployment flow ([#4970](https://github.com/AztecProtocol/aztec-packages/issues/4970)) ([6d15947](https://github.com/AztecProtocol/aztec-packages/commit/6d1594736e96cd744ea691a239fcd3a46bdade60))
* Remove open keyword from Noir ([#4967](https://github.com/AztecProtocol/aztec-packages/issues/4967)) ([401557e](https://github.com/AztecProtocol/aztec-packages/commit/401557e1119c1dc4968c16f51381f3306ed8e876))
* Run nargo fmt on each nargo project ([#5102](https://github.com/AztecProtocol/aztec-packages/issues/5102)) ([b327254](https://github.com/AztecProtocol/aztec-packages/commit/b32725421171f39d510619c8f78a39c182738725))
* Use context interface in mark-as-initialized ([#5142](https://github.com/AztecProtocol/aztec-packages/issues/5142)) ([932c1d5](https://github.com/AztecProtocol/aztec-packages/commit/932c1d5006ad793ee05ed7cdbae05d59c04334d8))

## [0.26.6](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.5...aztec-packages-v0.26.6) (2024-03-08)


### Features

* Basic public reverts ([#4870](https://github.com/AztecProtocol/aztec-packages/issues/4870)) ([5cccc78](https://github.com/AztecProtocol/aztec-packages/commit/5cccc78e47173006e7a78ab928a7f352d23418be))
* Deploying new inbox ([#5036](https://github.com/AztecProtocol/aztec-packages/issues/5036)) ([fed729d](https://github.com/AztecProtocol/aztec-packages/commit/fed729daaa667b78e9291a9eae5ab8225f82d572))
* Detect unknown note type ids in compute_note_hash ([#5086](https://github.com/AztecProtocol/aztec-packages/issues/5086)) ([6206bec](https://github.com/AztecProtocol/aztec-packages/commit/6206becb094bed9163e77b1f67b78bdd30a72c38))
* Easy deployment of protocol contracts in e2e ([#4983](https://github.com/AztecProtocol/aztec-packages/issues/4983)) ([480161f](https://github.com/AztecProtocol/aztec-packages/commit/480161f386465076d4a2811ead5633433aee7cd8))
* IPA documentation ([#4924](https://github.com/AztecProtocol/aztec-packages/issues/4924)) ([48bd22e](https://github.com/AztecProtocol/aztec-packages/commit/48bd22eaab6d9df38d856db943f35292a42ea928))
* Nullifier read requests in public kernel ([#4910](https://github.com/AztecProtocol/aztec-packages/issues/4910)) ([0e44247](https://github.com/AztecProtocol/aztec-packages/commit/0e442474ba76142bb0597e584cd9626b6c205ed6))
* Show bytecode size per function in CLI inspect-contract ([#5059](https://github.com/AztecProtocol/aztec-packages/issues/5059)) ([cb9fdc6](https://github.com/AztecProtocol/aztec-packages/commit/cb9fdc6b5069ee2ab8fb1f68f369e360039fa18b))
* Updating an SMT solver class ([#4981](https://github.com/AztecProtocol/aztec-packages/issues/4981)) ([4b94d58](https://github.com/AztecProtocol/aztec-packages/commit/4b94d580a7add893a305e453e0f9005694759dc4))


### Bug Fixes

* Canonical contract address ([#5030](https://github.com/AztecProtocol/aztec-packages/issues/5030)) ([b2af880](https://github.com/AztecProtocol/aztec-packages/commit/b2af8805587b7ccb002ff1b216ac3b57b2839d63))
* Flaky deployment test ([#5035](https://github.com/AztecProtocol/aztec-packages/issues/5035)) ([039eafc](https://github.com/AztecProtocol/aztec-packages/commit/039eafc4cea398fcded386a982dc52c74458e39a))
* Pull the correct platform image for noir ([#5097](https://github.com/AztecProtocol/aztec-packages/issues/5097)) ([3342371](https://github.com/AztecProtocol/aztec-packages/commit/3342371cbf21806664367f89ffa56a25c1b3ec13))
* Sleep function memory leak ([#5023](https://github.com/AztecProtocol/aztec-packages/issues/5023)) ([a72cfea](https://github.com/AztecProtocol/aztec-packages/commit/a72cfea60ef33e19e2e003fa3093bb46a4b75886)), closes [#4817](https://github.com/AztecProtocol/aztec-packages/issues/4817)
* Storage v2 ([#5027](https://github.com/AztecProtocol/aztec-packages/issues/5027)) ([fe3190e](https://github.com/AztecProtocol/aztec-packages/commit/fe3190ee66d5c340b6ef6a6fe53772e8e08c9463))
* Update protogalaxy cmake dependencies ([#5066](https://github.com/AztecProtocol/aztec-packages/issues/5066)) ([507c374](https://github.com/AztecProtocol/aztec-packages/commit/507c374b65c7947f4562fe736c28dc6500ad95b3))


### Miscellaneous

* Address warnings in noir test suite ([#4966](https://github.com/AztecProtocol/aztec-packages/issues/4966)) ([7ef4ef5](https://github.com/AztecProtocol/aztec-packages/commit/7ef4ef59d1188e3d370503bd69f4750fcf7d14b7))
* Bootstrap noir natively if nargo is invalid ([#5034](https://github.com/AztecProtocol/aztec-packages/issues/5034)) ([df089de](https://github.com/AztecProtocol/aztec-packages/commit/df089def1e562539ff8ce1f6ed6360256da7a067))
* Build avm transpiler if we are on mac ([#5039](https://github.com/AztecProtocol/aztec-packages/issues/5039)) ([c2966b9](https://github.com/AztecProtocol/aztec-packages/commit/c2966b977c314eb53f913fe43d5ca46a112a126d))
* **ci:** Re-enable certain bb solidity ACIR tests ([#5065](https://github.com/AztecProtocol/aztec-packages/issues/5065)) ([58e1ff4](https://github.com/AztecProtocol/aztec-packages/commit/58e1ff4ecf8dbc5e4504994a9e22b04d09d0535d))
* Cleanup of prover and verifier instances ([#4959](https://github.com/AztecProtocol/aztec-packages/issues/4959)) ([f2fdefd](https://github.com/AztecProtocol/aztec-packages/commit/f2fdefd1a7b4759abc767f273e5defa5bf7ddcc7))
* Delete bootstrap scripts from `noir/noir-repo` ([#5044](https://github.com/AztecProtocol/aztec-packages/issues/5044)) ([add91ca](https://github.com/AztecProtocol/aztec-packages/commit/add91caf4ff395bd5f1bd0d7609dfccb5858bba8))
* Disable `hello_world_example` noir test in aztec-packages CI ([#5061](https://github.com/AztecProtocol/aztec-packages/issues/5061)) ([1be9243](https://github.com/AztecProtocol/aztec-packages/commit/1be9243f17996429b4282413ef7db45b6229b537))
* Join-split example Part 1 ([#4965](https://github.com/AztecProtocol/aztec-packages/issues/4965)) ([b9de0f5](https://github.com/AztecProtocol/aztec-packages/commit/b9de0f52e89c05f2260afeae0ccc6c3ff63e69b6))
* Moving RootRollupInputs impl ([#5087](https://github.com/AztecProtocol/aztec-packages/issues/5087)) ([f3d9f9b](https://github.com/AztecProtocol/aztec-packages/commit/f3d9f9b53bf72190eba4e9bd66b575663cfdd993))
* Remove eccvm functionality to update the op queue and ensure ultra ops are populated through function ([#5084](https://github.com/AztecProtocol/aztec-packages/issues/5084)) ([77954ab](https://github.com/AztecProtocol/aztec-packages/commit/77954ab56de67e0e055f222d04dbeb353aa3c04b))


### Documentation

* Parity circuit naming fixes ([#5076](https://github.com/AztecProtocol/aztec-packages/issues/5076)) ([c255255](https://github.com/AztecProtocol/aztec-packages/commit/c2552552736a2ba1e9a91bb6a7c47a47c16c19b3))

## [0.26.5](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.4...aztec-packages-v0.26.5) (2024-03-07)


### Features

* Crowdfunding contract ([#4917](https://github.com/AztecProtocol/aztec-packages/issues/4917)) ([ba3aff2](https://github.com/AztecProtocol/aztec-packages/commit/ba3aff2d32f88218543082ab91060ae549f1666a))
* Integrated native ACVM ([#4903](https://github.com/AztecProtocol/aztec-packages/issues/4903)) ([3fd7025](https://github.com/AztecProtocol/aztec-packages/commit/3fd7025ab43e705cab4aa67ca057e54316a1715b))


### Bug Fixes

* Dependency for yarn-project-tests ([#5031](https://github.com/AztecProtocol/aztec-packages/issues/5031)) ([4b5db50](https://github.com/AztecProtocol/aztec-packages/commit/4b5db50df68380e787cf499efac78835f927bea6))
* **docs:** Update writing_token_contract.md ([#5020](https://github.com/AztecProtocol/aztec-packages/issues/5020)) ([5b0f38f](https://github.com/AztecProtocol/aztec-packages/commit/5b0f38f5e3b7f5c6ad950572341859d12bbf46bc))
* End to end dependency fix ([#5029](https://github.com/AztecProtocol/aztec-packages/issues/5029)) ([191ad93](https://github.com/AztecProtocol/aztec-packages/commit/191ad9314263aae7fde75e510a6b866631d5d3de))
* Missing dependency end-to-end =&gt; yarn-project ([#5018](https://github.com/AztecProtocol/aztec-packages/issues/5018)) ([f930bdd](https://github.com/AztecProtocol/aztec-packages/commit/f930bdd49bfdf77eed166634e07ef49c93ffce07))
* **revert:** "feat(avm): storage" ([#5019](https://github.com/AztecProtocol/aztec-packages/issues/5019)) ([ba31016](https://github.com/AztecProtocol/aztec-packages/commit/ba3101610217ec1ac9976fed0962790b319cb01c))


### Miscellaneous

* **boxes:** Refactor npx to improve readability, added upgrade option and manual versioning ([#4855](https://github.com/AztecProtocol/aztec-packages/issues/4855)) ([ef76d3f](https://github.com/AztecProtocol/aztec-packages/commit/ef76d3f37dfc338bda1742baf006129ff9b3ed74))
* Purging calldata hash ([#4984](https://github.com/AztecProtocol/aztec-packages/issues/4984)) ([f6f34b7](https://github.com/AztecProtocol/aztec-packages/commit/f6f34b7cebc757aa7974cd2c947815132ec703d6))


### Documentation

* Add versions section to updating doc ([#4916](https://github.com/AztecProtocol/aztec-packages/issues/4916)) ([d4d935f](https://github.com/AztecProtocol/aztec-packages/commit/d4d935f05ae7026420aca4550a2b80e196028299))

## [0.26.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.3...aztec-packages-v0.26.4) (2024-03-06)


### Features

* **avm:** ALU &lt;--&gt; MAIN inter table relation on intermediate registers copy ([#4945](https://github.com/AztecProtocol/aztec-packages/issues/4945)) ([8708131](https://github.com/AztecProtocol/aztec-packages/commit/870813173e0fc760338a06485722387fdd1dfcab)), closes [#4613](https://github.com/AztecProtocol/aztec-packages/issues/4613)
* Circuit checker class ([#4931](https://github.com/AztecProtocol/aztec-packages/issues/4931)) ([4eba266](https://github.com/AztecProtocol/aztec-packages/commit/4eba26675a39cf6c9539da57c7177ec28ee3a8fb))
* Compute out hash in circuits [#4561](https://github.com/AztecProtocol/aztec-packages/issues/4561) ([#4873](https://github.com/AztecProtocol/aztec-packages/issues/4873)) ([06a9116](https://github.com/AztecProtocol/aztec-packages/commit/06a9116959a6a193a605aebe2fc4e33751e3ef1a))


### Bug Fixes

* **ci:** Noir mirror base commit ([#4969](https://github.com/AztecProtocol/aztec-packages/issues/4969)) ([546c666](https://github.com/AztecProtocol/aztec-packages/commit/546c666c62f495d258fe44d164a3bc184a8e5fed))
* Fix release ([#4994](https://github.com/AztecProtocol/aztec-packages/issues/4994)) ([19a8728](https://github.com/AztecProtocol/aztec-packages/commit/19a872843b3eea1991fc76afab5f6d50fbe4a492))


### Miscellaneous

* Use public constructors where possible ([#4937](https://github.com/AztecProtocol/aztec-packages/issues/4937)) ([225aad6](https://github.com/AztecProtocol/aztec-packages/commit/225aad683ec940eaa06509b5a149797a179c865e))

## [0.26.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.2...aztec-packages-v0.26.3) (2024-03-06)


### Features

* New Inbox ([#4880](https://github.com/AztecProtocol/aztec-packages/issues/4880)) ([c5e8014](https://github.com/AztecProtocol/aztec-packages/commit/c5e80142ddb2c928639af02d59b2b9e9cc8b0a9b)), closes [#4825](https://github.com/AztecProtocol/aztec-packages/issues/4825)


### Bug Fixes

* Remove l1 contracts publishing ([#4985](https://github.com/AztecProtocol/aztec-packages/issues/4985)) ([fb6552c](https://github.com/AztecProtocol/aztec-packages/commit/fb6552c945bc25e4599a0500167a04a5e0177708))


### Miscellaneous

* Update bootstrap instructions in the readme ([#4968](https://github.com/AztecProtocol/aztec-packages/issues/4968)) ([959158b](https://github.com/AztecProtocol/aztec-packages/commit/959158b9ed5ffe367c5d0253a87aec734fd64128))

## [0.26.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.1...aztec-packages-v0.26.2) (2024-03-06)


### Bug Fixes

* Pw/disable generate config ([#4976](https://github.com/AztecProtocol/aztec-packages/issues/4976)) ([d7549fe](https://github.com/AztecProtocol/aztec-packages/commit/d7549fe27558678956d57c4f525d719922946fa9))

## [0.26.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.26.0...aztec-packages-v0.26.1) (2024-03-06)


### Features

* Adding fr compatibility to smt variables api ([#4884](https://github.com/AztecProtocol/aztec-packages/issues/4884)) ([c085cbb](https://github.com/AztecProtocol/aztec-packages/commit/c085cbb0840b29698db1fec0ed5d6aa19c9c36ea))
* **avm-simulator:** Implement EMITUNENCRYPTEDLOG ([#4926](https://github.com/AztecProtocol/aztec-packages/issues/4926)) ([5f3304e](https://github.com/AztecProtocol/aztec-packages/commit/5f3304ea834f03342a97c9839e3b2f850bf2919d))
* Choose constructor method in Contract.deploy ([#4939](https://github.com/AztecProtocol/aztec-packages/issues/4939)) ([e899e56](https://github.com/AztecProtocol/aztec-packages/commit/e899e56ed2423557d264d835f09820e89a8a4697))
* Indirect mem flag deserialisation ([#4877](https://github.com/AztecProtocol/aztec-packages/issues/4877)) ([4c6820f](https://github.com/AztecProtocol/aztec-packages/commit/4c6820f6359a2db4863502d36b188dd52d2d32b1))


### Miscellaneous

* Add missing jobs to CI end ([#4963](https://github.com/AztecProtocol/aztec-packages/issues/4963)) ([ff4110e](https://github.com/AztecProtocol/aztec-packages/commit/ff4110e684e3b229ecf1da7e63d7094f43f1d850))
* **avm-simulator:** Better type env getters ([#4950](https://github.com/AztecProtocol/aztec-packages/issues/4950)) ([8f97977](https://github.com/AztecProtocol/aztec-packages/commit/8f979779499e7dc39f9de8caaa65269abe6fa3bb))
* **avm-simulator:** Revive field comparison ([#4957](https://github.com/AztecProtocol/aztec-packages/issues/4957)) ([ee21374](https://github.com/AztecProtocol/aztec-packages/commit/ee2137457a17b7f51699c870751c4ad68d195819))
* **avm-simulator:** Test improvements ([#4946](https://github.com/AztecProtocol/aztec-packages/issues/4946)) ([f74e6a1](https://github.com/AztecProtocol/aztec-packages/commit/f74e6a1f58869e327677958245edfec8cf0bc130))
* Fix CCI config ([#4974](https://github.com/AztecProtocol/aztec-packages/issues/4974)) ([40178f0](https://github.com/AztecProtocol/aztec-packages/commit/40178f0a77c727e67e4a9257895f88471954554b))
* Remove commitment key copy out of instance ([#4893](https://github.com/AztecProtocol/aztec-packages/issues/4893)) ([6eb6778](https://github.com/AztecProtocol/aztec-packages/commit/6eb6778c2f4586e97a659e3368aa25016f97d3b9))
* **vscode:** Add avm-transpiler to vscode rust-analyzer settings ([#4952](https://github.com/AztecProtocol/aztec-packages/issues/4952)) ([db915e5](https://github.com/AztecProtocol/aztec-packages/commit/db915e50011b26d641175c22276ac6472379e8de))

## [0.26.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.25.0...aztec-packages-v0.26.0) (2024-03-05)


### ⚠ BREAKING CHANGES

* Internal as a macro ([#4898](https://github.com/AztecProtocol/aztec-packages/issues/4898))

### Features

* Add init check by default to public fns ([#4897](https://github.com/AztecProtocol/aztec-packages/issues/4897)) ([4550f25](https://github.com/AztecProtocol/aztec-packages/commit/4550f2596b51a985d6677191fd83bb2e621c5bc3))
* Enable public constructor functions ([#4896](https://github.com/AztecProtocol/aztec-packages/issues/4896)) ([7b06895](https://github.com/AztecProtocol/aztec-packages/commit/7b068957b41069a2ed8fd0f64ba1b95eb0299ee0))
* Internal as a macro ([#4898](https://github.com/AztecProtocol/aztec-packages/issues/4898)) ([73d640a](https://github.com/AztecProtocol/aztec-packages/commit/73d640a4a033f0c865d45da470ef40c1fb03a844))
* We no longer update version packages via scripts ([#4962](https://github.com/AztecProtocol/aztec-packages/issues/4962)) ([31d470b](https://github.com/AztecProtocol/aztec-packages/commit/31d470b5940408feb8beceacff35f0207f6d5588))


### Miscellaneous

* Disable failing test temporarily ([ec61974](https://github.com/AztecProtocol/aztec-packages/commit/ec6197407a924ea6f0133122cdfc49a064804b72))
* Fixed call nesting, tests and docs ([#4932](https://github.com/AztecProtocol/aztec-packages/issues/4932)) ([bd5c879](https://github.com/AztecProtocol/aztec-packages/commit/bd5c8793c91214ee2e85ded245e336953fe7abdf))
* Specify packages individually for release-please ([#4960](https://github.com/AztecProtocol/aztec-packages/issues/4960)) ([dddc35f](https://github.com/AztecProtocol/aztec-packages/commit/dddc35f30711a27d1bbcc3414ea53b33880e390f))
* Sync noir repo ([#4947](https://github.com/AztecProtocol/aztec-packages/issues/4947)) ([7ff9b71](https://github.com/AztecProtocol/aztec-packages/commit/7ff9b71d8d87fc93ae7dbd8ba63f5176b0cd17be))
* Unused vars cleanup + updated TODOs ([#4883](https://github.com/AztecProtocol/aztec-packages/issues/4883)) ([3747619](https://github.com/AztecProtocol/aztec-packages/commit/374761962fdc3711b6169dcceaba81add04b7082))
* Update escrow to use PrivateImmutable ([#4942](https://github.com/AztecProtocol/aztec-packages/issues/4942)) ([245d801](https://github.com/AztecProtocol/aztec-packages/commit/245d801240998d945e6d5d3371f32eb2b31b66e5))

## [0.25.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.24.0...aztec-packages-v0.25.0) (2024-03-05)


### ⚠ BREAKING CHANGES

* nullifier read requests in private ([#4764](https://github.com/AztecProtocol/aztec-packages/issues/4764))
* Use new deployment flow in ContractDeployer ([#4497](https://github.com/AztecProtocol/aztec-packages/issues/4497))
* renamings of state var wrappers ([#4739](https://github.com/AztecProtocol/aztec-packages/issues/4739))
* l1 to l2 message api takes sender as arg ([#4648](https://github.com/AztecProtocol/aztec-packages/issues/4648))
* autogenerate compute_note_hash_and_nullifier ([#4610](https://github.com/AztecProtocol/aztec-packages/issues/4610))

### Features

* Add aztec-nr private functions for initialization nullifier ([#4807](https://github.com/AztecProtocol/aztec-packages/issues/4807)) ([4feaea5](https://github.com/AztecProtocol/aztec-packages/commit/4feaea59267437a0841aa14f445cee7556a0c0b4))
* Add deploy contract helper to aztec-nr ([#4775](https://github.com/AztecProtocol/aztec-packages/issues/4775)) ([6018fc6](https://github.com/AztecProtocol/aztec-packages/commit/6018fc66adfe76afbade0ffde3f1c83e97eba9c0))
* Add tagged note structure  ([#4843](https://github.com/AztecProtocol/aztec-packages/issues/4843)) ([553c2c6](https://github.com/AztecProtocol/aztec-packages/commit/553c2c602702d683c455928fc386f3b554f536ef)), closes [#4572](https://github.com/AztecProtocol/aztec-packages/issues/4572)
* Additional op count timing ([#4722](https://github.com/AztecProtocol/aztec-packages/issues/4722)) ([f0cc760](https://github.com/AztecProtocol/aztec-packages/commit/f0cc76040a2de5d0f827afdb662591232c4ee1ed))
* Allow nullifier proofs in public ([#4892](https://github.com/AztecProtocol/aztec-packages/issues/4892)) ([f7a7243](https://github.com/AztecProtocol/aztec-packages/commit/f7a72436bb12e30d8a85c8cf9b3a460d5b380252))
* Analyze % of time spent on field arithmetic ([#4501](https://github.com/AztecProtocol/aztec-packages/issues/4501)) ([5ddfa16](https://github.com/AztecProtocol/aztec-packages/commit/5ddfa16391f1017219a997c322b061ebe6f34db2))
* AUTHWIT cancellations ([#4799](https://github.com/AztecProtocol/aztec-packages/issues/4799)) ([b7c2bc0](https://github.com/AztecProtocol/aztec-packages/commit/b7c2bc0e70faebb60e2051e0330e94937a1e3711))
* AUTHWIT generator ([#4798](https://github.com/AztecProtocol/aztec-packages/issues/4798)) ([efd70f4](https://github.com/AztecProtocol/aztec-packages/commit/efd70f4b8bb284815c5345bd16d79018ed2dd812))
* Autogenerate compute_note_hash_and_nullifier ([#4610](https://github.com/AztecProtocol/aztec-packages/issues/4610)) ([286e708](https://github.com/AztecProtocol/aztec-packages/commit/286e708c1016d60278060bb01f5d997f9a0bdfba))
* **avm-simulator:** Add NULLIFIEREXISTS opcode to avm simulator, transpiler, noir test, TS tests ([#4747](https://github.com/AztecProtocol/aztec-packages/issues/4747)) ([707f572](https://github.com/AztecProtocol/aztec-packages/commit/707f572ae8802a9c92b9fe8cff2ec16dfea00b9d))
* **avm-simulator:** Create cache for pending nullifiers and existence checks ([#4743](https://github.com/AztecProtocol/aztec-packages/issues/4743)) ([0f80579](https://github.com/AztecProtocol/aztec-packages/commit/0f80579823aa2de1271c8cdccc72e5f5ee935939))
* **avm-simulator:** Implement AVM message opcodes (simulator/transpiler/noir-test) ([#4852](https://github.com/AztecProtocol/aztec-packages/issues/4852)) ([c98325d](https://github.com/AztecProtocol/aztec-packages/commit/c98325d23897d23c09faddc4355958406d44faa9))
* **avm-simulator:** Implement NOTEHASHEXISTS ([#4882](https://github.com/AztecProtocol/aztec-packages/issues/4882)) ([d8c770b](https://github.com/AztecProtocol/aztec-packages/commit/d8c770bbf9e208adb31c6b0ea41e08f7c4f8818c))
* **avm-transpiler:** Add emitnotehash and emitnullifier opcodes to avm transpiler and simulator tests ([#4746](https://github.com/AztecProtocol/aztec-packages/issues/4746)) ([d44d9f1](https://github.com/AztecProtocol/aztec-packages/commit/d44d9f11be2a2d2652b70b1d333322440c6ef06c))
* **avm:** Enable main -&gt; mem clk lookups  ([#4591](https://github.com/AztecProtocol/aztec-packages/issues/4591)) ([0e503c1](https://github.com/AztecProtocol/aztec-packages/commit/0e503c14c0c20a93e162a90d8d049f094b64de7d))
* **avm:** Hashing opcodes ([#4526](https://github.com/AztecProtocol/aztec-packages/issues/4526)) ([fe10c70](https://github.com/AztecProtocol/aztec-packages/commit/fe10c7049b3597a96f76a27a22e9233bc3b8ce82))
* **avm:** Hashing to simulator ([#4527](https://github.com/AztecProtocol/aztec-packages/issues/4527)) ([9f67eec](https://github.com/AztecProtocol/aztec-packages/commit/9f67eec73c5d639df16e6b3bf45c4a1fc1c54bad))
* **avm:** Propagate tag err to the main trace for op_return and internal_return ([#4615](https://github.com/AztecProtocol/aztec-packages/issues/4615)) ([427f1d8](https://github.com/AztecProtocol/aztec-packages/commit/427f1d8567a3f68c3093c29a2999096746927548)), closes [#4598](https://github.com/AztecProtocol/aztec-packages/issues/4598)
* Avoid requiring arith gates in sequence ([#4869](https://github.com/AztecProtocol/aztec-packages/issues/4869)) ([0ab0a94](https://github.com/AztecProtocol/aztec-packages/commit/0ab0a94842ce9b174ba82b430a93cba188fe75b0))
* **bb:** Working msan preset ([#4618](https://github.com/AztecProtocol/aztec-packages/issues/4618)) ([0195ac8](https://github.com/AztecProtocol/aztec-packages/commit/0195ac89a13dc2a7b9caa5a8d8d29458a99c5f76))
* Benchmark Protogalaxy rounds ([#4316](https://github.com/AztecProtocol/aztec-packages/issues/4316)) ([91af28d](https://github.com/AztecProtocol/aztec-packages/commit/91af28d6e03d85b5c749740c82cf9114379c823a))
* Bitwise_not avm circuit ([#4548](https://github.com/AztecProtocol/aztec-packages/issues/4548)) ([3a7d31b](https://github.com/AztecProtocol/aztec-packages/commit/3a7d31b200e6e604eea06a40dcf5bf02b088ab79))
* Boxes refactor pt2 ([#4612](https://github.com/AztecProtocol/aztec-packages/issues/4612)) ([aad45b3](https://github.com/AztecProtocol/aztec-packages/commit/aad45b3bc2be50dc7223ccc3faf1c336613dffea))
* Boxes update ([#4498](https://github.com/AztecProtocol/aztec-packages/issues/4498)) ([382626c](https://github.com/AztecProtocol/aztec-packages/commit/382626cddaa175041695e2eb70ad3c350351ffe3))
* Check initializer by default in private functions ([#4832](https://github.com/AztecProtocol/aztec-packages/issues/4832)) ([3ff9fe0](https://github.com/AztecProtocol/aztec-packages/commit/3ff9fe0ad9591caebc313acecd3a2144f8434ae2))
* Define Aztec prelude ([#4929](https://github.com/AztecProtocol/aztec-packages/issues/4929)) ([8ffe5df](https://github.com/AztecProtocol/aztec-packages/commit/8ffe5df71b78ed5100f598f680fbb1fe49b546b3))
* Delegate calls ([#4586](https://github.com/AztecProtocol/aztec-packages/issues/4586)) ([e6d65a7](https://github.com/AztecProtocol/aztec-packages/commit/e6d65a7fe9ebe855dcac389775aae2ccc3fa311f))
* **devops:** Filter circleci config no-ops ([#4731](https://github.com/AztecProtocol/aztec-packages/issues/4731)) ([41984b4](https://github.com/AztecProtocol/aztec-packages/commit/41984b4e43fd3fd42522552ecb8ca1e54f32cdf1))
* **docs:** Autogenerated Aztec-nr reference docs ([#3481](https://github.com/AztecProtocol/aztec-packages/issues/3481)) ([aebf762](https://github.com/AztecProtocol/aztec-packages/commit/aebf762d37dee9985740f3bf2578a0cf69818050))
* **docs:** Docs meta doc ([#4767](https://github.com/AztecProtocol/aztec-packages/issues/4767)) ([0a645d3](https://github.com/AztecProtocol/aztec-packages/commit/0a645d3a5d3029501ccbba5e030146f7397301b0))
* **docs:** Meta doc typo fixes ([#4779](https://github.com/AztecProtocol/aztec-packages/issues/4779)) ([44df132](https://github.com/AztecProtocol/aztec-packages/commit/44df1327fb7018187bf15a4ae4c76218160a2914))
* **docs:** Note type IDs and compute_note_hash_and_nullifier page ([#4636](https://github.com/AztecProtocol/aztec-packages/issues/4636)) ([032874a](https://github.com/AztecProtocol/aztec-packages/commit/032874a031ce9a5dde7da20864fbd456061adc43))
* Equality avm circuit ([#4595](https://github.com/AztecProtocol/aztec-packages/issues/4595)) ([aad7b45](https://github.com/AztecProtocol/aztec-packages/commit/aad7b45aa6d3a4c3df259ea41fdde48bf01139b1))
* Execution Trace ([#4623](https://github.com/AztecProtocol/aztec-packages/issues/4623)) ([07ac589](https://github.com/AztecProtocol/aztec-packages/commit/07ac589d08964a44ea54a0d9fa0a21db73186aee))
* Gate blocks ([#4741](https://github.com/AztecProtocol/aztec-packages/issues/4741)) ([61067a5](https://github.com/AztecProtocol/aztec-packages/commit/61067a5cdedfd10fbc32e381083b031bc80fc6d6))
* Goblin documentation ([#4679](https://github.com/AztecProtocol/aztec-packages/issues/4679)) ([24d918f](https://github.com/AztecProtocol/aztec-packages/commit/24d918f7bd114f2641ae61bcf0da888e06f6520a))
* Goblin Translator Fuzzer ([#4752](https://github.com/AztecProtocol/aztec-packages/issues/4752)) ([7402517](https://github.com/AztecProtocol/aztec-packages/commit/74025170288e39e1d7516f57df94f22bc30f663c))
* GoblinUltra Bench ([#4671](https://github.com/AztecProtocol/aztec-packages/issues/4671)) ([319eea9](https://github.com/AztecProtocol/aztec-packages/commit/319eea9e4caf1d1ade00fedface5fab9bbf9db16))
* Implementing IPA optimisation ([#4363](https://github.com/AztecProtocol/aztec-packages/issues/4363)) ([13647c2](https://github.com/AztecProtocol/aztec-packages/commit/13647c24487116f971c81dfaf4ee4664870522d5))
* L1 to l2 message api takes sender as arg ([#4648](https://github.com/AztecProtocol/aztec-packages/issues/4648)) ([96f6b2a](https://github.com/AztecProtocol/aztec-packages/commit/96f6b2a6e5475d747191def24a122532eacd610d)), closes [#4559](https://github.com/AztecProtocol/aztec-packages/issues/4559)
* Login to ecr explicitly, faster bootstrap as we only do once. ([#4900](https://github.com/AztecProtocol/aztec-packages/issues/4900)) ([86d6749](https://github.com/AztecProtocol/aztec-packages/commit/86d6749615a533e0a9fbe0a1dca97b38fb14bb5f))
* Macros for initializer checks ([#4830](https://github.com/AztecProtocol/aztec-packages/issues/4830)) ([c7c24b2](https://github.com/AztecProtocol/aztec-packages/commit/c7c24b2d1e71a95d3af7a9fe9e39b439ec319e3d))
* Manual ClientIVC breakdown ([#4778](https://github.com/AztecProtocol/aztec-packages/issues/4778)) ([b4cfc89](https://github.com/AztecProtocol/aztec-packages/commit/b4cfc89c0d8286d2dfa3e04c58695d554951c920))
* Moving the unbox option to npx command ([#4718](https://github.com/AztecProtocol/aztec-packages/issues/4718)) ([4c3bb92](https://github.com/AztecProtocol/aztec-packages/commit/4c3bb9294fc10ff4663275c952e277eaa7ecd647))
* Native fee payment ([#4543](https://github.com/AztecProtocol/aztec-packages/issues/4543)) ([5d4702b](https://github.com/AztecProtocol/aztec-packages/commit/5d4702b7684393b54bef4cdca963077504b41a2a))
* Non revertible effects and tx phases ([#4629](https://github.com/AztecProtocol/aztec-packages/issues/4629)) ([c04d72f](https://github.com/AztecProtocol/aztec-packages/commit/c04d72fd363b32743cf906bfe986f82c5d5901fc))
* Nullifier read requests in private ([#4764](https://github.com/AztecProtocol/aztec-packages/issues/4764)) ([a049d1f](https://github.com/AztecProtocol/aztec-packages/commit/a049d1f571487f2cec25cb1bdeff5c177e25b91d))
* Outgoing messages to any address ([#4512](https://github.com/AztecProtocol/aztec-packages/issues/4512)) ([4d0e8d3](https://github.com/AztecProtocol/aztec-packages/commit/4d0e8d30fb604e72bd4ef62f5cf8928e0eaa2009))
* Parallel native/wasm bb builds. Better messaging around using ci cache. ([#4766](https://github.com/AztecProtocol/aztec-packages/issues/4766)) ([a924e55](https://github.com/AztecProtocol/aztec-packages/commit/a924e55393daa89fbba3a87cf019977286104b59))
* Parallelise kernel and function circuit construction in client IVC ([#4841](https://github.com/AztecProtocol/aztec-packages/issues/4841)) ([9c689d8](https://github.com/AztecProtocol/aztec-packages/commit/9c689d8d5a7d330dabafaa7d10c0cfc5e4694921))
* Public initializer check ([#4894](https://github.com/AztecProtocol/aztec-packages/issues/4894)) ([6b861bb](https://github.com/AztecProtocol/aztec-packages/commit/6b861bb06c7d0e51692953a946aba481bc78e2d1))
* Public refunds via FPC ([#4750](https://github.com/AztecProtocol/aztec-packages/issues/4750)) ([30502c9](https://github.com/AztecProtocol/aztec-packages/commit/30502c96fc2aa2a86cdad0f7edaec9cac97e6cf5))
* PublicImmutable impl ([#4758](https://github.com/AztecProtocol/aztec-packages/issues/4758)) ([87c976b](https://github.com/AztecProtocol/aztec-packages/commit/87c976bcf022300b2bd9dfa2a8c98f8fe7e45433)), closes [#4757](https://github.com/AztecProtocol/aztec-packages/issues/4757)
* Renamings of state var wrappers ([#4739](https://github.com/AztecProtocol/aztec-packages/issues/4739)) ([4667c27](https://github.com/AztecProtocol/aztec-packages/commit/4667c27695ad203f4d8fef73e13158ceed2cef7d))
* Separate addition gate after final RAM gate ([#4851](https://github.com/AztecProtocol/aztec-packages/issues/4851)) ([f329db4](https://github.com/AztecProtocol/aztec-packages/commit/f329db4ec08f013bf8f53eb73b18d3d98d98e2e4))
* Separate arithmetic gate in sort with edges ([#4866](https://github.com/AztecProtocol/aztec-packages/issues/4866)) ([40adc5c](https://github.com/AztecProtocol/aztec-packages/commit/40adc5cdc578c6ff6d6a9aa25c9a2f3506ec1677))
* Simplify public input copy cycles ([#4753](https://github.com/AztecProtocol/aztec-packages/issues/4753)) ([a714ee0](https://github.com/AztecProtocol/aztec-packages/commit/a714ee027262dba3a083e17878862cd1144a86a6))
* Static call support in aztec.nr and acir-simulator ([#4106](https://github.com/AztecProtocol/aztec-packages/issues/4106)) ([5f9546a](https://github.com/AztecProtocol/aztec-packages/commit/5f9546a50b72e29ec032e115a79ce5ceae2f26c0))
* Update header to match message extension ([#4627](https://github.com/AztecProtocol/aztec-packages/issues/4627)) ([dc01e1d](https://github.com/AztecProtocol/aztec-packages/commit/dc01e1d573795f2199b6b9c6249fb1e816d5c594))
* Update RAM/ROM memory records for new block structure ([#4806](https://github.com/AztecProtocol/aztec-packages/issues/4806)) ([65e4ab9](https://github.com/AztecProtocol/aztec-packages/commit/65e4ab93219118c8ac46a68bc6607ee9d11f6478))
* Use new deployment flow in ContractDeployer ([#4497](https://github.com/AztecProtocol/aztec-packages/issues/4497)) ([0702dc6](https://github.com/AztecProtocol/aztec-packages/commit/0702dc6988149258124184b85d38db930effe0e7))
* Use yarns topological build to get rid of explicit sequential steps, and let it solve. ([#4868](https://github.com/AztecProtocol/aztec-packages/issues/4868)) ([c909966](https://github.com/AztecProtocol/aztec-packages/commit/c909966ad6d0f1621d066f5861d38a128fe9c224))
* **yp:** Add algolia search to the yellow paper ([#4771](https://github.com/AztecProtocol/aztec-packages/issues/4771)) ([48dd78e](https://github.com/AztecProtocol/aztec-packages/commit/48dd78e06a2dc9452bea1a3156721ffd68e046a4))


### Bug Fixes

* Add new oracle contract to devnet in CI ([#4687](https://github.com/AztecProtocol/aztec-packages/issues/4687)) ([920fa10](https://github.com/AztecProtocol/aztec-packages/commit/920fa10d4d5fb476cd6d868439310452f6e8dcc5))
* Add registry contract to list ([#4694](https://github.com/AztecProtocol/aztec-packages/issues/4694)) ([3675e1d](https://github.com/AztecProtocol/aztec-packages/commit/3675e1d110eccf45986bbbcf35e29746474bb7aa))
* Add TODO with issue for num_gates bug ([#4847](https://github.com/AztecProtocol/aztec-packages/issues/4847)) ([f6c558b](https://github.com/AztecProtocol/aztec-packages/commit/f6c558b41d3e003e1626a853aff0b58705847e84))
* After noir move ([#4564](https://github.com/AztecProtocol/aztec-packages/issues/4564)) ([5f5bf16](https://github.com/AztecProtocol/aztec-packages/commit/5f5bf1604ce16a9d7c9f121ed79f9d287358510c))
* Align block structs w/ yp [#3868](https://github.com/AztecProtocol/aztec-packages/issues/3868) ([#4541](https://github.com/AztecProtocol/aztec-packages/issues/4541)) ([081da3c](https://github.com/AztecProtocol/aztec-packages/commit/081da3cb0b9e83f817a82314bb4be116e32e054c))
* Assembly benching ([#4640](https://github.com/AztecProtocol/aztec-packages/issues/4640)) ([f144745](https://github.com/AztecProtocol/aztec-packages/commit/f14474571210a46e7159cb9d2f0bc9374a837d3d))
* AZTEC_PORT variable for devnet ([#4700](https://github.com/AztecProtocol/aztec-packages/issues/4700)) ([097a888](https://github.com/AztecProtocol/aztec-packages/commit/097a888b1f60d285595dbae6ebac5af32f9ace67))
* Aztec-node terraform args ([#4669](https://github.com/AztecProtocol/aztec-packages/issues/4669)) ([4f37270](https://github.com/AztecProtocol/aztec-packages/commit/4f372703bcd2a13a7949cc3370356d0b376746ef))
* **bb:** Initialize element::infinity() ([#4664](https://github.com/AztecProtocol/aztec-packages/issues/4664)) ([6813540](https://github.com/AztecProtocol/aztec-packages/commit/6813540731149db1f0d8932598335f95937ada03))
* Boost the size of the non-revertible reads/writes ([#4688](https://github.com/AztecProtocol/aztec-packages/issues/4688)) ([9cb6daf](https://github.com/AztecProtocol/aztec-packages/commit/9cb6daff6330a5675a070334cc88773d6e0bae3a))
* **build-system:** Login to dockerhub ([#4716](https://github.com/AztecProtocol/aztec-packages/issues/4716)) ([5eb0c57](https://github.com/AztecProtocol/aztec-packages/commit/5eb0c577f34df5f111d17ec25000fc03d09d5497))
* Change function limit to private function limit ([#4785](https://github.com/AztecProtocol/aztec-packages/issues/4785)) ([2799f1f](https://github.com/AztecProtocol/aztec-packages/commit/2799f1fe1718fadd4bc0705449a8b4c79bc391b6))
* Ci merge check ([#4921](https://github.com/AztecProtocol/aztec-packages/issues/4921)) ([46063da](https://github.com/AztecProtocol/aztec-packages/commit/46063da1b42f109e8b0c5c4b1a07c15401899b30))
* **ci:** Bump puppeteer to fix yarn-project-base ([#4721](https://github.com/AztecProtocol/aztec-packages/issues/4721)) ([89af734](https://github.com/AztecProtocol/aztec-packages/commit/89af73421a83dfc79743e3e0287b246326d71b7d))
* Cpp build ([#4918](https://github.com/AztecProtocol/aztec-packages/issues/4918)) ([15df3c0](https://github.com/AztecProtocol/aztec-packages/commit/15df3c08168611f7f65f5837a937031d81bb3566))
* Dapp sub test ([#4938](https://github.com/AztecProtocol/aztec-packages/issues/4938)) ([827afd1](https://github.com/AztecProtocol/aztec-packages/commit/827afd10edfca8b2c8273742717f039981543194))
* Debug build ([#4666](https://github.com/AztecProtocol/aztec-packages/issues/4666)) ([acc27b1](https://github.com/AztecProtocol/aztec-packages/commit/acc27b1bd2ec21c7b5c71f02974bd49d29b4caa5))
* Depreciated ci image ([#4911](https://github.com/AztecProtocol/aztec-packages/issues/4911)) ([174fc10](https://github.com/AztecProtocol/aztec-packages/commit/174fc104d68e94b33d4d455f24e38b73a64b534a))
* **docs:** Update 0.22 migration_notes.md w/ proper note interface ([#4701](https://github.com/AztecProtocol/aztec-packages/issues/4701)) ([a972dc8](https://github.com/AztecProtocol/aztec-packages/commit/a972dc8b0d62ba8e3fbbb9aed7f523ebd2b06f59))
* **docs:** Update unconstrained function call image ([#4834](https://github.com/AztecProtocol/aztec-packages/issues/4834)) ([b0bc772](https://github.com/AztecProtocol/aztec-packages/commit/b0bc772017fd36671ce9250f52d6cc64b22f7386))
* **dsl:** Add full recursive verification test ([#4658](https://github.com/AztecProtocol/aztec-packages/issues/4658)) ([9e09772](https://github.com/AztecProtocol/aztec-packages/commit/9e0977261aea723d6ea68750788f29a40730c404))
* Expose port when running aztec img ([#4719](https://github.com/AztecProtocol/aztec-packages/issues/4719)) ([df40b15](https://github.com/AztecProtocol/aztec-packages/commit/df40b15524cee9799c5193c6adf2ad7a5ea92faf))
* Fetch Headers and Bodies separately [#4167](https://github.com/AztecProtocol/aztec-packages/issues/4167) ([#4632](https://github.com/AztecProtocol/aztec-packages/issues/4632)) ([0681b3a](https://github.com/AztecProtocol/aztec-packages/commit/0681b3a6fe99667cdaa6cb3954accf15795c42ea))
* Fix races in slab allocator and lookup tables and add prepending for op_queues ([#4754](https://github.com/AztecProtocol/aztec-packages/issues/4754)) ([0c99de7](https://github.com/AztecProtocol/aztec-packages/commit/0c99de7c4b9931989824f66dab83cc644578a75c))
* Fix Translator composer test instability ([#4751](https://github.com/AztecProtocol/aztec-packages/issues/4751)) ([842ba7a](https://github.com/AztecProtocol/aztec-packages/commit/842ba7a720d075632ad2c4b948f874a12cfa3ecd))
* G2.Serialize sporadic failure ([#4626](https://github.com/AztecProtocol/aztec-packages/issues/4626)) ([c9e6bb1](https://github.com/AztecProtocol/aztec-packages/commit/c9e6bb1391070b6551b313b85fe73742ff0966fc))
* Get_wires for ultra ([#4605](https://github.com/AztecProtocol/aztec-packages/issues/4605)) ([512110e](https://github.com/AztecProtocol/aztec-packages/commit/512110e4bdc353b01ee92fb5b2ff5f6e6f875fbb))
* Initializer checks across txs ([#4842](https://github.com/AztecProtocol/aztec-packages/issues/4842)) ([747fc33](https://github.com/AztecProtocol/aztec-packages/commit/747fc33590f9fe25ffcd3e538d7db49bfb98fae8))
* Issue if commitment hints when the same commitment appears twice within the same tx ([#4702](https://github.com/AztecProtocol/aztec-packages/issues/4702)) ([9c3c880](https://github.com/AztecProtocol/aztec-packages/commit/9c3c88015965554dfdb6568bc239214cbbe85002))
* L1 contract address config ([#4684](https://github.com/AztecProtocol/aztec-packages/issues/4684)) ([20e7605](https://github.com/AztecProtocol/aztec-packages/commit/20e76058e3de7d0d30d6c951fa74d6dd08a68d2c))
* Master borked arithmetic tests ([#4606](https://github.com/AztecProtocol/aztec-packages/issues/4606)) ([472c54a](https://github.com/AztecProtocol/aztec-packages/commit/472c54a7e89001f5f752da670cc25ec1a537da87))
* More robust noir sync ([#4734](https://github.com/AztecProtocol/aztec-packages/issues/4734)) ([f53946d](https://github.com/AztecProtocol/aztec-packages/commit/f53946df78d09e7634eb839d068c559fffa0e751))
* Msan build ([#4646](https://github.com/AztecProtocol/aztec-packages/issues/4646)) ([886cc75](https://github.com/AztecProtocol/aztec-packages/commit/886cc7585f935f4f12257444af7862b51dc91584))
* MSAN msgpack noise ([#4677](https://github.com/AztecProtocol/aztec-packages/issues/4677)) ([1abae28](https://github.com/AztecProtocol/aztec-packages/commit/1abae28580354f5ccc620dbd717bf079f39fb445))
* Noir test incorrect reporting ([#4925](https://github.com/AztecProtocol/aztec-packages/issues/4925)) ([d98db3a](https://github.com/AztecProtocol/aztec-packages/commit/d98db3aa7cbfdaf5f698d4f4f0eaf4a788a02199))
* P2p-bootstrap ECS command + /status route ([#4682](https://github.com/AztecProtocol/aztec-packages/issues/4682)) ([21ec23d](https://github.com/AztecProtocol/aztec-packages/commit/21ec23d54fa69c3515f0d9fa23cc7ea1168d7e6e))
* PXE devnet connectivity ([#4759](https://github.com/AztecProtocol/aztec-packages/issues/4759)) ([c2027e3](https://github.com/AztecProtocol/aztec-packages/commit/c2027e3f58279fc9fa7c8e5c1b7fdcf832555d90))
* Rebuilding on snap updates ([#4729](https://github.com/AztecProtocol/aztec-packages/issues/4729)) ([a2c0cae](https://github.com/AztecProtocol/aztec-packages/commit/a2c0caed4c48ce2d37d2370040ea059d80d93bfe)), closes [#4728](https://github.com/AztecProtocol/aztec-packages/issues/4728)
* Remove the `VerificationKey` from `ProverInstance` ([#4908](https://github.com/AztecProtocol/aztec-packages/issues/4908)) ([8619c08](https://github.com/AztecProtocol/aztec-packages/commit/8619c084cdfd061f284058b00a96f16fbbca65bf))
* Revert boxes update ([#4602](https://github.com/AztecProtocol/aztec-packages/issues/4602)) ([f5592b8](https://github.com/AztecProtocol/aztec-packages/commit/f5592b82cab37072f0a1140b77e15cfa68220d74))
* Temporarily skip failing deployment test ([e6ce08f](https://github.com/AztecProtocol/aztec-packages/commit/e6ce08f6d74db76a45e5dea69d5b7531ca99c769))
* Use size hint for ivc circuits ([#4802](https://github.com/AztecProtocol/aztec-packages/issues/4802)) ([035cff4](https://github.com/AztecProtocol/aztec-packages/commit/035cff451ca2171e08279b9d36b23f38b840efea))
* Use specific slither and slitherin versions ([#4621](https://github.com/AztecProtocol/aztec-packages/issues/4621)) ([9e7a451](https://github.com/AztecProtocol/aztec-packages/commit/9e7a4519ae6d5ded8b7369abf50eb2c46948abe7))
* **yp:** Update search API key ([#4800](https://github.com/AztecProtocol/aztec-packages/issues/4800)) ([1cb6396](https://github.com/AztecProtocol/aztec-packages/commit/1cb639631dab59b8a301f1e256d2f76bd52addd2))


### Miscellaneous

* 1 struct per file ([#4693](https://github.com/AztecProtocol/aztec-packages/issues/4693)) ([19d2bbe](https://github.com/AztecProtocol/aztec-packages/commit/19d2bbea913506761e9706073d13513d5533fedb)), closes [#4410](https://github.com/AztecProtocol/aztec-packages/issues/4410)
* Add authwit to migration notes ([#4914](https://github.com/AztecProtocol/aztec-packages/issues/4914)) ([e775ead](https://github.com/AztecProtocol/aztec-packages/commit/e775ead27c975027022813902183c9eda44d64a4))
* Add comments in kernel_prover.ts related to hints ([#4713](https://github.com/AztecProtocol/aztec-packages/issues/4713)) ([68162b6](https://github.com/AztecProtocol/aztec-packages/commit/68162b6799aef91f005539a5e613240698bc2a1c))
* Add custom inspect for base types ([#4890](https://github.com/AztecProtocol/aztec-packages/issues/4890)) ([a1b3c01](https://github.com/AztecProtocol/aztec-packages/commit/a1b3c01fa088400188348b85ac1933e14bd9bdf6))
* Add pow poly bench and link optimization issues ([#4725](https://github.com/AztecProtocol/aztec-packages/issues/4725)) ([faa9586](https://github.com/AztecProtocol/aztec-packages/commit/faa9586ef702e3f150e6aa8217dcbcd63611dea2))
* Add struct for each bigint modulus ([#4422](https://github.com/AztecProtocol/aztec-packages/issues/4422)) ([a2942b7](https://github.com/AztecProtocol/aztec-packages/commit/a2942b791c55aab85e2266a0ec66ffb5a993c2a4))
* Address comments ([#4772](https://github.com/AztecProtocol/aztec-packages/issues/4772)) ([10d90ab](https://github.com/AztecProtocol/aztec-packages/commit/10d90ab3a15de66f4b8a64464fe8e15f33a0589d))
* Addressing flakiness of `e2e_public_cross_chain_messaging` ([#4853](https://github.com/AztecProtocol/aztec-packages/issues/4853)) ([99bbaee](https://github.com/AztecProtocol/aztec-packages/commit/99bbaee6282ec9d7e6d853e43653d43eb68bf408))
* **avm-simulator:** Create a dedicated component just for tracing world state accesses ([#4733](https://github.com/AztecProtocol/aztec-packages/issues/4733)) ([0af89e6](https://github.com/AztecProtocol/aztec-packages/commit/0af89e6c1ff21a6079d42fe87d57d667a42cc491))
* **avm-simulator:** Pull out public storage caching and merging from the state journal ([#4730](https://github.com/AztecProtocol/aztec-packages/issues/4730)) ([b075401](https://github.com/AztecProtocol/aztec-packages/commit/b075401e53a6dbe95c413608fc3c30bf19648103))
* **avm-simulator:** Test cleanup using `expect.objectContaining()` ([#4863](https://github.com/AztecProtocol/aztec-packages/issues/4863)) ([c4ecfdd](https://github.com/AztecProtocol/aztec-packages/commit/c4ecfddeaa09b204977d31329aec7ba00f26e2d0))
* **avm-transpiler:** Minor rust fixes ([#4889](https://github.com/AztecProtocol/aztec-packages/issues/4889)) ([46ee6a8](https://github.com/AztecProtocol/aztec-packages/commit/46ee6a88f4c8972bf7c8b60caf14030760590b96))
* **avm-transpiler:** Prefix AVM opcode oracles with avmOpcode ([#4862](https://github.com/AztecProtocol/aztec-packages/issues/4862)) ([f07beee](https://github.com/AztecProtocol/aztec-packages/commit/f07beee3c220ccce892a984b1995e6f867c6895c))
* **avm:** Nit fixes on message opcodes ([#4915](https://github.com/AztecProtocol/aztec-packages/issues/4915)) ([c48f5ce](https://github.com/AztecProtocol/aztec-packages/commit/c48f5cebf56e3a4545fcc72bb9d619b1127dc1ba))
* **avm:** Remove some leftover files related to Avm-mini (replaced by Avm) ([#4715](https://github.com/AztecProtocol/aztec-packages/issues/4715)) ([8c697ce](https://github.com/AztecProtocol/aztec-packages/commit/8c697ce187b4bb1c66f1146ebbc39567a46f35f8))
* **aztec-nr:** Clarify in comments that nullifier computation does not need to include siloed note-hash for protocol security ([#2667](https://github.com/AztecProtocol/aztec-packages/issues/2667)) ([426513e](https://github.com/AztecProtocol/aztec-packages/commit/426513e39e79579c53f6a4a16f26c8f5d9631026)), closes [#2666](https://github.com/AztecProtocol/aztec-packages/issues/2666)
* **bb:** Allow dynamic plookup tables ([#4667](https://github.com/AztecProtocol/aztec-packages/issues/4667)) ([5920012](https://github.com/AztecProtocol/aztec-packages/commit/592001255a999abb7167f885a5def7f8651d63a7))
* **bb:** More namespaces under bb ([#4348](https://github.com/AztecProtocol/aztec-packages/issues/4348)) ([00ba983](https://github.com/AztecProtocol/aztec-packages/commit/00ba9837606f33ccbc5c0c40be22b11a736b1608))
* **bb:** Small test improvements ([#4568](https://github.com/AztecProtocol/aztec-packages/issues/4568)) ([e23d048](https://github.com/AztecProtocol/aztec-packages/commit/e23d048e916fa12966fe01d1a8c0d3bfb50c2943))
* **bb:** Use RefArray where possible ([#4686](https://github.com/AztecProtocol/aztec-packages/issues/4686)) ([5b4e1a6](https://github.com/AztecProtocol/aztec-packages/commit/5b4e1a61216655cebb58863d26d418b23881dd02))
* Bootstrap improvements. ([#4711](https://github.com/AztecProtocol/aztec-packages/issues/4711)) ([1375233](https://github.com/AztecProtocol/aztec-packages/commit/13752339334be9c8cc0ae500d0e932f76d18a77d))
* **boxes:** Adding frontend test to vanilla-js box ([cd1ca2e](https://github.com/AztecProtocol/aztec-packages/commit/cd1ca2e13c3b475e28f17ad74e09b439a1133de0))
* **boxes:** Adding react frontend tests ([086e478](https://github.com/AztecProtocol/aztec-packages/commit/086e4789985d4e9b4712c0556811ab88be51e387))
* Build nargo against Ubuntu 20 for better compatability ([#4710](https://github.com/AztecProtocol/aztec-packages/issues/4710)) ([e84759f](https://github.com/AztecProtocol/aztec-packages/commit/e84759f953b789f38624021814dc634e8dc1d5b7))
* **ci:** Enforce formatting of noir rust code ([#4765](https://github.com/AztecProtocol/aztec-packages/issues/4765)) ([d9a1853](https://github.com/AztecProtocol/aztec-packages/commit/d9a1853cc0474050f40ef52b196568b711f7eb07)), closes [#4763](https://github.com/AztecProtocol/aztec-packages/issues/4763)
* **ci:** Test noir-projects in CI ([#4604](https://github.com/AztecProtocol/aztec-packages/issues/4604)) ([2ac428f](https://github.com/AztecProtocol/aztec-packages/commit/2ac428fd048aaadbdd28eb4ff7b7692a149e6468))
* ContextInterface trait for private and public contexts ([#4808](https://github.com/AztecProtocol/aztec-packages/issues/4808)) ([237f870](https://github.com/AztecProtocol/aztec-packages/commit/237f870cfa9d83eb11530b0c64d3b3e5a6b0ad8d))
* Decouple ypb ([#4749](https://github.com/AztecProtocol/aztec-packages/issues/4749)) ([f3c65ce](https://github.com/AztecProtocol/aztec-packages/commit/f3c65ce75637bd47aca849a08b567b06a69318b0))
* Deploy docs to production only on releases ([#4928](https://github.com/AztecProtocol/aztec-packages/issues/4928)) ([c9eb856](https://github.com/AztecProtocol/aztec-packages/commit/c9eb856ab7307642c77a8bd808de49585449b1d3))
* Do not download foundry during L1 contracts fast bootstrap ([#4865](https://github.com/AztecProtocol/aztec-packages/issues/4865)) ([c4357c8](https://github.com/AztecProtocol/aztec-packages/commit/c4357c8c4af5f763a81939ff4abe19b5e0e40029))
* **docs:** Getting a bot to comment on docs PRs with docs previews ([#4600](https://github.com/AztecProtocol/aztec-packages/issues/4600)) ([8307dad](https://github.com/AztecProtocol/aztec-packages/commit/8307dadd853d5091841e169c841ab6b09c223efb))
* **docs:** Passing nothing if pull request is unbounded ([#4794](https://github.com/AztecProtocol/aztec-packages/issues/4794)) ([db3f785](https://github.com/AztecProtocol/aztec-packages/commit/db3f785348f92a3255edc6ccaf59c3ecede082c6))
* **docs:** Removing boxes page, will iterate later as part of DIP ([#4698](https://github.com/AztecProtocol/aztec-packages/issues/4698)) ([5c232af](https://github.com/AztecProtocol/aztec-packages/commit/5c232af1dfbbf3872fafc88fad41f6e64bc0d341))
* **docs:** Simple e2e tests to use in docs ([#4596](https://github.com/AztecProtocol/aztec-packages/issues/4596)) ([6ec9f57](https://github.com/AztecProtocol/aztec-packages/commit/6ec9f577afe860ca2986b03a00b5ebe87d6600f4))
* **docs:** Update aztecnr-getting-started.md CLI deploy command ([#4590](https://github.com/AztecProtocol/aztec-packages/issues/4590)) ([234ae3e](https://github.com/AztecProtocol/aztec-packages/commit/234ae3e773ace4097bfe9b9be9a563886dfaaffc))
* **docs:** Update communication images ([#4744](https://github.com/AztecProtocol/aztec-packages/issues/4744)) ([8968e6e](https://github.com/AztecProtocol/aztec-packages/commit/8968e6e1709d7e257cfc264c76d9e52500ccd99f))
* **docs:** Update getting started contract tutorial ([#4588](https://github.com/AztecProtocol/aztec-packages/issues/4588)) ([f417452](https://github.com/AztecProtocol/aztec-packages/commit/f4174527657db9e0c5168c98a896a93f1214e846))
* Ecr login retry ([#4617](https://github.com/AztecProtocol/aztec-packages/issues/4617)) ([c3a784f](https://github.com/AztecProtocol/aztec-packages/commit/c3a784f7dfc7c11e4069c0a81dbc9c3303b0d3d5))
* Fix docs ([#4923](https://github.com/AztecProtocol/aztec-packages/issues/4923)) ([edfba29](https://github.com/AztecProtocol/aztec-packages/commit/edfba29efea1faa10631dd76ea4e737f8d8bad79))
* Get rid of Honk UltraComposer ([#4875](https://github.com/AztecProtocol/aztec-packages/issues/4875)) ([7e52c29](https://github.com/AztecProtocol/aztec-packages/commit/7e52c2971b91dfb0f07c178b2adb4427363acd1e))
* Implement poseidon2 opcode ([#4446](https://github.com/AztecProtocol/aztec-packages/issues/4446)) ([491a8df](https://github.com/AztecProtocol/aztec-packages/commit/491a8dfe81a33a7552686f70833f6130da944142))
* Improve noir-contracts.js codegen ([#4789](https://github.com/AztecProtocol/aztec-packages/issues/4789)) ([d367cc4](https://github.com/AztecProtocol/aztec-packages/commit/d367cc45c72a8d4a6c4e207a38047f3e63bee3b9)), closes [#4707](https://github.com/AztecProtocol/aztec-packages/issues/4707)
* Integration test of body publishing ([#4795](https://github.com/AztecProtocol/aztec-packages/issues/4795)) ([e414846](https://github.com/AztecProtocol/aztec-packages/commit/e414846db11479f91f332fd4d5edf62b3eeae905))
* Make first iteration of protogalaxy more efficient ([#4630](https://github.com/AztecProtocol/aztec-packages/issues/4630)) ([4c7f24f](https://github.com/AztecProtocol/aztec-packages/commit/4c7f24f8ea8c21bc8114ead67d2082a06c9c5493))
* Min noir build ([#4812](https://github.com/AztecProtocol/aztec-packages/issues/4812)) ([01dd0a9](https://github.com/AztecProtocol/aztec-packages/commit/01dd0a9318de6c69d60e15d56b0fb29d2ec51b28))
* More interop tests ([#4699](https://github.com/AztecProtocol/aztec-packages/issues/4699)) ([a9971e1](https://github.com/AztecProtocol/aztec-packages/commit/a9971e10e7e9980946ebcbe7a7d4201c61d7bef0)), closes [#4412](https://github.com/AztecProtocol/aztec-packages/issues/4412)
* Move remaining data out of Honk UltraComposer ([#4848](https://github.com/AztecProtocol/aztec-packages/issues/4848)) ([823e071](https://github.com/AztecProtocol/aztec-packages/commit/823e071a0988cae906c13fa47e501fe9912788dc))
* Move vk computation out of Honk Ultra composer ([#4811](https://github.com/AztecProtocol/aztec-packages/issues/4811)) ([f354e89](https://github.com/AztecProtocol/aztec-packages/commit/f354e899b4b35dd6d06699f0dbff48f7ea9ed9c3))
* Moving hash functions to relevant classes ([#4551](https://github.com/AztecProtocol/aztec-packages/issues/4551)) ([731d7d0](https://github.com/AztecProtocol/aztec-packages/commit/731d7d012b1f5fb0f8ae3380f14683a37be0e65c))
* Moving types consts to constants.nr ([#4919](https://github.com/AztecProtocol/aztec-packages/issues/4919)) ([ecfcb78](https://github.com/AztecProtocol/aztec-packages/commit/ecfcb7876e487c9f7a8a31ff5438c15e342ba31b))
* **noir:** Extend 4681 bitsize refactor ([#4689](https://github.com/AztecProtocol/aztec-packages/issues/4689)) ([811d767](https://github.com/AztecProtocol/aztec-packages/commit/811d76771b472a2da0464c3038c15a489d49319c))
* PedersenHash(...) TS func returns Fr ([#4704](https://github.com/AztecProtocol/aztec-packages/issues/4704)) ([c5eeb4c](https://github.com/AztecProtocol/aztec-packages/commit/c5eeb4c4ba4cec3be6b3c9fc60b7105ca2f54867)), closes [#4614](https://github.com/AztecProtocol/aztec-packages/issues/4614)
* Pull noir for u64 as array lengths ([#4787](https://github.com/AztecProtocol/aztec-packages/issues/4787)) ([e69b586](https://github.com/AztecProtocol/aztec-packages/commit/e69b58660ff843350e1e098d8f1a84f4ce3d3c34))
* Purge SafeU120 ([#4819](https://github.com/AztecProtocol/aztec-packages/issues/4819)) ([9633b0f](https://github.com/AztecProtocol/aztec-packages/commit/9633b0fd4dfbdc80b3fc248b03486f2a73f37bed))
* Reduce size for rollup benchmark ([cf8bd85](https://github.com/AztecProtocol/aztec-packages/commit/cf8bd85376169cdeb6fbda40e19ae2601bbb3370))
* Remove import of `dep::aztec` from aztec_macros ([#4941](https://github.com/AztecProtocol/aztec-packages/issues/4941)) ([e696b1e](https://github.com/AztecProtocol/aztec-packages/commit/e696b1e7b4d6f5cc895c6dad7fb56f001ebbac6e))
* Remove last impls of compute_note_hash_and_nullifier ([#4943](https://github.com/AztecProtocol/aztec-packages/issues/4943)) ([ff66bb8](https://github.com/AztecProtocol/aztec-packages/commit/ff66bb83a610ac5d6390c1b648245e31cc958189))
* Remove legacy deployer ([#4777](https://github.com/AztecProtocol/aztec-packages/issues/4777)) ([20dc67b](https://github.com/AztecProtocol/aztec-packages/commit/20dc67b5b1de367787361e8406c09e670b12bac2))
* Remove original return from aztec fns ([#4804](https://github.com/AztecProtocol/aztec-packages/issues/4804)) ([9e246c1](https://github.com/AztecProtocol/aztec-packages/commit/9e246c1289fa40c35c4b28d2f0081dfdc2aa9d19))
* Remove TypeScript tooling from noir-projects. ([#4867](https://github.com/AztecProtocol/aztec-packages/issues/4867)) ([15c5399](https://github.com/AztecProtocol/aztec-packages/commit/15c5399a10719a8916ed82fe0ea510a8c6e8c6c7))
* Remove unnecessary casts ([#4906](https://github.com/AztecProtocol/aztec-packages/issues/4906)) ([7a62c2f](https://github.com/AztecProtocol/aztec-packages/commit/7a62c2f9dfc35080a3051c518fa63c26f86977d7))
* Remove VK computation Pg prover flow; improve benchmark to reflect possible optimization ([#4639](https://github.com/AztecProtocol/aztec-packages/issues/4639)) ([c1709b3](https://github.com/AztecProtocol/aztec-packages/commit/c1709b3d5fe615d980b2ebd9283fb841d9e6a85a))
* Remove WASMTIME_ENV_HACK ([#4714](https://github.com/AztecProtocol/aztec-packages/issues/4714)) ([50f89f1](https://github.com/AztecProtocol/aztec-packages/commit/50f89f1832154d526908c55ab296aaf9bacf3608))
* Removing msg-key ([#4856](https://github.com/AztecProtocol/aztec-packages/issues/4856)) ([2b6656d](https://github.com/AztecProtocol/aztec-packages/commit/2b6656dbbd3b16297ceb93df3403a7c7d80c9899)), closes [#4678](https://github.com/AztecProtocol/aztec-packages/issues/4678)
* Rename avm_mini to avm ([#4580](https://github.com/AztecProtocol/aztec-packages/issues/4580)) ([5896a92](https://github.com/AztecProtocol/aztec-packages/commit/5896a920bc4f5fd239d69795872567af6ccbe803)), closes [#4533](https://github.com/AztecProtocol/aztec-packages/issues/4533)
* Rename read request to note hash read request ([#4888](https://github.com/AztecProtocol/aztec-packages/issues/4888)) ([bd3f614](https://github.com/AztecProtocol/aztec-packages/commit/bd3f614009701ab6e7e0033be25c4f04def62ebf))
* Replacing use of `L2Tx` with `TxEffect` ([#4876](https://github.com/AztecProtocol/aztec-packages/issues/4876)) ([d9acaa4](https://github.com/AztecProtocol/aztec-packages/commit/d9acaa43140974c7d5e4380aead467552c932496))
* Specify rust-analyzer.linkedProjects after noir-repo move ([#4922](https://github.com/AztecProtocol/aztec-packages/issues/4922)) ([c22b8c6](https://github.com/AztecProtocol/aztec-packages/commit/c22b8c67483c5f28afd4e95b0a6b0f794224be79))
* Squash yp ypb + other build improvements. ([#4901](https://github.com/AztecProtocol/aztec-packages/issues/4901)) ([be5855c](https://github.com/AztecProtocol/aztec-packages/commit/be5855cdbd1993155bd228afbeafee2c447b46a5))
* Subscribe to a dapp with a token ([#4696](https://github.com/AztecProtocol/aztec-packages/issues/4696)) ([3bbe167](https://github.com/AztecProtocol/aztec-packages/commit/3bbe167b43f13dd87d0ebf0b3f5005ba7bb612e7))
* Switch noir pull to master branch ([#4581](https://github.com/AztecProtocol/aztec-packages/issues/4581)) ([a7889f8](https://github.com/AztecProtocol/aztec-packages/commit/a7889f8d21684099306b72a87e0fb57b3bba0cb4))
* **tests:** Add counter and private voting tests ([#4592](https://github.com/AztecProtocol/aztec-packages/issues/4592)) ([d3be5cc](https://github.com/AztecProtocol/aztec-packages/commit/d3be5cc5d2569f3c9c00f993d4c4df8118bf7e7b))
* Toy avm snake case ([#4584](https://github.com/AztecProtocol/aztec-packages/issues/4584)) ([d071768](https://github.com/AztecProtocol/aztec-packages/commit/d07176863011382c34af5d5c80c596f737369703))
* Updating encoding of TxEffects ([#4726](https://github.com/AztecProtocol/aztec-packages/issues/4726)) ([29b1ea3](https://github.com/AztecProtocol/aztec-packages/commit/29b1ea3db2fd86bb42b584f48d5933e53fa73978))
* Updating viem ([#4783](https://github.com/AztecProtocol/aztec-packages/issues/4783)) ([23bc26a](https://github.com/AztecProtocol/aztec-packages/commit/23bc26a4859d9777c3e6dd49e351a4e6b13a989a))
* Use shared immutable for slow tree ([#4831](https://github.com/AztecProtocol/aztec-packages/issues/4831)) ([821c25d](https://github.com/AztecProtocol/aztec-packages/commit/821c25dccf8b32c51cbca49842395755cf39037e)), closes [#4820](https://github.com/AztecProtocol/aztec-packages/issues/4820)
* Using Tuples in `TxEffect`s and renaming note commitments ([#4717](https://github.com/AztecProtocol/aztec-packages/issues/4717)) ([3dd3c46](https://github.com/AztecProtocol/aztec-packages/commit/3dd3c46591aac17f1d936c49aeb04a5f00e9ff0e))
* Yellow paper typo fix ([#4663](https://github.com/AztecProtocol/aztec-packages/issues/4663)) ([315fcb1](https://github.com/AztecProtocol/aztec-packages/commit/315fcb1f6bf3dcffab51af793cf2745619bed4be))
* **yellowpaper:** Fix notehashexists nullifierexists instructions ([#4625](https://github.com/AztecProtocol/aztec-packages/issues/4625)) ([5d38dc7](https://github.com/AztecProtocol/aztec-packages/commit/5d38dc79e44f6053d68228e061c9c65f117e072b))
* **yellowpaper:** Minor cleanup ([#4622](https://github.com/AztecProtocol/aztec-packages/issues/4622)) ([2d16966](https://github.com/AztecProtocol/aztec-packages/commit/2d169665ee7191a710f9586db0f37fd8d409678e))
* **yellowpaper:** Typos and other cleanup ([#4620](https://github.com/AztecProtocol/aztec-packages/issues/4620)) ([825c5c3](https://github.com/AztecProtocol/aztec-packages/commit/825c5c3446d8d5a31d886972551c0214158a2501))


### Documentation

* Add compression circuit outline ([#4599](https://github.com/AztecProtocol/aztec-packages/issues/4599)) ([2eca2aa](https://github.com/AztecProtocol/aztec-packages/commit/2eca2aa8796b7077e05f0bc1b71dd4d404ad36b3))
* Add Notes page to build section ([#4690](https://github.com/AztecProtocol/aztec-packages/issues/4690)) ([6582b09](https://github.com/AztecProtocol/aztec-packages/commit/6582b09956d03b1749c5727053ca23f7c266e535))
* Add prelude note to migration ([#4949](https://github.com/AztecProtocol/aztec-packages/issues/4949)) ([8342393](https://github.com/AztecProtocol/aztec-packages/commit/83423933f23e28ec7ca6e9a5c96c291ef40303df))
* Add section on nodes and actors ([#3975](https://github.com/AztecProtocol/aztec-packages/issues/3975)) ([379ded4](https://github.com/AztecProtocol/aztec-packages/commit/379ded49162d4f0a9fd2877c1e22d11ad74126b6))
* Address DA comments ([#4641](https://github.com/AztecProtocol/aztec-packages/issues/4641)) ([624ec4c](https://github.com/AztecProtocol/aztec-packages/commit/624ec4ce52479e3060f0d7e656426640407c0f43))
* Incorrect comment ([#4846](https://github.com/AztecProtocol/aztec-packages/issues/4846)) ([4979e02](https://github.com/AztecProtocol/aztec-packages/commit/4979e02dd359238547df0573aab3fe14c81a3602))
* Minor fixes state ([#4909](https://github.com/AztecProtocol/aztec-packages/issues/4909)) ([b027dbb](https://github.com/AztecProtocol/aztec-packages/commit/b027dbbc91298c9a159248e7792aaf0a12dbfcfd))
* Pass by brunny-eth ([#4579](https://github.com/AztecProtocol/aztec-packages/issues/4579)) ([5285010](https://github.com/AztecProtocol/aztec-packages/commit/5285010219fca950991f30d557b8082922fff449))
* Refactoring of private message delivery section of yellow paper ([#4628](https://github.com/AztecProtocol/aztec-packages/issues/4628)) ([5a2c534](https://github.com/AztecProtocol/aztec-packages/commit/5a2c534280fa45de8437b9cdac5600b6eb2eac67))
* Update LSP instructions ([#4920](https://github.com/AztecProtocol/aztec-packages/issues/4920)) ([a5e26e7](https://github.com/AztecProtocol/aztec-packages/commit/a5e26e7c283fb54b4acbc485d227df0b07505401))
* Updated bytecode section ([#4650](https://github.com/AztecProtocol/aztec-packages/issues/4650)) ([fa67330](https://github.com/AztecProtocol/aztec-packages/commit/fa67330ea466058d1613a2c7fa82351f81cf85de))
* Updated fees spec in yellow paper ([#4624](https://github.com/AztecProtocol/aztec-packages/issues/4624)) ([cdf67ea](https://github.com/AztecProtocol/aztec-packages/commit/cdf67ea74aed4ba8f465a981b32f82766a32641a))
* Updated yellow paper P2P network. ([#4652](https://github.com/AztecProtocol/aztec-packages/issues/4652)) ([d3ae287](https://github.com/AztecProtocol/aztec-packages/commit/d3ae28780ca33fe88166e7cceb3cc3c246926195))
* Yellow paper - AVM circuit Chiplets section ([#4642](https://github.com/AztecProtocol/aztec-packages/issues/4642)) ([d717dde](https://github.com/AztecProtocol/aztec-packages/commit/d717dde4054e47dbe56f7903075ea9a007777e54))
* **yellow-paper:** Changes to circuit sections ([#4616](https://github.com/AztecProtocol/aztec-packages/issues/4616)) ([3260081](https://github.com/AztecProtocol/aztec-packages/commit/3260081755bdb3bbd71aaedb2cb129c68110298a))
* **yellowpaper:** AVM `call` instructions, split out sections, cleanup ([#4594](https://github.com/AztecProtocol/aztec-packages/issues/4594)) ([e63f022](https://github.com/AztecProtocol/aztec-packages/commit/e63f02265d3d2b3c2f3e2a9e35ed6201753512f5))

## [0.24.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.23.0...aztec-packages-v0.24.0) (2024-02-13)


### ⚠ BREAKING CHANGES

* move noir out of yarn-project ([#4479](https://github.com/AztecProtocol/aztec-packages/issues/4479))
* note type ids ([#4500](https://github.com/AztecProtocol/aztec-packages/issues/4500))

### Features

* Add fee payment methods ([#4504](https://github.com/AztecProtocol/aztec-packages/issues/4504)) ([d107746](https://github.com/AztecProtocol/aztec-packages/commit/d10774605f7c39175034e664b9dfcca8f7327106))
* Add hashing to stdlib transcript ([#4161](https://github.com/AztecProtocol/aztec-packages/issues/4161)) ([e78b86f](https://github.com/AztecProtocol/aztec-packages/commit/e78b86f9d25cef977b4a3790cccd37a079c8a90f))
* Added cast opcode and cast calldata ([#4423](https://github.com/AztecProtocol/aztec-packages/issues/4423)) ([e58eda8](https://github.com/AztecProtocol/aztec-packages/commit/e58eda804cbdd8a7220013ac8befacbef243b856))
* Added cast opcode and cast calldata (https://github.com/AztecProtocol/aztec-packages/pull/4423) ([db803bd](https://github.com/AztecProtocol/aztec-packages/commit/db803bd50d2417eafc2a0eceb610113a5c11765e))
* **avm-transpiler:** Implement tags for SET and others ([#4545](https://github.com/AztecProtocol/aztec-packages/issues/4545)) ([3063bf3](https://github.com/AztecProtocol/aztec-packages/commit/3063bf326a1619eb51da00b9c1a570107904193f))
* **avm:** Implement addressing modes for MOV ([#4490](https://github.com/AztecProtocol/aztec-packages/issues/4490)) ([ab4eaf0](https://github.com/AztecProtocol/aztec-packages/commit/ab4eaf0af9272d413c25e175aaa1e8b07039ecdb))
* **avm:** Introduce small e2e test  ([#4470](https://github.com/AztecProtocol/aztec-packages/issues/4470)) ([7b4c6e7](https://github.com/AztecProtocol/aztec-packages/commit/7b4c6e769e0b52f61930de085d8e95fe5fdadec2))
* Aztec.js API for registering a contract class ([#4469](https://github.com/AztecProtocol/aztec-packages/issues/4469)) ([d566c74](https://github.com/AztecProtocol/aztec-packages/commit/d566c74786a1ea960e9beee4599c1fdedc7ae6eb))
* **docs:** DIP1 - Extracting how-tos ([#4251](https://github.com/AztecProtocol/aztec-packages/issues/4251)) ([9d50e24](https://github.com/AztecProtocol/aztec-packages/commit/9d50e24024f4c13de1ac27691f3981ded4a0125e))
* Enable gmock and upgrade gtest to 1.13 ([#4480](https://github.com/AztecProtocol/aztec-packages/issues/4480)) ([5fc02e7](https://github.com/AztecProtocol/aztec-packages/commit/5fc02e7f9227788a529c05efbc844a35ec810773))
* IVC bench ([#4515](https://github.com/AztecProtocol/aztec-packages/issues/4515)) ([d8ae42b](https://github.com/AztecProtocol/aztec-packages/commit/d8ae42b1d9ea626dc213739825576522552998ad))
* Nicer API for instance deployment ([#4493](https://github.com/AztecProtocol/aztec-packages/issues/4493)) ([99c3fba](https://github.com/AztecProtocol/aztec-packages/commit/99c3fbad02d0a2b873f46f79d70da84de85af310))
* Note type ids ([#4500](https://github.com/AztecProtocol/aztec-packages/issues/4500)) ([e1da2fd](https://github.com/AztecProtocol/aztec-packages/commit/e1da2fd509c75d7886b95655d233165e087cf2ed))
* Op count timers ([#4471](https://github.com/AztecProtocol/aztec-packages/issues/4471)) ([26918de](https://github.com/AztecProtocol/aztec-packages/commit/26918de4396269eda6c66efc745cf510460a885a))
* PG + Goblin ([#4399](https://github.com/AztecProtocol/aztec-packages/issues/4399)) ([295cd55](https://github.com/AztecProtocol/aztec-packages/commit/295cd5564048ca27316c508766a2dcfc3cc1bf7e))
* Prototype native merkle trees ([#4457](https://github.com/AztecProtocol/aztec-packages/issues/4457)) ([7d5e056](https://github.com/AztecProtocol/aztec-packages/commit/7d5e0563edf3c7397ca994033b703149242cc24c))
* Update rollup circuits and contracts in yp ([#4536](https://github.com/AztecProtocol/aztec-packages/issues/4536)) ([6e89d53](https://github.com/AztecProtocol/aztec-packages/commit/6e89d53dcb6db97ff54678ac7883b7fa0b29a109))


### Bug Fixes

* **bb:** Publishing bb for mac intel ([#4523](https://github.com/AztecProtocol/aztec-packages/issues/4523)) ([4982e3c](https://github.com/AztecProtocol/aztec-packages/commit/4982e3c4ab3b7414ff20c950986b5e0db56b0cfb))
* Broken links in docs [REDO] ([#4540](https://github.com/AztecProtocol/aztec-packages/issues/4540)) ([ce2a205](https://github.com/AztecProtocol/aztec-packages/commit/ce2a20561f758a735756d849cf05e7f5a60aa6a5))
* **build-system:** Image expiring ([#4521](https://github.com/AztecProtocol/aztec-packages/issues/4521)) ([1501afd](https://github.com/AztecProtocol/aztec-packages/commit/1501afde2cd2354ac6f3b5cb79dc3a743811fdec))
* Convert folding recursive verifier ops to batch mul ([#4517](https://github.com/AztecProtocol/aztec-packages/issues/4517)) ([3750b26](https://github.com/AztecProtocol/aztec-packages/commit/3750b262af14ec00edced670d1fbc3d79dfb6b11))
* Cycle_group validate_is_on_curve bug ([#4494](https://github.com/AztecProtocol/aztec-packages/issues/4494)) ([fecf3f7](https://github.com/AztecProtocol/aztec-packages/commit/fecf3f7618d1e016ea5c3afc97e4253639c1d983))
* Field divison / journal comparisions ([#4489](https://github.com/AztecProtocol/aztec-packages/issues/4489)) ([15c06c5](https://github.com/AztecProtocol/aztec-packages/commit/15c06c5281ef1905690d20d9b2759059d1a41961))
* Master ([#4547](https://github.com/AztecProtocol/aztec-packages/issues/4547)) ([490ca26](https://github.com/AztecProtocol/aztec-packages/commit/490ca26539fe9a951f2f1cfeed9c40be55793d90))
* Mirror_noir_subrepo.yml ([#4550](https://github.com/AztecProtocol/aztec-packages/issues/4550)) ([f8d8311](https://github.com/AztecProtocol/aztec-packages/commit/f8d83115705590bcad5c0b7473faed8156a5dbfd))
* Mul with endomorphism ([#4538](https://github.com/AztecProtocol/aztec-packages/issues/4538)) ([1f4c90d](https://github.com/AztecProtocol/aztec-packages/commit/1f4c90da7901e27d8c2abaf248fac0b51bd188f7))
* **noir-mirror:** Don't update .gitrepo on push  ([#4555](https://github.com/AztecProtocol/aztec-packages/issues/4555)) ([686140a](https://github.com/AztecProtocol/aztec-packages/commit/686140abd10c7537c097b47dff298f9d77440d64))
* Recreate jest.config.ts for sequencer-client ([#4553](https://github.com/AztecProtocol/aztec-packages/issues/4553)) ([d172f0b](https://github.com/AztecProtocol/aztec-packages/commit/d172f0bd85e65b12d14f53bc623e6a4312134b79))
* StandardCircuitBuilder create_logic_constraint and uint logic_operator ([#4530](https://github.com/AztecProtocol/aztec-packages/issues/4530)) ([ce51d20](https://github.com/AztecProtocol/aztec-packages/commit/ce51d206ab54f769654422109fb7baa3d8ce2d72))
* Use ordered-binary value encoding for multi maps ([#4565](https://github.com/AztecProtocol/aztec-packages/issues/4565)) ([04ae0d2](https://github.com/AztecProtocol/aztec-packages/commit/04ae0d2a7a09cbbab631375d660dfc8ab2efd52c))


### Miscellaneous

* Aligning some naming in `BaseOrMergeRollupPublicInputs` ([#4510](https://github.com/AztecProtocol/aztec-packages/issues/4510)) ([47d66f9](https://github.com/AztecProtocol/aztec-packages/commit/47d66f9119cad1a82b621d30acab88f47886e063))
* **avm-circuit:** Tests use OpCode enum's  instead of hardcoded values ([#4554](https://github.com/AztecProtocol/aztec-packages/issues/4554)) ([ca4dd60](https://github.com/AztecProtocol/aztec-packages/commit/ca4dd60394934347b3d7f754b26275d0d3d538f1))
* **avm-simulator:** Reduce boilerplate in AVM memory types ([#4542](https://github.com/AztecProtocol/aztec-packages/issues/4542)) ([da2f5ed](https://github.com/AztecProtocol/aztec-packages/commit/da2f5ed3fee642550addf92049a7b21e07938222))
* **avm:** Add/improve tests for AvmContext, tagged memory, etc ([#4484](https://github.com/AztecProtocol/aztec-packages/issues/4484)) ([2fccdf2](https://github.com/AztecProtocol/aztec-packages/commit/2fccdf2408912b0b6b4abef71ab55dce01bdae73))
* **avm:** Remove field support for comparators and bitwise ops ([#4516](https://github.com/AztecProtocol/aztec-packages/issues/4516)) ([87a9663](https://github.com/AztecProtocol/aztec-packages/commit/87a96635e5f873fcb4918fc05a3f6fbe66986ad7))
* **avm:** Use some matchers gtest functionalities to improve unit tests ([#4502](https://github.com/AztecProtocol/aztec-packages/issues/4502)) ([bf4fc6c](https://github.com/AztecProtocol/aztec-packages/commit/bf4fc6c7d50957236d56b311dd0272b1dceca92f)), closes [#4495](https://github.com/AztecProtocol/aztec-packages/issues/4495)
* Cleanup of `abi.nr` in `aztec-nr` ([#4473](https://github.com/AztecProtocol/aztec-packages/issues/4473)) ([6d9c73a](https://github.com/AztecProtocol/aztec-packages/commit/6d9c73aa2bc88ffabf233db2afd9d53556745e7e))
* Cleanup of `abi.nr` in `aztec-nr` (https://github.com/AztecProtocol/aztec-packages/pull/4473) [skip ci] ([db803bd](https://github.com/AztecProtocol/aztec-packages/commit/db803bd50d2417eafc2a0eceb610113a5c11765e))
* Create constraints for sha256 compression opcode ([#4503](https://github.com/AztecProtocol/aztec-packages/issues/4503)) ([64bef49](https://github.com/AztecProtocol/aztec-packages/commit/64bef495d5ba25bb1d4b191e139618f5c491420d))
* Little cpp style improvements ([#4528](https://github.com/AztecProtocol/aztec-packages/issues/4528)) ([dcc9ba4](https://github.com/AztecProtocol/aztec-packages/commit/dcc9ba47b34201566d9433dce6be7da7ab54ccea))
* Move noir out of yarn-project ([#4479](https://github.com/AztecProtocol/aztec-packages/issues/4479)) ([1fe674b](https://github.com/AztecProtocol/aztec-packages/commit/1fe674b046c694e1cbbbb2edaf5a855828bb5340)), closes [#4107](https://github.com/AztecProtocol/aztec-packages/issues/4107)
* Pull noir ([#4546](https://github.com/AztecProtocol/aztec-packages/issues/4546)) ([acf5cf2](https://github.com/AztecProtocol/aztec-packages/commit/acf5cf231e883daaa5c3e85d7739281a994cfb9e))
* Pull noir (https://github.com/AztecProtocol/aztec-packages/pull/4546) ([db803bd](https://github.com/AztecProtocol/aztec-packages/commit/db803bd50d2417eafc2a0eceb610113a5c11765e))
* Redo noir subrepo force push ([#4514](https://github.com/AztecProtocol/aztec-packages/issues/4514)) ([7b519a4](https://github.com/AztecProtocol/aztec-packages/commit/7b519a4b752dede36d8f2516c798d81fd36f4f73))
* Remove .oldValue in contract state update request ([#4499](https://github.com/AztecProtocol/aztec-packages/issues/4499)) ([a796bef](https://github.com/AztecProtocol/aztec-packages/commit/a796bef62dc2786f372afd2e933d9ca102aa2f1d))
* Removing redundant utilities ([#4532](https://github.com/AztecProtocol/aztec-packages/issues/4532)) ([79bf445](https://github.com/AztecProtocol/aztec-packages/commit/79bf44581ea3039a33ea63bb1a2ed429bfa0ece8)), closes [#3470](https://github.com/AztecProtocol/aztec-packages/issues/3470)
* Rename kernel circuits and disambiguate inputs ([#4535](https://github.com/AztecProtocol/aztec-packages/issues/4535)) ([ed6d521](https://github.com/AztecProtocol/aztec-packages/commit/ed6d5218d28d2b6f556ea70188d1b7d331285dc9))
* Replace relative paths to noir-protocol-circuits ([f1accbf](https://github.com/AztecProtocol/aztec-packages/commit/f1accbfdbd7fb3af3feb2fdcf5266143954bc1e6))
* Replace relative paths to noir-protocol-circuits ([20ee430](https://github.com/AztecProtocol/aztec-packages/commit/20ee43056591b576fb9f4473137ccafb5afb9232))
* Replace relative paths to noir-protocol-circuits ([101ab59](https://github.com/AztecProtocol/aztec-packages/commit/101ab597a6dfea1ef4c5da461cdec17e57912d4a))
* Simulator utils cleanup ([#4507](https://github.com/AztecProtocol/aztec-packages/issues/4507)) ([1dd0ebf](https://github.com/AztecProtocol/aztec-packages/commit/1dd0ebfdec2942ceecb958a15c84085ca8bb3d5a))
* Sync to noir-lang/noir ([db803bd](https://github.com/AztecProtocol/aztec-packages/commit/db803bd50d2417eafc2a0eceb610113a5c11765e))
* Testing all values in `PublicGlobalVariables` and `PrivateGlobalVariables` ([#4481](https://github.com/AztecProtocol/aztec-packages/issues/4481)) ([bc25f9b](https://github.com/AztecProtocol/aztec-packages/commit/bc25f9bd8448136b70763d232d62867e4bd4ec4e))
* Testing historical header in contexts ([#4509](https://github.com/AztecProtocol/aztec-packages/issues/4509)) ([c00229a](https://github.com/AztecProtocol/aztec-packages/commit/c00229a12681ba273322e86370188496c05e4691))
* Updating field conversion code without pointer hack ([#4537](https://github.com/AztecProtocol/aztec-packages/issues/4537)) ([94f436e](https://github.com/AztecProtocol/aztec-packages/commit/94f436ed12f17d2671dbaea8bf581fc0cda0986d))
* Uses sha256compression opcode in Noir and implements acvm solver for it ([#4511](https://github.com/AztecProtocol/aztec-packages/issues/4511)) ([9dc05bc](https://github.com/AztecProtocol/aztec-packages/commit/9dc05bc3d47c57981e584661fcc7b5480e21d7d8))


### Documentation

* Describe the new message box model ([#4485](https://github.com/AztecProtocol/aztec-packages/issues/4485)) ([14cc1dd](https://github.com/AztecProtocol/aztec-packages/commit/14cc1dd422ae55aa381cf1f39e157e34ac4ee90b))
* Review of docs, so far ([#4505](https://github.com/AztecProtocol/aztec-packages/issues/4505)) ([140c508](https://github.com/AztecProtocol/aztec-packages/commit/140c5080b98107821a968fbb0e9e716a9360bf90))
* **yellowpaper:** Avm tree-access operations ([#4552](https://github.com/AztecProtocol/aztec-packages/issues/4552)) ([913f4bd](https://github.com/AztecProtocol/aztec-packages/commit/913f4bde56c6602b6db6c73c4d6001bca9c46ca4))
* **yellowpaper:** Separate section for AVM state ([#4440](https://github.com/AztecProtocol/aztec-packages/issues/4440)) ([7881f09](https://github.com/AztecProtocol/aztec-packages/commit/7881f09aa7628ac3adf447a7b696a614dcc47fee))

## [0.23.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.22.0...aztec-packages-v0.23.0) (2024-02-07)


### ⚠ BREAKING CHANGES

* rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420)
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014)
* init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200)
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) [skip ci]

### Features

* Add additional error types to verifier contract and revert early ([#4464](https://github.com/AztecProtocol/aztec-packages/issues/4464)) ([5e16063](https://github.com/AztecProtocol/aztec-packages/commit/5e160632bb7d48e676583e1b62b604c25fc4af4e))
* Add bit size to const opcode (https://github.com/AztecProtocol/aztec-packages/pull/4385) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Add expression width into acir (https://github.com/AztecProtocol/aztec-packages/pull/4014) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Allow brillig to read arrays directly from memory (https://github.com/AztecProtocol/aztec-packages/pull/4460) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Allow nested arrays and vectors in Brillig foreign calls ([#4478](https://github.com/AztecProtocol/aztec-packages/issues/4478)) ([bbfa337](https://github.com/AztecProtocol/aztec-packages/commit/bbfa3374d20b44c49870e21c61cbb2ab5f7ae117))
* Allow nested arrays and vectors in Brillig foreign calls (https://github.com/AztecProtocol/aztec-packages/pull/4478) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* **avm:** Back in avm context with macro - refactor context (https://github.com/AztecProtocol/aztec-packages/pull/4438) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* **avm:** Generic bytecode deserialization ([#4441](https://github.com/AztecProtocol/aztec-packages/issues/4441)) ([934fabc](https://github.com/AztecProtocol/aztec-packages/commit/934fabc8d3706e601eb3dca546c4545b58a10006)), closes [#4304](https://github.com/AztecProtocol/aztec-packages/issues/4304)
* **avm:** Support variable size SET opcode ([#4465](https://github.com/AztecProtocol/aztec-packages/issues/4465)) ([545b334](https://github.com/AztecProtocol/aztec-packages/commit/545b3341f73e5b20c3ac39b75f4783e7bdecac5d))
* **aztec-nr:** Initial work for aztec public vm macro (https://github.com/AztecProtocol/aztec-packages/pull/4400) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* **bb:** Op counting mode ([#4437](https://github.com/AztecProtocol/aztec-packages/issues/4437)) ([5d00cff](https://github.com/AztecProtocol/aztec-packages/commit/5d00cff86a1f76f5279dad6a0bd4e02c8211b225))
* Canonical instance deployer contract ([#4436](https://github.com/AztecProtocol/aztec-packages/issues/4436)) ([b4acc8c](https://github.com/AztecProtocol/aztec-packages/commit/b4acc8c6227f1551998aab9a300891b560479b9c))
* Init storage macro (https://github.com/AztecProtocol/aztec-packages/pull/4200) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Updating global vars with fees ([#4421](https://github.com/AztecProtocol/aztec-packages/issues/4421)) ([34109eb](https://github.com/AztecProtocol/aztec-packages/commit/34109eb9c068df87bbb04164a1531605f3a2c47d)), closes [#3824](https://github.com/AztecProtocol/aztec-packages/issues/3824)


### Bug Fixes

* Delay rming bins till right before installing them. ([#4474](https://github.com/AztecProtocol/aztec-packages/issues/4474)) ([fabeac8](https://github.com/AztecProtocol/aztec-packages/commit/fabeac8b3f9971763a9a723ac6983e5ea8330f46))
* **docs:** Add redirect for top google hit giving 404 ([#4487](https://github.com/AztecProtocol/aztec-packages/issues/4487)) ([e1d3f5a](https://github.com/AztecProtocol/aztec-packages/commit/e1d3f5ad6f45966845983a01eed8167afe7c137f))
* **docs:** Update mdx files to md ([#4459](https://github.com/AztecProtocol/aztec-packages/issues/4459)) ([e67d94b](https://github.com/AztecProtocol/aztec-packages/commit/e67d94b9c335b94b2ca01ebb71f2da54747b9ee1))
* **docs:** Update private voting tutorial cli commands ([#4472](https://github.com/AztecProtocol/aztec-packages/issues/4472)) ([0a8905a](https://github.com/AztecProtocol/aztec-packages/commit/0a8905a46fba47d1a773fd9cf562ef6f51197236))
* Parse instance deployed event ([#4482](https://github.com/AztecProtocol/aztec-packages/issues/4482)) ([62b171a](https://github.com/AztecProtocol/aztec-packages/commit/62b171a1d217324fc61fad049ab32ce97a2fc2fb))


### Miscellaneous

* Able to run noir-sync manually ([#4486](https://github.com/AztecProtocol/aztec-packages/issues/4486)) ([2082fed](https://github.com/AztecProtocol/aztec-packages/commit/2082fedfb03d4882a269881f51c5337263bc539b))
* **acir:** Move `is_recursive` flag to be part of the circuit definition (https://github.com/AztecProtocol/aztec-packages/pull/4221) [skip ci] ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Add bigint solver in ACVM and add a unit test for bigints in Noir (https://github.com/AztecProtocol/aztec-packages/pull/4415) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* **avm:** Add SET serialize comments and simplify signature ([#4476](https://github.com/AztecProtocol/aztec-packages/issues/4476)) ([84dbdd3](https://github.com/AztecProtocol/aztec-packages/commit/84dbdd35684ebc9929097c4ea9cfd59584fb8cbb))
* Lift rollup address check & deplot kv-store to npm ([#4483](https://github.com/AztecProtocol/aztec-packages/issues/4483)) ([92d0aa4](https://github.com/AztecProtocol/aztec-packages/commit/92d0aa40ef9add4b433feed8862ba4286dc7036c))
* Nuked `OptionallyRevealedData` ([#4456](https://github.com/AztecProtocol/aztec-packages/issues/4456)) ([83a3136](https://github.com/AztecProtocol/aztec-packages/commit/83a3136ac1553184b42ca7d96609697098eb80f4))
* Rename bigint_neg into bigint_sub (https://github.com/AztecProtocol/aztec-packages/pull/4420) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Replace relative paths to noir-protocol-circuits ([902bbd4](https://github.com/AztecProtocol/aztec-packages/commit/902bbd4af8c77e209ad1f29e0fe03b55a384b142))
* Surpress chained macro warning (https://github.com/AztecProtocol/aztec-packages/pull/4396) ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Sync to noir-lang/noir ([4113cfd](https://github.com/AztecProtocol/aztec-packages/commit/4113cfdfd5cf43a4dff98cdc398daf3a0d891dd3))
* Unhardcode canonical addresses of deployer and registerer contracts ([#4467](https://github.com/AztecProtocol/aztec-packages/issues/4467)) ([2c82b62](https://github.com/AztecProtocol/aztec-packages/commit/2c82b62951ac001c4c1574b539aeff76d4e6d014))

## [0.22.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.21.0...aztec-packages-v0.22.0) (2024-02-06)


### ⚠ BREAKING CHANGES

* rename bigint_neg into bigint_sub ([#4420](https://github.com/AztecProtocol/aztec-packages/issues/4420))
* Add expression width into acir ([#4014](https://github.com/AztecProtocol/aztec-packages/issues/4014))
* Use NoteSerialize and NoteDeserialize traits for note specific serialization  ([#4383](https://github.com/AztecProtocol/aztec-packages/issues/4383))
* Unencrypted logs are not strings ([#4392](https://github.com/AztecProtocol/aztec-packages/issues/4392))
* init storage macro ([#4200](https://github.com/AztecProtocol/aztec-packages/issues/4200))
* **acir:** Move `is_recursive` flag to be part of the circuit definition ([#4221](https://github.com/AztecProtocol/aztec-packages/issues/4221))
* introduce compute_note_hash_for_(consumption/insertion) ([#4344](https://github.com/AztecProtocol/aztec-packages/issues/4344))
* replace Note::compute_note_hash with Note::compute_note_content_hash ([#4342](https://github.com/AztecProtocol/aztec-packages/issues/4342))
* Include contract class id in deployment info ([#4223](https://github.com/AztecProtocol/aztec-packages/issues/4223))
* Serialize, Deserialize and NoteInterface as Traits ([#4135](https://github.com/AztecProtocol/aztec-packages/issues/4135))

### Features

* Add aztec node to client execution and nuke state info provider [#4320](https://github.com/AztecProtocol/aztec-packages/issues/4320) ([#4401](https://github.com/AztecProtocol/aztec-packages/issues/4401)) ([2dec0cc](https://github.com/AztecProtocol/aztec-packages/commit/2dec0cc4bf5bc592443d65e3a9923fc2a4f076a3))
* Add bit size to const opcode ([#4385](https://github.com/AztecProtocol/aztec-packages/issues/4385)) ([b2a000e](https://github.com/AztecProtocol/aztec-packages/commit/b2a000e5f366721b514653456db804a704242b20))
* Add expression width into acir ([#4014](https://github.com/AztecProtocol/aztec-packages/issues/4014)) ([f09e8fc](https://github.com/AztecProtocol/aztec-packages/commit/f09e8fc3fdaf9a0e5b9f927e345bf9e819e2024c))
* Add meta_hwm to PrivateCircuitPublicInputs ([#4341](https://github.com/AztecProtocol/aztec-packages/issues/4341)) ([4f248b5](https://github.com/AztecProtocol/aztec-packages/commit/4f248b55af8119ed588754dff3cf14eb0559c8d2))
* Add poseidon2 hashing to native transcript ([#3718](https://github.com/AztecProtocol/aztec-packages/issues/3718)) ([afcfa71](https://github.com/AztecProtocol/aztec-packages/commit/afcfa71da760680dfe02c39cf2de068a4297b3e7))
* Adding slitherin detectors ([#4246](https://github.com/AztecProtocol/aztec-packages/issues/4246)) ([7cdc186](https://github.com/AztecProtocol/aztec-packages/commit/7cdc18692313017a70b9c8d761b7540eb96a9369))
* Allow brillig to read arrays directly from memory ([#4460](https://github.com/AztecProtocol/aztec-packages/issues/4460)) ([f99392d](https://github.com/AztecProtocol/aztec-packages/commit/f99392dace572889b34ccd000f8af252c92c3b5e))
* Allow using of current block in inclusion proofs ([#4285](https://github.com/AztecProtocol/aztec-packages/issues/4285)) ([728c5ac](https://github.com/AztecProtocol/aztec-packages/commit/728c5ac5dd534ce28c2ac84a8f720ea85c36308c)), closes [#4274](https://github.com/AztecProtocol/aztec-packages/issues/4274)
* **avm-transpiler:** Brillig to AVM transpiler ([#4227](https://github.com/AztecProtocol/aztec-packages/issues/4227)) ([c366c6e](https://github.com/AztecProtocol/aztec-packages/commit/c366c6e6d5c9f28a5dc92a303dcab4a23fb2d84e))
* **avm:** Add command to call avm proving in bb binary ([#4369](https://github.com/AztecProtocol/aztec-packages/issues/4369)) ([4f6d607](https://github.com/AztecProtocol/aztec-packages/commit/4f6d607d7dce36819d84ba6ce69bbd57e0ad79a0)), closes [#4039](https://github.com/AztecProtocol/aztec-packages/issues/4039)
* **avm:** Add revert tracking to the journal ([#4349](https://github.com/AztecProtocol/aztec-packages/issues/4349)) ([1615803](https://github.com/AztecProtocol/aztec-packages/commit/161580312a753fb2f0507c11fb10d895b5073e3e))
* **avm:** Back in avm context with macro - refactor context ([#4438](https://github.com/AztecProtocol/aztec-packages/issues/4438)) ([ccf9b17](https://github.com/AztecProtocol/aztec-packages/commit/ccf9b17495ec46df6494fa93e1c848c87a05d071))
* **avm:** Complete SET instruction ([#4378](https://github.com/AztecProtocol/aztec-packages/issues/4378)) ([013891f](https://github.com/AztecProtocol/aztec-packages/commit/013891fc3a65066a315652ad0ac41a9622b59573))
* **avm:** Implement avm state getter opcodes within noir contracts ([#4402](https://github.com/AztecProtocol/aztec-packages/issues/4402)) ([9f2a6eb](https://github.com/AztecProtocol/aztec-packages/commit/9f2a6eb80f796a9be1c9c5b6a143dc70e5ec3c43))
* **avm:** Implement serialization for all existing operations ([#4338](https://github.com/AztecProtocol/aztec-packages/issues/4338)) ([13e0683](https://github.com/AztecProtocol/aztec-packages/commit/13e0683034e3a7ec02f73be36c82e5b3a9fe7151))
* **avm:** Keep history of reads and writes in journal ([#4315](https://github.com/AztecProtocol/aztec-packages/issues/4315)) ([cdf1baf](https://github.com/AztecProtocol/aztec-packages/commit/cdf1baf017c4833bc621ba4dd3681dd1a745e259))
* **aztec-nr:** Initial work for aztec public vm macro ([#4400](https://github.com/AztecProtocol/aztec-packages/issues/4400)) ([0024590](https://github.com/AztecProtocol/aztec-packages/commit/00245900d1cc7511f3fa71fe461944fbe7094d5a))
* **bb:** Wasmtime and remote benchmarking ([#4204](https://github.com/AztecProtocol/aztec-packages/issues/4204)) ([fd27808](https://github.com/AztecProtocol/aztec-packages/commit/fd27808721b1f32b4828db5465b502cca2f1ce6c))
* Contract class registerer contract ([#4403](https://github.com/AztecProtocol/aztec-packages/issues/4403)) ([d953090](https://github.com/AztecProtocol/aztec-packages/commit/d953090ca9eba0184d10c0b8ddbc60998bc155f0)), closes [#4069](https://github.com/AztecProtocol/aztec-packages/issues/4069) [#4070](https://github.com/AztecProtocol/aztec-packages/issues/4070)
* Crude stable var implementation ([#4289](https://github.com/AztecProtocol/aztec-packages/issues/4289)) ([5f9eee4](https://github.com/AztecProtocol/aztec-packages/commit/5f9eee48579a507512612e283b4106ddf9d72555))
* **docs:** Docs deeper dive into unconstrained functions ([#4233](https://github.com/AztecProtocol/aztec-packages/issues/4233)) ([6af548e](https://github.com/AztecProtocol/aztec-packages/commit/6af548e369d5d20bcbd08d346f5c6f89b7363f39))
* Emit single functions from class registerer ([#4429](https://github.com/AztecProtocol/aztec-packages/issues/4429)) ([19e03ad](https://github.com/AztecProtocol/aztec-packages/commit/19e03adc71ab7561d33dc9d75b9bdb7c19883fc9)), closes [#4427](https://github.com/AztecProtocol/aztec-packages/issues/4427)
* Extend Historical Access APIs [#4179](https://github.com/AztecProtocol/aztec-packages/issues/4179) ([#4375](https://github.com/AztecProtocol/aztec-packages/issues/4375)) ([c918d8d](https://github.com/AztecProtocol/aztec-packages/commit/c918d8d1a6ba306afd2feab97ddb8527b76d1a82))
* Folding `GoblinUltra` instances in ProtoGalaxy ([#4340](https://github.com/AztecProtocol/aztec-packages/issues/4340)) ([8569e7c](https://github.com/AztecProtocol/aztec-packages/commit/8569e7c091c3db424a3f1c70b0749489d8574ad2))
* Hashing output of `serialize()` in noir + more tests ([#4365](https://github.com/AztecProtocol/aztec-packages/issues/4365)) ([5a71bb9](https://github.com/AztecProtocol/aztec-packages/commit/5a71bb95a57bf22189e2611035e5f1faae92426b))
* Implementation for bigint opcodes ([#4288](https://github.com/AztecProtocol/aztec-packages/issues/4288)) ([b61dace](https://github.com/AztecProtocol/aztec-packages/commit/b61dacee47f57a8fce6657f28b64e7a3128d0dba))
* Improve ivc bench ([#4242](https://github.com/AztecProtocol/aztec-packages/issues/4242)) ([9d28354](https://github.com/AztecProtocol/aztec-packages/commit/9d28354ecefc9f7db71c7d2f40da7eae30e133c5))
* Include contract class id in deployment info ([#4223](https://github.com/AztecProtocol/aztec-packages/issues/4223)) ([0ed4126](https://github.com/AztecProtocol/aztec-packages/commit/0ed41261ae43e21f695c35ad753e07adfaaa55f9)), closes [#4054](https://github.com/AztecProtocol/aztec-packages/issues/4054)
* Init storage macro ([#4200](https://github.com/AztecProtocol/aztec-packages/issues/4200)) ([11d9697](https://github.com/AztecProtocol/aztec-packages/commit/11d9697f8c1248e92341638a5587c7e24b09425a))
* Memory only brillig ([#4215](https://github.com/AztecProtocol/aztec-packages/issues/4215)) ([018177b](https://github.com/AztecProtocol/aztec-packages/commit/018177bc757cce3258c153a56f1f7a871fec681c))
* Nullified note retrieval in get_notes and view_notes ([#4238](https://github.com/AztecProtocol/aztec-packages/issues/4238)) ([8d02eb7](https://github.com/AztecProtocol/aztec-packages/commit/8d02eb71c96eb01726ec828e0e6934d0f30121ed))
* Private calls and initialization of undeployed contracts ([#4362](https://github.com/AztecProtocol/aztec-packages/issues/4362)) ([f31c181](https://github.com/AztecProtocol/aztec-packages/commit/f31c181f187c2aca90c91834a434b7d2e563af84)), closes [#4057](https://github.com/AztecProtocol/aztec-packages/issues/4057) [#4058](https://github.com/AztecProtocol/aztec-packages/issues/4058) [#4059](https://github.com/AztecProtocol/aztec-packages/issues/4059)
* Revert early in verifier contract for malformed proof inputs ([#4453](https://github.com/AztecProtocol/aztec-packages/issues/4453)) ([d4a7716](https://github.com/AztecProtocol/aztec-packages/commit/d4a7716800a5f67ec55f7f85beeb439f11b11d4d))
* Sequencer processes transactions in phases ([#4345](https://github.com/AztecProtocol/aztec-packages/issues/4345)) ([78cc709](https://github.com/AztecProtocol/aztec-packages/commit/78cc709ea5f9fc137472a76e5155216ca439d292))
* Unencrypted logs are not strings ([#4392](https://github.com/AztecProtocol/aztec-packages/issues/4392)) ([25a7ea7](https://github.com/AztecProtocol/aztec-packages/commit/25a7ea76effa98b09051cde383fdcce95e314166))
* Validate verification key on contract deployment ([#4450](https://github.com/AztecProtocol/aztec-packages/issues/4450)) ([00f9966](https://github.com/AztecProtocol/aztec-packages/commit/00f996631130b9a284f29adff4ce5bcc5ad70b1b))
* Verify function against contract class id in private kernel ([#4337](https://github.com/AztecProtocol/aztec-packages/issues/4337)) ([e1d832d](https://github.com/AztecProtocol/aztec-packages/commit/e1d832dbf6bf06b192538bc768871848484a4f14)), closes [#4056](https://github.com/AztecProtocol/aztec-packages/issues/4056)


### Bug Fixes

* **avm-transpiler:** Avm-transpiler bootstrap by tying down rust version ([#4347](https://github.com/AztecProtocol/aztec-packages/issues/4347)) ([09d0730](https://github.com/AztecProtocol/aztec-packages/commit/09d0730bad4be2f4954cbb6d27538f7860d0f21f))
* **avm-transpiler:** Bump rust toolchain version for transpiler ([#4356](https://github.com/AztecProtocol/aztec-packages/issues/4356)) ([75e30b9](https://github.com/AztecProtocol/aztec-packages/commit/75e30b999feeda0f7526669f3f0f08ca6c4acac2))
* **avm:** Fix SendL2ToL1Message implementation ([#4367](https://github.com/AztecProtocol/aztec-packages/issues/4367)) ([ee560c3](https://github.com/AztecProtocol/aztec-packages/commit/ee560c32873a085a68288357471b1a54d8cb9c6f))
* Aztec binary fixes ([#4273](https://github.com/AztecProtocol/aztec-packages/issues/4273)) ([84e1f7d](https://github.com/AztecProtocol/aztec-packages/commit/84e1f7dd0e005351bb742b015270ab2fd575136d))
* Bb build ([#4317](https://github.com/AztecProtocol/aztec-packages/issues/4317)) ([82f5f03](https://github.com/AztecProtocol/aztec-packages/commit/82f5f03acdaee8e23b149369cb9e6f89f257b757))
* **docs:** Another one ([#4455](https://github.com/AztecProtocol/aztec-packages/issues/4455)) ([538f308](https://github.com/AztecProtocol/aztec-packages/commit/538f3081f7ac158d983e182ce848254f63335740))
* **docs:** Update import ([#4451](https://github.com/AztecProtocol/aztec-packages/issues/4451)) ([a4bc954](https://github.com/AztecProtocol/aztec-packages/commit/a4bc954006269fd064d76300aa515604a6bbdd29))
* Load contract artifact from json ([#4352](https://github.com/AztecProtocol/aztec-packages/issues/4352)) ([47a0a79](https://github.com/AztecProtocol/aztec-packages/commit/47a0a79f6beaa241eafc94fcae84103488a9dcef))
* Mac build ([#4336](https://github.com/AztecProtocol/aztec-packages/issues/4336)) ([aeb4cf0](https://github.com/AztecProtocol/aztec-packages/commit/aeb4cf0d9cec6127cac947c4f0de8e853b2f34e0))
* **noir-contracts:** Disable transpilation for now ([#4372](https://github.com/AztecProtocol/aztec-packages/issues/4372)) ([37662b7](https://github.com/AztecProtocol/aztec-packages/commit/37662b78da3811fd5d5f4d4d33d69d4c5fd873e3))
* Nr codegen to use new protocol types path ([#4353](https://github.com/AztecProtocol/aztec-packages/issues/4353)) ([84e63b1](https://github.com/AztecProtocol/aztec-packages/commit/84e63b12dcc45130ddef499dca383b09c9844b8b)), closes [#4193](https://github.com/AztecProtocol/aztec-packages/issues/4193)
* Relative LogFn import ([#4328](https://github.com/AztecProtocol/aztec-packages/issues/4328)) ([1faead5](https://github.com/AztecProtocol/aztec-packages/commit/1faead5bf5e07417e2d4452a2e3ff096a273a41a))
* Release the size of goblin translator ([#4259](https://github.com/AztecProtocol/aztec-packages/issues/4259)) ([6e1d958](https://github.com/AztecProtocol/aztec-packages/commit/6e1d958badafdbe4abdc0c221047186c5da69be4))
* Transpiler build ([#4386](https://github.com/AztecProtocol/aztec-packages/issues/4386)) ([032ddc5](https://github.com/AztecProtocol/aztec-packages/commit/032ddc53840e79b3f324b99b82f0aebfa3c83bfe))


### Miscellaneous

* `PublicCircuitPublicInputs` and `PrivateCircuitPublicInputs` cleanup ([#4360](https://github.com/AztecProtocol/aztec-packages/issues/4360)) ([b92d690](https://github.com/AztecProtocol/aztec-packages/commit/b92d6904fc9ad2cda30de1245fd546e00a5523e1))
* `toFields()`/`fromFields(...)` methods in more classes ([#4335](https://github.com/AztecProtocol/aztec-packages/issues/4335)) ([433b9eb](https://github.com/AztecProtocol/aztec-packages/commit/433b9ebdb505b21bef40c174e4feec0e6ca211e8))
* Acir-simulator -&gt; simulator ([#4439](https://github.com/AztecProtocol/aztec-packages/issues/4439)) ([bccd809](https://github.com/AztecProtocol/aztec-packages/commit/bccd809183f18a0d6fc05bfcdffa78ba1169e894))
* **acir:** Move `is_recursive` flag to be part of the circuit definition ([#4221](https://github.com/AztecProtocol/aztec-packages/issues/4221)) ([9c965a7](https://github.com/AztecProtocol/aztec-packages/commit/9c965a7c9e652dfeaba2f09152e5db287407473d))
* Add bigint solver in ACVM and add a unit test for bigints in Noir ([#4415](https://github.com/AztecProtocol/aztec-packages/issues/4415)) ([e4a2fe9](https://github.com/AztecProtocol/aztec-packages/commit/e4a2fe906f5e02ebcc1fc7a8b7a5d96f3b11fcb5))
* Add bootstrap_cache for avm-transpiler ([#4357](https://github.com/AztecProtocol/aztec-packages/issues/4357)) ([bfebebb](https://github.com/AztecProtocol/aztec-packages/commit/bfebebb89fc9a9b87f19237642cec9a221abf712))
* Add disclaimer ([#4393](https://github.com/AztecProtocol/aztec-packages/issues/4393)) ([6895f52](https://github.com/AztecProtocol/aztec-packages/commit/6895f522220ee689acb178a9ee6271b132fd6cd0))
* Add migration note for serialization change ([#4414](https://github.com/AztecProtocol/aztec-packages/issues/4414)) ([968a3a0](https://github.com/AztecProtocol/aztec-packages/commit/968a3a0734c202cccec5f322a0dd272f66cbeb1c))
* **avm:** Make interpreter a function not a class ([#4272](https://github.com/AztecProtocol/aztec-packages/issues/4272)) ([14e8c5c](https://github.com/AztecProtocol/aztec-packages/commit/14e8c5c325ad8459e3c81cc7e443ca277dd072a9))
* **avm:** Refactor AVM Simulator and fix issues ([#4424](https://github.com/AztecProtocol/aztec-packages/issues/4424)) ([a6179bd](https://github.com/AztecProtocol/aztec-packages/commit/a6179bdb52070d71dd04a3721f987a89920f4d98))
* Call stack item cleanup ([#4381](https://github.com/AztecProtocol/aztec-packages/issues/4381)) ([341b0a1](https://github.com/AztecProtocol/aztec-packages/commit/341b0a177f35d2e46d9f3e011f1543a20628244b))
* Check loading Nargo artifacts works in the cli ([#4355](https://github.com/AztecProtocol/aztec-packages/issues/4355)) ([43b58b3](https://github.com/AztecProtocol/aztec-packages/commit/43b58b346cd788cc4bfa187626bf53b518ad5bb4))
* Cleanup + various doc improvements ([#4282](https://github.com/AztecProtocol/aztec-packages/issues/4282)) ([648229c](https://github.com/AztecProtocol/aztec-packages/commit/648229c24e01b2eeeeb0b361b65c0d62c0adf8ea)), closes [#4264](https://github.com/AztecProtocol/aztec-packages/issues/4264)
* Collapse bb::honk ([#4318](https://github.com/AztecProtocol/aztec-packages/issues/4318)) ([5853af4](https://github.com/AztecProtocol/aztec-packages/commit/5853af448a86ed02901609f4786e86fe1651880e))
* Consistent naming of serialization method ([#4379](https://github.com/AztecProtocol/aztec-packages/issues/4379)) ([148d5dc](https://github.com/AztecProtocol/aztec-packages/commit/148d5dc754329eabcc42430b4ee06993ec2d4224))
* Do not run forge fmt because not everyone has forge installed ([#4430](https://github.com/AztecProtocol/aztec-packages/issues/4430)) ([ecb6c3f](https://github.com/AztecProtocol/aztec-packages/commit/ecb6c3fdff93e8c194acaef9de45d8e740a14bb0))
* **docs:** Update broken link ref in slow_updates_tree.md ([#4339](https://github.com/AztecProtocol/aztec-packages/issues/4339)) ([2599d7f](https://github.com/AztecProtocol/aztec-packages/commit/2599d7f1ea3f616375a7d439c8bde016b6f2b876))
* **docs:** Updating concepts/communication pages images ([#4368](https://github.com/AztecProtocol/aztec-packages/issues/4368)) ([92fb2b0](https://github.com/AztecProtocol/aztec-packages/commit/92fb2b091d6a6143d5f736c4e1e15e454f14a162)), closes [#3857](https://github.com/AztecProtocol/aztec-packages/issues/3857)
* Eth address tech debt cleanup ([#4442](https://github.com/AztecProtocol/aztec-packages/issues/4442)) ([153989f](https://github.com/AztecProtocol/aztec-packages/commit/153989f636b0b76522597fdf60e1f2af9e318b10))
* Extract merge from UC and simplify ([#4343](https://github.com/AztecProtocol/aztec-packages/issues/4343)) ([54fd794](https://github.com/AztecProtocol/aztec-packages/commit/54fd7949cdbb0e213c37ce331f7546e2827f4c17))
* Fix bb wasm build when using remote cache ([#4397](https://github.com/AztecProtocol/aztec-packages/issues/4397)) ([14e57cb](https://github.com/AztecProtocol/aztec-packages/commit/14e57cb285571208c5f88f0eaf500b1e7859ef04))
* Fix clippy warnings in `avm-transpiler` ([#4416](https://github.com/AztecProtocol/aztec-packages/issues/4416)) ([e54ecd2](https://github.com/AztecProtocol/aztec-packages/commit/e54ecd25ad375901eee859f4a85745b5047c190f))
* Fix some circular imports ([#4445](https://github.com/AztecProtocol/aztec-packages/issues/4445)) ([e6a9c68](https://github.com/AztecProtocol/aztec-packages/commit/e6a9c68148deffb6a1352cf2ed75281568ebef39))
* Format l1-contracts after generating constants ([#4448](https://github.com/AztecProtocol/aztec-packages/issues/4448)) ([de11994](https://github.com/AztecProtocol/aztec-packages/commit/de11994d19e84b641984b198c594592147d9c2ec))
* Git subrepo commit (merge) noir ([#4321](https://github.com/AztecProtocol/aztec-packages/issues/4321)) ([348d18a](https://github.com/AztecProtocol/aztec-packages/commit/348d18aa3c864fc80fc791029b2d91ee9e7e33d4))
* Git subrepo pull (merge) noir ([#4331](https://github.com/AztecProtocol/aztec-packages/issues/4331)) ([683f782](https://github.com/AztecProtocol/aztec-packages/commit/683f782e08b007f82505c19f093eed12cd5f48eb))
* Implementing `deserialize()` in Noir structs ([#4384](https://github.com/AztecProtocol/aztec-packages/issues/4384)) ([e63bbae](https://github.com/AztecProtocol/aztec-packages/commit/e63bbaefebb9f0048eb9b2c80ea392bfd86831c1))
* Introduce compute_note_hash_for_(consumption/insertion) ([#4344](https://github.com/AztecProtocol/aztec-packages/issues/4344)) ([26a0d49](https://github.com/AztecProtocol/aztec-packages/commit/26a0d49de177a1e7faecc4dede453de4c46808bb))
* Optimize prove_note_validity [#4418](https://github.com/AztecProtocol/aztec-packages/issues/4418) ([#4426](https://github.com/AztecProtocol/aztec-packages/issues/4426)) ([4de2540](https://github.com/AztecProtocol/aztec-packages/commit/4de25403b4bbdabbd0bfef84ae8a685be682bf84))
* Poseidon2 hash uses span instead of vector ([#4003](https://github.com/AztecProtocol/aztec-packages/issues/4003)) ([f63e7a9](https://github.com/AztecProtocol/aztec-packages/commit/f63e7a94b1ba555eecbe08b7114e8b6ad0b82bc0))
* Reenable private kernel function tree checks ([#4358](https://github.com/AztecProtocol/aztec-packages/issues/4358)) ([e7db0da](https://github.com/AztecProtocol/aztec-packages/commit/e7db0da2a055567e859b347bba30c3cc18f32f68))
* Remove hardcoded storage slot values ([#4398](https://github.com/AztecProtocol/aztec-packages/issues/4398)) ([d2294a4](https://github.com/AztecProtocol/aztec-packages/commit/d2294a4d58e76a7bc9be8f153b47a5a5d5d87db1))
* Rename bigint_neg into bigint_sub ([#4420](https://github.com/AztecProtocol/aztec-packages/issues/4420)) ([57824fe](https://github.com/AztecProtocol/aztec-packages/commit/57824feff268153a7a33b90a3dc68d5bc98a2471))
* Replace Note::compute_note_hash with Note::compute_note_content_hash ([#4342](https://github.com/AztecProtocol/aztec-packages/issues/4342)) ([8368659](https://github.com/AztecProtocol/aztec-packages/commit/836865983c2a0bc6878bde1e30dca56f97f9e1a2))
* Replace relative paths to noir-protocol-circuits ([23de650](https://github.com/AztecProtocol/aztec-packages/commit/23de6504f02b0f93799f57d41b5b4e005e227228))
* Replace relative paths to noir-protocol-circuits ([b8d427f](https://github.com/AztecProtocol/aztec-packages/commit/b8d427fbd735f586c2ea4101e00aa57bc24814aa))
* Replace relative paths to noir-protocol-circuits ([113dec1](https://github.com/AztecProtocol/aztec-packages/commit/113dec1c00293b79aacc506dd6cdfd976a46bcc7))
* Replace relative paths to noir-protocol-circuits ([a79093b](https://github.com/AztecProtocol/aztec-packages/commit/a79093bc9c43efd6c8e1c4c2ceb19d39284e914e))
* Replace relative paths to noir-protocol-circuits ([808b4eb](https://github.com/AztecProtocol/aztec-packages/commit/808b4eb21b74df2b352b571f2f93e949036bd626))
* Serialize, Deserialize and NoteInterface as Traits ([#4135](https://github.com/AztecProtocol/aztec-packages/issues/4135)) ([9e6605c](https://github.com/AztecProtocol/aztec-packages/commit/9e6605cf7cc6e778b681e4e0c39788ab58249f55))
* Simpler noir sync ([#4376](https://github.com/AztecProtocol/aztec-packages/issues/4376)) ([665b35e](https://github.com/AztecProtocol/aztec-packages/commit/665b35ea1f667be057df3d6bb2ca26beb8f3b461))
* Surpress chained macro warning ([#4396](https://github.com/AztecProtocol/aztec-packages/issues/4396)) ([5e9c790](https://github.com/AztecProtocol/aztec-packages/commit/5e9c79057ff7e372c71e221756fc7577aa06baeb))
* Switch to macos-14 for m1 runners ([#3456](https://github.com/AztecProtocol/aztec-packages/issues/3456)) ([ca5b6f8](https://github.com/AztecProtocol/aztec-packages/commit/ca5b6f8b23adf78dbc8c76f08514edeaa3eebfab))
* Testing `toFields()` length ([#4364](https://github.com/AztecProtocol/aztec-packages/issues/4364)) ([5d3fce3](https://github.com/AztecProtocol/aztec-packages/commit/5d3fce35a51151eddde6982913c5d4bd865e450d))
* Typing contents of `MessageLoadOracleInputs` ([#4351](https://github.com/AztecProtocol/aztec-packages/issues/4351)) ([433babd](https://github.com/AztecProtocol/aztec-packages/commit/433babdadfc3fa5e14634e43aafb9efd9c3c2313))
* Update docs for historical state ([#4461](https://github.com/AztecProtocol/aztec-packages/issues/4461)) ([16a2eca](https://github.com/AztecProtocol/aztec-packages/commit/16a2eca1dbc5e8f4ca99c46363ed00eaa54dc97e))
* Update docs on comparators ([#4281](https://github.com/AztecProtocol/aztec-packages/issues/4281)) ([cc2ce9c](https://github.com/AztecProtocol/aztec-packages/commit/cc2ce9c012a11206bd2528771774aa817fa7a922))
* Updating block hash to be header.hash() ([#4286](https://github.com/AztecProtocol/aztec-packages/issues/4286)) ([d4125e1](https://github.com/AztecProtocol/aztec-packages/commit/d4125e12459a0375e9fa2cb8b83f700654219cea))
* Use NoteSerialize and NoteDeserialize traits for note specific serialization  ([#4383](https://github.com/AztecProtocol/aztec-packages/issues/4383)) ([14dd0b8](https://github.com/AztecProtocol/aztec-packages/commit/14dd0b885721d4b9024ceb6569e929269ec9ad23))


### Documentation

* Add simple api description for note_getter_options.status ([#4329](https://github.com/AztecProtocol/aztec-packages/issues/4329)) ([cc17afe](https://github.com/AztecProtocol/aztec-packages/commit/cc17afe73da1f2cad18080797f30210908caa7f6))
* Document stable public state usage ([#4324](https://github.com/AztecProtocol/aztec-packages/issues/4324)) ([13f709b](https://github.com/AztecProtocol/aztec-packages/commit/13f709b2b94d17af34da2e609c2b764977eb5d6b)), closes [#4325](https://github.com/AztecProtocol/aztec-packages/issues/4325)
* Minor quickstart fixes ([#4330](https://github.com/AztecProtocol/aztec-packages/issues/4330)) ([f85a870](https://github.com/AztecProtocol/aztec-packages/commit/f85a87084e3fdba20b6f74d7e53b2e95925e8548))
* Update contract deployment section in YP ([#4290](https://github.com/AztecProtocol/aztec-packages/issues/4290)) ([e99a882](https://github.com/AztecProtocol/aztec-packages/commit/e99a882fcc69041a34ecd7febafe46d661b76094))
* **yellow-paper:** Update kernel with changes from contract deployment ([#4432](https://github.com/AztecProtocol/aztec-packages/issues/4432)) ([201a80e](https://github.com/AztecProtocol/aztec-packages/commit/201a80e747d3754dc2d20c25d4a2c10cc0847f9e))
* **yp:** AVM circuit - user memory section ([#4323](https://github.com/AztecProtocol/aztec-packages/issues/4323)) ([8928fb1](https://github.com/AztecProtocol/aztec-packages/commit/8928fb1f46ce6403eb425548d254e8cfbb6ae6a9)), closes [#4043](https://github.com/AztecProtocol/aztec-packages/issues/4043)

## [0.21.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.20.0...aztec-packages-v0.21.0) (2024-01-30)


### ⚠ BREAKING CHANGES

* aztec binary ([#3927](https://github.com/AztecProtocol/aztec-packages/issues/3927))
* add opcode for sha256 compression function ([#4229](https://github.com/AztecProtocol/aztec-packages/issues/4229))
* add opcode for poseidon2 permutation ([#4214](https://github.com/AztecProtocol/aztec-packages/issues/4214))
* remove ec_double opcode ([#4210](https://github.com/AztecProtocol/aztec-packages/issues/4210))
* Updates singleton usage ([#4186](https://github.com/AztecProtocol/aztec-packages/issues/4186))
* Add big int opcodes (without implementation) ([#4050](https://github.com/AztecProtocol/aztec-packages/issues/4050))

### Features

* **3738:** AVM basic arithmetic operations for non ff types ([#3881](https://github.com/AztecProtocol/aztec-packages/issues/3881)) ([457a3f9](https://github.com/AztecProtocol/aztec-packages/commit/457a3f9b0c05f58cc88ef43763c5d55b6debaf05)), closes [#3996](https://github.com/AztecProtocol/aztec-packages/issues/3996)
* Accrued substate instructions ([#4197](https://github.com/AztecProtocol/aztec-packages/issues/4197)) ([bfe30d2](https://github.com/AztecProtocol/aztec-packages/commit/bfe30d27785fb659bea5bb716ccf0eadb0f19167))
* Add big int opcodes (without implementation) ([#4050](https://github.com/AztecProtocol/aztec-packages/issues/4050)) ([bcab9ce](https://github.com/AztecProtocol/aztec-packages/commit/bcab9ceab62bede3bc1c105b3e639e7c64e3217a))
* Add comparators to get/view note ([#3136](https://github.com/AztecProtocol/aztec-packages/issues/3136)) ([#4205](https://github.com/AztecProtocol/aztec-packages/issues/4205)) ([6de36b3](https://github.com/AztecProtocol/aztec-packages/commit/6de36b39825785ec96a967b1ff4f2cbb83943ebb))
* Add inclusion check l1-&gt;l2 ([#4141](https://github.com/AztecProtocol/aztec-packages/issues/4141)) ([bef65c3](https://github.com/AztecProtocol/aztec-packages/commit/bef65c38c58427325b4481ab794f0fb4f12196b0))
* Add opcode for poseidon2 permutation ([#4214](https://github.com/AztecProtocol/aztec-packages/issues/4214)) ([53c5ba5](https://github.com/AztecProtocol/aztec-packages/commit/53c5ba5fa2a86aba16bba8aa8d3a594a789e3e24))
* Add opcode for sha256 compression function ([#4229](https://github.com/AztecProtocol/aztec-packages/issues/4229)) ([ac25ff7](https://github.com/AztecProtocol/aztec-packages/commit/ac25ff737a934a5f260605f4497e4074c3ed5824))
* Adding slither to l1-contracts  ([#4226](https://github.com/AztecProtocol/aztec-packages/issues/4226)) ([b4dc31d](https://github.com/AztecProtocol/aztec-packages/commit/b4dc31dd9fb02c096db6c40d848aea9f03e36a8c))
* **avm:** Add tests for memory and bitwise instructions ([#4184](https://github.com/AztecProtocol/aztec-packages/issues/4184)) ([6dac650](https://github.com/AztecProtocol/aztec-packages/commit/6dac6504fdbe85c61ffd7aad7c37cc1b52ebf6d4))
* **avm:** Bytecode avm control flow ([#4253](https://github.com/AztecProtocol/aztec-packages/issues/4253)) ([fb1d742](https://github.com/AztecProtocol/aztec-packages/commit/fb1d7420860a35e68b987e790abdaba18595219b)), closes [#4209](https://github.com/AztecProtocol/aztec-packages/issues/4209)
* **avm:** Bytecode parsing and proof generation ([#4191](https://github.com/AztecProtocol/aztec-packages/issues/4191)) ([6c70548](https://github.com/AztecProtocol/aztec-packages/commit/6c70548a98c8e01bc7925d98ece9a2eda4139f69)), closes [#3791](https://github.com/AztecProtocol/aztec-packages/issues/3791)
* **avm:** Environment getters ([#4203](https://github.com/AztecProtocol/aztec-packages/issues/4203)) ([60d2377](https://github.com/AztecProtocol/aztec-packages/commit/60d237771d129fc9a75e5f0806fd2d002c6e92c8))
* **avm:** Implement comparator opcodes ([#4232](https://github.com/AztecProtocol/aztec-packages/issues/4232)) ([973ff2f](https://github.com/AztecProtocol/aztec-packages/commit/973ff2f0ad11b78b5dcab1537abe5cb611af8db2))
* **avm:** Initial external calls ([#4194](https://github.com/AztecProtocol/aztec-packages/issues/4194)) ([d8aa966](https://github.com/AztecProtocol/aztec-packages/commit/d8aa9662730028824a4e7de2cd1dc6b95359ff21))
* **avm:** Link up storage ([#4150](https://github.com/AztecProtocol/aztec-packages/issues/4150)) ([3e86870](https://github.com/AztecProtocol/aztec-packages/commit/3e868705ca9b1ca05d962a7f5399a41ce470b120))
* **avm:** Revert instruction ([#4206](https://github.com/AztecProtocol/aztec-packages/issues/4206)) ([bd6e797](https://github.com/AztecProtocol/aztec-packages/commit/bd6e79727bb207978634d70df3bb213a222a4bb7))
* **avm:** Tagged memory ([#4213](https://github.com/AztecProtocol/aztec-packages/issues/4213)) ([e5ff2f6](https://github.com/AztecProtocol/aztec-packages/commit/e5ff2f60e20e9e85972515d845d5d45a0117409f))
* Aztec binary ([#3927](https://github.com/AztecProtocol/aztec-packages/issues/3927)) ([12356d9](https://github.com/AztecProtocol/aztec-packages/commit/12356d9e34994a239d5612798c1bc82fa3d26562))
* Contract classes and instances ([#4192](https://github.com/AztecProtocol/aztec-packages/issues/4192)) ([1858126](https://github.com/AztecProtocol/aztec-packages/commit/18581265dfd6d6aff42f9b90fd8425159d501f46)), closes [#4053](https://github.com/AztecProtocol/aztec-packages/issues/4053)
* Deserialize AztecAddress when contract's view function returns it in Aztec.js [#3641](https://github.com/AztecProtocol/aztec-packages/issues/3641) ([#4224](https://github.com/AztecProtocol/aztec-packages/issues/4224)) ([11f400f](https://github.com/AztecProtocol/aztec-packages/commit/11f400f6580d4c3fee52a5e97d84fcdf0dbad779))
* **docs:** DIP1: Extract Explanations ([#4228](https://github.com/AztecProtocol/aztec-packages/issues/4228)) ([3b25737](https://github.com/AztecProtocol/aztec-packages/commit/3b25737324e45bdfb49233f73065569301282cc0))
* **docs:** Historical trees docs ([#3895](https://github.com/AztecProtocol/aztec-packages/issues/3895)) ([8c3efba](https://github.com/AztecProtocol/aztec-packages/commit/8c3efba92f74905709760f3d8838df50076aaa92))
* **docs:** PXE docs ([#4021](https://github.com/AztecProtocol/aztec-packages/issues/4021)) ([a656034](https://github.com/AztecProtocol/aztec-packages/commit/a6560343fb333a6f725bc9d8c41e8594ea2e00b0))
* Implement bigint in Noir, using bigint opcodes ([#4198](https://github.com/AztecProtocol/aztec-packages/issues/4198)) ([3720415](https://github.com/AztecProtocol/aztec-packages/commit/3720415c8bf2b6f3292d961795eb13f08cb9dff5))
* Implement Embedded EC add and double opcodes ([#3982](https://github.com/AztecProtocol/aztec-packages/issues/3982)) ([ccb7bff](https://github.com/AztecProtocol/aztec-packages/commit/ccb7bff8e16ea9c8bc4bd48754db59857137507e))
* Limit exposed functions on note utils ([#4207](https://github.com/AztecProtocol/aztec-packages/issues/4207)) ([8338f39](https://github.com/AztecProtocol/aztec-packages/commit/8338f390fd826bc85f6789a1124ae34251a042dd))
* Nullifier key validation ([#4176](https://github.com/AztecProtocol/aztec-packages/issues/4176)) ([1c72c0d](https://github.com/AztecProtocol/aztec-packages/commit/1c72c0d2978af94cb147f143977557fa1540c419))
* Produce graph of internal Barretenberg dependencies ([#4225](https://github.com/AztecProtocol/aztec-packages/issues/4225)) ([88e7923](https://github.com/AztecProtocol/aztec-packages/commit/88e7923ed2ecd747b65f72c5955016c6a1b80b9f))
* Recursive folding and decider verifier for Protogalaxy ([#4156](https://github.com/AztecProtocol/aztec-packages/issues/4156)) ([9342048](https://github.com/AztecProtocol/aztec-packages/commit/93420480603b2dfa126e5bddb08cd768b7093352))
* Remove ec_double opcode ([#4210](https://github.com/AztecProtocol/aztec-packages/issues/4210)) ([75f26c4](https://github.com/AztecProtocol/aztec-packages/commit/75f26c4f2a9cf185891234eab6ec4f213d31fc50))
* Replace single bit range constraints with basic bool gates ([#4164](https://github.com/AztecProtocol/aztec-packages/issues/4164)) ([0a3553b](https://github.com/AztecProtocol/aztec-packages/commit/0a3553b10e02374843181901709933975dc36bb4))
* Updates singleton usage ([#4186](https://github.com/AztecProtocol/aztec-packages/issues/4186)) ([301f0e6](https://github.com/AztecProtocol/aztec-packages/commit/301f0e6d0832a999a31d0e9a5b4e8267474de6ab))


### Bug Fixes

* **avm:** Fix usage of Fr with tagged memory ([#4240](https://github.com/AztecProtocol/aztec-packages/issues/4240)) ([b82e70c](https://github.com/AztecProtocol/aztec-packages/commit/b82e70c61771c8a3cef4026dc522f2c99147180b))
* **bb:** .gitignore ([#4201](https://github.com/AztecProtocol/aztec-packages/issues/4201)) ([a56e418](https://github.com/AztecProtocol/aztec-packages/commit/a56e418b0fe90b77b7a9fd6bcb0e40cd15260fd6))
* **docs:** Add missing deps to token tutorial references ([#4265](https://github.com/AztecProtocol/aztec-packages/issues/4265)) ([d7e2d9c](https://github.com/AztecProtocol/aztec-packages/commit/d7e2d9c80262dd4dff714caac575785b3bf14482))
* Generic Honk dependencies ([#4239](https://github.com/AztecProtocol/aztec-packages/issues/4239)) ([382dfbe](https://github.com/AztecProtocol/aztec-packages/commit/382dfbed6aa4c6da7b3c897f8a5f9639843d7037))


### Miscellaneous

* Add note getter test to cci ([#4236](https://github.com/AztecProtocol/aztec-packages/issues/4236)) ([e1184ff](https://github.com/AztecProtocol/aztec-packages/commit/e1184ffc2f4ac2d2de0e8a106614bdd266a06cf4))
* **avm-simulator:** Cleanup, tags as first instruction constructor args ([#4244](https://github.com/AztecProtocol/aztec-packages/issues/4244)) ([e46b865](https://github.com/AztecProtocol/aztec-packages/commit/e46b865f4c53c8916061147c85fd298db31a384f))
* **avm:** Remove the state manager in favour of journal ([#4195](https://github.com/AztecProtocol/aztec-packages/issues/4195)) ([40f9324](https://github.com/AztecProtocol/aztec-packages/commit/40f9324a88fef3f762f5e6a21ccd3f200f8b8c4a))
* **bb:** Rearrange namespaces ([#4147](https://github.com/AztecProtocol/aztec-packages/issues/4147)) ([5de0a8e](https://github.com/AztecProtocol/aztec-packages/commit/5de0a8e8dce2483230cccb1d716613966089f2f6))
* Cleaning up circuits test setup ([#4235](https://github.com/AztecProtocol/aztec-packages/issues/4235)) ([fa6915a](https://github.com/AztecProtocol/aztec-packages/commit/fa6915a5f35c3b1b1283f122666d15f836ac682b)), closes [#4237](https://github.com/AztecProtocol/aztec-packages/issues/4237)
* Delete C++ PK circuits ([#4219](https://github.com/AztecProtocol/aztec-packages/issues/4219)) ([9136d32](https://github.com/AztecProtocol/aztec-packages/commit/9136d32268db350779d51e45884368be3a694220))
* Delete MemoryDB ([#4241](https://github.com/AztecProtocol/aztec-packages/issues/4241)) ([9e6250a](https://github.com/AztecProtocol/aztec-packages/commit/9e6250aacbe2d47aa71dee9fa5e43c66eec73e75))
* **docs:** Fix a few links to docs ([#4260](https://github.com/AztecProtocol/aztec-packages/issues/4260)) ([1c8ea49](https://github.com/AztecProtocol/aztec-packages/commit/1c8ea497fb1d64da64cb240917a60d57bd1efef8))
* **docs:** Fix autogen docs ([#4261](https://github.com/AztecProtocol/aztec-packages/issues/4261)) ([3b9927a](https://github.com/AztecProtocol/aztec-packages/commit/3b9927ab7ef2e7e50193bbfcb6ab4db66734e481))
* **docs:** Fix public and private storage not in docs ([#4257](https://github.com/AztecProtocol/aztec-packages/issues/4257)) ([48ceafd](https://github.com/AztecProtocol/aztec-packages/commit/48ceafd085f56464e65ae17a6f4931fdbdb575b6))
* **docs:** Fix token bridge tutorial ([#3935](https://github.com/AztecProtocol/aztec-packages/issues/3935)) ([84c9fdb](https://github.com/AztecProtocol/aztec-packages/commit/84c9fdbecf0b44a0897badcc12e5a9b333da4ec0))
* **docs:** Split contract storage pages ([#4202](https://github.com/AztecProtocol/aztec-packages/issues/4202)) ([1e05f33](https://github.com/AztecProtocol/aztec-packages/commit/1e05f33c58feb30f073e6dd5369cbed336343e54))
* Fix typo in yellow paper ([#4247](https://github.com/AztecProtocol/aztec-packages/issues/4247)) ([ac82e6b](https://github.com/AztecProtocol/aztec-packages/commit/ac82e6ba57a2a868e79399248fa2505c383e241c))
* Fixes test file from [#4205](https://github.com/AztecProtocol/aztec-packages/issues/4205) ([#4216](https://github.com/AztecProtocol/aztec-packages/issues/4216)) ([18a9b72](https://github.com/AztecProtocol/aztec-packages/commit/18a9b72dc9df95f517bb90fbcf0ebe45e7430d9a))
* Git subrepo pull (merge) noir ([#4252](https://github.com/AztecProtocol/aztec-packages/issues/4252)) ([80be57d](https://github.com/AztecProtocol/aztec-packages/commit/80be57d612ebdd0aac9384c4051e01823c9222da))
* Nuking old `BlockHeader` ([#4154](https://github.com/AztecProtocol/aztec-packages/issues/4154)) ([997791a](https://github.com/AztecProtocol/aztec-packages/commit/997791a06061eaab6c219948576565b457051ba2)), closes [#3937](https://github.com/AztecProtocol/aztec-packages/issues/3937) [#3564](https://github.com/AztecProtocol/aztec-packages/issues/3564) [#4134](https://github.com/AztecProtocol/aztec-packages/issues/4134)
* Remove flaky e2e p2p test ([#4181](https://github.com/AztecProtocol/aztec-packages/issues/4181)) ([688e4af](https://github.com/AztecProtocol/aztec-packages/commit/688e4afc2b8be8b7766baf5180c89fe985ebf6cd))
* Remove mandatory jsdoc ([#4180](https://github.com/AztecProtocol/aztec-packages/issues/4180)) ([9625b43](https://github.com/AztecProtocol/aztec-packages/commit/9625b4350a54c43f55841b508e3f86e7d7d6635b)), closes [#3860](https://github.com/AztecProtocol/aztec-packages/issues/3860)
* Remove stubbed docs ([#4196](https://github.com/AztecProtocol/aztec-packages/issues/4196)) ([25a4bc4](https://github.com/AztecProtocol/aztec-packages/commit/25a4bc490a53304110e7e1f79e99f4c8b7639164))
* Replace leveldb with lmdb for merkle trees ([#4119](https://github.com/AztecProtocol/aztec-packages/issues/4119)) ([84967b2](https://github.com/AztecProtocol/aztec-packages/commit/84967b246180e8ae94db98b32c4ed5439958d8d2)), closes [#3362](https://github.com/AztecProtocol/aztec-packages/issues/3362)
* Replace relative paths to noir-protocol-circuits ([a9839a8](https://github.com/AztecProtocol/aztec-packages/commit/a9839a8b9c0dce68d8af3b17e1140f669cd1a8d1))
* Replace relative paths to noir-protocol-circuits ([2ef6e56](https://github.com/AztecProtocol/aztec-packages/commit/2ef6e56052c6d35ef0cbc1e07cd3d06fca504747))
* Replace relative paths to noir-protocol-circuits ([426c17d](https://github.com/AztecProtocol/aztec-packages/commit/426c17d23981eaccf3a93c4a86d366a12141895f))
* Replace relative paths to noir-protocol-circuits ([12adb71](https://github.com/AztecProtocol/aztec-packages/commit/12adb71d3c7c03eca8b315d3d1192368d315ca45))
* Replace relative paths to noir-protocol-circuits ([fcd048d](https://github.com/AztecProtocol/aztec-packages/commit/fcd048d6a1ef6d84506f29d4e2aa49657fe6ca21))
* Unifying Header serialization accross domains ([#4230](https://github.com/AztecProtocol/aztec-packages/issues/4230)) ([92080a0](https://github.com/AztecProtocol/aztec-packages/commit/92080a02819e563680a18e38ce49d983275f2bf5))
* Update .gitrepo with correct parent hash ([#4279](https://github.com/AztecProtocol/aztec-packages/issues/4279)) ([9253c8a](https://github.com/AztecProtocol/aztec-packages/commit/9253c8a6944ef36e2d61ba6bf86953afcbb4966f))


### Documentation

* **bb:** How to use docker_interactive.sh ([#4220](https://github.com/AztecProtocol/aztec-packages/issues/4220)) ([f44c6b1](https://github.com/AztecProtocol/aztec-packages/commit/f44c6b173856331a6ca4d00d50436671735172a2))
* Update welcome page and dev pages ([#4143](https://github.com/AztecProtocol/aztec-packages/issues/4143)) ([d2a86ff](https://github.com/AztecProtocol/aztec-packages/commit/d2a86ff1f1eb79a47f6297f665c11e8aafcb584b))

## [0.20.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.19.0...aztec-packages-v0.20.0) (2024-01-22)


### ⚠ BREAKING CHANGES

* nullifier key ([#4166](https://github.com/AztecProtocol/aztec-packages/issues/4166))
* Unify ABIs between nargo and yarn-project ([#3989](https://github.com/AztecProtocol/aztec-packages/issues/3989))

### Features

* **avm:** Add internal jump and return, adjust control flow ([#4140](https://github.com/AztecProtocol/aztec-packages/issues/4140)) ([b77afb1](https://github.com/AztecProtocol/aztec-packages/commit/b77afb14c609cfc04663a318eecaa8cbd3b2fa85))
* **avm:** Better field arithmetic ([#4142](https://github.com/AztecProtocol/aztec-packages/issues/4142)) ([7308e31](https://github.com/AztecProtocol/aztec-packages/commit/7308e31c981ec7c03d0474811b2979e6f418f348))
* **avm:** Encode TS AVM instructions as bytecode, especially for testing ([#4115](https://github.com/AztecProtocol/aztec-packages/issues/4115)) ([de6e2ed](https://github.com/AztecProtocol/aztec-packages/commit/de6e2edc09b823542f45e14b45c32ec10894f6c7))
* **avm:** Improve interpreter errors and tests ([#4173](https://github.com/AztecProtocol/aztec-packages/issues/4173)) ([f0fb594](https://github.com/AztecProtocol/aztec-packages/commit/f0fb5942386e2e091cb6d1b23108ac74c1c6b75d))
* Benchmark commit function ([#4178](https://github.com/AztecProtocol/aztec-packages/issues/4178)) ([ea84085](https://github.com/AztecProtocol/aztec-packages/commit/ea840857d8134c9af6f233b414f6d990cd2abd6d))
* Goblin acir composer ([#4112](https://github.com/AztecProtocol/aztec-packages/issues/4112)) ([5e85b92](https://github.com/AztecProtocol/aztec-packages/commit/5e85b92f48bc31fe55315de9f45c4907e417cb6a))
* Nullifier key ([#4166](https://github.com/AztecProtocol/aztec-packages/issues/4166)) ([7c07665](https://github.com/AztecProtocol/aztec-packages/commit/7c076653169771223a378f6c01bd9d3e3aafb682))
* **public-vm:** Avm journal ([#3945](https://github.com/AztecProtocol/aztec-packages/issues/3945)) ([5658468](https://github.com/AztecProtocol/aztec-packages/commit/56584683340cd29b26adbcc60d3cd58fe889e8ad))
* Publish block body separately ([#4118](https://github.com/AztecProtocol/aztec-packages/issues/4118)) ([a04e1e3](https://github.com/AztecProtocol/aztec-packages/commit/a04e1e351fe0b42dfa6ef7d1894dffbcff19b187)), closes [#3944](https://github.com/AztecProtocol/aztec-packages/issues/3944)
* Unify ABIs between nargo and yarn-project ([#3989](https://github.com/AztecProtocol/aztec-packages/issues/3989)) ([d083438](https://github.com/AztecProtocol/aztec-packages/commit/d0834380a749a48d31e3075f831f3279f18fc01e))
* Update noir ([#4082](https://github.com/AztecProtocol/aztec-packages/issues/4082)) ([0e6037a](https://github.com/AztecProtocol/aztec-packages/commit/0e6037ad48fc1eb8ab3524d64755f4f27f64bf36))
* Updating L2 Block encoding and `Rollup.process` function ([#4015](https://github.com/AztecProtocol/aztec-packages/issues/4015)) ([2d8eb37](https://github.com/AztecProtocol/aztec-packages/commit/2d8eb37edbc9164639fe6d20140364288e2b72a9)), closes [#3936](https://github.com/AztecProtocol/aztec-packages/issues/3936) [#4010](https://github.com/AztecProtocol/aztec-packages/issues/4010) [#4011](https://github.com/AztecProtocol/aztec-packages/issues/4011)


### Bug Fixes

* Bb.js version in yarn lockfile ([7b96760](https://github.com/AztecProtocol/aztec-packages/commit/7b96760f7d201d984bab885b996b218bd427be22))
* **build:** Publish bb.js from CCI ([#4151](https://github.com/AztecProtocol/aztec-packages/issues/4151)) ([09dbfcd](https://github.com/AztecProtocol/aztec-packages/commit/09dbfcd7e8d3b15cf79686eea44a4032f2aa4bbb))
* Make CMake version warning fatal ([#4144](https://github.com/AztecProtocol/aztec-packages/issues/4144)) ([b1443fa](https://github.com/AztecProtocol/aztec-packages/commit/b1443faf9d8f308dbad6d0aa365b1feb8385557d))
* Misleading error message in `PublicState::read` ([#4149](https://github.com/AztecProtocol/aztec-packages/issues/4149)) ([fa4d919](https://github.com/AztecProtocol/aztec-packages/commit/fa4d919d25b5253d1f39b7f2db183771f461fe1b))
* Nargo destination path in bootstrap cache ([#4103](https://github.com/AztecProtocol/aztec-packages/issues/4103)) ([4901309](https://github.com/AztecProtocol/aztec-packages/commit/490130979a5d3a3e703c7e28417835aa86fa8cd7))
* Reinstate Ultra arith rec verifier test ([#3886](https://github.com/AztecProtocol/aztec-packages/issues/3886)) ([995973b](https://github.com/AztecProtocol/aztec-packages/commit/995973b0226ddd7ae4cb5c3501859bec10f4eb93))
* Upload_benchmarks_to_s3.sh missing exit ([#4046](https://github.com/AztecProtocol/aztec-packages/issues/4046)) ([52a9327](https://github.com/AztecProtocol/aztec-packages/commit/52a93279e43ae6780e8bfc253ee0570a443ed472))


### Miscellaneous

* Archiver store ([#3966](https://github.com/AztecProtocol/aztec-packages/issues/3966)) ([af2be87](https://github.com/AztecProtocol/aztec-packages/commit/af2be878b49aceb668480e5a291aed7dea5319ba))
* **avm:** List avm opcodes in a (enum =&gt; class) map in TS ([#4113](https://github.com/AztecProtocol/aztec-packages/issues/4113)) ([dee564a](https://github.com/AztecProtocol/aztec-packages/commit/dee564a16293ff8f0aa2c6ba9fb0d1d6235ba251))
* **bb:** More concise namespaces, plookup =&gt; bb::plookup ([#4146](https://github.com/AztecProtocol/aztec-packages/issues/4146)) ([14d39ed](https://github.com/AztecProtocol/aztec-packages/commit/14d39edbe1a6753849581a664184d4e98baf923d))
* **bb:** Namespace plonk::stdlib =&gt; stdlib ([#4117](https://github.com/AztecProtocol/aztec-packages/issues/4117)) ([cd2f67f](https://github.com/AztecProtocol/aztec-packages/commit/cd2f67f5cbc471b9120f7c7070b96ba0d4994865))
* **bb:** Namespace proof_system=&gt;bb ([#4116](https://github.com/AztecProtocol/aztec-packages/issues/4116)) ([7438db3](https://github.com/AztecProtocol/aztec-packages/commit/7438db31b29860aa2c0af54afa8413711a24e1eb))
* **docs:** Aztec-up doesnt need `latest`, remove warnings around sandbox/cli npm pkgs ([#4138](https://github.com/AztecProtocol/aztec-packages/issues/4138)) ([2bbf7a9](https://github.com/AztecProtocol/aztec-packages/commit/2bbf7a919172bea9842b4e65129160e0e4ee0050))
* **docs:** Update js release notes for 0.18.0 ([#4051](https://github.com/AztecProtocol/aztec-packages/issues/4051)) ([bdbe963](https://github.com/AztecProtocol/aztec-packages/commit/bdbe963b114813a49017daebe5b34757692328ce))
* **docs:** Update lsp install instructions ([#4110](https://github.com/AztecProtocol/aztec-packages/issues/4110)) ([3138816](https://github.com/AztecProtocol/aztec-packages/commit/3138816b8b7480bd85eab8739e1d0bc72a5a5361)), closes [#4098](https://github.com/AztecProtocol/aztec-packages/issues/4098)
* Dont mirror build-system mirror_repos.yml ([#4067](https://github.com/AztecProtocol/aztec-packages/issues/4067)) ([04f8e0d](https://github.com/AztecProtocol/aztec-packages/commit/04f8e0dfde5a3d4f54621726891aedc071212a8a))
* Fixes many broken urls ([#4109](https://github.com/AztecProtocol/aztec-packages/issues/4109)) ([41ae75c](https://github.com/AztecProtocol/aztec-packages/commit/41ae75cdee6285729551965972e8cb039ff3045a))
* Remove dependency cycles in `sequencer-client` ([#4017](https://github.com/AztecProtocol/aztec-packages/issues/4017)) ([fe4538b](https://github.com/AztecProtocol/aztec-packages/commit/fe4538b36c1cdb19c0c7245f7a73c9f5ee131a4a))
* Remove lodash times in favor of foundation fn ([#3877](https://github.com/AztecProtocol/aztec-packages/issues/3877)) ([a10eef0](https://github.com/AztecProtocol/aztec-packages/commit/a10eef0a77cb52e00ffedb10ed325bd63fa0c971))
* Remove mutex dependency ([#4160](https://github.com/AztecProtocol/aztec-packages/issues/4160)) ([3b82be0](https://github.com/AztecProtocol/aztec-packages/commit/3b82be0f266c838c823bbe26cfea99337d7180a9))
* Remove unnecessary computation ([#4133](https://github.com/AztecProtocol/aztec-packages/issues/4133)) ([f35bdb8](https://github.com/AztecProtocol/aztec-packages/commit/f35bdb84722dbd01535da5542a0ec7c1fb96e5c7))
* Remove unused noir-version json ([#4105](https://github.com/AztecProtocol/aztec-packages/issues/4105)) ([afca819](https://github.com/AztecProtocol/aztec-packages/commit/afca819166b9b5882b5d94062ac50cbb8dc590fb))
* Remove unwanted submodules ([#4085](https://github.com/AztecProtocol/aztec-packages/issues/4085)) ([dda7c9c](https://github.com/AztecProtocol/aztec-packages/commit/dda7c9c4fa8da54d28b99b1cf601328030485503))
* Replace relative paths to noir-protocol-circuits ([59feeb5](https://github.com/AztecProtocol/aztec-packages/commit/59feeb5ed55ab022b99b83a31cf89bdabc0003c5))
* Replace relative paths to noir-protocol-circuits ([44d9136](https://github.com/AztecProtocol/aztec-packages/commit/44d91361274a54f0e59f0ad5f8869f3f3aa49c2b))
* Replace relative paths to noir-protocol-circuits ([84b0bad](https://github.com/AztecProtocol/aztec-packages/commit/84b0bad61a5fc751fdf01ae907bb9b7df3bece7b))
* Simplify and fix DocsExample contract, e2e singleton + codegen to not show internal methods ([#4169](https://github.com/AztecProtocol/aztec-packages/issues/4169)) ([38d262e](https://github.com/AztecProtocol/aztec-packages/commit/38d262eddd4fca80a7726d17303fc084257ad460))
* Update noir ([#4168](https://github.com/AztecProtocol/aztec-packages/issues/4168)) ([d40ad06](https://github.com/AztecProtocol/aztec-packages/commit/d40ad063606219119b41ba3acc883e62245bd035))


### Documentation

* Update migration notes ([#4175](https://github.com/AztecProtocol/aztec-packages/issues/4175)) ([dbc8174](https://github.com/AztecProtocol/aztec-packages/commit/dbc8174f010f15422a9485b567b60834a0a4aa2f))
* **yellow-paper:** Update circuit sections for nullifier keys and static calls ([#4155](https://github.com/AztecProtocol/aztec-packages/issues/4155)) ([ed71a57](https://github.com/AztecProtocol/aztec-packages/commit/ed71a573bca18912e4590a5f8c8044778ad439c5))
* **yellowpaper:** Refresh of avm instruction set ([#4081](https://github.com/AztecProtocol/aztec-packages/issues/4081)) ([52162ee](https://github.com/AztecProtocol/aztec-packages/commit/52162eed9ace2726c143a34520115c8530876187))

## [0.19.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.18.0...aztec-packages-v0.19.0) (2024-01-17)


### ⚠ BREAKING CHANGES

* Start witness of ACIR generated by Noir start at zero not one ([#3961](https://github.com/AztecProtocol/aztec-packages/issues/3961))

### Features

* Avm executor - layout and groundwork ([#3928](https://github.com/AztecProtocol/aztec-packages/issues/3928)) ([432f6d1](https://github.com/AztecProtocol/aztec-packages/commit/432f6d1ffa56754e0bbcfc99368e417196963efe))
* Remove dangerous function ([#4007](https://github.com/AztecProtocol/aztec-packages/issues/4007)) ([b3790eb](https://github.com/AztecProtocol/aztec-packages/commit/b3790ebfc3f6f62a30dc1b222b4cafaef8effb98))
* Track hashing ([#4030](https://github.com/AztecProtocol/aztec-packages/issues/4030)) ([09090e8](https://github.com/AztecProtocol/aztec-packages/commit/09090e877f1535c5badc5cb3740653f91646391e))


### Bug Fixes

* Do not publish empty contract data ([#4022](https://github.com/AztecProtocol/aztec-packages/issues/4022)) ([3ce3ef7](https://github.com/AztecProtocol/aztec-packages/commit/3ce3ef71a8a8dc2101a6c73eb0a90940855950f6)), closes [#2970](https://github.com/AztecProtocol/aztec-packages/issues/2970)
* Fix various warnings in `noir-protocol-circuits` ([#4048](https://github.com/AztecProtocol/aztec-packages/issues/4048)) ([470d046](https://github.com/AztecProtocol/aztec-packages/commit/470d046fe54c8b4e76d20ca3dbe8e128355b384f))
* Start witness of ACIR generated by Noir start at zero not one ([#3961](https://github.com/AztecProtocol/aztec-packages/issues/3961)) ([4cdc096](https://github.com/AztecProtocol/aztec-packages/commit/4cdc0963777de138bf5275dd657a738ae6f020d3))


### Miscellaneous

* Barretenberg =&gt; bb namespace shortening ([#4066](https://github.com/AztecProtocol/aztec-packages/issues/4066)) ([e6b66b8](https://github.com/AztecProtocol/aztec-packages/commit/e6b66b856db498e6fc465212f3645cf2c196c31a))
* Keep track of blocks emitting/cancelling messages ([#4028](https://github.com/AztecProtocol/aztec-packages/issues/4028)) ([d250a0e](https://github.com/AztecProtocol/aztec-packages/commit/d250a0e4e34d435f888627dd768a4f5367d060b5))
* Resume CCI runs on release-please branch ([#4034](https://github.com/AztecProtocol/aztec-packages/issues/4034)) ([fd58251](https://github.com/AztecProtocol/aztec-packages/commit/fd582515027f9e82dee877304bc7333f95554704))
* Sync from noir repo ([#4047](https://github.com/AztecProtocol/aztec-packages/issues/4047)) ([328c581](https://github.com/AztecProtocol/aztec-packages/commit/328c5812eb42244570293c22884634c439dd26f9))


### Documentation

* **yellowpaper:** Update to AVM section after Zac's feedback ([#4008](https://github.com/AztecProtocol/aztec-packages/issues/4008)) ([87127ca](https://github.com/AztecProtocol/aztec-packages/commit/87127ca2c22e49b06b46bb3597b6a2fe184b74d0))

## [0.18.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.17.0...aztec-packages-v0.18.0) (2024-01-16)


### ⚠ BREAKING CHANGES

* Remove `Directive::Quotient` ([#4019](https://github.com/AztecProtocol/aztec-packages/issues/4019))
* define key type in maps ([#3841](https://github.com/AztecProtocol/aztec-packages/issues/3841))
* implement keccakf1600 in brillig ([#3914](https://github.com/AztecProtocol/aztec-packages/issues/3914))
* add blake3 opcode to brillig ([#3913](https://github.com/AztecProtocol/aztec-packages/issues/3913))
* Remove opcode supported from the backend ([#3889](https://github.com/AztecProtocol/aztec-packages/issues/3889))

### Features

* `PartialStateReference` and `StateReference` structs ([#3827](https://github.com/AztecProtocol/aztec-packages/issues/3827)) ([3ba0369](https://github.com/AztecProtocol/aztec-packages/commit/3ba03695c4e5aff2ae20a10b4b9ab6fd38a815d7))
* `StateDiffHints` ([#3919](https://github.com/AztecProtocol/aztec-packages/issues/3919)) ([8774795](https://github.com/AztecProtocol/aztec-packages/commit/87747959ee02d0c4b60e45c4cef0c9718ec6904a)), closes [#3916](https://github.com/AztecProtocol/aztec-packages/issues/3916)
* Acir cleanup ([#3845](https://github.com/AztecProtocol/aztec-packages/issues/3845)) ([390b84c](https://github.com/AztecProtocol/aztec-packages/commit/390b84ced39ea8ed76c291019e63d18a772f7c3c))
* Add ACIR opcodes for ECADD and ECDOUBLE ([#3878](https://github.com/AztecProtocol/aztec-packages/issues/3878)) ([537630f](https://github.com/AztecProtocol/aztec-packages/commit/537630ffe1b3747a03aa821687e33db04b1fe3ad))
* Add blake3 opcode to brillig ([#3913](https://github.com/AztecProtocol/aztec-packages/issues/3913)) ([34fad0a](https://github.com/AztecProtocol/aztec-packages/commit/34fad0a76c58139b4b56f372aa6f39f897286b3a))
* Add data availability oracle ([#3897](https://github.com/AztecProtocol/aztec-packages/issues/3897)) ([5441753](https://github.com/AztecProtocol/aztec-packages/commit/5441753efd8347318a47abfbed422c43c7cf819b)), closes [#3890](https://github.com/AztecProtocol/aztec-packages/issues/3890)
* Add str support for args + add name/symbol/decimal to token ([#3862](https://github.com/AztecProtocol/aztec-packages/issues/3862)) ([0bf5d8c](https://github.com/AztecProtocol/aztec-packages/commit/0bf5d8c154d53882d67a11f6d515788c797f069f))
* Bench bb in pr's, docker shell utils ([#3561](https://github.com/AztecProtocol/aztec-packages/issues/3561)) ([5408919](https://github.com/AztecProtocol/aztec-packages/commit/54089190a4532988cec9f88d199263aeafa2c2f3))
* Benchmark protogalaxy prover ([#3958](https://github.com/AztecProtocol/aztec-packages/issues/3958)) ([5843722](https://github.com/AztecProtocol/aztec-packages/commit/5843722ff5e888bf858016c6d005af37fac42e1c))
* Benchmarks for basic functionality and IPA improvements ([#4004](https://github.com/AztecProtocol/aztec-packages/issues/4004)) ([fd1f619](https://github.com/AztecProtocol/aztec-packages/commit/fd1f619f443916c172b6311aa71a84153145ef4d))
* Body hash as buffer in TS + minor cleanup ([#4012](https://github.com/AztecProtocol/aztec-packages/issues/4012)) ([e28a6bf](https://github.com/AztecProtocol/aztec-packages/commit/e28a6bf14cee1f7bade14337c74ecdcba1350899))
* Bootstrap cache v2 ([#3876](https://github.com/AztecProtocol/aztec-packages/issues/3876)) ([331598d](https://github.com/AztecProtocol/aztec-packages/commit/331598d369ab9bb91dcc48d50bdd8df0684f0b79))
* Counters in private functions ([#3850](https://github.com/AztecProtocol/aztec-packages/issues/3850)) ([23bbf75](https://github.com/AztecProtocol/aztec-packages/commit/23bbf7559dc6c5fbacff4d67c0e899c944750b75))
* Generate json blocks for tests ([#3923](https://github.com/AztecProtocol/aztec-packages/issues/3923)) ([a09fd2a](https://github.com/AztecProtocol/aztec-packages/commit/a09fd2a92c540d65abea89113abd9a217b3ee7e0))
* Implement keccakf1600 in brillig ([#3914](https://github.com/AztecProtocol/aztec-packages/issues/3914)) ([a182381](https://github.com/AztecProtocol/aztec-packages/commit/a18238180cbd6c71f75fcfcb1a093ac29c839aeb))
* Parallel IPA ([#3882](https://github.com/AztecProtocol/aztec-packages/issues/3882)) ([7002a33](https://github.com/AztecProtocol/aztec-packages/commit/7002a332da3bb9a75d5164a068a2bd9ea1ad211a))
* Pil lookups w/ xor table example ([#3880](https://github.com/AztecProtocol/aztec-packages/issues/3880)) ([544d24e](https://github.com/AztecProtocol/aztec-packages/commit/544d24e419a604c4720988315239e365f06205b1))
* Poseidon2 stdlib impl ([#3551](https://github.com/AztecProtocol/aztec-packages/issues/3551)) ([50b4a72](https://github.com/AztecProtocol/aztec-packages/commit/50b4a728b4c20503f6ab56c07feaa29d767cec10))
* Protogalaxy Decider and complete folding tests ([#3657](https://github.com/AztecProtocol/aztec-packages/issues/3657)) ([cfdaf9c](https://github.com/AztecProtocol/aztec-packages/commit/cfdaf9c1980356764a0bed88bc01358b8e807bd0))
* Reexport `protocol_types` from `aztec-nr` ([#3926](https://github.com/AztecProtocol/aztec-packages/issues/3926)) ([9bd22f7](https://github.com/AztecProtocol/aztec-packages/commit/9bd22f7dc0ddf105b15737a81c3f4bfb0f3ad408))
* Relations vs widgets benchmarking ([#3931](https://github.com/AztecProtocol/aztec-packages/issues/3931)) ([3af64ef](https://github.com/AztecProtocol/aztec-packages/commit/3af64eff3a32922849cb0fd1977ee89a6ea7cd07))
* Remove opcode supported from the backend ([#3889](https://github.com/AztecProtocol/aztec-packages/issues/3889)) ([1fd135c](https://github.com/AztecProtocol/aztec-packages/commit/1fd135cb61a0b0419a339743c2a4fa9890d62720))
* Reorganize acir composer ([#3957](https://github.com/AztecProtocol/aztec-packages/issues/3957)) ([e6232e8](https://github.com/AztecProtocol/aztec-packages/commit/e6232e8ded1fa731565b17b77b0b2be80b2ef6e2))
* Replace bitwise ORs in `U256:from_bytes32` with addition ([#3947](https://github.com/AztecProtocol/aztec-packages/issues/3947)) ([efd7660](https://github.com/AztecProtocol/aztec-packages/commit/efd7660f41ddf3474fccee26d56d8e1458f423cf))
* Standalone calldata test ([#3842](https://github.com/AztecProtocol/aztec-packages/issues/3842)) ([7353a35](https://github.com/AztecProtocol/aztec-packages/commit/7353a358aa3f364d1d31fd00c73a9e1a4b6dff4e))
* Sync with main noir repo ([#3939](https://github.com/AztecProtocol/aztec-packages/issues/3939)) ([69c7e99](https://github.com/AztecProtocol/aztec-packages/commit/69c7e99bf53893685fe838763a53664b095fdabf))
* Update noir ([#3979](https://github.com/AztecProtocol/aztec-packages/issues/3979)) ([271de71](https://github.com/AztecProtocol/aztec-packages/commit/271de71e7cc9402bd85ea0ff22811f9a6b47403a))
* Verify state hash is correct before publishing to L1 ([#3915](https://github.com/AztecProtocol/aztec-packages/issues/3915)) ([a53c261](https://github.com/AztecProtocol/aztec-packages/commit/a53c26139e2d02de75faa761209cdb09d0a1b94e))


### Bug Fixes

* **aztec-nr:** Broken nargo url ([#3925](https://github.com/AztecProtocol/aztec-packages/issues/3925)) ([034bc30](https://github.com/AztecProtocol/aztec-packages/commit/034bc300fa4688958776afbc83276bacca7ce6ad))
* Back out the buggy branch condition ([#3988](https://github.com/AztecProtocol/aztec-packages/issues/3988)) ([9f61ed1](https://github.com/AztecProtocol/aztec-packages/commit/9f61ed1ff4cb696032816ba9c752b070b1021bfb))
* Bb unnecessary env var ([#3901](https://github.com/AztecProtocol/aztec-packages/issues/3901)) ([f127e5a](https://github.com/AztecProtocol/aztec-packages/commit/f127e5a4176d00e641c8f2308ebf105f415cb914))
* Broken links by paterson1 ([#3902](https://github.com/AztecProtocol/aztec-packages/issues/3902)) ([6783aaa](https://github.com/AztecProtocol/aztec-packages/commit/6783aaa486a634fd81b1f16e931ecc30184dd278))
* **cli:** Unbox command should run as release ([#3974](https://github.com/AztecProtocol/aztec-packages/issues/3974)) ([80c3805](https://github.com/AztecProtocol/aztec-packages/commit/80c3805320cb95b46f7595e8604b785ae48011b7))
* **cli:** Unbox should set up the accounts package ([#3972](https://github.com/AztecProtocol/aztec-packages/issues/3972)) ([065e988](https://github.com/AztecProtocol/aztec-packages/commit/065e988f0f6a7106f74c66a709d995ec3bc88207))
* Docusaurus start command ([#3968](https://github.com/AztecProtocol/aztec-packages/issues/3968)) ([87c0b07](https://github.com/AztecProtocol/aztec-packages/commit/87c0b07ca9fa75f912ac6a454f19b8ebbde606a2))
* Dont spam logs with yarn install ([#4027](https://github.com/AztecProtocol/aztec-packages/issues/4027)) ([949c5ab](https://github.com/AztecProtocol/aztec-packages/commit/949c5abf1df399f691f17c19fab64f0e36476219))
* Fix compilation of `token` box ([#3981](https://github.com/AztecProtocol/aztec-packages/issues/3981)) ([0f994d0](https://github.com/AztecProtocol/aztec-packages/commit/0f994d063622ad20397c806cdb62fbfb1191f5fe))
* Mirror_noir_subrepo.yml erroring ([#3954](https://github.com/AztecProtocol/aztec-packages/issues/3954)) ([2ac1b9c](https://github.com/AztecProtocol/aztec-packages/commit/2ac1b9c82ece6edbb3e91e998c2cb5693277e8d9))
* Quote the glob string for ignore branches for protocol-circuits-gate-diff ([#3990](https://github.com/AztecProtocol/aztec-packages/issues/3990)) ([a43889d](https://github.com/AztecProtocol/aztec-packages/commit/a43889de4191d532a58e1b497cda6965bcc3fbe6))
* Reprocess notes in pxe when a new contract is added ([#3867](https://github.com/AztecProtocol/aztec-packages/issues/3867)) ([ccbff99](https://github.com/AztecProtocol/aztec-packages/commit/ccbff99386b5ed8b9432e452f43778953f8c47dd))
* Segment tree insertion stats by depth ([#4029](https://github.com/AztecProtocol/aztec-packages/issues/4029)) ([2787bae](https://github.com/AztecProtocol/aztec-packages/commit/2787baeb4cbbd1805fab24bae8b6842b12cb488c))
* Store blockhash alongside blocks ([#3950](https://github.com/AztecProtocol/aztec-packages/issues/3950)) ([12b07fa](https://github.com/AztecProtocol/aztec-packages/commit/12b07fa76b00414493e9631dc8b11b36df8ca0c4)), closes [#3870](https://github.com/AztecProtocol/aztec-packages/issues/3870)
* Swap branch exclusion for release-please on report gates diff workflow ([#3994](https://github.com/AztecProtocol/aztec-packages/issues/3994)) ([70b2ffd](https://github.com/AztecProtocol/aztec-packages/commit/70b2ffd7e94461a6ae7986c69a8651c4d43f4114))
* Typos in authwit.md by czepluch ([#3921](https://github.com/AztecProtocol/aztec-packages/issues/3921)) ([4b9d0f4](https://github.com/AztecProtocol/aztec-packages/commit/4b9d0f4bcf14d26cd3bdbeb27f69183e1e7d9cd5))
* Yellowpaper docusaurus start command ([#3969](https://github.com/AztecProtocol/aztec-packages/issues/3969)) ([4977cbc](https://github.com/AztecProtocol/aztec-packages/commit/4977cbcac58e82073653ce6a792fb08e0305fe5d))


### Miscellaneous

* Add a link back to `aztec-packages` in noir sync PR ([#4018](https://github.com/AztecProtocol/aztec-packages/issues/4018)) ([7d89f3b](https://github.com/AztecProtocol/aztec-packages/commit/7d89f3b674c33090e85bff3a1446ba2207c94921))
* Catch up note processors could be synced more efficiently ([#3933](https://github.com/AztecProtocol/aztec-packages/issues/3933)) ([df54f33](https://github.com/AztecProtocol/aztec-packages/commit/df54f3305d7054fc5c89673ef6a784286d3e6283))
* **ci:** Require boxes CI to succeed ([#3983](https://github.com/AztecProtocol/aztec-packages/issues/3983)) ([93cbea1](https://github.com/AztecProtocol/aztec-packages/commit/93cbea1eeb40a705eb21d9571746ee71825bcf7c))
* Cleanup duplicated methods for structs after traits ([#3912](https://github.com/AztecProtocol/aztec-packages/issues/3912)) ([60b59da](https://github.com/AztecProtocol/aztec-packages/commit/60b59daca5957d0b1b0ec4b727f1328247d37d4e))
* Cleanup sandbox dependent tests ([#3861](https://github.com/AztecProtocol/aztec-packages/issues/3861)) ([158c5be](https://github.com/AztecProtocol/aztec-packages/commit/158c5be0f166a89effe044acb120d260f538faf7))
* Codegen acir opcodes after renaming arithmetic to assertzero ([#3896](https://github.com/AztecProtocol/aztec-packages/issues/3896)) ([c710ce1](https://github.com/AztecProtocol/aztec-packages/commit/c710ce19eaa3fbcf7c83957e7341a6ca10677ef1))
* Define key type in maps ([#3841](https://github.com/AztecProtocol/aztec-packages/issues/3841)) ([cf15adb](https://github.com/AztecProtocol/aztec-packages/commit/cf15adb89dca172a4c4899050b3d17b6091cf1c5))
* Delete the compiler from `noir-compiler` ([#3959](https://github.com/AztecProtocol/aztec-packages/issues/3959)) ([9aa0986](https://github.com/AztecProtocol/aztec-packages/commit/9aa0986633391c0db343c6c6ca84b1405ba44d7b))
* Deploy canary release of bb.js to npm via CCI ([#3917](https://github.com/AztecProtocol/aztec-packages/issues/3917)) ([bdeb10c](https://github.com/AztecProtocol/aztec-packages/commit/bdeb10c43135e0ef5a1bd1b086678b728b3b9e02))
* Do not fail jq dependencies check for package with no deps ([#3894](https://github.com/AztecProtocol/aztec-packages/issues/3894)) ([22c65bf](https://github.com/AztecProtocol/aztec-packages/commit/22c65bf74f263940fa8de90c1d960aa3c67bb126))
* Do not pass redundant txNullifier when computing notes ([#3943](https://github.com/AztecProtocol/aztec-packages/issues/3943)) ([9355cda](https://github.com/AztecProtocol/aztec-packages/commit/9355cdac7b46be0bb4bfe8d06db7e7b7bcbec547))
* Do not run CCI on release-please branches ([#3984](https://github.com/AztecProtocol/aztec-packages/issues/3984)) ([c38dbd2](https://github.com/AztecProtocol/aztec-packages/commit/c38dbd2fe13dfcf019da3877c17ca8d148f45233))
* **docs:** Fix ts code in token bridge tutorial ([#3888](https://github.com/AztecProtocol/aztec-packages/issues/3888)) ([f53f8ed](https://github.com/AztecProtocol/aztec-packages/commit/f53f8ed8b844c0d1188f0db646964f46475c1810))
* **docs:** Move map keys to new release in migration guide ([#3977](https://github.com/AztecProtocol/aztec-packages/issues/3977)) ([6356d94](https://github.com/AztecProtocol/aztec-packages/commit/6356d94be43bdaa32753202009c4bfa5387db9d0))
* **docs:** Update missing @aztec/accounts missing import in testing.md ([#3903](https://github.com/AztecProtocol/aztec-packages/issues/3903)) ([755668a](https://github.com/AztecProtocol/aztec-packages/commit/755668a243f78ccf9d28a4a2585efa3970ac6bfb))
* Document `witness_buf_to_witness_data` ([#3940](https://github.com/AztecProtocol/aztec-packages/issues/3940)) ([fbaa726](https://github.com/AztecProtocol/aztec-packages/commit/fbaa72641c50cc7f05712e266416f12c4edf8fe9))
* End to end test node & pxe persistence ([#3911](https://github.com/AztecProtocol/aztec-packages/issues/3911)) ([6164ccd](https://github.com/AztecProtocol/aztec-packages/commit/6164ccd4a54aca6eb175c90b9521676eec6ccce7))
* Enforce immutable yarn installs in CI ([#3964](https://github.com/AztecProtocol/aztec-packages/issues/3964)) ([f3104ac](https://github.com/AztecProtocol/aztec-packages/commit/f3104acdb135aff00b275ba35f41cbb8cfb4c84f))
* Fix rust tests ([#3963](https://github.com/AztecProtocol/aztec-packages/issues/3963)) ([a907c3b](https://github.com/AztecProtocol/aztec-packages/commit/a907c3b9d21c653162161091e8c97e924a0e8542))
* Fix sidebar index links ([#3942](https://github.com/AztecProtocol/aztec-packages/issues/3942)) ([984f1f8](https://github.com/AztecProtocol/aztec-packages/commit/984f1f8fedcb8294615a6e62b479a44e51f8cfd4))
* Git subrepo commit (merge) noir ([#3955](https://github.com/AztecProtocol/aztec-packages/issues/3955)) ([2c2bc69](https://github.com/AztecProtocol/aztec-packages/commit/2c2bc69ba4d8bc94d1fe598252c9b509add98825))
* Introduce EventSelector class ([#3960](https://github.com/AztecProtocol/aztec-packages/issues/3960)) ([7315f2c](https://github.com/AztecProtocol/aztec-packages/commit/7315f2c3aa44ac26a7ecf9378cb1cf1af97985af))
* Investigate P2P test ([#3929](https://github.com/AztecProtocol/aztec-packages/issues/3929)) ([0fca2c4](https://github.com/AztecProtocol/aztec-packages/commit/0fca2c4084f2033f86d0df60536cba1341538dbd))
* Move types to circuit-types ([#3967](https://github.com/AztecProtocol/aztec-packages/issues/3967)) ([f81b7c0](https://github.com/AztecProtocol/aztec-packages/commit/f81b7c050a22e326de3e2009dced856e18c1686f))
* Persistence uses TokenContract ([#3930](https://github.com/AztecProtocol/aztec-packages/issues/3930)) ([1a052c4](https://github.com/AztecProtocol/aztec-packages/commit/1a052c4e0651cb4f1572d928343375c7d77781d4))
* Pull in noir from upstream ([#3904](https://github.com/AztecProtocol/aztec-packages/issues/3904)) ([ab07e7e](https://github.com/AztecProtocol/aztec-packages/commit/ab07e7e96c3ec267d0e555049a5039e8bcba3dd1))
* Recreated types package without circuits.js dependency ([#3970](https://github.com/AztecProtocol/aztec-packages/issues/3970)) ([fc1d539](https://github.com/AztecProtocol/aztec-packages/commit/fc1d5399f24edafdc10cf9810f0cc6268008a3d8))
* Refactor serialisation functions and sibling paths ([#3980](https://github.com/AztecProtocol/aztec-packages/issues/3980)) ([1a936fc](https://github.com/AztecProtocol/aztec-packages/commit/1a936fcc373ec930f73ef0d6f060af1c8262dadb))
* Remove 'extern template's, expand macros ([#3953](https://github.com/AztecProtocol/aztec-packages/issues/3953)) ([5fe9908](https://github.com/AztecProtocol/aztec-packages/commit/5fe99085963cec32a2d411b95ab8887578a90253))
* Remove `Directive::Quotient` ([#4019](https://github.com/AztecProtocol/aztec-packages/issues/4019)) ([824d76f](https://github.com/AztecProtocol/aztec-packages/commit/824d76f363180821678238f1474a00520f781758))
* Remove sandbox and cli npm pkgs ([#3567](https://github.com/AztecProtocol/aztec-packages/issues/3567)) ([a8cf1bf](https://github.com/AztecProtocol/aztec-packages/commit/a8cf1bf2a49fc2a7b60aff91b23d77a096b54c2c))
* Reorganize benchmarks ([#3909](https://github.com/AztecProtocol/aztec-packages/issues/3909)) ([730766b](https://github.com/AztecProtocol/aztec-packages/commit/730766b07d9521c0ec6c0606042b506edbc5db48))
* Replace `AztecU128` with `U128` ([#3951](https://github.com/AztecProtocol/aztec-packages/issues/3951)) ([e3b288d](https://github.com/AztecProtocol/aztec-packages/commit/e3b288d3f59a3cc0549c02d5e67b7eb729ce68ba))
* Replace relative paths to noir-protocol-circuits ([2c25f04](https://github.com/AztecProtocol/aztec-packages/commit/2c25f0427f33c2ddc0ebfa5e86d772b8336dd747))
* Replace relative paths to noir-protocol-circuits ([0c8b770](https://github.com/AztecProtocol/aztec-packages/commit/0c8b770faf0801e90d3fc692abecb9a9dfff30d8))
* Replace relative paths to noir-protocol-circuits ([d3819ba](https://github.com/AztecProtocol/aztec-packages/commit/d3819ba372e401652178ba8d959440e7cdf78833))
* Standardise toml parsers ([#3910](https://github.com/AztecProtocol/aztec-packages/issues/3910)) ([963035f](https://github.com/AztecProtocol/aztec-packages/commit/963035f47c5dda5ee371c631808253ddf004c2dd))
* Sync noir ([#4025](https://github.com/AztecProtocol/aztec-packages/issues/4025)) ([4e90d7b](https://github.com/AztecProtocol/aztec-packages/commit/4e90d7bf3cfda51affca142aadb8d6d80655316a))
* Sync Noir repo ([#4020](https://github.com/AztecProtocol/aztec-packages/issues/4020)) ([876603e](https://github.com/AztecProtocol/aztec-packages/commit/876603e9e397b336e2895b56399f68fcd9636632))
* **yellowpaper:** Cleanup avm sidebar, fix filename case ([#3952](https://github.com/AztecProtocol/aztec-packages/issues/3952)) ([5211060](https://github.com/AztecProtocol/aztec-packages/commit/5211060ffb0253c94b030b3b3c3dc7148abdf190))
* Yp docs sidebar (with some auto-formatting) ([#3893](https://github.com/AztecProtocol/aztec-packages/issues/3893)) ([f7b007a](https://github.com/AztecProtocol/aztec-packages/commit/f7b007a6001772a8c3148ef105529e53709d4715))


### Documentation

* Noir git subrepo usage ([#3962](https://github.com/AztecProtocol/aztec-packages/issues/3962)) ([2e4c9de](https://github.com/AztecProtocol/aztec-packages/commit/2e4c9de3cea1421b04e1c9276fc0566fccdb6872))
* Streamlined pr template ([#3932](https://github.com/AztecProtocol/aztec-packages/issues/3932)) ([5ec1559](https://github.com/AztecProtocol/aztec-packages/commit/5ec1559663473ca8de8df41ebf3dc65d2efc1917))
* **yellowpaper:** Avm call pointers, bytecode lookups, circuit io ([#3898](https://github.com/AztecProtocol/aztec-packages/issues/3898)) ([45e1ed2](https://github.com/AztecProtocol/aztec-packages/commit/45e1ed2b3088a43b87ba51e0aa5f7a5f04348b65))
* **yellowpaper:** Avm circuit architecture ([#3934](https://github.com/AztecProtocol/aztec-packages/issues/3934)) ([6aed1d0](https://github.com/AztecProtocol/aztec-packages/commit/6aed1d053def3cfdd3073afdfc62db37dc42dca3))
* **yellowpaper:** First draft of avm circuit memory ([#3865](https://github.com/AztecProtocol/aztec-packages/issues/3865)) ([f689297](https://github.com/AztecProtocol/aztec-packages/commit/f6892972e7d89796a8ac4cc9bdde2f99dd9008e5))
* **yellowpaper:** Logs ([#4016](https://github.com/AztecProtocol/aztec-packages/issues/4016)) ([d734c79](https://github.com/AztecProtocol/aztec-packages/commit/d734c79936157f4c3858675eabd6591a8afa959f))
* **yellowpaper:** Update AVM spec for with "daGasLeft", some cleanup ([#3956](https://github.com/AztecProtocol/aztec-packages/issues/3956)) ([a9537fb](https://github.com/AztecProtocol/aztec-packages/commit/a9537fb40db7ed20b32cc7f90b36174eed52e782))

## [0.17.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.9...aztec-packages-v0.17.0) (2024-01-09)


### ⚠ BREAKING CHANGES

* Remove aggregation objects from RecursionConstraint ([#3885](https://github.com/AztecProtocol/aztec-packages/issues/3885))
* Noir development branch (serialization changes) ([#3858](https://github.com/AztecProtocol/aztec-packages/issues/3858))
* **aztec.js:** Move accounts out of aztec.js into new package ([#3844](https://github.com/AztecProtocol/aztec-packages/issues/3844))
* Add Side effect counter struct for ordering ([#3608](https://github.com/AztecProtocol/aztec-packages/issues/3608))
* typing partial address, deduplicating `Point`, `Point` -> `GrumpkinPoint` ([#3814](https://github.com/AztecProtocol/aztec-packages/issues/3814))
* moving `compute_selector` to `FunctionSelector` ([#3806](https://github.com/AztecProtocol/aztec-packages/issues/3806))
* moving compute_address func to AztecAddress ([#3801](https://github.com/AztecProtocol/aztec-packages/issues/3801))
* updated note hash and nullifier macro ([#3777](https://github.com/AztecProtocol/aztec-packages/issues/3777))
* return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735))
* deduplicating circuit types + typing everything ([#3594](https://github.com/AztecProtocol/aztec-packages/issues/3594))

### Features

* A script which runs `nargo fmt` in all packages + running it ([#3803](https://github.com/AztecProtocol/aztec-packages/issues/3803)) ([5f0ebd6](https://github.com/AztecProtocol/aztec-packages/commit/5f0ebd636265a82d5bcb2b8eb383231a276dea3f))
* Add new metrics ([#3855](https://github.com/AztecProtocol/aztec-packages/issues/3855)) ([a2b267b](https://github.com/AztecProtocol/aztec-packages/commit/a2b267bf4af666a1848c428c2e6430fe45875bbc))
* Adding option to set initial and max memory ([#3265](https://github.com/AztecProtocol/aztec-packages/issues/3265)) ([0ad75fe](https://github.com/AztecProtocol/aztec-packages/commit/0ad75fe745099119726976f964a92d1587f32fbf))
* **avm-main:** Pil -&gt; permutations ([#3650](https://github.com/AztecProtocol/aztec-packages/issues/3650)) ([c52acf6](https://github.com/AztecProtocol/aztec-packages/commit/c52acf64cf00443867f8f578a1c25cda49583faf))
* **avm-mini:** Call and return opcodes ([#3704](https://github.com/AztecProtocol/aztec-packages/issues/3704)) ([e534204](https://github.com/AztecProtocol/aztec-packages/commit/e534204c760db31eb1347cd76e85d151a1fb8305))
* **avm:** Add standalone jump opcode ([#3781](https://github.com/AztecProtocol/aztec-packages/issues/3781)) ([b1b2e7c](https://github.com/AztecProtocol/aztec-packages/commit/b1b2e7ca28ba56cf0bae0f906734df00458714b9))
* **avm:** VM circuit handles tagged memory ([#3725](https://github.com/AztecProtocol/aztec-packages/issues/3725)) ([739fe90](https://github.com/AztecProtocol/aztec-packages/commit/739fe90a50891d99b03a8f34da556c8725673f80)), closes [#3644](https://github.com/AztecProtocol/aztec-packages/issues/3644)
* **aztec.js:** Move accounts out of aztec.js into new package ([#3844](https://github.com/AztecProtocol/aztec-packages/issues/3844)) ([afd7b6d](https://github.com/AztecProtocol/aztec-packages/commit/afd7b6d0618cdc0ee7611c531f5c18167a9a5832)), closes [#3807](https://github.com/AztecProtocol/aztec-packages/issues/3807)
* Barretenberg doxygen CI ([#3818](https://github.com/AztecProtocol/aztec-packages/issues/3818)) ([022a918](https://github.com/AztecProtocol/aztec-packages/commit/022a918911817b1897fd69ea72da84054450c8cb))
* Bb uses goblin ([#3636](https://github.com/AztecProtocol/aztec-packages/issues/3636)) ([d093266](https://github.com/AztecProtocol/aztec-packages/commit/d09326636140dbd68d3efb8bc4ec2b6948e2bfe1))
* Compile base rollup as a circuit ([#3739](https://github.com/AztecProtocol/aztec-packages/issues/3739)) ([5118d44](https://github.com/AztecProtocol/aztec-packages/commit/5118d44a68b6cb017d84dba58908209766d3bf50))
* Contract inclusion proof ([#3680](https://github.com/AztecProtocol/aztec-packages/issues/3680)) ([43aa603](https://github.com/AztecProtocol/aztec-packages/commit/43aa603f4736583c7743945cdaeae8abf4eb7b7e))
* Correct circuit construction from acir ([#3757](https://github.com/AztecProtocol/aztec-packages/issues/3757)) ([a876ab8](https://github.com/AztecProtocol/aztec-packages/commit/a876ab8a61108be06bd5d884d727058e7e54a383))
* Deduplicating circuit types + typing everything ([#3594](https://github.com/AztecProtocol/aztec-packages/issues/3594)) ([fcb04a7](https://github.com/AztecProtocol/aztec-packages/commit/fcb04a76cd9460ca732a1dbc3c21843c92beca29)), closes [#3592](https://github.com/AztecProtocol/aztec-packages/issues/3592) [#3059](https://github.com/AztecProtocol/aztec-packages/issues/3059)
* Goblin and eccvm bench ([#3606](https://github.com/AztecProtocol/aztec-packages/issues/3606)) ([1fe63b2](https://github.com/AztecProtocol/aztec-packages/commit/1fe63b2cf5b83fef576bb99294700743929d5ec7))
* Goblinize the final ecc ops in ZM ([#3741](https://github.com/AztecProtocol/aztec-packages/issues/3741)) ([3048d08](https://github.com/AztecProtocol/aztec-packages/commit/3048d0820c89f3bcce38913d3744cf5be1ece14f))
* Launch the monorepo in a codespace. ([#3829](https://github.com/AztecProtocol/aztec-packages/issues/3829)) ([f5a4a78](https://github.com/AztecProtocol/aztec-packages/commit/f5a4a7842fbf3d380ca946403c49f7734f1ca6e7))
* Moving `compute_selector` to `FunctionSelector` ([#3806](https://github.com/AztecProtocol/aztec-packages/issues/3806)) ([bbaebf4](https://github.com/AztecProtocol/aztec-packages/commit/bbaebf4ab6c4272a2819800a1edb6d328d9bfea5)), closes [#3681](https://github.com/AztecProtocol/aztec-packages/issues/3681)
* Moving compute_address func to AztecAddress ([#3801](https://github.com/AztecProtocol/aztec-packages/issues/3801)) ([3107aad](https://github.com/AztecProtocol/aztec-packages/commit/3107aad0b436b96f16005eb00346528f6f58bb5e)), closes [#3794](https://github.com/AztecProtocol/aztec-packages/issues/3794)
* Node version check in `yarn-project/bootstrap.sh` ([#3780](https://github.com/AztecProtocol/aztec-packages/issues/3780)) ([c29e4ee](https://github.com/AztecProtocol/aztec-packages/commit/c29e4ee2b6cd2b0f74af3a69fa6b3d83b8284a64))
* Noir development branch (serialization changes) ([#3858](https://github.com/AztecProtocol/aztec-packages/issues/3858)) ([d2ae2cd](https://github.com/AztecProtocol/aztec-packages/commit/d2ae2cd529b0ef132c0b6c7c35938066c89d809c))
* Only one tx per base rollup ([#3742](https://github.com/AztecProtocol/aztec-packages/issues/3742)) ([9eef247](https://github.com/AztecProtocol/aztec-packages/commit/9eef247a7ba986041bff2d7b459ceb811bba21ce))
* ProverPolynomials owns its memory  ([#3560](https://github.com/AztecProtocol/aztec-packages/issues/3560)) ([a4aba00](https://github.com/AztecProtocol/aztec-packages/commit/a4aba0061929c96bf9cccb64916f96011688a3e1))
* Public data tree as indexed tree ([#3566](https://github.com/AztecProtocol/aztec-packages/issues/3566)) ([4711ef7](https://github.com/AztecProtocol/aztec-packages/commit/4711ef76ea1c4f62fc2f6341dbc22f715fb11a67))
* PXE adds note processors for stored accounts ([#3673](https://github.com/AztecProtocol/aztec-packages/issues/3673)) ([93f9315](https://github.com/AztecProtocol/aztec-packages/commit/93f9315d2e9f1d51195ec831f6dd3e6fe16c468e))
* Return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735)) ([bd5614c](https://github.com/AztecProtocol/aztec-packages/commit/bd5614c2ee04065e149d3df48f1ace9c0ce3858f))
* Serialize synchronize and simulateTx calls by the pxe via SerialQueue ([#3817](https://github.com/AztecProtocol/aztec-packages/issues/3817)) ([e893675](https://github.com/AztecProtocol/aztec-packages/commit/e8936754104b74fbc93c1e70df7d38f356be5fc6))
* Specific membership witness functions in aztec-nr ([#3674](https://github.com/AztecProtocol/aztec-packages/issues/3674)) ([3403877](https://github.com/AztecProtocol/aztec-packages/commit/3403877812108ccab090b4f1c3d348639049a1ec)), closes [#3663](https://github.com/AztecProtocol/aztec-packages/issues/3663)
* Tree ids in noir ([#3809](https://github.com/AztecProtocol/aztec-packages/issues/3809)) ([ec2e36e](https://github.com/AztecProtocol/aztec-packages/commit/ec2e36e6657dac1d1d392e43730b2ddbd2aa88b0))
* Txpool persistence ([#3672](https://github.com/AztecProtocol/aztec-packages/issues/3672)) ([4dd076c](https://github.com/AztecProtocol/aztec-packages/commit/4dd076cfc037c4008b2f9bfe112421970ae10b64)), closes [#3365](https://github.com/AztecProtocol/aztec-packages/issues/3365)
* Typing partial address, deduplicating `Point`, `Point` -&gt; `GrumpkinPoint` ([#3814](https://github.com/AztecProtocol/aztec-packages/issues/3814)) ([44458be](https://github.com/AztecProtocol/aztec-packages/commit/44458be658f2c91423a83c7f126cabe66ee9a273)), closes [#3682](https://github.com/AztecProtocol/aztec-packages/issues/3682)
* Update to latest noir and update noir compiler ([#3696](https://github.com/AztecProtocol/aztec-packages/issues/3696)) ([62a17a4](https://github.com/AztecProtocol/aztec-packages/commit/62a17a4758264c62bb7a9273bf95263dcfea930c))
* Updated note hash and nullifier macro ([#3777](https://github.com/AztecProtocol/aztec-packages/issues/3777)) ([e83dd2b](https://github.com/AztecProtocol/aztec-packages/commit/e83dd2b351bf7494dda13b662d0644e8a3dde092)), closes [#3669](https://github.com/AztecProtocol/aztec-packages/issues/3669)


### Bug Fixes

* AWS deploy_service regex + faucet dockerfile ([#3699](https://github.com/AztecProtocol/aztec-packages/issues/3699)) ([260c7c3](https://github.com/AztecProtocol/aztec-packages/commit/260c7c33f74e051bfb9ea9fbfed8ca6f05fa1da2))
* Broken aztec-nr imports ([#3693](https://github.com/AztecProtocol/aztec-packages/issues/3693)) ([7c8814e](https://github.com/AztecProtocol/aztec-packages/commit/7c8814ed5640e0050f74097f3c578312c8eb67f3))
* Build scripts if statements ([#3700](https://github.com/AztecProtocol/aztec-packages/issues/3700)) ([4847c19](https://github.com/AztecProtocol/aztec-packages/commit/4847c199000178aa26a78fec2a941edb94868ba5))
* **ci:** Contracts_deployed check ([#3703](https://github.com/AztecProtocol/aztec-packages/issues/3703)) ([6c4bf75](https://github.com/AztecProtocol/aztec-packages/commit/6c4bf75c5482c9e2bf4df0a8d473337bd9f4b50e))
* **ci:** Redeploy triggers ([#3677](https://github.com/AztecProtocol/aztec-packages/issues/3677)) ([cc515da](https://github.com/AztecProtocol/aztec-packages/commit/cc515dabf981d4fd0938f8650c200b54288586a2))
* CRS not needed for gate_count. Grumpkin not needed for non-goblin. ([#3872](https://github.com/AztecProtocol/aztec-packages/issues/3872)) ([8cda00d](https://github.com/AztecProtocol/aztec-packages/commit/8cda00d94946ed7e8dfc1dbafdefae3e6d1af682))
* Deploy l1 contracts script ([#3713](https://github.com/AztecProtocol/aztec-packages/issues/3713)) ([309be4b](https://github.com/AztecProtocol/aztec-packages/commit/309be4bf0d74ec6c9e5fe636809bc318a9d6dc3f))
* Disable goblin bbjs tests ([#3836](https://github.com/AztecProtocol/aztec-packages/issues/3836)) ([1f5b2c6](https://github.com/AztecProtocol/aztec-packages/commit/1f5b2c606def0c7203cbd7497264c95bbfa708e1))
* Docker user permissions ([#3711](https://github.com/AztecProtocol/aztec-packages/issues/3711)) ([35316fc](https://github.com/AztecProtocol/aztec-packages/commit/35316fcdd95f96616601d4f4b4e1f17e256f909a))
* **docs:** Fix docs build during releases ([#3815](https://github.com/AztecProtocol/aztec-packages/issues/3815)) ([2e0776a](https://github.com/AztecProtocol/aztec-packages/commit/2e0776a0abb0b3186360d6052b48f7769e110495))
* **docs:** Force docs build using latest released code always ([#3762](https://github.com/AztecProtocol/aztec-packages/issues/3762)) ([5545ee6](https://github.com/AztecProtocol/aztec-packages/commit/5545ee654272c1e805470bb7d7b4161fca9c44e8))
* **docs:** Make git repo available when building docs ([#3761](https://github.com/AztecProtocol/aztec-packages/issues/3761)) ([bce2d99](https://github.com/AztecProtocol/aztec-packages/commit/bce2d9944004dfeed3fd4e75808e68b4369ff3b6))
* **docs:** Show latest released code on published site ([#3716](https://github.com/AztecProtocol/aztec-packages/issues/3716)) ([f1eb6d5](https://github.com/AztecProtocol/aztec-packages/commit/f1eb6d5b6e3221e5a5b8f55632dba066d4cfbc24))
* Event macro ([#3784](https://github.com/AztecProtocol/aztec-packages/issues/3784)) ([3af2438](https://github.com/AztecProtocol/aztec-packages/commit/3af24383ced5f95b2babfdeb7d1572a8a9c28967)), closes [#3655](https://github.com/AztecProtocol/aztec-packages/issues/3655)
* Fix for faucet and node deployment config ([#3722](https://github.com/AztecProtocol/aztec-packages/issues/3722)) ([a60b71a](https://github.com/AztecProtocol/aztec-packages/commit/a60b71a5826d8ff2dbbd1c89e66a4fd9d858a2aa))
* Flaky e2e-p2p test ([#3831](https://github.com/AztecProtocol/aztec-packages/issues/3831)) ([5b1e9f2](https://github.com/AztecProtocol/aztec-packages/commit/5b1e9f2b909130a6198f1c395fdf7a360c2ea0d3))
* Issue with `run_nargo_fmt.sh` + minor yellow paper naming improvements ([#3833](https://github.com/AztecProtocol/aztec-packages/issues/3833)) ([8e692c1](https://github.com/AztecProtocol/aztec-packages/commit/8e692c18ef3c5231718840f46243299cba46ab60))
* Map relative path to protocol circuits ([#3694](https://github.com/AztecProtocol/aztec-packages/issues/3694)) ([125ab1d](https://github.com/AztecProtocol/aztec-packages/commit/125ab1d31710a8cc34407fd4c6a2644105d10abb))
* Noir-protocol circuits ([#3734](https://github.com/AztecProtocol/aztec-packages/issues/3734)) ([34e2505](https://github.com/AztecProtocol/aztec-packages/commit/34e2505632048b553dca1f82faa1454f913dcf2b))
* Reenable goblin bbjs for a single test ([#3838](https://github.com/AztecProtocol/aztec-packages/issues/3838)) ([30e47a0](https://github.com/AztecProtocol/aztec-packages/commit/30e47a005c39ae0af80ef33b83251d04046191dc))
* Setup aztec-cli cache ([#3698](https://github.com/AztecProtocol/aztec-packages/issues/3698)) ([48b7474](https://github.com/AztecProtocol/aztec-packages/commit/48b7474f1ca97d8ff659bb06a67eb0f8fd08cfd7))
* Stale pseudocode in yellow paper process func ([#3869](https://github.com/AztecProtocol/aztec-packages/issues/3869)) ([4a73e3d](https://github.com/AztecProtocol/aztec-packages/commit/4a73e3dfb46592bf6783876c3b2f6fa8a0c25d7b))
* Subrepo commit ([b5bfb0b](https://github.com/AztecProtocol/aztec-packages/commit/b5bfb0bac4dfea3b8978a99502ebaf8ad974dac5))
* There is no main.js ([#3691](https://github.com/AztecProtocol/aztec-packages/issues/3691)) ([58ba060](https://github.com/AztecProtocol/aztec-packages/commit/58ba0607ada5c939d9196829347a9ff157c8ac0d))
* Unpick world state circulars. ([#3721](https://github.com/AztecProtocol/aztec-packages/issues/3721)) ([84f4671](https://github.com/AztecProtocol/aztec-packages/commit/84f4671f527ea9f4c043f37bcf72989a273d6a45))
* Update for new p2p bootstrap node names ([#3710](https://github.com/AztecProtocol/aztec-packages/issues/3710)) ([c7b29b3](https://github.com/AztecProtocol/aztec-packages/commit/c7b29b3d8a3cf33ab4f29dbf5e71737926c53bde))
* Update toy to new master ([78cf525](https://github.com/AztecProtocol/aztec-packages/commit/78cf525dcacba77386779a74b6f806fba47f1bc7))
* Use lookup instead of resolve to ensure consider /etc/hosts ([#3720](https://github.com/AztecProtocol/aztec-packages/issues/3720)) ([eb8413e](https://github.com/AztecProtocol/aztec-packages/commit/eb8413e38f245724b21ac60438c64862a4654311))


### Miscellaneous

* Add GH action to notify gate count differences ([#3724](https://github.com/AztecProtocol/aztec-packages/issues/3724)) ([c0a24fb](https://github.com/AztecProtocol/aztec-packages/commit/c0a24fb6cab1f401bd93352ef544d763804cee52)), closes [#3467](https://github.com/AztecProtocol/aztec-packages/issues/3467)
* Add Side effect counter struct for ordering ([#3608](https://github.com/AztecProtocol/aztec-packages/issues/3608)) ([c58b197](https://github.com/AztecProtocol/aztec-packages/commit/c58b197512297a292cfddd253d8d951b207829a0))
* Add small how to diagram section ([#3804](https://github.com/AztecProtocol/aztec-packages/issues/3804)) ([df581f0](https://github.com/AztecProtocol/aztec-packages/commit/df581f02a23d0957c1db03a6d3432b89be670fdc))
* Added cryptography section to yellow paper ([#3647](https://github.com/AztecProtocol/aztec-packages/issues/3647)) ([286028b](https://github.com/AztecProtocol/aztec-packages/commit/286028bc4df04fafa6a5a379409a9e5b64fa7477))
* Adding some clarification after a question on discourse ([#3823](https://github.com/AztecProtocol/aztec-packages/issues/3823)) ([f3d37d7](https://github.com/AztecProtocol/aztec-packages/commit/f3d37d7661473e514f8ad11495c2b284690c67e2))
* Align bb.js testing ([#3840](https://github.com/AztecProtocol/aztec-packages/issues/3840)) ([c489727](https://github.com/AztecProtocol/aztec-packages/commit/c4897270515f23891a32807dd2be046be12d5095))
* **avm:** Avm memory trace building ([#3835](https://github.com/AztecProtocol/aztec-packages/issues/3835)) ([b7766d6](https://github.com/AztecProtocol/aztec-packages/commit/b7766d68727c92f92abc91131a4332db25d805dd))
* Aztec js circulars ([#3723](https://github.com/AztecProtocol/aztec-packages/issues/3723)) ([378407d](https://github.com/AztecProtocol/aztec-packages/commit/378407d4f4569ed9c1d06a3cbff6eb8e5c81e9db))
* Bring boxes back to CI. Build and run using docker/docker-compose. ([#3727](https://github.com/AztecProtocol/aztec-packages/issues/3727)) ([4a1c0df](https://github.com/AztecProtocol/aztec-packages/commit/4a1c0df76f26530521daaaa60945fead106b555e))
* Build protocol circuits on CI and stop committing artifacts ([#3816](https://github.com/AztecProtocol/aztec-packages/issues/3816)) ([fa1c456](https://github.com/AztecProtocol/aztec-packages/commit/fa1c45679b2d8ed9cacb2ae283c167fb7fd66969))
* Checking noir formatting in CI ([#3828](https://github.com/AztecProtocol/aztec-packages/issues/3828)) ([b53bacf](https://github.com/AztecProtocol/aztec-packages/commit/b53bacfcde8217e15c8329f42ff546064f9b44bc)), closes [#3825](https://github.com/AztecProtocol/aztec-packages/issues/3825)
* Cleaning inconsistency ([#3851](https://github.com/AztecProtocol/aztec-packages/issues/3851)) ([9bbd70a](https://github.com/AztecProtocol/aztec-packages/commit/9bbd70af1666b52307b03399cea77e6f64a0ac02))
* Cleanup recursion interface ([#3744](https://github.com/AztecProtocol/aztec-packages/issues/3744)) ([fde0ac3](https://github.com/AztecProtocol/aztec-packages/commit/fde0ac3e96fe6e2edcdb1e6919d372e96181eda5))
* **docs:** Add block productions ([#3770](https://github.com/AztecProtocol/aztec-packages/issues/3770)) ([f091f49](https://github.com/AztecProtocol/aztec-packages/commit/f091f49eb5387bdf8482d73487cfa2aecef7a2a4))
* **docs:** Add high level overview of a tx ([#3763](https://github.com/AztecProtocol/aztec-packages/issues/3763)) ([9a55e57](https://github.com/AztecProtocol/aztec-packages/commit/9a55e57d3348dc368dc86c635f20aaedcd8c1dda))
* **docs:** Remove npm reference section from testing page ([#3719](https://github.com/AztecProtocol/aztec-packages/issues/3719)) ([1484c11](https://github.com/AztecProtocol/aztec-packages/commit/1484c110ab1a6ebf7a17dd53ed1d9785457d7693))
* **docs:** Remove references to npm packages ([#3676](https://github.com/AztecProtocol/aztec-packages/issues/3676)) ([bd5355f](https://github.com/AztecProtocol/aztec-packages/commit/bd5355fcd1b2ade72fe7c34af69a5194283ff9bf))
* **docs:** Starting a migration notes section ([#3853](https://github.com/AztecProtocol/aztec-packages/issues/3853)) ([060f39a](https://github.com/AztecProtocol/aztec-packages/commit/060f39aa7112d5bc454684a94ecc02eb404d8b49))
* **docs:** Update deps in tutorials ([#3708](https://github.com/AztecProtocol/aztec-packages/issues/3708)) ([f3d93aa](https://github.com/AztecProtocol/aztec-packages/commit/f3d93aa41392a36ea467c71aefdae2741d28d27a))
* **docs:** Update install script ([#3847](https://github.com/AztecProtocol/aztec-packages/issues/3847)) ([7003853](https://github.com/AztecProtocol/aztec-packages/commit/7003853a633858ff171c159b9d94c56bbd65efb1))
* **docs:** Update reference link ([#3768](https://github.com/AztecProtocol/aztec-packages/issues/3768)) ([18edb98](https://github.com/AztecProtocol/aztec-packages/commit/18edb983a7c5bdde25c7cc241ff3c83c36f24fc0))
* **docs:** Update testing pages ([#3733](https://github.com/AztecProtocol/aztec-packages/issues/3733)) ([1c68e3b](https://github.com/AztecProtocol/aztec-packages/commit/1c68e3b2142ed6303238d2fe27c41b255a625052))
* **docs:** Update token bridge tutorial ([#3773](https://github.com/AztecProtocol/aztec-packages/issues/3773)) ([764cb46](https://github.com/AztecProtocol/aztec-packages/commit/764cb464f48adac3342d8252483b1e8e769a5219))
* **docs:** Update trees page ([#3732](https://github.com/AztecProtocol/aztec-packages/issues/3732)) ([b265531](https://github.com/AztecProtocol/aztec-packages/commit/b265531037b93a1a9f3296529cbae22892803b0d))
* **dsl:** Abstract nested aggregation object from ACIR ([#3765](https://github.com/AztecProtocol/aztec-packages/issues/3765)) ([92f72e4](https://github.com/AztecProtocol/aztec-packages/commit/92f72e44d4b57a3078da6bd1bb39dd0f615785be))
* Increase benchmark warning threshold for trial decrypt ([#3602](https://github.com/AztecProtocol/aztec-packages/issues/3602)) ([913943e](https://github.com/AztecProtocol/aztec-packages/commit/913943ed19f28f6c564d40a462e0e0a7bab228ce))
* Just nargo compile. ([#3775](https://github.com/AztecProtocol/aztec-packages/issues/3775)) ([3d08ef9](https://github.com/AztecProtocol/aztec-packages/commit/3d08ef9d828b81ddb1fc6e9aebae6896fe4ee946))
* Move boxes out of yarn-project ([#3688](https://github.com/AztecProtocol/aztec-packages/issues/3688)) ([472596c](https://github.com/AztecProtocol/aztec-packages/commit/472596ce8908b9d2039acf0e9d08a043f9830596))
* Noir sync ([#3884](https://github.com/AztecProtocol/aztec-packages/issues/3884)) ([217de09](https://github.com/AztecProtocol/aztec-packages/commit/217de090b15feb38d5cccfb21867cfd94edf0061))
* Remove aggregation objects from RecursionConstraint ([#3885](https://github.com/AztecProtocol/aztec-packages/issues/3885)) ([9a80008](https://github.com/AztecProtocol/aztec-packages/commit/9a80008c623a9d26e1b82c9e86561c304ef185f1))
* Remove HashToField128Security ACIR opcode ([#3631](https://github.com/AztecProtocol/aztec-packages/issues/3631)) ([1d6d3c9](https://github.com/AztecProtocol/aztec-packages/commit/1d6d3c94f327de1f20ef7d78302d3957db70019e))
* Removing leaf data type + related cleanup ([#3794](https://github.com/AztecProtocol/aztec-packages/issues/3794)) ([3030cc8](https://github.com/AztecProtocol/aztec-packages/commit/3030cc8a38578aab0762fb322288632dee8fb580))
* Rename generate-ts/nr commands to codegen. ([#3843](https://github.com/AztecProtocol/aztec-packages/issues/3843)) ([1fcb964](https://github.com/AztecProtocol/aztec-packages/commit/1fcb964265907b07a20691c23ba26b0c2b127c97))
* Replace relative paths to noir-protocol-circuits ([c2fed18](https://github.com/AztecProtocol/aztec-packages/commit/c2fed18fbb9c922af1b35258e28d20a68fe69fea))
* Replace relative paths to noir-protocol-circuits ([3accd8a](https://github.com/AztecProtocol/aztec-packages/commit/3accd8ad4fa2bb6987db0532f4ff7ef27b240ba9))
* Replace relative paths to noir-protocol-circuits ([346590b](https://github.com/AztecProtocol/aztec-packages/commit/346590b098fd4e2521b3de1158f641cd0afe2641))
* Replace relative paths to noir-protocol-circuits ([861d928](https://github.com/AztecProtocol/aztec-packages/commit/861d928e9f1141d7dae805e5305c56f1c9644d2e))
* Show noir tag alongside commit on sandbox startup ([#3750](https://github.com/AztecProtocol/aztec-packages/issues/3750)) ([009f66d](https://github.com/AztecProtocol/aztec-packages/commit/009f66d997066f075b8fa834dcbdab660b95f535))
* Update governance vote ballot ([#3789](https://github.com/AztecProtocol/aztec-packages/issues/3789)) ([f8976ad](https://github.com/AztecProtocol/aztec-packages/commit/f8976add22835c1383114c8732e66c08d95db7cd))
* Update how_to_contribute.md ([#3759](https://github.com/AztecProtocol/aztec-packages/issues/3759)) ([4567ec4](https://github.com/AztecProtocol/aztec-packages/commit/4567ec44e6872e149b12c3a2840e75b5063ec302))
* Update privacy main.md ([#3760](https://github.com/AztecProtocol/aztec-packages/issues/3760)) ([c3d8b5d](https://github.com/AztecProtocol/aztec-packages/commit/c3d8b5d46ee443917d06a8dba46b8b5cfbf2ed55))
* Use repo BB for gate diff ([#3852](https://github.com/AztecProtocol/aztec-packages/issues/3852)) ([506e719](https://github.com/AztecProtocol/aztec-packages/commit/506e7196bc9a34f064d3be0f2899a0891a3043eb))
* Use simple "flat" CRS. ([#3748](https://github.com/AztecProtocol/aztec-packages/issues/3748)) ([5c6c2ca](https://github.com/AztecProtocol/aztec-packages/commit/5c6c2caf212fb22856df41fd15464dda37e10dab))
* Use traits in noir-protocol-circuits ([#3832](https://github.com/AztecProtocol/aztec-packages/issues/3832)) ([88fcf8d](https://github.com/AztecProtocol/aztec-packages/commit/88fcf8d34b28d803475c68d2d32424eafed0c9af))


### Documentation

* A layout of logs section of yellow paper ([#3582](https://github.com/AztecProtocol/aztec-packages/issues/3582)) ([8c759f6](https://github.com/AztecProtocol/aztec-packages/commit/8c759f60a3a5b16d9711b57bf99b106fc1f8f253))
* Add current thinking on upgrades ([#3743](https://github.com/AztecProtocol/aztec-packages/issues/3743)) ([9f3d972](https://github.com/AztecProtocol/aztec-packages/commit/9f3d972e4244e4c20b2cae6852ee5c2049906fa9))
* Add da doc ([#3736](https://github.com/AztecProtocol/aztec-packages/issues/3736)) ([193f3f2](https://github.com/AztecProtocol/aztec-packages/commit/193f3f2b4eb80c2a206630357a711afaa3e8aee2)), closes [#3645](https://github.com/AztecProtocol/aztec-packages/issues/3645)
* Bytecode ([#3701](https://github.com/AztecProtocol/aztec-packages/issues/3701)) ([912df7e](https://github.com/AztecProtocol/aztec-packages/commit/912df7eede3494551d69ecfe5c0326c8c4140f63))
* Extend state documentation ([#3731](https://github.com/AztecProtocol/aztec-packages/issues/3731)) ([a99cbd6](https://github.com/AztecProtocol/aztec-packages/commit/a99cbd68ffd53177316ea8810ab1c7409723d23d))
* Remove mentions of noir-compiler ([#3702](https://github.com/AztecProtocol/aztec-packages/issues/3702)) ([ea7cd50](https://github.com/AztecProtocol/aztec-packages/commit/ea7cd50a10fddc99c4198e6ffe66aa8459378305))
* Yellow paper rollup circuits and state update ([#3558](https://github.com/AztecProtocol/aztec-packages/issues/3558)) ([b2d6376](https://github.com/AztecProtocol/aztec-packages/commit/b2d6376d5be2dadd989d9749c26444b552067433))
* **yellow-paper:** Circuits ([#3782](https://github.com/AztecProtocol/aztec-packages/issues/3782)) ([a935ca3](https://github.com/AztecProtocol/aztec-packages/commit/a935ca3abb55b126a4eb8d2fe58d6929e6102221))
* **yellow-paper:** Contract deployment ([#3624](https://github.com/AztecProtocol/aztec-packages/issues/3624)) ([b282867](https://github.com/AztecProtocol/aztec-packages/commit/b28286770ee79146b97c5bd73b00f1c67392dd7d)), closes [#3104](https://github.com/AztecProtocol/aztec-packages/issues/3104)
* **yellow-paper:** Drop pokodl request in key derivation ([#3837](https://github.com/AztecProtocol/aztec-packages/issues/3837)) ([a3920fb](https://github.com/AztecProtocol/aztec-packages/commit/a3920fb5051f0f91313a40f17a8e049d18780d71))
* **yellow-paper:** Update keys and addresses ([#3707](https://github.com/AztecProtocol/aztec-packages/issues/3707)) ([56992ae](https://github.com/AztecProtocol/aztec-packages/commit/56992ae59b76e79bd6404e3400e640b8665e69f9))
* **yellowpaper:** AVM high-level execution ([#3717](https://github.com/AztecProtocol/aztec-packages/issues/3717)) ([2ded221](https://github.com/AztecProtocol/aztec-packages/commit/2ded221e546a30ecede020eaf12cfb33c37b5c60))
* **yellowpaper:** AVM intro sections ([#3692](https://github.com/AztecProtocol/aztec-packages/issues/3692)) ([c48e76c](https://github.com/AztecProtocol/aztec-packages/commit/c48e76c19d369b3bff6432c3dc7d922e811b6451))
* **yellowpaper:** Avm nested call returns, updating calling context ([#3749](https://github.com/AztecProtocol/aztec-packages/issues/3749)) ([a1c701d](https://github.com/AztecProtocol/aztec-packages/commit/a1c701de943df0df811db96b81fe59976c79ae45))
* **yellowpaper:** Finish AVM Context definitions ([#3709](https://github.com/AztecProtocol/aztec-packages/issues/3709)) ([4cfb427](https://github.com/AztecProtocol/aztec-packages/commit/4cfb427aa6df3a73deb1a0025aa0b6bc61b3cc69))
* **yellowpaper:** Private kernel circuits ([#3559](https://github.com/AztecProtocol/aztec-packages/issues/3559)) ([056e553](https://github.com/AztecProtocol/aztec-packages/commit/056e553066f25bd74f85c0d746734d5111f0c102))

## [0.16.9](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.8...aztec-packages-v0.16.9) (2023-12-13)


### Bug Fixes

* **ci:** Deploy_npm script ([#3678](https://github.com/AztecProtocol/aztec-packages/issues/3678)) ([9d7c58d](https://github.com/AztecProtocol/aztec-packages/commit/9d7c58d4fe0f91c453c47d6be813325cff08907b))

## [0.16.8](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.7...aztec-packages-v0.16.8) (2023-12-13)


### Features

* Block header block number oracle ([#3648](https://github.com/AztecProtocol/aztec-packages/issues/3648)) ([ac1edc1](https://github.com/AztecProtocol/aztec-packages/commit/ac1edc1d30352cc3e3dcbfce4e69fe38777cc0df))
* Complete folding prover and verifier for ultra instances ([#3419](https://github.com/AztecProtocol/aztec-packages/issues/3419)) ([bb86ce9](https://github.com/AztecProtocol/aztec-packages/commit/bb86ce9a27e09b8a336af04b787b81d5f1d49ac8))
* Copy constructors for builders ([#3635](https://github.com/AztecProtocol/aztec-packages/issues/3635)) ([b82b0c5](https://github.com/AztecProtocol/aztec-packages/commit/b82b0c579c4a315c9b4eaf3e9726275633603b5a))
* Enabling nullifier tree snapshot ([#3670](https://github.com/AztecProtocol/aztec-packages/issues/3670)) ([b47d49d](https://github.com/AztecProtocol/aztec-packages/commit/b47d49def5b8ae9db3cab9afd1367c68b15cb766))
* Libraryfying historic access ([#3658](https://github.com/AztecProtocol/aztec-packages/issues/3658)) ([6877ca1](https://github.com/AztecProtocol/aztec-packages/commit/6877ca1c906743afbf488ac0f7e662bf4486f6a6))
* Log-derivative based generic permutations for AVM ([#3428](https://github.com/AztecProtocol/aztec-packages/issues/3428)) ([379b5ad](https://github.com/AztecProtocol/aztec-packages/commit/379b5adc259ac69b01e61b852172cdfc87cf9350))
* Merge recursive verifier ([#3588](https://github.com/AztecProtocol/aztec-packages/issues/3588)) ([cdd9259](https://github.com/AztecProtocol/aztec-packages/commit/cdd92595c313617189a530e0bfda987db211ae6b))
* New install script and container wrappers. ([#3617](https://github.com/AztecProtocol/aztec-packages/issues/3617)) ([c7f1878](https://github.com/AztecProtocol/aztec-packages/commit/c7f1878777bf76dbfd451761dbe6dc78903e45e2))
* Persist pxe state ([#3628](https://github.com/AztecProtocol/aztec-packages/issues/3628)) ([9ccbbd9](https://github.com/AztecProtocol/aztec-packages/commit/9ccbbd96e50f22646f0527c7d15944cecabca662))
* Update command handles Dockerized sandbox ([#3656](https://github.com/AztecProtocol/aztec-packages/issues/3656)) ([7c85750](https://github.com/AztecProtocol/aztec-packages/commit/7c85750089e022240376751086d515d993409d98))


### Bug Fixes

* Aztec sandbox compose fixes ([#3634](https://github.com/AztecProtocol/aztec-packages/issues/3634)) ([765a19c](https://github.com/AztecProtocol/aztec-packages/commit/765a19c3aad3a2793a764b970b7cc8a819094da7))
* Broken uint256_t implicit copy ([#3625](https://github.com/AztecProtocol/aztec-packages/issues/3625)) ([1a6b44d](https://github.com/AztecProtocol/aztec-packages/commit/1a6b44d67e077eb5904ab30255454693d6a1edac))
* **ci:** Rebuild versioned cli / sandbox images ([#3613](https://github.com/AztecProtocol/aztec-packages/issues/3613)) ([6a53fbc](https://github.com/AztecProtocol/aztec-packages/commit/6a53fbc48624d6ecbcc8dc6fd69564fdb3db342e))
* Make lsp work in docker, plus some other install tweaks. ([#3661](https://github.com/AztecProtocol/aztec-packages/issues/3661)) ([53eb54f](https://github.com/AztecProtocol/aztec-packages/commit/53eb54fb3416711dbc411e6037bc2da18d523cc3))
* **noir-compiler:** Compile time error if ctor is missing ([#3649](https://github.com/AztecProtocol/aztec-packages/issues/3649)) ([12249bf](https://github.com/AztecProtocol/aztec-packages/commit/12249bf91d5934f798856a75cdd9de8a5822604b))
* Sandbox node mode api prefix ([#3662](https://github.com/AztecProtocol/aztec-packages/issues/3662)) ([fd6eefe](https://github.com/AztecProtocol/aztec-packages/commit/fd6eefe310e291e9625184dee9177a9b219e82f2))
* Top level init bb.js, but better scoped imports to not incur cost too early ([#3629](https://github.com/AztecProtocol/aztec-packages/issues/3629)) ([cea862d](https://github.com/AztecProtocol/aztec-packages/commit/cea862dd7feec714a34eba6a3cf7a2a174a59a1b))


### Miscellaneous

* **ci:** Combine deploy / release jobs + canary update ([#3610](https://github.com/AztecProtocol/aztec-packages/issues/3610)) ([0888c05](https://github.com/AztecProtocol/aztec-packages/commit/0888c05098c94151462e30528a32322ae793d9ac)), closes [#3579](https://github.com/AztecProtocol/aztec-packages/issues/3579)
* **docs:** Update implementation references in token contract tutorial ([#3626](https://github.com/AztecProtocol/aztec-packages/issues/3626)) ([a2cee4f](https://github.com/AztecProtocol/aztec-packages/commit/a2cee4ff1df294b1253f4a495a158794b47bfe66))
* Nuke fib ([#3607](https://github.com/AztecProtocol/aztec-packages/issues/3607)) ([48e2e3d](https://github.com/AztecProtocol/aztec-packages/commit/48e2e3d261a7091cb0b87565ec8bc9ae595b3022))
* Reduced spam logging in archiver ([#3671](https://github.com/AztecProtocol/aztec-packages/issues/3671)) ([e749daa](https://github.com/AztecProtocol/aztec-packages/commit/e749daa7f1e9e34cad93d64e76e04d2525c6f458))
* Run the protocol circuits noir tests in CI ([#3660](https://github.com/AztecProtocol/aztec-packages/issues/3660)) ([383e123](https://github.com/AztecProtocol/aztec-packages/commit/383e1231c1df23e1365e94642fe0d4d670d84c8f)), closes [#3205](https://github.com/AztecProtocol/aztec-packages/issues/3205)


### Documentation

* Updated yellow paper for fees ([#3659](https://github.com/AztecProtocol/aztec-packages/issues/3659)) ([5513624](https://github.com/AztecProtocol/aztec-packages/commit/55136246ffa457c73426190438c254743c841675))
* **yellowpaper:** Rewrite section on tagged memory, misc rewording/cleanup ([#3523](https://github.com/AztecProtocol/aztec-packages/issues/3523)) ([fe849e3](https://github.com/AztecProtocol/aztec-packages/commit/fe849e323526ea132c2a3ab41c8351a24f6e9cf4))
* **yellowpaper:** Update `cast` instruction description with truncation operation ([#3621](https://github.com/AztecProtocol/aztec-packages/issues/3621)) ([2cede41](https://github.com/AztecProtocol/aztec-packages/commit/2cede412d25ef27acc7347e31389ae5d780f1b0b))

## [0.16.7](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.6...aztec-packages-v0.16.7) (2023-12-06)


### Features

* Encapsulated Goblin ([#3524](https://github.com/AztecProtocol/aztec-packages/issues/3524)) ([2f08423](https://github.com/AztecProtocol/aztec-packages/commit/2f08423e37942f991634fe6c45de52feb1f159cf))


### Bug Fixes

* Extract whole archive instead of subset ([#3604](https://github.com/AztecProtocol/aztec-packages/issues/3604)) ([cb000d8](https://github.com/AztecProtocol/aztec-packages/commit/cb000d828dcea0ec5025bceadd322b1d260c0111))


### Documentation

* **yellow-paper:** Note hash, nullifier, and public data trees ([#3518](https://github.com/AztecProtocol/aztec-packages/issues/3518)) ([0e2db8b](https://github.com/AztecProtocol/aztec-packages/commit/0e2db8b0a819dfe44dd5c76ff89aaa1f403d2071))

## [0.16.6](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.5...aztec-packages-v0.16.6) (2023-12-06)


### Bug Fixes

* **pxe:** Initialise aztecjs on pxe startup ([#3601](https://github.com/AztecProtocol/aztec-packages/issues/3601)) ([ceb2ed2](https://github.com/AztecProtocol/aztec-packages/commit/ceb2ed2618398c6af56e69ec0a9f58b808547f30))
* Remove api_prefix local ([#3599](https://github.com/AztecProtocol/aztec-packages/issues/3599)) ([0d8dd8d](https://github.com/AztecProtocol/aztec-packages/commit/0d8dd8d14fa002b4dadcd7ea70e01c5b263edaee))


### Miscellaneous

* **yellow_paper:** Fixes to my work on public private messages ([#3507](https://github.com/AztecProtocol/aztec-packages/issues/3507)) ([33a4f63](https://github.com/AztecProtocol/aztec-packages/commit/33a4f63dc8004d144d891fb8016d85471c64e880))

## [0.16.5](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.4...aztec-packages-v0.16.5) (2023-12-06)


### Features

* Add EFS file storage to devnet nodes ([#3584](https://github.com/AztecProtocol/aztec-packages/issues/3584)) ([5b590eb](https://github.com/AztecProtocol/aztec-packages/commit/5b590eb06fab7ecfcd62aa78a04e035dc8db6b41))


### Bug Fixes

* **ci:** Aztec node devnet healthchecks ([#3598](https://github.com/AztecProtocol/aztec-packages/issues/3598)) ([1a9d742](https://github.com/AztecProtocol/aztec-packages/commit/1a9d742cb21ea71df33eb8931b0faecc96e84508))
* **ci:** Count for EFS AZ2 ([#3597](https://github.com/AztecProtocol/aztec-packages/issues/3597)) ([d427bca](https://github.com/AztecProtocol/aztec-packages/commit/d427bca1c53aacc499f0895bb172f88d96e9347e))
* **ci:** L1-contracts npm release ([#3596](https://github.com/AztecProtocol/aztec-packages/issues/3596)) ([008df50](https://github.com/AztecProtocol/aztec-packages/commit/008df5018e8f924ac93ad5d9d712727c51952c54))
* **ci:** Node health-check + contract address env vars ([#3578](https://github.com/AztecProtocol/aztec-packages/issues/3578)) ([fffc700](https://github.com/AztecProtocol/aztec-packages/commit/fffc7007cf5a5fb5e721c63d4abff5184d40c9c0))


### Miscellaneous

* Make noir-circuit independent of aztec-nr ([#3591](https://github.com/AztecProtocol/aztec-packages/issues/3591)) ([3013354](https://github.com/AztecProtocol/aztec-packages/commit/301335479f45837e61e1b434566dff98a0867a37))
* Remove foundation and types deps from boxes ([#3389](https://github.com/AztecProtocol/aztec-packages/issues/3389)) ([eade352](https://github.com/AztecProtocol/aztec-packages/commit/eade352a56b2365b5213962733735e45a6d46fb0))
* Renaming blockstree to archive ([#3569](https://github.com/AztecProtocol/aztec-packages/issues/3569)) ([6c200e9](https://github.com/AztecProtocol/aztec-packages/commit/6c200e932b6a4bb218059e7b9f92f97c70aa8195))
* Trivial change roundup ([#3556](https://github.com/AztecProtocol/aztec-packages/issues/3556)) ([ff893b2](https://github.com/AztecProtocol/aztec-packages/commit/ff893b236091b480b6de18ebaab57c62dcdfe1d4))


### Documentation

* Add libstdc++-12-dev to setup instructions ([#3585](https://github.com/AztecProtocol/aztec-packages/issues/3585)) ([9773e8c](https://github.com/AztecProtocol/aztec-packages/commit/9773e8c3b4789f0dd6b5fdaf0f283b9bd7c9812f))

## [0.16.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.3...aztec-packages-v0.16.4) (2023-12-05)


### Bug Fixes

* **ci:** Separate step for l1-contracts npm release ([#3581](https://github.com/AztecProtocol/aztec-packages/issues/3581)) ([7745975](https://github.com/AztecProtocol/aztec-packages/commit/7745975731a009c9010291b9174d321941754760))

## [0.16.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.2...aztec-packages-v0.16.3) (2023-12-05)


### Bug Fixes

* Npm release of l1-contracts ([#3571](https://github.com/AztecProtocol/aztec-packages/issues/3571)) ([487419b](https://github.com/AztecProtocol/aztec-packages/commit/487419be549903a3d42b1232cce02139b2ac556f))


### Miscellaneous

* CLI's startup time was pushing almost 2s. This gets the basic 'help' down to 0.16. ([#3529](https://github.com/AztecProtocol/aztec-packages/issues/3529)) ([396df13](https://github.com/AztecProtocol/aztec-packages/commit/396df13389cdcb8b8b0d5a92a4b3d1c2bffcb7a7))


### Documentation

* Documenting issue with `context.block_header` ([#3565](https://github.com/AztecProtocol/aztec-packages/issues/3565)) ([1237e26](https://github.com/AztecProtocol/aztec-packages/commit/1237e2658d90114c03a6b838cbab80005aa3a661))

## [0.16.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.1...aztec-packages-v0.16.2) (2023-12-05)


### Features

* Add tree snapshots ([#3468](https://github.com/AztecProtocol/aztec-packages/issues/3468)) ([7a86bb3](https://github.com/AztecProtocol/aztec-packages/commit/7a86bb3a5e2bd9db60c1b70e11ced29deca83ff6))
* **AVM:** First version for mini AVM (ADD, RETURN, CALLDATACOPY) ([#3439](https://github.com/AztecProtocol/aztec-packages/issues/3439)) ([b3af146](https://github.com/AztecProtocol/aztec-packages/commit/b3af1463ed6b7858252ab4779f8c747a6de47363))
* Circuit optimized indexed tree batch insertion ([#3367](https://github.com/AztecProtocol/aztec-packages/issues/3367)) ([187d2f7](https://github.com/AztecProtocol/aztec-packages/commit/187d2f79d9390e43ec2e2ce6a0db0d6718cc1716))
* Devnet ([#3473](https://github.com/AztecProtocol/aztec-packages/issues/3473)) ([97c40c2](https://github.com/AztecProtocol/aztec-packages/commit/97c40c26098dc615e95e8555458401afc88d9516))
* **docs:** Add simple private voting tutorial   ([#3402](https://github.com/AztecProtocol/aztec-packages/issues/3402)) ([a6e0352](https://github.com/AztecProtocol/aztec-packages/commit/a6e035275fc07f11d0354d0794eaa15d937ba278))
* **docs:** Document slow update tree ([#3416](https://github.com/AztecProtocol/aztec-packages/issues/3416)) ([8e9f103](https://github.com/AztecProtocol/aztec-packages/commit/8e9f10349936ee414526915a93f4ec1070de17e4))
* Flavor refactor, reduce duplication ([#3407](https://github.com/AztecProtocol/aztec-packages/issues/3407)) ([8d6b013](https://github.com/AztecProtocol/aztec-packages/commit/8d6b01304d797f7cbb576b23a7e115390d113c79))
* Inclusion and non-inclusion proofs experiment ([#3255](https://github.com/AztecProtocol/aztec-packages/issues/3255)) ([b911e65](https://github.com/AztecProtocol/aztec-packages/commit/b911e6546bea5b3e2301b02459c5db8a1ff9024e)), closes [#2572](https://github.com/AztecProtocol/aztec-packages/issues/2572) [#2584](https://github.com/AztecProtocol/aztec-packages/issues/2584)
* New Poseidon2 circuit builder gates ([#3346](https://github.com/AztecProtocol/aztec-packages/issues/3346)) ([91cb369](https://github.com/AztecProtocol/aztec-packages/commit/91cb369aa7ecbf457965f53057cafa2c2e6f1214))
* New Poseidon2 relations ([#3406](https://github.com/AztecProtocol/aztec-packages/issues/3406)) ([14b9736](https://github.com/AztecProtocol/aztec-packages/commit/14b9736925c6da33133bd24ee283fb4c199082a5))
* Pull latest noir for brillig optimizations ([#3464](https://github.com/AztecProtocol/aztec-packages/issues/3464)) ([d356bac](https://github.com/AztecProtocol/aztec-packages/commit/d356bac740d203fbb9363a0127ca1d433358e029))
* Refactor StandardIndexedTree for abstract leaves and preimages and optimized it ([#3530](https://github.com/AztecProtocol/aztec-packages/issues/3530)) ([63b9cdc](https://github.com/AztecProtocol/aztec-packages/commit/63b9cdc5823df540c73b3e53d8e3c4117deb3b02))
* Removing historical roots from circuits ([#3544](https://github.com/AztecProtocol/aztec-packages/issues/3544)) ([9f682cb](https://github.com/AztecProtocol/aztec-packages/commit/9f682cb8cf37eb392c4979f62fdec7126fb4d102))
* Seperate pil files for sub machines ([#3454](https://github.com/AztecProtocol/aztec-packages/issues/3454)) ([d09d6f5](https://github.com/AztecProtocol/aztec-packages/commit/d09d6f5a5f2c7e2a58658a640a6a6d6ba4294701))
* Throw compile time error if contract has too many fns ([#3536](https://github.com/AztecProtocol/aztec-packages/issues/3536)) ([ad66ad0](https://github.com/AztecProtocol/aztec-packages/commit/ad66ad0811181def6ef13c646acfc06261958787))
* Use tree snapshots in aztec-node/pxe/oracles ([#3504](https://github.com/AztecProtocol/aztec-packages/issues/3504)) ([6e40427](https://github.com/AztecProtocol/aztec-packages/commit/6e4042757feb852dca77c957fc52f41e5b30f848))
* Yellow paper cross-chain communication ([#3477](https://github.com/AztecProtocol/aztec-packages/issues/3477)) ([d51df8c](https://github.com/AztecProtocol/aztec-packages/commit/d51df8cf6d756e03ffa577b9e35b92a9b723e6c1))


### Bug Fixes

* Check version, chainid and sender for cross-chain l1 to l2 msgs ([#3457](https://github.com/AztecProtocol/aztec-packages/issues/3457)) ([d251703](https://github.com/AztecProtocol/aztec-packages/commit/d251703213c42c427ed3e0f8ff1098edf3b6a2e3))
* **ci:** Add DEPLOY_TAG in fork log group ([#3510](https://github.com/AztecProtocol/aztec-packages/issues/3510)) ([f021041](https://github.com/AztecProtocol/aztec-packages/commit/f02104136f2d98325baa21792ea10245abffab76))
* **ci:** Check if l1 contracts img has been deployed ([#3531](https://github.com/AztecProtocol/aztec-packages/issues/3531)) ([ac1f03c](https://github.com/AztecProtocol/aztec-packages/commit/ac1f03c995457df161ce59b181664950871b6436))
* **ci:** Comment out LB listeners (for now) ([#3519](https://github.com/AztecProtocol/aztec-packages/issues/3519)) ([640aabc](https://github.com/AztecProtocol/aztec-packages/commit/640aabc414876a3dacb5287e2705380a9fafca9f))
* **ci:** Count for bootnode discovery service ([#3517](https://github.com/AztecProtocol/aztec-packages/issues/3517)) ([2a38788](https://github.com/AztecProtocol/aztec-packages/commit/2a38788ee7857162a9af391323f53187e670dedc))
* **ci:** Define REPOSITORY in deploy_l1_contracts ([#3514](https://github.com/AztecProtocol/aztec-packages/issues/3514)) ([b246d1b](https://github.com/AztecProtocol/aztec-packages/commit/b246d1ba3a899af5e7566944a9d79be62827cdd5))
* **ci:** Don't deploy to npm on master merge ([#3502](https://github.com/AztecProtocol/aztec-packages/issues/3502)) ([a138860](https://github.com/AztecProtocol/aztec-packages/commit/a138860bf4032be9688c5ffb5d95b12bcb6d459e))
* **ci:** Env vars for deploying l1-contracts ([#3513](https://github.com/AztecProtocol/aztec-packages/issues/3513)) ([27106b2](https://github.com/AztecProtocol/aztec-packages/commit/27106b2e2845cb32ea229a8527b86a691a668f20))
* **ci:** Export FORK_API_KEY from setup_env ([#3512](https://github.com/AztecProtocol/aztec-packages/issues/3512)) ([7e81e2c](https://github.com/AztecProtocol/aztec-packages/commit/7e81e2c53deaf2b5efcc6b0567fc1240540471eb))
* **ci:** Fix docker architecture for devnet packages ([#3505](https://github.com/AztecProtocol/aztec-packages/issues/3505)) ([66d0287](https://github.com/AztecProtocol/aztec-packages/commit/66d02879a33ded27e188b90b1d7ac6b551830acc))
* **ci:** Fix faucet vars + don't deploy contracts from node ([#3553](https://github.com/AztecProtocol/aztec-packages/issues/3553)) ([c7176f6](https://github.com/AztecProtocol/aztec-packages/commit/c7176f6c04486a3f261a48958ccadba684f33521))
* **ci:** L1 contracts directories ([#3545](https://github.com/AztecProtocol/aztec-packages/issues/3545)) ([63dd0c8](https://github.com/AztecProtocol/aztec-packages/commit/63dd0c8852ca7605a2407458b355b3776a96b37c))
* **ci:** Login to ecr to fetch contracts image ([#3538](https://github.com/AztecProtocol/aztec-packages/issues/3538)) ([b033538](https://github.com/AztecProtocol/aztec-packages/commit/b0335383c884d81562c2911ecae9d889f1076254))
* **ci:** Remove unused ADDRESS vars & export private key vars ([#3520](https://github.com/AztecProtocol/aztec-packages/issues/3520)) ([d889359](https://github.com/AztecProtocol/aztec-packages/commit/d8893590a8f6f7b1d0a60279a6a2bc9fd0b5c154))
* **ci:** Set default value for $TO_TAINT ([#3508](https://github.com/AztecProtocol/aztec-packages/issues/3508)) ([8b6688a](https://github.com/AztecProtocol/aztec-packages/commit/8b6688a7975a748f910f67ee17dbc61fd1df7001))
* **ci:** Terraform listener resources ([#3534](https://github.com/AztecProtocol/aztec-packages/issues/3534)) ([c3b9cce](https://github.com/AztecProtocol/aztec-packages/commit/c3b9cce96599451fce79fd3318176da4708bfa6a))
* **ci:** Terraform_deploy for devnet ([#3516](https://github.com/AztecProtocol/aztec-packages/issues/3516)) ([ba3803e](https://github.com/AztecProtocol/aztec-packages/commit/ba3803ec7c208804f8da5ee81b9989f4640a2fc1))
* **ci:** Tf variable references & formatting([#3522](https://github.com/AztecProtocol/aztec-packages/issues/3522)) ([d37cf52](https://github.com/AztecProtocol/aztec-packages/commit/d37cf520348e17acdc9de93bc2cf83560ccf57d5))
* Disable e2e-slow-tree ([#3459](https://github.com/AztecProtocol/aztec-packages/issues/3459)) ([5927103](https://github.com/AztecProtocol/aztec-packages/commit/59271039b3a087a4f33b11701929cebf2eadb61d))
* **docs:** Update package name of aztec-cli ([#3474](https://github.com/AztecProtocol/aztec-packages/issues/3474)) ([98d7ba0](https://github.com/AztecProtocol/aztec-packages/commit/98d7ba0c1d8c809f1bcb05e517412f99e46f95ae))
* Double slash in deployed faucet routes ([#3555](https://github.com/AztecProtocol/aztec-packages/issues/3555)) ([6c704a5](https://github.com/AztecProtocol/aztec-packages/commit/6c704a5502746e8a002e039ce8c73e8e207ca9d0))
* Faucet lb_listener priority ([#3554](https://github.com/AztecProtocol/aztec-packages/issues/3554)) ([3f56dd7](https://github.com/AztecProtocol/aztec-packages/commit/3f56dd7cacfda0eb7a4bf0c38ec804a85e6881d2))
* Handling low_nullifier.next_value equal to 0 ([#3562](https://github.com/AztecProtocol/aztec-packages/issues/3562)) ([c800502](https://github.com/AztecProtocol/aztec-packages/commit/c8005023d80a2a4e15d3a3bea10072371e3c5842)), closes [#3550](https://github.com/AztecProtocol/aztec-packages/issues/3550)
* Remove x86_64 form l1-contracts img tag ([#3549](https://github.com/AztecProtocol/aztec-packages/issues/3549)) ([6828f1a](https://github.com/AztecProtocol/aztec-packages/commit/6828f1ac33755ca6ccf42096d741d5ea326dae66))
* Throw error if fn sig has whitespaces ([#3509](https://github.com/AztecProtocol/aztec-packages/issues/3509)) ([7671063](https://github.com/AztecProtocol/aztec-packages/commit/7671063a2cb32c45a751c33f6ed5e1b8bea8608f)), closes [#3055](https://github.com/AztecProtocol/aztec-packages/issues/3055)


### Miscellaneous

* (yellow paper) public-vm section of yellow paper ([#3493](https://github.com/AztecProtocol/aztec-packages/issues/3493)) ([8ff3780](https://github.com/AztecProtocol/aztec-packages/commit/8ff378005f78126260cb0950a8167ec40efd14aa))
* Add mermaid diagram support ([#3499](https://github.com/AztecProtocol/aztec-packages/issues/3499)) ([537d552](https://github.com/AztecProtocol/aztec-packages/commit/537d552009676a7dfed2d75e7f73a572591699af))
* Add yellow paper build check to CI ([#3490](https://github.com/AztecProtocol/aztec-packages/issues/3490)) ([3ebd2f2](https://github.com/AztecProtocol/aztec-packages/commit/3ebd2f25646c7db170d22c62f41888d0c417d644))
* **avm:** Enable AVM unit tests in CI ([#3463](https://github.com/AztecProtocol/aztec-packages/issues/3463)) ([051dda9](https://github.com/AztecProtocol/aztec-packages/commit/051dda9c50f1d9f11f5063ddf51c9986a6998b43)), closes [#3461](https://github.com/AztecProtocol/aztec-packages/issues/3461)
* **bb:** Pointer_view to reference-based get_all ([#3495](https://github.com/AztecProtocol/aztec-packages/issues/3495)) ([50d7327](https://github.com/AztecProtocol/aztec-packages/commit/50d73271919306a05ac3a7c2e7d37363b6761248))
* **bb:** Reuse entities from GoblinUltra in GoblinUltraRecursive ([#3521](https://github.com/AztecProtocol/aztec-packages/issues/3521)) ([8259636](https://github.com/AztecProtocol/aztec-packages/commit/8259636c016c0adecb052f176e78444fb5481c38))
* Build the acir test vectors as part of CI. ([#3447](https://github.com/AztecProtocol/aztec-packages/issues/3447)) ([1a2d1f8](https://github.com/AztecProtocol/aztec-packages/commit/1a2d1f822d0e1fabd322c2c4d0473629edd56380))
* Containers reduced to ~100MB total. ~30s installation. ([#3487](https://github.com/AztecProtocol/aztec-packages/issues/3487)) ([b49cef2](https://github.com/AztecProtocol/aztec-packages/commit/b49cef21e30f06bce23f421b533e64728278cbf8))
* **docs:** Fix broken Noir stdlib link ([#3496](https://github.com/AztecProtocol/aztec-packages/issues/3496)) ([787d59a](https://github.com/AztecProtocol/aztec-packages/commit/787d59a1a583788773a0e5d75a9079328ce2a21d))
* Field-agnostic and reusable transcript ([#3433](https://github.com/AztecProtocol/aztec-packages/issues/3433)) ([d78775a](https://github.com/AztecProtocol/aztec-packages/commit/d78775adb9574a3d76c3fca8cf940cdef460ae10))
* Fix broken link in txs in yellow paper ([#3484](https://github.com/AztecProtocol/aztec-packages/issues/3484)) ([798565d](https://github.com/AztecProtocol/aztec-packages/commit/798565d5a8a5cb096c9b2efb6d41de1c449d2c4e))
* Fix yellow paper build error ([32881a4](https://github.com/AztecProtocol/aztec-packages/commit/32881a4d0912e0287b558a4785b6d60c50f84335))
* Fixed typo in build system ([#3501](https://github.com/AztecProtocol/aztec-packages/issues/3501)) ([3a80ac2](https://github.com/AztecProtocol/aztec-packages/commit/3a80ac2caf5f1f847f5e6b2a7b526b81a211de29))
* Increase functions per contract from 16 to 32 ([#3503](https://github.com/AztecProtocol/aztec-packages/issues/3503)) ([ebdeea3](https://github.com/AztecProtocol/aztec-packages/commit/ebdeea3f4bc721d5708b44ba1f89ba24eb0e25d5))
* Naming fixes ([#3476](https://github.com/AztecProtocol/aztec-packages/issues/3476)) ([1db30bf](https://github.com/AztecProtocol/aztec-packages/commit/1db30bf0d61a7b2920ab1aedaef58bc0922ec78e))
* Optimise bb.js package size and sandox/cli dockerfiles to unbloat final containers. ([#3462](https://github.com/AztecProtocol/aztec-packages/issues/3462)) ([cb3db5d](https://github.com/AztecProtocol/aztec-packages/commit/cb3db5d0f1f8912f1a97258e5043eb0f69eff551))
* Pin node version in docker base images and bump nvmrc ([#3537](https://github.com/AztecProtocol/aztec-packages/issues/3537)) ([5d3895a](https://github.com/AztecProtocol/aztec-packages/commit/5d3895aefb7812eb6bd8017baf43533959ad69b4))
* Recursive verifier updates ([#3452](https://github.com/AztecProtocol/aztec-packages/issues/3452)) ([dbb4a12](https://github.com/AztecProtocol/aztec-packages/commit/dbb4a1205528bdd8217ea2d15ccf060e2aa9b7d2))
* Refactor `WitnessEntities` to be able to derive `WitnessCommitments` from it ([#3479](https://github.com/AztecProtocol/aztec-packages/issues/3479)) ([9c9b561](https://github.com/AztecProtocol/aztec-packages/commit/9c9b561f392de5fce11cefe4d72e4f33f2567f41))
* Remove temporary logging ([#3466](https://github.com/AztecProtocol/aztec-packages/issues/3466)) ([8c8387b](https://github.com/AztecProtocol/aztec-packages/commit/8c8387b6b18335ca23f62c3d4c942415b7449462))
* Transcript handled through shared_ptr ([#3434](https://github.com/AztecProtocol/aztec-packages/issues/3434)) ([30fca33](https://github.com/AztecProtocol/aztec-packages/commit/30fca3307ee7e33d81fd51c3d280c6362baef0b9))
* Typo fixes ([#3488](https://github.com/AztecProtocol/aztec-packages/issues/3488)) ([d9a44dc](https://github.com/AztecProtocol/aztec-packages/commit/d9a44dc2e655752e1c6503ac85b64169ec7e4754))
* **yellow_paper:** Public&lt;&gt;private messaging ([#3491](https://github.com/AztecProtocol/aztec-packages/issues/3491)) ([6ecc406](https://github.com/AztecProtocol/aztec-packages/commit/6ecc406159a022e5d57267dcaea48e0df25bbda0))


### Documentation

* Add transaction section to yellow paper ([#3418](https://github.com/AztecProtocol/aztec-packages/issues/3418)) ([44bf30b](https://github.com/AztecProtocol/aztec-packages/commit/44bf30b0af5a546e375d068790e9fa7e94d6ca52))
* Apply comments from Jan on contracts ([#3539](https://github.com/AztecProtocol/aztec-packages/issues/3539)) ([e351873](https://github.com/AztecProtocol/aztec-packages/commit/e351873cadb5cbca5d1d61016e6f9a9e7479bff9))
* Fees update in yellow paper ([#3486](https://github.com/AztecProtocol/aztec-packages/issues/3486)) ([a8b2608](https://github.com/AztecProtocol/aztec-packages/commit/a8b26086306bfec6e7808f4858a08644e84336f4))
* First go at generated AVM instruction set doc ([#3469](https://github.com/AztecProtocol/aztec-packages/issues/3469)) ([8cc54a4](https://github.com/AztecProtocol/aztec-packages/commit/8cc54a48917ff319a5c2b706e01cfbf5ebca013e))
* Further update to the yellow paper ([#3542](https://github.com/AztecProtocol/aztec-packages/issues/3542)) ([751bb6a](https://github.com/AztecProtocol/aztec-packages/commit/751bb6a2075705931b3035117512a93769142707))
* Yellow paper updates ([#3478](https://github.com/AztecProtocol/aztec-packages/issues/3478)) ([11f754d](https://github.com/AztecProtocol/aztec-packages/commit/11f754d256cc164ca2d50b9923aeba1612e7f48b))
* Yellow paper updates for private message delivery ([#3472](https://github.com/AztecProtocol/aztec-packages/issues/3472)) ([6ba9e18](https://github.com/AztecProtocol/aztec-packages/commit/6ba9e18820c85acca692d2af03e4d800c29ab6dc))
* **yellow-paper:** Sync, enqueued, and static calls ([#3494](https://github.com/AztecProtocol/aztec-packages/issues/3494)) ([00835c6](https://github.com/AztecProtocol/aztec-packages/commit/00835c67b460074fe16e19b27a47ac37273e743b)), closes [#3108](https://github.com/AztecProtocol/aztec-packages/issues/3108)
* **yellowpaper:** Instruction set updates and fixes ([#3515](https://github.com/AztecProtocol/aztec-packages/issues/3515)) ([bfb61dd](https://github.com/AztecProtocol/aztec-packages/commit/bfb61dd1412e856adc912f0e3133cd6f8c9e8fbf))

## [0.16.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.16.0...aztec-packages-v0.16.1) (2023-11-28)


### Features

* Added poseidon2 hash function to barretenberg/crypto ([#3118](https://github.com/AztecProtocol/aztec-packages/issues/3118)) ([d47782b](https://github.com/AztecProtocol/aztec-packages/commit/d47782bb480f7e016dbc77cf962978ddca0632aa))
* Aztec CI files in Noir ([#3430](https://github.com/AztecProtocol/aztec-packages/issues/3430)) ([1621f3a](https://github.com/AztecProtocol/aztec-packages/commit/1621f3a1cec3ad16fe7e87160f9b43d3f9490dbd))
* Persistent archiver store ([#3410](https://github.com/AztecProtocol/aztec-packages/issues/3410)) ([4735bde](https://github.com/AztecProtocol/aztec-packages/commit/4735bdebc059a323f4be0c4adf2d4ba644eeecc0)), closes [#3361](https://github.com/AztecProtocol/aztec-packages/issues/3361)


### Bug Fixes

* **ci:** Don't leave DRY_DEPLOY unset ([#3449](https://github.com/AztecProtocol/aztec-packages/issues/3449)) ([454e316](https://github.com/AztecProtocol/aztec-packages/commit/454e316a48056e944519220aa40ffe8286e2a3bd))
* **ci:** Publishing dockerhub manifests ([#3451](https://github.com/AztecProtocol/aztec-packages/issues/3451)) ([a59e7f0](https://github.com/AztecProtocol/aztec-packages/commit/a59e7f020e80916e501811c762876c36692742fc))
* Hotfix noir sync ([#3436](https://github.com/AztecProtocol/aztec-packages/issues/3436)) ([c4e4745](https://github.com/AztecProtocol/aztec-packages/commit/c4e4745df22634a2649f8e7b6e116dc6b399e31f))


### Miscellaneous

* **docs:** Core concepts page in getting-started ([#3401](https://github.com/AztecProtocol/aztec-packages/issues/3401)) ([1a62f73](https://github.com/AztecProtocol/aztec-packages/commit/1a62f73006b406c105bf5b98f2a099690ba83af6))
* Point acir tests at noir master branch ([#3440](https://github.com/AztecProtocol/aztec-packages/issues/3440)) ([106e690](https://github.com/AztecProtocol/aztec-packages/commit/106e690993cdc10db85903d91af873c04744c05f))


### Documentation

* Further updates to the gas and fees whitepaper ([#3448](https://github.com/AztecProtocol/aztec-packages/issues/3448)) ([4152ba6](https://github.com/AztecProtocol/aztec-packages/commit/4152ba60432180dba3f7af0d30eff708828d40f1))
* Updates to gas and fees yellow paper ([#3438](https://github.com/AztecProtocol/aztec-packages/issues/3438)) ([5f0e1ca](https://github.com/AztecProtocol/aztec-packages/commit/5f0e1cad2872d9d16953fa3bc8d43f6cf2709d96))

## [0.16.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.15.1...aztec-packages-v0.16.0) (2023-11-27)


### ⚠ BREAKING CHANGES

* deprecate circuits/cpp ([#3421](https://github.com/AztecProtocol/aztec-packages/issues/3421))
* call stack validation optimisation. ([#3387](https://github.com/AztecProtocol/aztec-packages/issues/3387))

### Features

* Base rollup in noir ([#3257](https://github.com/AztecProtocol/aztec-packages/issues/3257)) ([4a1e9c3](https://github.com/AztecProtocol/aztec-packages/commit/4a1e9c32b1aae52811e348d67cab468cdab89bd6))
* Call stack validation optimisation. ([#3387](https://github.com/AztecProtocol/aztec-packages/issues/3387)) ([d06d5db](https://github.com/AztecProtocol/aztec-packages/commit/d06d5db376ac4bfb3240046be904205082e77a33))
* Goblin proof construction ([#3332](https://github.com/AztecProtocol/aztec-packages/issues/3332)) ([6a7ebb6](https://github.com/AztecProtocol/aztec-packages/commit/6a7ebb60e4ecf0ae0d047814e22ecd88c9c7528f))
* More logs relevant for debugging failures of 2 pixies test ([#3370](https://github.com/AztecProtocol/aztec-packages/issues/3370)) ([683a0f3](https://github.com/AztecProtocol/aztec-packages/commit/683a0f38ac61aa4f9ef8f0b29d3e4f736cdd2771))
* Noir subrepo. ([#3369](https://github.com/AztecProtocol/aztec-packages/issues/3369)) ([d94d88b](https://github.com/AztecProtocol/aztec-packages/commit/d94d88bf626ddbe41dd1b7fe3eb0f11619dde97a))
* Noir_wasm compilation of noir programs ([#3272](https://github.com/AztecProtocol/aztec-packages/issues/3272)) ([f9981d5](https://github.com/AztecProtocol/aztec-packages/commit/f9981d5a9b719f0a3220cf069a2bd6ac8c483437))
* Rollback public state changes on failure ([#3393](https://github.com/AztecProtocol/aztec-packages/issues/3393)) ([0e276fb](https://github.com/AztecProtocol/aztec-packages/commit/0e276fb9f2ce046467032dfdd8210c776ff7e0d2))


### Bug Fixes

* **docs:** Doc explaining noir debug_log ([#3322](https://github.com/AztecProtocol/aztec-packages/issues/3322)) ([eed023d](https://github.com/AztecProtocol/aztec-packages/commit/eed023d2f8c642ee1dab8c9910d7a1651622070c))
* Naming inconsistency in private kernel ([#3384](https://github.com/AztecProtocol/aztec-packages/issues/3384)) ([4743486](https://github.com/AztecProtocol/aztec-packages/commit/4743486411ed56a49660f053f0645e3895c0d44c))
* Race condition in `PXE.getTxReceipt(...)` ([#3411](https://github.com/AztecProtocol/aztec-packages/issues/3411)) ([9557a66](https://github.com/AztecProtocol/aztec-packages/commit/9557a66dce6104e794a7ab20172738954d4315ba))


### Miscellaneous

* Deprecate circuits/cpp ([#3421](https://github.com/AztecProtocol/aztec-packages/issues/3421)) ([4973cfb](https://github.com/AztecProtocol/aztec-packages/commit/4973cfbf352449d32017dd6ebce36e75a608ff41))
* Deterministically deduplicate `cached_partial_non_native_field_multiplication` across wasm32 and native compilations ([#3425](https://github.com/AztecProtocol/aztec-packages/issues/3425)) ([5524933](https://github.com/AztecProtocol/aztec-packages/commit/55249336212764da4b85634e7d35e8fedb147619))
* **docs:** Common patterns and anti patterns in aztec.nr ([#3413](https://github.com/AztecProtocol/aztec-packages/issues/3413)) ([65bd855](https://github.com/AztecProtocol/aztec-packages/commit/65bd8556875a8680bb44fc7ec9321c5df8d8ad38))
* Fix and reenable e2e quick start ([#3403](https://github.com/AztecProtocol/aztec-packages/issues/3403)) ([112740e](https://github.com/AztecProtocol/aztec-packages/commit/112740eb51f512d2099d7ba3537bb55fde3797e5)), closes [#3356](https://github.com/AztecProtocol/aztec-packages/issues/3356)
* Fix intermittent failures for block-building e2e test ([#3404](https://github.com/AztecProtocol/aztec-packages/issues/3404)) ([e76e2d4](https://github.com/AztecProtocol/aztec-packages/commit/e76e2d4190399ccc917852be7873fffd2f0acf9a)), closes [#3358](https://github.com/AztecProtocol/aztec-packages/issues/3358)
* Formatted `noir-contracts` and `aztec-nr` ([a73c4aa](https://github.com/AztecProtocol/aztec-packages/commit/a73c4aacc9c8ecd5a2d83f5d07bdf76bcbb13eed))
* Initial clone of noir to subrepo ([#3409](https://github.com/AztecProtocol/aztec-packages/issues/3409)) ([8f1cb83](https://github.com/AztecProtocol/aztec-packages/commit/8f1cb832cd0adeff0da69da293bb45a3748583e7))
* **noir-contracts:** Remove redundant return value of 1 ([#3415](https://github.com/AztecProtocol/aztec-packages/issues/3415)) ([2001d47](https://github.com/AztecProtocol/aztec-packages/commit/2001d47408e2bcb704c304ef80e87fbdf0e2d11e)), closes [#2615](https://github.com/AztecProtocol/aztec-packages/issues/2615)
* Plumbs noir subrepo into yarn-project. ([#3420](https://github.com/AztecProtocol/aztec-packages/issues/3420)) ([63173c4](https://github.com/AztecProtocol/aztec-packages/commit/63173c45db127288bc4b079229239a650fc5d4be))
* Remove pxe / node /p2p-bootstrap docker images ([#3396](https://github.com/AztecProtocol/aztec-packages/issues/3396)) ([c236143](https://github.com/AztecProtocol/aztec-packages/commit/c236143388ccc34b3ad1c3f48ad3f7872d447c4c))
* Skip artifacts for prettier ([#3399](https://github.com/AztecProtocol/aztec-packages/issues/3399)) ([98d9e04](https://github.com/AztecProtocol/aztec-packages/commit/98d9e04b32464dbffbb5f109bf69bd3f42236ac3))
* Update path to acir artifacts ([#3426](https://github.com/AztecProtocol/aztec-packages/issues/3426)) ([f56f88d](https://github.com/AztecProtocol/aztec-packages/commit/f56f88de05a0ebfcc34c279ae869956a48baa0f4))

## [0.15.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.15.0...aztec-packages-v0.15.1) (2023-11-21)


### Features

* **bb:** Add ability to write pk to file or stdout ([#3335](https://github.com/AztecProtocol/aztec-packages/issues/3335)) ([c99862c](https://github.com/AztecProtocol/aztec-packages/commit/c99862c9602d7d37f7fef348e9f014fb137adab1))
* DataBus PoC (UltraHonk as extension of Ultra) ([#3181](https://github.com/AztecProtocol/aztec-packages/issues/3181)) ([dd9dd84](https://github.com/AztecProtocol/aztec-packages/commit/dd9dd84e9cfc93f8605f28aa25fa36b0004052cb))
* Deploy docs from CCI w/ netlify-cli ([#3348](https://github.com/AztecProtocol/aztec-packages/issues/3348)) ([624d733](https://github.com/AztecProtocol/aztec-packages/commit/624d7339d6bd9755156487a07c553b00f92c3b4b))
* Fold batching challenge (alpha) ([#3291](https://github.com/AztecProtocol/aztec-packages/issues/3291)) ([bc99a4f](https://github.com/AztecProtocol/aztec-packages/commit/bc99a4f644824727920b0b4a38ec5ba915d5c0ce))
* Open transcript polys as univariates in ECCVM ([#3331](https://github.com/AztecProtocol/aztec-packages/issues/3331)) ([436b22e](https://github.com/AztecProtocol/aztec-packages/commit/436b22e35bf8a41f78def237889f2afd2ca79830))
* Sandbox packages ([#3360](https://github.com/AztecProtocol/aztec-packages/issues/3360)) ([0dc2d58](https://github.com/AztecProtocol/aztec-packages/commit/0dc2d586c60587f62e50bb7af0862d1a3f828688))
* Slow updates experimentation ([#2732](https://github.com/AztecProtocol/aztec-packages/issues/2732)) ([193e6c8](https://github.com/AztecProtocol/aztec-packages/commit/193e6c8e0afd1646f3b90c30c250fc4c087a4dde))
* ZM updates for Translator concatenated polys ([#3343](https://github.com/AztecProtocol/aztec-packages/issues/3343)) ([0e425db](https://github.com/AztecProtocol/aztec-packages/commit/0e425dbfc99af9fc2598a957acd8b71f3fd45fe9))


### Bug Fixes

* Bootstrap bbjs. ([#3337](https://github.com/AztecProtocol/aztec-packages/issues/3337)) ([06aedcb](https://github.com/AztecProtocol/aztec-packages/commit/06aedcbfd601e243d3486763c1306e20c1ae3688))
* Noir-compiler breadth-first resolver ([#3307](https://github.com/AztecProtocol/aztec-packages/issues/3307)) ([02348cf](https://github.com/AztecProtocol/aztec-packages/commit/02348cf94ff21d585ca43c22be69433af9cd3b98))
* Update command looks at devDeps ([#3276](https://github.com/AztecProtocol/aztec-packages/issues/3276)) ([54ee38d](https://github.com/AztecProtocol/aztec-packages/commit/54ee38d94f904a94cec948b9db9ca833f097d9c1)), closes [#3275](https://github.com/AztecProtocol/aztec-packages/issues/3275)
* Updating pedersen benchmarks ([#3211](https://github.com/AztecProtocol/aztec-packages/issues/3211)) ([7e89ff3](https://github.com/AztecProtocol/aztec-packages/commit/7e89ff363521dd65e0c9f0c098b3bacea33c2764))
* Warn on circular imports. ([#3350](https://github.com/AztecProtocol/aztec-packages/issues/3350)) ([5bfbddb](https://github.com/AztecProtocol/aztec-packages/commit/5bfbddb21bc81dd47698f0c8796d0c8dc0a498e0))


### Miscellaneous

* All hashes in ts ([#3333](https://github.com/AztecProtocol/aztec-packages/issues/3333)) ([6307e12](https://github.com/AztecProtocol/aztec-packages/commit/6307e129770af7791dc5a477859b75ebb112a653))
* Compute function tree root in ts. ([#3326](https://github.com/AztecProtocol/aztec-packages/issues/3326)) ([48d8c7f](https://github.com/AztecProtocol/aztec-packages/commit/48d8c7fd53c11b2d84c8f8e9e137ce0bb0dc3604))
* **docs:** Suggest CLI install per project ([#3267](https://github.com/AztecProtocol/aztec-packages/issues/3267)) ([b4c967b](https://github.com/AztecProtocol/aztec-packages/commit/b4c967bcb222e410030fe6066b32aa1802ddb15b))
* Enforce bracing around blocks. Generally considered easier to read and less error prone. ([#3349](https://github.com/AztecProtocol/aztec-packages/issues/3349)) ([ee11dec](https://github.com/AztecProtocol/aztec-packages/commit/ee11decb8d3ba65d2a74aedf72396b57fccb1db6))
* Fix circulars in foundation. Also cleanup fields and optimise to be buffer underlying. ([#3351](https://github.com/AztecProtocol/aztec-packages/issues/3351)) ([c4bf8d3](https://github.com/AztecProtocol/aztec-packages/commit/c4bf8d371550e27ee8982ec3ea2a6848bd02a46f))
* Public kernel tests ([#3325](https://github.com/AztecProtocol/aztec-packages/issues/3325)) ([bace972](https://github.com/AztecProtocol/aztec-packages/commit/bace9722cbb5903ea28ebc8e32ddfa4cf784e62a))


### Documentation

* Fixed errors in Gas and Fees yellow paper ([#3363](https://github.com/AztecProtocol/aztec-packages/issues/3363)) ([d818206](https://github.com/AztecProtocol/aztec-packages/commit/d818206391801b69fff9ff63d4aeefa1bb3fb72a))
* Initial network section of yellow paper ([#3341](https://github.com/AztecProtocol/aztec-packages/issues/3341)) ([5a18615](https://github.com/AztecProtocol/aztec-packages/commit/5a18615fe68a25adf33f9d158c03cf9d68fbcfc6))
* Yellow paper section on Gas and Fees ([#3327](https://github.com/AztecProtocol/aztec-packages/issues/3327)) ([caa7e10](https://github.com/AztecProtocol/aztec-packages/commit/caa7e10565f2b9177085fca5fc3366ccea5f0d33))

## [0.15.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.14.2...aztec-packages-v0.15.0) (2023-11-16)


### ⚠ BREAKING CHANGES

* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114))

### Features

* **bb:** Add msan preset ([#3284](https://github.com/AztecProtocol/aztec-packages/issues/3284)) ([bcf025c](https://github.com/AztecProtocol/aztec-packages/commit/bcf025ceef07fb2bf5b07f96e7818425ae59e79a))
* Enable merge and root rollup circuits in noir ([#3248](https://github.com/AztecProtocol/aztec-packages/issues/3248)) ([68555fc](https://github.com/AztecProtocol/aztec-packages/commit/68555fca71746579c7551a78a13b965400d2c865))
* Protogalaxy combiner quotient ([#3245](https://github.com/AztecProtocol/aztec-packages/issues/3245)) ([db0f3ab](https://github.com/AztecProtocol/aztec-packages/commit/db0f3ab9b3d74e0527116a773bf11d26e6bf7736))
* Public kernel in noir  ([#3186](https://github.com/AztecProtocol/aztec-packages/issues/3186)) ([15a522b](https://github.com/AztecProtocol/aztec-packages/commit/15a522ba731820851f1bf505bc2663314e4efc30))
* Ultra honk arith from ultra ([#3274](https://github.com/AztecProtocol/aztec-packages/issues/3274)) ([ec2b805](https://github.com/AztecProtocol/aztec-packages/commit/ec2b805e5b35805e2c5e394ae2b6181865e22aa3))


### Bug Fixes

* Debug build ([#3283](https://github.com/AztecProtocol/aztec-packages/issues/3283)) ([aca2624](https://github.com/AztecProtocol/aztec-packages/commit/aca2624df2d07782f6879d32efc891318b985344))
* Fix block constraint key divergence bug. ([#3256](https://github.com/AztecProtocol/aztec-packages/issues/3256)) ([1c71a0c](https://github.com/AztecProtocol/aztec-packages/commit/1c71a0cf38cf463efe1964126a6a5741c27bd2eb))
* Main.md typo ([#3278](https://github.com/AztecProtocol/aztec-packages/issues/3278)) ([cb87c4d](https://github.com/AztecProtocol/aztec-packages/commit/cb87c4df5e37a689e8ea32a138f794bbe099f884))
* Typo fix roundup ([#3302](https://github.com/AztecProtocol/aztec-packages/issues/3302)) ([9dd778d](https://github.com/AztecProtocol/aztec-packages/commit/9dd778d6856b87107b88e4e8e38d0fc6fc6479fc))


### Miscellaneous

* **bb:** Remove -Wfatal-errors ([#3318](https://github.com/AztecProtocol/aztec-packages/issues/3318)) ([4229173](https://github.com/AztecProtocol/aztec-packages/commit/4229173e7d794ba7800b34dcc8565d7f3ea5525d))
* Clarify that barretenberg mirror should not take PRs ([#3303](https://github.com/AztecProtocol/aztec-packages/issues/3303)) ([13f1a1d](https://github.com/AztecProtocol/aztec-packages/commit/13f1a1d4f8cd12ac8f38e2d1a2c6715f2871f4c8))
* Clean up Plonk widgets ([#3305](https://github.com/AztecProtocol/aztec-packages/issues/3305)) ([4623d91](https://github.com/AztecProtocol/aztec-packages/commit/4623d916d5e8d048cf3c5e06f02d937cf91e6180))
* **docs:** Aztec.nr logging page ([#3281](https://github.com/AztecProtocol/aztec-packages/issues/3281)) ([11e6ca7](https://github.com/AztecProtocol/aztec-packages/commit/11e6ca732c90dc25eceda00f8ac30620a064ebf6))
* **docs:** Update netlify.toml and fix build ([#3304](https://github.com/AztecProtocol/aztec-packages/issues/3304)) ([df76636](https://github.com/AztecProtocol/aztec-packages/commit/df76636293091e2761721eff6f2bdf7243b642e1))
* Explicitly instantiate Goblin translator relations ([#3239](https://github.com/AztecProtocol/aztec-packages/issues/3239)) ([e3b5fb0](https://github.com/AztecProtocol/aztec-packages/commit/e3b5fb0681839bd003804a9e066118dd4693502d))
* Plain struct flavor entities ([#3277](https://github.com/AztecProtocol/aztec-packages/issues/3277)) ([f109512](https://github.com/AztecProtocol/aztec-packages/commit/f1095124af96d2d69522c8677e5e02cd55063c99))
* Remove bn254 instantiation of eccvm plus naming changes ([#3330](https://github.com/AztecProtocol/aztec-packages/issues/3330)) ([23d1e2d](https://github.com/AztecProtocol/aztec-packages/commit/23d1e2d307757c42f6a070afcb22f800fae94555))
* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114)) ([87eeb71](https://github.com/AztecProtocol/aztec-packages/commit/87eeb715014996ec329de969df85684083b18f83))
* Revert build-debug folder for debug preset ([#3324](https://github.com/AztecProtocol/aztec-packages/issues/3324)) ([43a2e6b](https://github.com/AztecProtocol/aztec-packages/commit/43a2e6b68853d5c22fac4563949c83baf443827c))
* Towards plain struct flavor entities ([#3216](https://github.com/AztecProtocol/aztec-packages/issues/3216)) ([3ba89cf](https://github.com/AztecProtocol/aztec-packages/commit/3ba89cf6fe3821b1149f482ee28c5e0716878b15))
* Typo fixes based on cspell ([#3319](https://github.com/AztecProtocol/aztec-packages/issues/3319)) ([8ae44dd](https://github.com/AztecProtocol/aztec-packages/commit/8ae44dd702987db524ab5e3edd6545881614f56b))

## [0.14.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.14.1...aztec-packages-v0.14.2) (2023-11-07)


### Features

* Load private tests and docs ([#3243](https://github.com/AztecProtocol/aztec-packages/issues/3243)) ([f3d8aae](https://github.com/AztecProtocol/aztec-packages/commit/f3d8aae1354f54090c7d61445bf54c3f3d974b09)), closes [#1285](https://github.com/AztecProtocol/aztec-packages/issues/1285)
* Run solidity tests for all acir artifacts ([#3161](https://github.com/AztecProtocol/aztec-packages/issues/3161)) ([d09f667](https://github.com/AztecProtocol/aztec-packages/commit/d09f66748fcbb7739b17940a36806abb72091ee1))


### Bug Fixes

* Wait for accounts to catch up with notes when deployed ([#2834](https://github.com/AztecProtocol/aztec-packages/issues/2834)) ([a8f3119](https://github.com/AztecProtocol/aztec-packages/commit/a8f31199a916f63111212be3973a398ccaf2089d))


### Miscellaneous

* Add noir-protocol-circuits to deploy_npm ([#3268](https://github.com/AztecProtocol/aztec-packages/issues/3268)) ([1a22cae](https://github.com/AztecProtocol/aztec-packages/commit/1a22cae3ffe2b9dc947aba96d631eea4ad403953))
* Aztec-cli better volume mounting strategy ([#3138](https://github.com/AztecProtocol/aztec-packages/issues/3138)) ([d40460e](https://github.com/AztecProtocol/aztec-packages/commit/d40460e261c916f5d4735716215452d5df3c12ea))
* Disable circuits tasks ([#3253](https://github.com/AztecProtocol/aztec-packages/issues/3253)) ([e8945f8](https://github.com/AztecProtocol/aztec-packages/commit/e8945f80260649f30beabdd6195e6e9ef554b36f))

## [0.14.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.14.0...aztec-packages-v0.14.1) (2023-11-07)


### Bug Fixes

* Remove aztec.nr version check from noir-compiler ([#3263](https://github.com/AztecProtocol/aztec-packages/issues/3263)) ([e2e4775](https://github.com/AztecProtocol/aztec-packages/commit/e2e477576eabb7006d7c9e74cc363c15306d9fde))

## [0.14.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.13.1...aztec-packages-v0.14.0) (2023-11-07)


### ⚠ BREAKING CHANGES

* make noir_wasm the default compiler ([#3090](https://github.com/AztecProtocol/aztec-packages/issues/3090))
* adding all the (note, nonce) pairs in `PXE.addNote` and hiding `PXE.getNoteNonces` ([#3196](https://github.com/AztecProtocol/aztec-packages/issues/3196))
* API inconsistency fix ([#3190](https://github.com/AztecProtocol/aztec-packages/issues/3190))
* tree leaf value as `Fr` everywhere in our public API ([#3173](https://github.com/AztecProtocol/aztec-packages/issues/3173))

### Features

* Add cli command update aztec dependencies ([#3128](https://github.com/AztecProtocol/aztec-packages/issues/3128)) ([0c05d8b](https://github.com/AztecProtocol/aztec-packages/commit/0c05d8bb5c5b9fc5e8d4490e7b748ea23821d4fd))
* Add root rollup circuit ([#3217](https://github.com/AztecProtocol/aztec-packages/issues/3217)) ([fb4f7af](https://github.com/AztecProtocol/aztec-packages/commit/fb4f7af0b37a9b291e1dd2729f46a3b7a92d208b))
* Adding all the (note, nonce) pairs in `PXE.addNote` and hiding `PXE.getNoteNonces` ([#3196](https://github.com/AztecProtocol/aztec-packages/issues/3196)) ([8c41664](https://github.com/AztecProtocol/aztec-packages/commit/8c41664f3aa1000f08ad36b9bc67dbbd2e8dd71f))
* API inconsistency fix ([#3190](https://github.com/AztecProtocol/aztec-packages/issues/3190)) ([272eda1](https://github.com/AztecProtocol/aztec-packages/commit/272eda153ab0db20969c7f186cc95e7a1bcd3a9b))
* **docs:** Aztec.nr errors in docs ([#3113](https://github.com/AztecProtocol/aztec-packages/issues/3113)) ([fb1e80b](https://github.com/AztecProtocol/aztec-packages/commit/fb1e80b4de9dcc7992d96c6e5e3b0456584fe65b))
* **docs:** New getting started flow  ([#2957](https://github.com/AztecProtocol/aztec-packages/issues/2957)) ([f23f868](https://github.com/AztecProtocol/aztec-packages/commit/f23f8687558f59de2813595342876cd3de3869de))
* Enable pkc in noir ([#3194](https://github.com/AztecProtocol/aztec-packages/issues/3194)) ([1ef892b](https://github.com/AztecProtocol/aztec-packages/commit/1ef892bac3b1ebb56cb78c819288bd7b87fa46ca))
* Extract types to a types crate ([#3203](https://github.com/AztecProtocol/aztec-packages/issues/3203)) ([4161be9](https://github.com/AztecProtocol/aztec-packages/commit/4161be99e46a07cd4b62aed79791e47865271406))
* Gperftools ([#3096](https://github.com/AztecProtocol/aztec-packages/issues/3096)) ([ea2f9a7](https://github.com/AztecProtocol/aztec-packages/commit/ea2f9a72674ae7fd3e810a12026bfc26c693e1c1))
* Initial storage slots docs ([#2842](https://github.com/AztecProtocol/aztec-packages/issues/2842)) ([e8bcd03](https://github.com/AztecProtocol/aztec-packages/commit/e8bcd037c04e822c4894c156ed9450a5e84d8f15))
* Make noir_wasm the default compiler ([#3090](https://github.com/AztecProtocol/aztec-packages/issues/3090)) ([ca52a3e](https://github.com/AztecProtocol/aztec-packages/commit/ca52a3e28d15725f8ca6050a9b3c224a9828df14))
* Migrate cpp private kernel tests to noir ([#3165](https://github.com/AztecProtocol/aztec-packages/issues/3165)) ([daee2f9](https://github.com/AztecProtocol/aztec-packages/commit/daee2f94dbadf02170a595aa6c14e286abbcd8a4))
* More test info in tx receipt ([#3221](https://github.com/AztecProtocol/aztec-packages/issues/3221)) ([a7354dc](https://github.com/AztecProtocol/aztec-packages/commit/a7354dc7e7642b4a4a9ea9ee015242a83ef00cbc)), closes [#3218](https://github.com/AztecProtocol/aztec-packages/issues/3218)
* Tag  artifacts with the compiler version ([#3220](https://github.com/AztecProtocol/aztec-packages/issues/3220)) ([c7490c5](https://github.com/AztecProtocol/aztec-packages/commit/c7490c58639ba70d6273e78f31dba5ad6eaaa654))
* Tree leaf value as `Fr` everywhere in our public API ([#3173](https://github.com/AztecProtocol/aztec-packages/issues/3173)) ([09464ca](https://github.com/AztecProtocol/aztec-packages/commit/09464cacb55ead3e2c5f7c111b0d8af65be7060b))


### Bug Fixes

* Attempt to fix spot request hangs. ([#3241](https://github.com/AztecProtocol/aztec-packages/issues/3241)) ([a062026](https://github.com/AztecProtocol/aztec-packages/commit/a062026db1993cbd1283c8db9fcb6a445a03b302))
* Better update steps for dockerized sandbox ([#3204](https://github.com/AztecProtocol/aztec-packages/issues/3204)) ([3ef0bee](https://github.com/AztecProtocol/aztec-packages/commit/3ef0bee4f09b6512a3a9cc48343deba63ab7fe36))
* Build cli image before releasing ([#3140](https://github.com/AztecProtocol/aztec-packages/issues/3140)) ([09c3b75](https://github.com/AztecProtocol/aztec-packages/commit/09c3b75154e688161c96e4c8bb72aee03a4b3c46))
* Cleanup gen_inner_proof_files.sh script. ([#3242](https://github.com/AztecProtocol/aztec-packages/issues/3242)) ([ee57e00](https://github.com/AztecProtocol/aztec-packages/commit/ee57e00da06a2daea571cac579a5f6ef9e039d5e))
* Corrects typo in repo readme ([#3236](https://github.com/AztecProtocol/aztec-packages/issues/3236)) ([0ed8c79](https://github.com/AztecProtocol/aztec-packages/commit/0ed8c79677c3679e19e7d90317b7bd3e12c4248f))
* Remove noirup from noir-contract's bootstrap ([#3252](https://github.com/AztecProtocol/aztec-packages/issues/3252)) ([d10342e](https://github.com/AztecProtocol/aztec-packages/commit/d10342eb6c7a04d40254a12418448c3cbf5a800d))
* Remove unused import ([#3200](https://github.com/AztecProtocol/aztec-packages/issues/3200)) ([520bba4](https://github.com/AztecProtocol/aztec-packages/commit/520bba4f163224adfd4ddcbc9cdadde752704b61))
* Temporary fix for bb prove w/ ram rom blocks ([#3215](https://github.com/AztecProtocol/aztec-packages/issues/3215)) ([af93a33](https://github.com/AztecProtocol/aztec-packages/commit/af93a33fdd5d73648d31b4e4f7347d29b8892405))
* Update noir-contracts path to types ([#3247](https://github.com/AztecProtocol/aztec-packages/issues/3247)) ([c5fc95d](https://github.com/AztecProtocol/aztec-packages/commit/c5fc95df4b111b15a74d97f2513746e0d29c02b9))
* Wasm-compiler `bin` package type ([#3254](https://github.com/AztecProtocol/aztec-packages/issues/3254)) ([2d50f11](https://github.com/AztecProtocol/aztec-packages/commit/2d50f11db873bc814a7caf4e0bce6d5694df7132))
* Yarn prepare ([#3251](https://github.com/AztecProtocol/aztec-packages/issues/3251)) ([d02726f](https://github.com/AztecProtocol/aztec-packages/commit/d02726fd412edab0c3fbc90e851513054d040f23))


### Miscellaneous

* Add initial skeleton code for root/merge/mase rollups for Noir ([#3178](https://github.com/AztecProtocol/aztec-packages/issues/3178)) ([7b0d076](https://github.com/AztecProtocol/aztec-packages/commit/7b0d07672cbe107d089bb7cb7c02fd6703d4b42d))
* Bump noir ([#3197](https://github.com/AztecProtocol/aztec-packages/issues/3197)) ([aa2042d](https://github.com/AztecProtocol/aztec-packages/commit/aa2042d3844844e68c111e330c2abf204db83f59))
* Clean up and refactor arithmetization ([#3164](https://github.com/AztecProtocol/aztec-packages/issues/3164)) ([0370b13](https://github.com/AztecProtocol/aztec-packages/commit/0370b135c723458852894363383bbe9275eb0e56))
* Continuation of note naming update ([#3137](https://github.com/AztecProtocol/aztec-packages/issues/3137)) ([582150f](https://github.com/AztecProtocol/aztec-packages/commit/582150f92fc0b5cf3114a07bd5761add5fbfdca4))
* Disable canary. ([#3244](https://github.com/AztecProtocol/aztec-packages/issues/3244)) ([1a56173](https://github.com/AztecProtocol/aztec-packages/commit/1a5617351bee036c8c01ded61fb994782cc6e240))
* Docs: fix broken link in functions.md ([#3183](https://github.com/AztecProtocol/aztec-packages/issues/3183)) ([fb53f7a](https://github.com/AztecProtocol/aztec-packages/commit/fb53f7a2ec6ca3e253fe909d180e758571dd2bfb))
* **docs:** Fix docs build ([#3249](https://github.com/AztecProtocol/aztec-packages/issues/3249)) ([ec2c0cf](https://github.com/AztecProtocol/aztec-packages/commit/ec2c0cff7d05a315b9b7d758329a0675f0489b34))
* Fix typo in aztec sandbox ([#3191](https://github.com/AztecProtocol/aztec-packages/issues/3191)) ([ed144b1](https://github.com/AztecProtocol/aztec-packages/commit/ed144b1942e040a685621db341f11f87eae9ca6c))
* More boiler plate code for merge rollup ([#3182](https://github.com/AztecProtocol/aztec-packages/issues/3182)) ([ffafcef](https://github.com/AztecProtocol/aztec-packages/commit/ffafcefa726c8b4340650b3c3230a0ee83599e93))
* Move flavors ([#3188](https://github.com/AztecProtocol/aztec-packages/issues/3188)) ([f1ff849](https://github.com/AztecProtocol/aztec-packages/commit/f1ff849d90b3914bf8c24bf54ded8d98b7ffa961))
* Move honk/pcs ([#3187](https://github.com/AztecProtocol/aztec-packages/issues/3187)) ([3870ff8](https://github.com/AztecProtocol/aztec-packages/commit/3870ff8f829c29556d633693875cf30ce8d724eb))
* Move log deriv lookup accum to library ([#3226](https://github.com/AztecProtocol/aztec-packages/issues/3226)) ([189d1bb](https://github.com/AztecProtocol/aztec-packages/commit/189d1bbd6691d0237d69acb012238e97589ee257))
* Move sumcheck ([#3189](https://github.com/AztecProtocol/aztec-packages/issues/3189)) ([410cae3](https://github.com/AztecProtocol/aztec-packages/commit/410cae39aba1387571308567a8022cc51b6d25d1))
* Move transcripts ([#3176](https://github.com/AztecProtocol/aztec-packages/issues/3176)) ([7372d19](https://github.com/AztecProtocol/aztec-packages/commit/7372d19f64737eabfa917f7368a5bf99068f48d5))
* Noir circuit tests ([#3229](https://github.com/AztecProtocol/aztec-packages/issues/3229)) ([dbfb086](https://github.com/AztecProtocol/aztec-packages/commit/dbfb0862a26944e240c84406c7d856c022b25f47))
* Private kernel circuits ([#3240](https://github.com/AztecProtocol/aztec-packages/issues/3240)) ([b7fbe19](https://github.com/AztecProtocol/aztec-packages/commit/b7fbe19a04a1097df7597ac8a32996994840f57c))
* Prune 0 values from L2Tx ([#3224](https://github.com/AztecProtocol/aztec-packages/issues/3224)) ([2de206d](https://github.com/AztecProtocol/aztec-packages/commit/2de206d7959078f5289441e35788d83661a2c9f6))
* Refactor e2e to not use @aztec/circuit-types ([#3175](https://github.com/AztecProtocol/aztec-packages/issues/3175)) ([006a07a](https://github.com/AztecProtocol/aztec-packages/commit/006a07a1dd89e4e4e792d7a3e8332351f8b32351)), closes [#3157](https://github.com/AztecProtocol/aztec-packages/issues/3157)
* Remove extra println in noir protoco circuit ([#3219](https://github.com/AztecProtocol/aztec-packages/issues/3219)) ([3295fdd](https://github.com/AztecProtocol/aztec-packages/commit/3295fdd99d92b215f2cbc2707980a7055557cb2a))
* Split out relations, PG, Honk variants ([#3238](https://github.com/AztecProtocol/aztec-packages/issues/3238)) ([8abd39f](https://github.com/AztecProtocol/aztec-packages/commit/8abd39f5f8a434d96fe259df9c5940787bd705f1))
* Update Noir version ([#3082](https://github.com/AztecProtocol/aztec-packages/issues/3082)) ([59eb6af](https://github.com/AztecProtocol/aztec-packages/commit/59eb6af2da9801fb6927f4b98615b2c3f8f3ec28))


### Documentation

* Fix bad shareable key crypto. better explanations ([#3228](https://github.com/AztecProtocol/aztec-packages/issues/3228)) ([e4a0c4a](https://github.com/AztecProtocol/aztec-packages/commit/e4a0c4ac04c6d50cee8742b025dc9f58e2fcac19))
* Fix in nullifier secrets doc ([#3167](https://github.com/AztecProtocol/aztec-packages/issues/3167)) ([8c70845](https://github.com/AztecProtocol/aztec-packages/commit/8c708450002b41f6fcd54c96aa2f91fb2e427978))
* Move Updating.md file, add aztec tag to nargo update instruction ([#3213](https://github.com/AztecProtocol/aztec-packages/issues/3213)) ([9f71c5d](https://github.com/AztecProtocol/aztec-packages/commit/9f71c5d79df8e8fadf3829b65b8d45fe3445b803))
* Update docs ([#3223](https://github.com/AztecProtocol/aztec-packages/issues/3223)) ([7977064](https://github.com/AztecProtocol/aztec-packages/commit/7977064106e49869888e4684f133ae024d7c3a13))
* Update storage doc ([#3212](https://github.com/AztecProtocol/aztec-packages/issues/3212)) ([d707d4e](https://github.com/AztecProtocol/aztec-packages/commit/d707d4e3fa85bb52c80a10582f2607850e05fc68))
* Updated stale tree docs ([#3166](https://github.com/AztecProtocol/aztec-packages/issues/3166)) ([3d5c98c](https://github.com/AztecProtocol/aztec-packages/commit/3d5c98c3eeb76103c331bfcbefc4127ae39836c7))

## [0.13.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.13.0...aztec-packages-v0.13.1) (2023-10-31)


### Bug Fixes

* Exposing `PXE.getBlock`, exporting `createAztecNodeClient` from `aztec.js` ([#3139](https://github.com/AztecProtocol/aztec-packages/issues/3139)) ([7af345e](https://github.com/AztecProtocol/aztec-packages/commit/7af345e9e96c81a05447a514dc7d27d113ba0948))
* Revert push cli docker image to docker hub ([#3142](https://github.com/AztecProtocol/aztec-packages/issues/3142)) ([7119382](https://github.com/AztecProtocol/aztec-packages/commit/7119382b4646b4bfe6fc43ba217dc62e2f089cd4))

## [0.13.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.12.0...aztec-packages-v0.13.0) (2023-10-31)


### ⚠ BREAKING CHANGES

* PXE.getNotes(...) + refactor of note types ([#3051](https://github.com/AztecProtocol/aztec-packages/issues/3051))

### Features

* `FieldNote` ([#3037](https://github.com/AztecProtocol/aztec-packages/issues/3037)) ([3d1ffd0](https://github.com/AztecProtocol/aztec-packages/commit/3d1ffd08938240405af7f02c12236486e0e9e0ba))
* Add Aztec Boxes page to docs ([#2569](https://github.com/AztecProtocol/aztec-packages/issues/2569)) ([997c15c](https://github.com/AztecProtocol/aztec-packages/commit/997c15c9438d629b9f1e9435b8ceaeb54264b092))
* Adding structure to Transcript ([#2937](https://github.com/AztecProtocol/aztec-packages/issues/2937)) ([db67aa1](https://github.com/AztecProtocol/aztec-packages/commit/db67aa1eb6ae9669d98301efbbb146d6265d58f4))
* Compile noir contracts with noir_wasm ([#2737](https://github.com/AztecProtocol/aztec-packages/issues/2737)) ([524cecf](https://github.com/AztecProtocol/aztec-packages/commit/524cecf40a6148515d92b193024d2ce587e3e5a8))
* Dockerize aztec-cli ([#3031](https://github.com/AztecProtocol/aztec-packages/issues/3031)) ([ec2e3c2](https://github.com/AztecProtocol/aztec-packages/commit/ec2e3c22d307d9c3c265cc97b0e5359e94669acc))
* Efficient ZM quotient computation ([#3016](https://github.com/AztecProtocol/aztec-packages/issues/3016)) ([ebda5fc](https://github.com/AztecProtocol/aztec-packages/commit/ebda5fcbc7321cb3f91b0c7a742b7cbd88a15179))
* **feature_branch:** Private Kernel Circuit ([#2740](https://github.com/AztecProtocol/aztec-packages/issues/2740)) ([f800a36](https://github.com/AztecProtocol/aztec-packages/commit/f800a36d351a18c1c06ee1acff90c6002a699a92))
* Measure plonk rounds ([#3065](https://github.com/AztecProtocol/aztec-packages/issues/3065)) ([c8e1d8b](https://github.com/AztecProtocol/aztec-packages/commit/c8e1d8b9244c3955f0fea6a34a3cc28a81a29d2c))
* Migrate the init kernel CPP tests to noir ([#3091](https://github.com/AztecProtocol/aztec-packages/issues/3091)) ([906429f](https://github.com/AztecProtocol/aztec-packages/commit/906429fa42628b1ccfc79f9d269301b3af133174))
* New script to output table of benchmarks for README pasting. ([#2780](https://github.com/AztecProtocol/aztec-packages/issues/2780)) ([6c20b45](https://github.com/AztecProtocol/aztec-packages/commit/6c20b45993ee9cbd319ab8351e2722e0c912f427))
* Pedersen in typescript. ([#3111](https://github.com/AztecProtocol/aztec-packages/issues/3111)) ([933f1b2](https://github.com/AztecProtocol/aztec-packages/commit/933f1b2c24a3a4bdaafd31e1158ba702ee9874c9))
* Protogalaxy folding of challenges ([#2935](https://github.com/AztecProtocol/aztec-packages/issues/2935)) ([7ed30e8](https://github.com/AztecProtocol/aztec-packages/commit/7ed30e83d2bea8399b7acd477c4dfc739417f96d))
* PXE.getNotes(...) + refactor of note types ([#3051](https://github.com/AztecProtocol/aztec-packages/issues/3051)) ([16abb5a](https://github.com/AztecProtocol/aztec-packages/commit/16abb5ae2e48ed7fcfe3dd2fbf9a111620f30b53))
* Zeromorph with concatenation (Goblin Translator part 10) ([#3006](https://github.com/AztecProtocol/aztec-packages/issues/3006)) ([70b0f17](https://github.com/AztecProtocol/aztec-packages/commit/70b0f17101f3b378df3e9a0247230b9ebf67239a))


### Bug Fixes

* Bad contract txs publishing contract data ([#2673](https://github.com/AztecProtocol/aztec-packages/issues/2673)) ([ccd4611](https://github.com/AztecProtocol/aztec-packages/commit/ccd4611be86c0d73e50c755739afa06dd123c809))
* Better error message for compute_note_hash_and_nullifier. ([#3097](https://github.com/AztecProtocol/aztec-packages/issues/3097)) ([57bec53](https://github.com/AztecProtocol/aztec-packages/commit/57bec53bd5960f5c2b8e78d944c2efd6d0722254))
* Broken `FieldNote` test ([#3135](https://github.com/AztecProtocol/aztec-packages/issues/3135)) ([fe78ecf](https://github.com/AztecProtocol/aztec-packages/commit/fe78ecf7eef1f4b6f92b792dab5aa3bc3dffa322))
* Docker-compose up, rather than run. ([#3081](https://github.com/AztecProtocol/aztec-packages/issues/3081)) ([242f780](https://github.com/AztecProtocol/aztec-packages/commit/242f7806a21a8706779545df15b40f33a7745695))
* Formatting ([#3070](https://github.com/AztecProtocol/aztec-packages/issues/3070)) ([e1633d3](https://github.com/AztecProtocol/aztec-packages/commit/e1633d349a00bd9bdc2bcd2bbcc820c4d26e9928))
* Minor stale naming fix ([#3117](https://github.com/AztecProtocol/aztec-packages/issues/3117)) ([a6786ae](https://github.com/AztecProtocol/aztec-packages/commit/a6786ae615071cfe5a74ca4fcec55ce1e735a902))
* Push cli docker image to docker hub ([#3120](https://github.com/AztecProtocol/aztec-packages/issues/3120)) ([ccad50f](https://github.com/AztecProtocol/aztec-packages/commit/ccad50f0c8761be7298a72770972b75895f5618b))
* Remove duplicate terraform resource definition ([#3066](https://github.com/AztecProtocol/aztec-packages/issues/3066)) ([d5abadb](https://github.com/AztecProtocol/aztec-packages/commit/d5abadb34f843d6f0e5831e905d1241d14df6237))
* Retry request spot ([#3116](https://github.com/AztecProtocol/aztec-packages/issues/3116)) ([82de5f1](https://github.com/AztecProtocol/aztec-packages/commit/82de5f1075e05cf92072e083d92f28feaba80c0a))


### Miscellaneous

* Add stdlib tests for pedersen commitment ([#3075](https://github.com/AztecProtocol/aztec-packages/issues/3075)) ([87fa621](https://github.com/AztecProtocol/aztec-packages/commit/87fa621347e55f82e36c70515c1824161eee5282))
* Automatic c_binds for commit should return a point instead of an Fr element ([#3072](https://github.com/AztecProtocol/aztec-packages/issues/3072)) ([2e289a5](https://github.com/AztecProtocol/aztec-packages/commit/2e289a5d11d28496ac47220bede03268065e0cb7))
* Cleanup remaining mentions of `compress` with pedersen in cpp and ts ([#3074](https://github.com/AztecProtocol/aztec-packages/issues/3074)) ([52cf383](https://github.com/AztecProtocol/aztec-packages/commit/52cf3831794a6ab497c9a40f85859f4cc8ac4700))
* E2e on spots [ci rebuild] ([#3068](https://github.com/AztecProtocol/aztec-packages/issues/3068)) ([15db6bf](https://github.com/AztecProtocol/aztec-packages/commit/15db6bf32a7c2c1f109392761cd2a16b51115ac9))
* Fix dapp_testing e2e race condition ([#3094](https://github.com/AztecProtocol/aztec-packages/issues/3094)) ([89e7c21](https://github.com/AztecProtocol/aztec-packages/commit/89e7c2172ee0db4929a9d2d64a570dae250f85da))
* Remove docs mirror ([#3122](https://github.com/AztecProtocol/aztec-packages/issues/3122)) ([3fa51e2](https://github.com/AztecProtocol/aztec-packages/commit/3fa51e28b480513887b18d0bf6538c178f4d9681))
* Remove endomorphism coefficient from ecc_add_gate ([#3115](https://github.com/AztecProtocol/aztec-packages/issues/3115)) ([d294987](https://github.com/AztecProtocol/aztec-packages/commit/d294987ad25fb69d2934dfade2bf7063ff64bef2))
* Remove unecessary calls to `pedersen__init` ([#3079](https://github.com/AztecProtocol/aztec-packages/issues/3079)) ([84f8db2](https://github.com/AztecProtocol/aztec-packages/commit/84f8db20f482242ac29a23eb4c8876f14f060b4c))
* Remove unused pedersen c_binds ([#3058](https://github.com/AztecProtocol/aztec-packages/issues/3058)) ([e71e5f9](https://github.com/AztecProtocol/aztec-packages/commit/e71e5f94ba920208e7cc9b2b1b9d62678b699812))
* Removes pedersen commit native pairs method ([#3073](https://github.com/AztecProtocol/aztec-packages/issues/3073)) ([69a34c7](https://github.com/AztecProtocol/aztec-packages/commit/69a34c72c9dccbd54072553ed1ecf0460b16db69))
* Rename private-kernel subpackage to protocol-circuits ([#3134](https://github.com/AztecProtocol/aztec-packages/issues/3134)) ([3e07104](https://github.com/AztecProtocol/aztec-packages/commit/3e071046d7b0f280cc3b5c426399902425aa1039))


### Documentation

* Initial keys spec ([#3035](https://github.com/AztecProtocol/aztec-packages/issues/3035)) ([4b24c58](https://github.com/AztecProtocol/aztec-packages/commit/4b24c580b8770a5926209ca4e29a4cf0644bb383))

## [0.12.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.11.1...aztec-packages-v0.12.0) (2023-10-26)


### ⚠ BREAKING CHANGES

* remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033))

### Features

* Add function selector to cli to make it easier for to call functions ([#3053](https://github.com/AztecProtocol/aztec-packages/issues/3053)) ([e0f0a8e](https://github.com/AztecProtocol/aztec-packages/commit/e0f0a8e4558a42a17d183515d5f2912d0cf8723c)), closes [#1996](https://github.com/AztecProtocol/aztec-packages/issues/1996)
* Added correctness tests for several small relations in Goblin Translator (Goblin Translator part 8) ([#2963](https://github.com/AztecProtocol/aztec-packages/issues/2963)) ([4c83250](https://github.com/AztecProtocol/aztec-packages/commit/4c8325093e7d76158a767dcf2854f1cfd274c5ff))
* AWS mainnet fork ([#2986](https://github.com/AztecProtocol/aztec-packages/issues/2986)) ([f491362](https://github.com/AztecProtocol/aztec-packages/commit/f491362329071983e5b16c3e7caa79342f2e93fa))
* Correctness tests for decomposition and non-native field relations (Goblin Translator Part 9) ([#2981](https://github.com/AztecProtocol/aztec-packages/issues/2981)) ([cdc830d](https://github.com/AztecProtocol/aztec-packages/commit/cdc830dd8731d9f8fed85bb46b3ed6771796f526))
* Enable sol verifier tests in ci ([#2997](https://github.com/AztecProtocol/aztec-packages/issues/2997)) ([058de1e](https://github.com/AztecProtocol/aztec-packages/commit/058de1ea92b1c19f76867b93769d8de4bb9a6f55))
* Goblin Translator flavor and permutation correctness (Goblin Translator part 7) ([#2961](https://github.com/AztecProtocol/aztec-packages/issues/2961)) ([737f17f](https://github.com/AztecProtocol/aztec-packages/commit/737f17fdff5a213dd1424c4e668bce41b95b349a))
* Linking errors ([#3004](https://github.com/AztecProtocol/aztec-packages/issues/3004)) ([388a47b](https://github.com/AztecProtocol/aztec-packages/commit/388a47ba402fac0a4b9832845d8b964e28aadb85)), closes [#2969](https://github.com/AztecProtocol/aztec-packages/issues/2969)


### Bug Fixes

* Fix clang-16 check ([#3030](https://github.com/AztecProtocol/aztec-packages/issues/3030)) ([7a5a8b3](https://github.com/AztecProtocol/aztec-packages/commit/7a5a8b3b79c18b45aa29eacc05e9bfb26090cc95))
* Fix docusaurus yellow paper build ([#3063](https://github.com/AztecProtocol/aztec-packages/issues/3063)) ([db54c1f](https://github.com/AztecProtocol/aztec-packages/commit/db54c1f1efc037e9cc73053a5832b764247c3bf5))
* Run deploy step for mainnet fork ([#3052](https://github.com/AztecProtocol/aztec-packages/issues/3052)) ([9b6be22](https://github.com/AztecProtocol/aztec-packages/commit/9b6be227169c9ee923744601ff3072b134e12f41))
* Try fix publish bb ([#3036](https://github.com/AztecProtocol/aztec-packages/issues/3036)) ([51248b5](https://github.com/AztecProtocol/aztec-packages/commit/51248b5af22a8d87b4d87a23444ccea5a3c3a982))
* Unboxing - nargo.toml injection of "-" for "_" ([#3018](https://github.com/AztecProtocol/aztec-packages/issues/3018)) ([83d6c51](https://github.com/AztecProtocol/aztec-packages/commit/83d6c511d3e717202e2eb665579bc70e53fd5370))


### Miscellaneous

* **acir_tests:** Add script to regenerate double_verify_proof inputs ([#3005](https://github.com/AztecProtocol/aztec-packages/issues/3005)) ([9c4eab2](https://github.com/AztecProtocol/aztec-packages/commit/9c4eab27d6a8a774d49f40ccea92faf305caf500))
* Add portal contract option to deploy subcommand of aztec-cli ([#3032](https://github.com/AztecProtocol/aztec-packages/issues/3032)) ([546b410](https://github.com/AztecProtocol/aztec-packages/commit/546b41045ee021239a8d7656c6703eab688f1a0d))
* Fix `pedersen_compress_with_hash_index` c_bind function ([#3054](https://github.com/AztecProtocol/aztec-packages/issues/3054)) ([a136f6e](https://github.com/AztecProtocol/aztec-packages/commit/a136f6e70725500739b518e1bfc96b680c3cb1b2))
* Msg sender is 0 when no entrypoint is called ([#3024](https://github.com/AztecProtocol/aztec-packages/issues/3024)) ([53c6680](https://github.com/AztecProtocol/aztec-packages/commit/53c6680a28672e2fbeea54e24b05abc3a9dc3fd1)), closes [#2949](https://github.com/AztecProtocol/aztec-packages/issues/2949)
* Optimize pedersen hash for the common usecase by not allocating when input fits in scratch space ([#3056](https://github.com/AztecProtocol/aztec-packages/issues/3056)) ([a0d290d](https://github.com/AztecProtocol/aztec-packages/commit/a0d290d3bab6c42809d57d86b5cd5e3948e35abd))
* Proxy redundant `hash` methods ([#3046](https://github.com/AztecProtocol/aztec-packages/issues/3046)) ([df389b5](https://github.com/AztecProtocol/aztec-packages/commit/df389b5f593a202bc644479a6c3dff884b7d3652))
* Remove "non-core artifact" nargo generated files ([#3026](https://github.com/AztecProtocol/aztec-packages/issues/3026)) ([03ebb8e](https://github.com/AztecProtocol/aztec-packages/commit/03ebb8e1d243507b4056b748af7c093f131eaf4d)), closes [#2977](https://github.com/AztecProtocol/aztec-packages/issues/2977)
* Remove `pedersen_buffer_to_field` from c_bind ([#3045](https://github.com/AztecProtocol/aztec-packages/issues/3045)) ([de7e63b](https://github.com/AztecProtocol/aztec-packages/commit/de7e63bf7e1184333c1eaadf2387fef6bf163871))
* Remove pedersen hash oracle ([#3023](https://github.com/AztecProtocol/aztec-packages/issues/3023)) ([0e6958c](https://github.com/AztecProtocol/aztec-packages/commit/0e6958c94e6d00d4132f08baa2cd63141ff8aae7))
* Remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033)) ([a8ea391](https://github.com/AztecProtocol/aztec-packages/commit/a8ea391c95a9fe4fa26a3fa987f52114a40c664a))
* Rename pedersen typescript methods to be called `hash` instead of compress ([#3047](https://github.com/AztecProtocol/aztec-packages/issues/3047)) ([2f7cc5f](https://github.com/AztecProtocol/aztec-packages/commit/2f7cc5fd3242b04fa996b71dbd7282444e82e903))
* Run check rebuild on boxes ([#3000](https://github.com/AztecProtocol/aztec-packages/issues/3000)) ([c503d91](https://github.com/AztecProtocol/aztec-packages/commit/c503d91aba42dc87acd50e9986a32bc93707fbc1))
* Same prettier in boxes and everywhere else ([#3025](https://github.com/AztecProtocol/aztec-packages/issues/3025)) ([0769d20](https://github.com/AztecProtocol/aztec-packages/commit/0769d2077bcc1f31ba36de2f8a9576427036cbed)), closes [#2978](https://github.com/AztecProtocol/aztec-packages/issues/2978)

## [0.11.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.11.0...aztec-packages-v0.11.1) (2023-10-24)


### Features

* ProverPlookupAuxiliaryWidget kernel bench ([#2924](https://github.com/AztecProtocol/aztec-packages/issues/2924)) ([faffc39](https://github.com/AztecProtocol/aztec-packages/commit/faffc39a379c9f215978e4867c3d24dbc638f0b4))


### Bug Fixes

* **ci:** Publish-bb, use clang 16.04 ([#3019](https://github.com/AztecProtocol/aztec-packages/issues/3019)) ([703a964](https://github.com/AztecProtocol/aztec-packages/commit/703a9646a18bf7a9817d4aa6f3fb185c912a6fe7))

## [0.11.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.10.1...aztec-packages-v0.11.0) (2023-10-24)


### ⚠ BREAKING CHANGES

* consistent `deploy` method params ([#2975](https://github.com/AztecProtocol/aztec-packages/issues/2975))

### Features

* Consistent `deploy` method params ([#2975](https://github.com/AztecProtocol/aztec-packages/issues/2975)) ([c50aefb](https://github.com/AztecProtocol/aztec-packages/commit/c50aefb63966e8ec3ae65d051d0b47b13de1d330))
* Pedersen hash in acir format ([#2990](https://github.com/AztecProtocol/aztec-packages/issues/2990)) ([2a4c548](https://github.com/AztecProtocol/aztec-packages/commit/2a4c548bc816a5f379ee841e26bb30411deef56b))


### Bug Fixes

* TokenBox ([#3003](https://github.com/AztecProtocol/aztec-packages/issues/3003)) ([1ad6647](https://github.com/AztecProtocol/aztec-packages/commit/1ad6647ac5ad3a13b52eb78db64d13d941d9718d))


### Miscellaneous

* Update acir_tests reference branch ([#2993](https://github.com/AztecProtocol/aztec-packages/issues/2993)) ([91813a5](https://github.com/AztecProtocol/aztec-packages/commit/91813a55b8503c279ccd38b1d83463b97b86d064))

## [0.10.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.10.0...aztec-packages-v0.10.1) (2023-10-24)


### Features

* Change nullifier tree height to 20 ([#2988](https://github.com/AztecProtocol/aztec-packages/issues/2988)) ([118f9d5](https://github.com/AztecProtocol/aztec-packages/commit/118f9d5eaf1b4339a7e3758d77dffc0b26735fad))


### Bug Fixes

* Aztec-sandbox docker-compose directory ([#2989](https://github.com/AztecProtocol/aztec-packages/issues/2989)) ([a9678d1](https://github.com/AztecProtocol/aztec-packages/commit/a9678d184f680306e5670a3f1b44047ea573d2b1))

## [0.10.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.9.0...aztec-packages-v0.10.0) (2023-10-24)


### ⚠ BREAKING CHANGES

* Emitting encrypted log by default ([#2926](https://github.com/AztecProtocol/aztec-packages/issues/2926))

### Features

* Added register-account command to cli ([#2980](https://github.com/AztecProtocol/aztec-packages/issues/2980)) ([0977a90](https://github.com/AztecProtocol/aztec-packages/commit/0977a90ca3bd8258bfbfb65e7bda92dd1cc7688e))
* **docs:** Fix portals tutorial formatting ([#2929](https://github.com/AztecProtocol/aztec-packages/issues/2929)) ([ab19b67](https://github.com/AztecProtocol/aztec-packages/commit/ab19b671ad5bcbeedc34550bdfdba4cb63bb32ad))
* Emitting encrypted log by default ([#2926](https://github.com/AztecProtocol/aztec-packages/issues/2926)) ([1ea2d4f](https://github.com/AztecProtocol/aztec-packages/commit/1ea2d4fd329b82cb69861431635d699596311a82)), closes [#2912](https://github.com/AztecProtocol/aztec-packages/issues/2912)
* Goblin translator non-native field relation (Goblin Translator part 6) ([#2871](https://github.com/AztecProtocol/aztec-packages/issues/2871)) ([c4d8d96](https://github.com/AztecProtocol/aztec-packages/commit/c4d8d963171cf936242e04639154fccc86a0942f))
* Honk profiling by pass, tsan preset ([#2982](https://github.com/AztecProtocol/aztec-packages/issues/2982)) ([a1592fd](https://github.com/AztecProtocol/aztec-packages/commit/a1592fdcde661e09826852fc28bb4aa4c5521863))
* Incorporate docs feedback and add "intermediate" level intros to some pages ([#2598](https://github.com/AztecProtocol/aztec-packages/issues/2598)) ([78f9f52](https://github.com/AztecProtocol/aztec-packages/commit/78f9f52af70ad630ce4ade8348a0766b0c1476ad))
* Nuking `Pokeable` contract ([#2939](https://github.com/AztecProtocol/aztec-packages/issues/2939)) ([583d6fb](https://github.com/AztecProtocol/aztec-packages/commit/583d6fbcdb44a2ffd5175c8bf6d87a87c5f4fa21))
* Protogalaxy Combiner ([#2436](https://github.com/AztecProtocol/aztec-packages/issues/2436)) ([a60c70d](https://github.com/AztecProtocol/aztec-packages/commit/a60c70dca1d920ad88511f77be3ad186afab7bdb))
* Protogalaxy perturbator! ([#2624](https://github.com/AztecProtocol/aztec-packages/issues/2624)) ([509dee6](https://github.com/AztecProtocol/aztec-packages/commit/509dee6108781f3dcd09b3c111be59f42798cac0))
* Refactor pedersen hash standard ([#2592](https://github.com/AztecProtocol/aztec-packages/issues/2592)) ([3085676](https://github.com/AztecProtocol/aztec-packages/commit/3085676dd8a68ac43abc3e5c7843ff437df91d7d))
* Widget benchmarking ([#2897](https://github.com/AztecProtocol/aztec-packages/issues/2897)) ([0e927e9](https://github.com/AztecProtocol/aztec-packages/commit/0e927e9233d7418b9fba4a0142f606e2f92a1f40))


### Bug Fixes

* Add @jest/types to box deps ([#2903](https://github.com/AztecProtocol/aztec-packages/issues/2903)) ([db3fa62](https://github.com/AztecProtocol/aztec-packages/commit/db3fa62e45ce21880c4bead293758c2efe70d4ba))
* Add lint rule for focused tests ([#2901](https://github.com/AztecProtocol/aztec-packages/issues/2901)) ([fd1a1a8](https://github.com/AztecProtocol/aztec-packages/commit/fd1a1a86f21986f16344d2dbd6296a28088a1188))
* Avoid tsc OOM by unignoring an old contract artifact ([#2932](https://github.com/AztecProtocol/aztec-packages/issues/2932)) ([7310600](https://github.com/AztecProtocol/aztec-packages/commit/73106008f464328935997028ca18698965b579a5))
* Bad it.only in tests ([#2900](https://github.com/AztecProtocol/aztec-packages/issues/2900)) ([a1f3af1](https://github.com/AztecProtocol/aztec-packages/commit/a1f3af152aaa37bca105f90581b032fb16f1f9d0))
* Boxes boostrap dont use ts-node directly and add .prettierignore ([#2890](https://github.com/AztecProtocol/aztec-packages/issues/2890)) ([a3b1804](https://github.com/AztecProtocol/aztec-packages/commit/a3b18048479ee3bed3931615e5eabd27efacd404))
* Confusing "Unknown complete address" error ([#2967](https://github.com/AztecProtocol/aztec-packages/issues/2967)) ([3a8f54a](https://github.com/AztecProtocol/aztec-packages/commit/3a8f54a8330620669380cbd1b06551aa10703ec3))
* Force jest to quit, otherwise CI can rack up to 3hrs of credits per job. ([#2899](https://github.com/AztecProtocol/aztec-packages/issues/2899)) ([ba2f671](https://github.com/AztecProtocol/aztec-packages/commit/ba2f671c79ac3c2aa19c769c3db56a27a7e0854f))
* Honk sumcheck performance ([#2925](https://github.com/AztecProtocol/aztec-packages/issues/2925)) ([5fbfe6e](https://github.com/AztecProtocol/aztec-packages/commit/5fbfe6eeccdb23f734fb36f30d1e33340f9fb07a))
* Pending commitments contract using the wrong number of arguments ([#2959](https://github.com/AztecProtocol/aztec-packages/issues/2959)) ([655c322](https://github.com/AztecProtocol/aztec-packages/commit/655c322ab0e71074b3f747c95bfafbd6b7008217))
* Prettierignore in boxes ([#2902](https://github.com/AztecProtocol/aztec-packages/issues/2902)) ([8f7a200](https://github.com/AztecProtocol/aztec-packages/commit/8f7a200e809a9dc6ac8e1beaf3bbf1fd83e5a1fb))
* Randomness in `AddressNote` ([#2965](https://github.com/AztecProtocol/aztec-packages/issues/2965)) ([4dc49a9](https://github.com/AztecProtocol/aztec-packages/commit/4dc49a92428216928d918d893c40745957e5b983))
* Yarn lock ([#2923](https://github.com/AztecProtocol/aztec-packages/issues/2923)) ([7042bc6](https://github.com/AztecProtocol/aztec-packages/commit/7042bc6130f8473b6c59bf9a0146ea8b2c3c7483))


### Miscellaneous

* `Private Data Tree` --&gt; `Note Hash Tree` ([#2945](https://github.com/AztecProtocol/aztec-packages/issues/2945)) ([abaec9c](https://github.com/AztecProtocol/aztec-packages/commit/abaec9c16b300c84fce82242a7a734a4bf0ac0db)), closes [#2906](https://github.com/AztecProtocol/aztec-packages/issues/2906)
* Apply hash abstraction over aztec-nr ([#2958](https://github.com/AztecProtocol/aztec-packages/issues/2958)) ([52f01ae](https://github.com/AztecProtocol/aztec-packages/commit/52f01aea277cb5a522043edc1066d0fda522f8f6))
* **docs:** Add Singleton and ImmutableSingleton `view_note` methods ([#2934](https://github.com/AztecProtocol/aztec-packages/issues/2934)) ([c1497f8](https://github.com/AztecProtocol/aztec-packages/commit/c1497f88f522cb9cbcb44fb6e69522854b604950))
* Fix box frontend styling ([#2919](https://github.com/AztecProtocol/aztec-packages/issues/2919)) ([7e9e8cc](https://github.com/AztecProtocol/aztec-packages/commit/7e9e8cced3cd5af9b99ce3719d9f969cbc61d383))
* Less noisy benchmark reports ([#2916](https://github.com/AztecProtocol/aztec-packages/issues/2916)) ([0df166c](https://github.com/AztecProtocol/aztec-packages/commit/0df166c891a00fa3b8a1c6b69e2c36a0fb45391f))
* Remove unused nix files ([#2933](https://github.com/AztecProtocol/aztec-packages/issues/2933)) ([3174f84](https://github.com/AztecProtocol/aztec-packages/commit/3174f84fe9d92b353d1b2c307ed5757ee941ce00))
* Run all e2e tests against sandbox ([#2891](https://github.com/AztecProtocol/aztec-packages/issues/2891)) ([6c4e26c](https://github.com/AztecProtocol/aztec-packages/commit/6c4e26cfcfbe4b3f3cfd26f549e585ad275373df))
* Token box copies noir source files from noir-contracts on bootstrap ([#2940](https://github.com/AztecProtocol/aztec-packages/issues/2940)) ([a467b96](https://github.com/AztecProtocol/aztec-packages/commit/a467b9601fabe3f6038d9231132f910079763a9b))


### Documentation

* Fix: update cheat codes to connect to ethRpcUrl ([#2922](https://github.com/AztecProtocol/aztec-packages/issues/2922)) ([4ffe9be](https://github.com/AztecProtocol/aztec-packages/commit/4ffe9befc06e9b322fe28a34ba4818c66459c6cd))

## [0.9.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.14...aztec-packages-v0.9.0) (2023-10-17)


### ⚠ BREAKING CHANGES

* nuking `PublicToken` and `PrivateAirdropToken` ([#2873](https://github.com/AztecProtocol/aztec-packages/issues/2873))
* Change blake3 to blake2 in private kernel ([#2861](https://github.com/AztecProtocol/aztec-packages/issues/2861))
* nuking private token ([#2822](https://github.com/AztecProtocol/aztec-packages/issues/2822))

### Features

* Add input support for chained transactions ("pending_read_requests" in private kernel circuit) ([#2869](https://github.com/AztecProtocol/aztec-packages/issues/2869)) ([c1dff38](https://github.com/AztecProtocol/aztec-packages/commit/c1dff38c2f878add4f0bcc41ef7d95ac1def19fb))
* Bump msgpack ([#2884](https://github.com/AztecProtocol/aztec-packages/issues/2884)) ([d7b7fb1](https://github.com/AztecProtocol/aztec-packages/commit/d7b7fb1d70cfb6a592d4cf24c0da92ed9acc7d38))
* Contract ts interface to use only aztec.js imports ([#2876](https://github.com/AztecProtocol/aztec-packages/issues/2876)) ([6952a1a](https://github.com/AztecProtocol/aztec-packages/commit/6952a1ab95b1febd0f1767e3560b2a8cc59622d2))
* Download msgpack ([#2885](https://github.com/AztecProtocol/aztec-packages/issues/2885)) ([8ac8beb](https://github.com/AztecProtocol/aztec-packages/commit/8ac8bebaa8dad39df6f3d6f622e215574062ac52))
* Faucet ([#2856](https://github.com/AztecProtocol/aztec-packages/issues/2856)) ([5bad35f](https://github.com/AztecProtocol/aztec-packages/commit/5bad35f3c0b5048511062f40cd5d45d69bf16355))
* Nuking `PublicToken` and `PrivateAirdropToken` ([#2873](https://github.com/AztecProtocol/aztec-packages/issues/2873)) ([c74311d](https://github.com/AztecProtocol/aztec-packages/commit/c74311d97ade2cac419e4a5999d1187b7a2c1473))
* Nuking private token ([#2822](https://github.com/AztecProtocol/aztec-packages/issues/2822)) ([5d93a47](https://github.com/AztecProtocol/aztec-packages/commit/5d93a470122aaddfcbd1e59e54568357df871098)), closes [#2350](https://github.com/AztecProtocol/aztec-packages/issues/2350)


### Bug Fixes

* Aztec node to save outbox adddress to config ([#2867](https://github.com/AztecProtocol/aztec-packages/issues/2867)) ([b6418a6](https://github.com/AztecProtocol/aztec-packages/commit/b6418a6bf225fcc53e250474172da3b047f5e511))
* Create data dir on node boot ([#2864](https://github.com/AztecProtocol/aztec-packages/issues/2864)) ([2d498b3](https://github.com/AztecProtocol/aztec-packages/commit/2d498b352364debf59af940f0a69c453651a4ad0))
* Don't repeatedly scan for missing messages ([#2886](https://github.com/AztecProtocol/aztec-packages/issues/2886)) ([3fe1cc8](https://github.com/AztecProtocol/aztec-packages/commit/3fe1cc857b83c20bdd5701f685334316db34dd85))
* Fix trailing pipe causing everything to rebuild. Sorry... ([d13ba75](https://github.com/AztecProtocol/aztec-packages/commit/d13ba75bf5bdc7c11b848ca9c8a281f9eec6b015))
* Pad L1 to L2 messages upon retrieval from L1 ([#2879](https://github.com/AztecProtocol/aztec-packages/issues/2879)) ([457669e](https://github.com/AztecProtocol/aztec-packages/commit/457669e81d654c0b77fcf2c7bf98eb335f0914ff))
* Sequencer aborts in-progress block ([#2883](https://github.com/AztecProtocol/aztec-packages/issues/2883)) ([b0915a8](https://github.com/AztecProtocol/aztec-packages/commit/b0915a8d618ac2e8d1401c41527af85648e0b2eb))


### Miscellaneous

* Change blake3 to blake2 in private kernel ([#2861](https://github.com/AztecProtocol/aztec-packages/issues/2861)) ([d629940](https://github.com/AztecProtocol/aztec-packages/commit/d62994073d0476bb62fab16c02fdc484da9edc44))
* Clean canary env & fixes ([#2880](https://github.com/AztecProtocol/aztec-packages/issues/2880)) ([20ad577](https://github.com/AztecProtocol/aztec-packages/commit/20ad57795ac3c66e88ac36af8ceca9235ad86e6b))
* Making anvil silent again ([#2866](https://github.com/AztecProtocol/aztec-packages/issues/2866)) ([90ae5dc](https://github.com/AztecProtocol/aztec-packages/commit/90ae5dc0efa93272950c734ad645b418de7a014c))
* Spell check on forbidden words. ([#2887](https://github.com/AztecProtocol/aztec-packages/issues/2887)) ([06bc4f9](https://github.com/AztecProtocol/aztec-packages/commit/06bc4f952e0e3ae853aaef7b2002eea67c1a1ee0))


### Documentation

* Initial 'protocol description' toc ([#2844](https://github.com/AztecProtocol/aztec-packages/issues/2844)) ([cb18f45](https://github.com/AztecProtocol/aztec-packages/commit/cb18f455d02b00b30da20c6afbeb806921b3a1cf))

## [0.8.14](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.13...aztec-packages-v0.8.14) (2023-10-13)


### Bug Fixes

* Deploy_defaults for canary-end ([#2854](https://github.com/AztecProtocol/aztec-packages/issues/2854)) ([7b189a8](https://github.com/AztecProtocol/aztec-packages/commit/7b189a83114a4206da425c375a77542af0b7df48))
* **docker:** Use entrypoint for mult line commands in docker ([#2853](https://github.com/AztecProtocol/aztec-packages/issues/2853)) ([ab99cd0](https://github.com/AztecProtocol/aztec-packages/commit/ab99cd0f0731b7951286ae2a1667a73f1d406a1a))

## [0.8.13](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.12...aztec-packages-v0.8.13) (2023-10-13)


### Features

* Add deployed contract to PXE from CLI ([#2850](https://github.com/AztecProtocol/aztec-packages/issues/2850)) ([5bad3e3](https://github.com/AztecProtocol/aztec-packages/commit/5bad3e344ee5842d86aebe443bb001e27d1e735b))
* **docs_tutorials:** Token Portal & Uniswap Tutorial ([#2726](https://github.com/AztecProtocol/aztec-packages/issues/2726)) ([dbef55f](https://github.com/AztecProtocol/aztec-packages/commit/dbef55fc63a296e720e270616b8ae7bd642b8a28))


### Bug Fixes

* Added registry contract address to node terraform ([#2851](https://github.com/AztecProtocol/aztec-packages/issues/2851)) ([bfc5feb](https://github.com/AztecProtocol/aztec-packages/commit/bfc5feb1bad76a5a1a4c7deb5ecd674f9ab42a9b))
* Create canary dockerhub manifest ([#2849](https://github.com/AztecProtocol/aztec-packages/issues/2849)) ([1d7bd26](https://github.com/AztecProtocol/aztec-packages/commit/1d7bd26874af4f3c608ce707c81b844e929cc742))
* Fix check_circuit in goblin translator (resulted in flimsy test) ([#2827](https://github.com/AztecProtocol/aztec-packages/issues/2827)) ([98b1679](https://github.com/AztecProtocol/aztec-packages/commit/98b16793b0e84360af8dc70934636d11d7bc7e29))

## [0.8.12](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.11...aztec-packages-v0.8.12) (2023-10-13)


### Features

* Private token box upgrade to new Token contract ([#2824](https://github.com/AztecProtocol/aztec-packages/issues/2824)) ([22794a5](https://github.com/AztecProtocol/aztec-packages/commit/22794a57bbf45cac72dd69bf9838c63b240e6e22))
* Use privacy consistently ([#2833](https://github.com/AztecProtocol/aztec-packages/issues/2833)) ([89b9b6a](https://github.com/AztecProtocol/aztec-packages/commit/89b9b6ac6eeed10484a4c0892d43d9374864ee1d))


### Bug Fixes

* Copied box nargo toml trailing slash ([#2819](https://github.com/AztecProtocol/aztec-packages/issues/2819)) ([ecd2a64](https://github.com/AztecProtocol/aztec-packages/commit/ecd2a64a517e34ada4770d26e6d7f9c578ee82aa))
* Fix rebuild pattern slashes. ([#2843](https://github.com/AztecProtocol/aztec-packages/issues/2843)) ([e32517e](https://github.com/AztecProtocol/aztec-packages/commit/e32517e9eae791b32f94b3816413392ccf0ba096))
* Trigger yarn-project rebuild for .sh files ([#2846](https://github.com/AztecProtocol/aztec-packages/issues/2846)) ([c956254](https://github.com/AztecProtocol/aztec-packages/commit/c95625439e3c779568d4ddf2f0d0ed93519fb4ac))

## [0.8.11](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.10...aztec-packages-v0.8.11) (2023-10-13)


### Features

* **archiver:** Use registry to fetch searchStartBlock ([#2830](https://github.com/AztecProtocol/aztec-packages/issues/2830)) ([e5bc067](https://github.com/AztecProtocol/aztec-packages/commit/e5bc0672b631f21debf96a85a206080e2d9a838c))
* Configure sandbox for network ([#2818](https://github.com/AztecProtocol/aztec-packages/issues/2818)) ([d393a59](https://github.com/AztecProtocol/aztec-packages/commit/d393a5954bb5d80dddf602d1828ab9c9f6e092cb))
* **docker-sandbox:** Allow forks in sandbox ([#2831](https://github.com/AztecProtocol/aztec-packages/issues/2831)) ([ed8431c](https://github.com/AztecProtocol/aztec-packages/commit/ed8431c736ab67dc825316b8ea35ca5c7f078563)), closes [#2726](https://github.com/AztecProtocol/aztec-packages/issues/2726)
* Goblin Translator Decomposition relation (Goblin Translator part 4) ([#2802](https://github.com/AztecProtocol/aztec-packages/issues/2802)) ([3c3cd9f](https://github.com/AztecProtocol/aztec-packages/commit/3c3cd9f62640b505b55916648df6ccddf524cdfc))
* Goblin Translator GenPermSort relation (Goblin Translator part 3) ([#2795](https://github.com/AztecProtocol/aztec-packages/issues/2795)) ([b36fdc4](https://github.com/AztecProtocol/aztec-packages/commit/b36fdc481d16e56fe244c5a10a5223199f9f2e6b))
* Goblin translator opcode constraint and accumulator transfer relations (Goblin Translator part 5) ([#2805](https://github.com/AztecProtocol/aztec-packages/issues/2805)) ([b3d1f28](https://github.com/AztecProtocol/aztec-packages/commit/b3d1f280913494322baee369e6ee4f04353891b3))
* Goblin Translator Permutation relation (Goblin Translator part 2) ([#2790](https://github.com/AztecProtocol/aztec-packages/issues/2790)) ([9a354c9](https://github.com/AztecProtocol/aztec-packages/commit/9a354c94c91f8f2927ca66d0de65b5b893066710))
* Integrate ZeroMorph into Honk ([#2774](https://github.com/AztecProtocol/aztec-packages/issues/2774)) ([ea86869](https://github.com/AztecProtocol/aztec-packages/commit/ea86869e92da3fbf921314fdbca31fdb85a6e274))
* NPM canary deployment ([#2731](https://github.com/AztecProtocol/aztec-packages/issues/2731)) ([7d48ed3](https://github.com/AztecProtocol/aztec-packages/commit/7d48ed3beb70f0ed183407e87dad0fb9310fcf13))
* Purge non native token + reorder params in token portal ([#2723](https://github.com/AztecProtocol/aztec-packages/issues/2723)) ([447dade](https://github.com/AztecProtocol/aztec-packages/commit/447dade3cc21bdd20a24b13fb5d958efea6fed08))
* Throw compile error if read/write public state from private ([#2804](https://github.com/AztecProtocol/aztec-packages/issues/2804)) ([a3649df](https://github.com/AztecProtocol/aztec-packages/commit/a3649df0691e76e108388aebd780748e844ee8c5))
* Unencrypted log filtering ([#2600](https://github.com/AztecProtocol/aztec-packages/issues/2600)) ([7ae554a](https://github.com/AztecProtocol/aztec-packages/commit/7ae554a7c4d725c1ae67b083a0286d15fb76ad0b)), closes [#1498](https://github.com/AztecProtocol/aztec-packages/issues/1498) [#1500](https://github.com/AztecProtocol/aztec-packages/issues/1500)
* Update goblin translator circuit builder (Goblin Translator part 1) ([#2764](https://github.com/AztecProtocol/aztec-packages/issues/2764)) ([32c69ae](https://github.com/AztecProtocol/aztec-packages/commit/32c69ae36ed431482d286e228fd830256e8bd1b5))


### Bug Fixes

* Outdated `noir:clean` ([#2821](https://github.com/AztecProtocol/aztec-packages/issues/2821)) ([2ea199f](https://github.com/AztecProtocol/aztec-packages/commit/2ea199fcd99db73ea2969af7ce0e99501d2cbb5d))


### Miscellaneous

* Benchmark tx sizes in p2p pool ([#2810](https://github.com/AztecProtocol/aztec-packages/issues/2810)) ([f63219c](https://github.com/AztecProtocol/aztec-packages/commit/f63219c91e076a96a49ed16a779a3124fef202c4))
* Change acir_tests branch to point to master ([#2815](https://github.com/AztecProtocol/aztec-packages/issues/2815)) ([73f229d](https://github.com/AztecProtocol/aztec-packages/commit/73f229d3123301818262439a2a98767146a1a58c))
* Fix typo ([#2839](https://github.com/AztecProtocol/aztec-packages/issues/2839)) ([5afdf91](https://github.com/AztecProtocol/aztec-packages/commit/5afdf9105f4980d3ed86ca5fb3a2d6b8e9c33f70))
* From &lt; genesis allowed in getBlocks ([#2816](https://github.com/AztecProtocol/aztec-packages/issues/2816)) ([5622b50](https://github.com/AztecProtocol/aztec-packages/commit/5622b506513f7f1fb491a6be011f90eca1ea96f3))
* Remove Ultra Grumpkin flavor ([#2825](https://github.com/AztecProtocol/aztec-packages/issues/2825)) ([bde77b8](https://github.com/AztecProtocol/aztec-packages/commit/bde77b8e6e91fa734e06453e67a50597480b2ec1))
* Remove work queue from honk ([#2814](https://github.com/AztecProtocol/aztec-packages/issues/2814)) ([bca7d12](https://github.com/AztecProtocol/aztec-packages/commit/bca7d126d2ec583977ee5bdf77a90263d059dc44))
* Spell check ([#2817](https://github.com/AztecProtocol/aztec-packages/issues/2817)) ([4777a11](https://github.com/AztecProtocol/aztec-packages/commit/4777a113491c4c9901b4589a9a6cb1e1148c0288))


### Documentation

* Slight changes to update portal page ([#2799](https://github.com/AztecProtocol/aztec-packages/issues/2799)) ([eb65819](https://github.com/AztecProtocol/aztec-packages/commit/eb65819957a0e5e5c2240ad4f299222133a27edd))
* Update aztec_connect_sunset.mdx ([#2808](https://github.com/AztecProtocol/aztec-packages/issues/2808)) ([5f659a7](https://github.com/AztecProtocol/aztec-packages/commit/5f659a708980c60d015d4292c05e5fd50e7c7f1f))

## [0.8.10](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.9...aztec-packages-v0.8.10) (2023-10-11)


### Features

* Adding Fr back as a BB export (ts) ([#2770](https://github.com/AztecProtocol/aztec-packages/issues/2770)) ([d9ac808](https://github.com/AztecProtocol/aztec-packages/commit/d9ac8080a5525b9792b7b3f10c40583536bb256c))
* Bb faster init ([#2776](https://github.com/AztecProtocol/aztec-packages/issues/2776)) ([c794533](https://github.com/AztecProtocol/aztec-packages/commit/c794533754a9706d362d0374209df9eb5b6bfdc7))
* Deploy l1 contracts npm pkg ([#2754](https://github.com/AztecProtocol/aztec-packages/issues/2754)) ([e317c47](https://github.com/AztecProtocol/aztec-packages/commit/e317c47471f0dc2ef9c95f917406dc1f85dd87e4))
* Docs: Add foundational concepts, ACIR and Sequencer pages ([#2716](https://github.com/AztecProtocol/aztec-packages/issues/2716)) ([9d10326](https://github.com/AztecProtocol/aztec-packages/commit/9d103265e8cde02add16c1a920add5b290a8fc92))
* Events in contract artifacts ([#2786](https://github.com/AztecProtocol/aztec-packages/issues/2786)) ([b8cb7df](https://github.com/AztecProtocol/aztec-packages/commit/b8cb7dfdb68784d60f29249fd49140bde1c8e581)), closes [#2324](https://github.com/AztecProtocol/aztec-packages/issues/2324)
* IAC for a prototype devnet ([#2720](https://github.com/AztecProtocol/aztec-packages/issues/2720)) ([b30839e](https://github.com/AztecProtocol/aztec-packages/commit/b30839e9e5b88124443d35140f84610bbc0a7855))
* **l1-contracts:** Remove remappings of [@aztec](https://github.com/aztec) ([#2797](https://github.com/AztecProtocol/aztec-packages/issues/2797)) ([aac8b37](https://github.com/AztecProtocol/aztec-packages/commit/aac8b37431d4e69db60388cf72c114297977248a))
* LLVM xray presets ([#2525](https://github.com/AztecProtocol/aztec-packages/issues/2525)) ([23a1ee9](https://github.com/AztecProtocol/aztec-packages/commit/23a1ee91da6003d1b5798640c8ccecbd226beef7))
* Separate aggregation protocol ([#2736](https://github.com/AztecProtocol/aztec-packages/issues/2736)) ([ad16937](https://github.com/AztecProtocol/aztec-packages/commit/ad169374943ef49c32eabc66483a7be28a711565))
* Simplify relation containers ([#2619](https://github.com/AztecProtocol/aztec-packages/issues/2619)) ([99c5127](https://github.com/AztecProtocol/aztec-packages/commit/99c5127ac5c10e6637534870a689a95238ae997c))
* ZeroMorph ([#2664](https://github.com/AztecProtocol/aztec-packages/issues/2664)) ([a006e5a](https://github.com/AztecProtocol/aztec-packages/commit/a006e5a0e0a30f8dfe992e3ac8a05f6c276f9300))


### Miscellaneous

* Acir format cleanup ([#2779](https://github.com/AztecProtocol/aztec-packages/issues/2779)) ([5ea373f](https://github.com/AztecProtocol/aztec-packages/commit/5ea373f7d653f7322a108297113a2deb379e1400))
* Add md to rebuild patterns ([#2798](https://github.com/AztecProtocol/aztec-packages/issues/2798)) ([3f4297d](https://github.com/AztecProtocol/aztec-packages/commit/3f4297dbc924ca76fdfba44975c64316f2236deb))
* Make canary uniswap test similar to e2e ([#2767](https://github.com/AztecProtocol/aztec-packages/issues/2767)) ([93d458b](https://github.com/AztecProtocol/aztec-packages/commit/93d458bbbf6c88861b72f00e8fe8beb753857765))
* Measure block building times, history processing times, and db sizes ([#2733](https://github.com/AztecProtocol/aztec-packages/issues/2733)) ([0cc553a](https://github.com/AztecProtocol/aztec-packages/commit/0cc553ab7740c0479582674fce2626a30f3093a9))
* Moved `AddressNote` to `aztec.nr` ([#2752](https://github.com/AztecProtocol/aztec-packages/issues/2752)) ([5f99066](https://github.com/AztecProtocol/aztec-packages/commit/5f99066113480292c8bc56247eca1adb4d49ad5c))
* No calls to pedersen from TS ([#2724](https://github.com/AztecProtocol/aztec-packages/issues/2724)) ([78e44c3](https://github.com/AztecProtocol/aztec-packages/commit/78e44c33bb98fa405f104aafa74b44ce791f239f))
* Remove stale comments ([#2788](https://github.com/AztecProtocol/aztec-packages/issues/2788)) ([d9c458d](https://github.com/AztecProtocol/aztec-packages/commit/d9c458d233d4c4a2ade50cdb6c1fc713e654cb55))
* Renaming abi as artifact ([#2756](https://github.com/AztecProtocol/aztec-packages/issues/2756)) ([c0abcfd](https://github.com/AztecProtocol/aztec-packages/commit/c0abcfd9dfcceb4a2c81561bd89beb9381d20461))
* Rewrite benchmark scripts in ts ([#2765](https://github.com/AztecProtocol/aztec-packages/issues/2765)) ([8efa374](https://github.com/AztecProtocol/aztec-packages/commit/8efa3741ca7503cd38a7de75d5768f1b4d1be287))
* Stop whinging about this ownership stuff. ([#2775](https://github.com/AztecProtocol/aztec-packages/issues/2775)) ([3dd6900](https://github.com/AztecProtocol/aztec-packages/commit/3dd6900f96a7dc855643be0e4aba0cfe9fa8a16e))
* Update ACIR serialisation format ([#2771](https://github.com/AztecProtocol/aztec-packages/issues/2771)) ([6d85527](https://github.com/AztecProtocol/aztec-packages/commit/6d855270f8c069edac62536ccc391a0cab764323))
* Use global crs in more places. Less pain. ([#2772](https://github.com/AztecProtocol/aztec-packages/issues/2772)) ([b819980](https://github.com/AztecProtocol/aztec-packages/commit/b8199802bad3c05ebe4d1ded5338a09a04e0ed7e))


### Documentation

* Add yellow-paper directory ([#2773](https://github.com/AztecProtocol/aztec-packages/issues/2773)) ([03de545](https://github.com/AztecProtocol/aztec-packages/commit/03de545b62ab8d6755fae27b6f2e2bce3575e40e))
* Adding some authwit docs ([#2711](https://github.com/AztecProtocol/aztec-packages/issues/2711)) ([afc23f4](https://github.com/AztecProtocol/aztec-packages/commit/afc23f4652c478298e86f8895f41b21e727a89a6))
* Update overview.mdx ([#2746](https://github.com/AztecProtocol/aztec-packages/issues/2746)) ([082ab56](https://github.com/AztecProtocol/aztec-packages/commit/082ab56d4735a8f08922e36a9897a17fb4fd2c3c))
* Update site title and tagline ([#2769](https://github.com/AztecProtocol/aztec-packages/issues/2769)) ([bbb0b60](https://github.com/AztecProtocol/aztec-packages/commit/bbb0b60d07bc2efa6754b1ad3839735272eeb896))

## [0.8.9](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.8...aztec-packages-v0.8.9) (2023-10-10)


### Features

* Auto-recompile the boxes and fix broken frontend CompleteAddress import ([#2727](https://github.com/AztecProtocol/aztec-packages/issues/2727)) ([4ec4ea0](https://github.com/AztecProtocol/aztec-packages/commit/4ec4ea061e2d003da905d6c2026608b41cdca044))


### Bug Fixes

* Default export in noir-version ([#2757](https://github.com/AztecProtocol/aztec-packages/issues/2757)) ([6ff7bed](https://github.com/AztecProtocol/aztec-packages/commit/6ff7bed1722f8e7afa4b4c495216ca20ea47f42a))


### Documentation

* Add preview image ([#2759](https://github.com/AztecProtocol/aztec-packages/issues/2759)) ([45597af](https://github.com/AztecProtocol/aztec-packages/commit/45597af2a75ffeb8ecd91028f30f159910821673))

## [0.8.8](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.7...aztec-packages-v0.8.8) (2023-10-09)


### Features

* Actually compute selectors ([#2686](https://github.com/AztecProtocol/aztec-packages/issues/2686)) ([dcb65e1](https://github.com/AztecProtocol/aztec-packages/commit/dcb65e1286f8ac4c09e03f65bb3837e02b44ace2))
* Add otterscan to sandbox ([#2648](https://github.com/AztecProtocol/aztec-packages/issues/2648)) ([6986649](https://github.com/AztecProtocol/aztec-packages/commit/69866498c77ba1b38ca88b766172d5dc46acc4f7))
* **aztec.js:** Remove attach method ([#2715](https://github.com/AztecProtocol/aztec-packages/issues/2715)) ([c03c654](https://github.com/AztecProtocol/aztec-packages/commit/c03c654631d6e70bb3f2c6abcd9fa046ce8554b8))
* Create .gitattributes in aztec-nr ([#2661](https://github.com/AztecProtocol/aztec-packages/issues/2661)) ([8084fc3](https://github.com/AztecProtocol/aztec-packages/commit/8084fc3a6a880517284d4aac78a355b9882e88a8))
* GCC 13 preset ([#2623](https://github.com/AztecProtocol/aztec-packages/issues/2623)) ([4881414](https://github.com/AztecProtocol/aztec-packages/commit/4881414ceb30590674c244ef9bc4c8416eacd6bc))
* Update noir to v0.16 ([#2718](https://github.com/AztecProtocol/aztec-packages/issues/2718)) ([e8d0675](https://github.com/AztecProtocol/aztec-packages/commit/e8d0675bfb99369ce488943e127ed03d8ecbe9dc))


### Bug Fixes

* Avoid ambiguity on blank and blank-react (prefix issue) ([#2729](https://github.com/AztecProtocol/aztec-packages/issues/2729)) ([68cdb3f](https://github.com/AztecProtocol/aztec-packages/commit/68cdb3f82cad9b7274c7c4902c2f5919b0acb96b))
* Block encoding ([#2719](https://github.com/AztecProtocol/aztec-packages/issues/2719)) ([c4796ac](https://github.com/AztecProtocol/aztec-packages/commit/c4796ac4ca6b1150cc1ac08fc44fba5a02e1bcf4))
* Canary tests to use a fork ([#2739](https://github.com/AztecProtocol/aztec-packages/issues/2739)) ([4906142](https://github.com/AztecProtocol/aztec-packages/commit/4906142ec611ea82296bcccd7aeefcd929a8d006))
* Challenge generation update ([#2628](https://github.com/AztecProtocol/aztec-packages/issues/2628)) ([68c1fab](https://github.com/AztecProtocol/aztec-packages/commit/68c1fab51e3a339032b719ce966ed34787f33dab))
* Docs: Sandbox version numbers ([#2708](https://github.com/AztecProtocol/aztec-packages/issues/2708)) ([34b0209](https://github.com/AztecProtocol/aztec-packages/commit/34b020974f63f2486c55b821c3c48d583a5e54d0))
* Docs: Update Sandbox page to use #include_aztec_version ([#2703](https://github.com/AztecProtocol/aztec-packages/issues/2703)) ([d5b78af](https://github.com/AztecProtocol/aztec-packages/commit/d5b78af731e4838ecd03a9267dab639681b06512))
* Remove npx from extract_tag_version ([#2697](https://github.com/AztecProtocol/aztec-packages/issues/2697)) ([fe4484a](https://github.com/AztecProtocol/aztec-packages/commit/fe4484a8b9eeb3c997650e94794b0db3b4f4e404))
* Version in sandbox deployment ([#2730](https://github.com/AztecProtocol/aztec-packages/issues/2730)) ([b1d8efd](https://github.com/AztecProtocol/aztec-packages/commit/b1d8efd62e31a49498870cab4c447ace7d5cc1a1))


### Miscellaneous

* `foundation/src/serialization` tech debt ([#2722](https://github.com/AztecProtocol/aztec-packages/issues/2722)) ([e92154b](https://github.com/AztecProtocol/aztec-packages/commit/e92154b891ef6362cec511e1371f8d9ff3007e89))
* Add node10 entrypoint to Foundation ([#2706](https://github.com/AztecProtocol/aztec-packages/issues/2706)) ([30c7935](https://github.com/AztecProtocol/aztec-packages/commit/30c793504951d4eb4f0a192a023fa42fc5d827d1))
* Add storage slot to docs ([#2601](https://github.com/AztecProtocol/aztec-packages/issues/2601)) ([a7710f0](https://github.com/AztecProtocol/aztec-packages/commit/a7710f0849801a85e6907ac0072dd65140ae086a))
* Add visibility modifiers ([#2728](https://github.com/AztecProtocol/aztec-packages/issues/2728)) ([d9ae189](https://github.com/AztecProtocol/aztec-packages/commit/d9ae189bcee43a193d262d2e819c55966494cce7))
* **benchmark:** Measure time to decrypt notes in pxe ([#2714](https://github.com/AztecProtocol/aztec-packages/issues/2714)) ([33a230a](https://github.com/AztecProtocol/aztec-packages/commit/33a230a77488baedb7e93528e296ec47631803c7))
* Build boxes as part of workspace ([#2725](https://github.com/AztecProtocol/aztec-packages/issues/2725)) ([d18349f](https://github.com/AztecProtocol/aztec-packages/commit/d18349f3435677200734a1db625ed80de35c469a))
* Bump ACIR deserializer ([#2675](https://github.com/AztecProtocol/aztec-packages/issues/2675)) ([502ee87](https://github.com/AztecProtocol/aztec-packages/commit/502ee872d6360bf4bc5b83c672eeb64c58944073))
* **circuits:** Delete old code that set a different generator index per vector entry in pedersen commitment ([#2700](https://github.com/AztecProtocol/aztec-packages/issues/2700)) ([4eabfd1](https://github.com/AztecProtocol/aztec-packages/commit/4eabfd1241cce2b2a0c230f600bda3af88f511dd))
* **log:** Show log level in debug logs ([#2717](https://github.com/AztecProtocol/aztec-packages/issues/2717)) ([2b87381](https://github.com/AztecProtocol/aztec-packages/commit/2b873819ad5bade5104813c4ca2624727090ea9e))
* Move { Fr } imports to foundation/fields ([#2712](https://github.com/AztecProtocol/aztec-packages/issues/2712)) ([f6fc7f2](https://github.com/AztecProtocol/aztec-packages/commit/f6fc7f20dfe94c7be9d791d369750234b94c1bbd))
* **uniswap_tests:** Test edge cases around uniswap flow ([#2620](https://github.com/AztecProtocol/aztec-packages/issues/2620)) ([7a58fe9](https://github.com/AztecProtocol/aztec-packages/commit/7a58fe928b658f92afc6914672d64f8742db35bc))
* Use `serialize` functions in `getInitialWitness` ([#2713](https://github.com/AztecProtocol/aztec-packages/issues/2713)) ([93cc668](https://github.com/AztecProtocol/aztec-packages/commit/93cc668d360ae1c599af5e347df7cd8341c59cda))

## [0.8.7](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.6...aztec-packages-v0.8.7) (2023-10-04)


### Bug Fixes

* Copy over wasm in yarn-project ([#2693](https://github.com/AztecProtocol/aztec-packages/issues/2693)) ([033e234](https://github.com/AztecProtocol/aztec-packages/commit/033e2340d53c425b2c76563d2bfda814b4c9cc06))

## [0.8.6](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.5...aztec-packages-v0.8.6) (2023-10-04)


### Bug Fixes

* Do not fail if npm package has not yet been deployed ([#2690](https://github.com/AztecProtocol/aztec-packages/issues/2690)) ([4a52888](https://github.com/AztecProtocol/aztec-packages/commit/4a52888273610134db63c208ed9ea66e58f55585))

## [0.8.5](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.4...aztec-packages-v0.8.5) (2023-10-04)


### Bug Fixes

* Ensure resources ignition directory exists ([#2684](https://github.com/AztecProtocol/aztec-packages/issues/2684)) ([f4f2cd0](https://github.com/AztecProtocol/aztec-packages/commit/f4f2cd04523381824c4e930af81cec79cc50eef9))
* Include resources folder in circuits.js package.json ([#2689](https://github.com/AztecProtocol/aztec-packages/issues/2689)) ([34ed2c2](https://github.com/AztecProtocol/aztec-packages/commit/34ed2c2d4a8e12b649f428dd655a840a1282b69a))
* Npm publish order ([#2687](https://github.com/AztecProtocol/aztec-packages/issues/2687)) ([876c0b1](https://github.com/AztecProtocol/aztec-packages/commit/876c0b1cccbe69d97964d9e90201ffd26adaca3d))

## [0.8.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.3...aztec-packages-v0.8.4) (2023-10-04)


### Bug Fixes

* `deploy_dockerhub.sh` permissions ([#2682](https://github.com/AztecProtocol/aztec-packages/issues/2682)) ([628127d](https://github.com/AztecProtocol/aztec-packages/commit/628127dec9a8903e6d03672d65dab3a195079d9c))
* Deploy npm fixes ([#2685](https://github.com/AztecProtocol/aztec-packages/issues/2685)) ([0b788c6](https://github.com/AztecProtocol/aztec-packages/commit/0b788c615d037754244803c57c1d59dbf8559b88))
* Foundation package is not private ([71d6cda](https://github.com/AztecProtocol/aztec-packages/commit/71d6cda350242c5f0228653111adee09619eafe3))

## [0.8.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.2...aztec-packages-v0.8.3) (2023-10-04)


### Bug Fixes

* Do not depend on npx for check rebuild script ([#2681](https://github.com/AztecProtocol/aztec-packages/issues/2681)) ([20ffbbc](https://github.com/AztecProtocol/aztec-packages/commit/20ffbbc2d906f92f345fae1d6c62954b49fb1c90))
* Remove package json properties whitelist ([#2680](https://github.com/AztecProtocol/aztec-packages/issues/2680)) ([ef499a0](https://github.com/AztecProtocol/aztec-packages/commit/ef499a06f5fcf545e4c8bad6fd59d5f9376c863c))


### Miscellaneous

* Update authwit computation ([#2651](https://github.com/AztecProtocol/aztec-packages/issues/2651)) ([fdbe2b2](https://github.com/AztecProtocol/aztec-packages/commit/fdbe2b2c6a3fc9918921bde5dadbe4d37a64ce11)), closes [#2448](https://github.com/AztecProtocol/aztec-packages/issues/2448)

## [0.8.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.1...aztec-packages-v0.8.2) (2023-10-04)


### Features

* Constrain return notes from oracle call. ([#2639](https://github.com/AztecProtocol/aztec-packages/issues/2639)) ([248be1b](https://github.com/AztecProtocol/aztec-packages/commit/248be1bd44a801808117abd29ba4538aca294af0))
* Multiple pixies 1 Sandbox ([#2492](https://github.com/AztecProtocol/aztec-packages/issues/2492)) ([572d572](https://github.com/AztecProtocol/aztec-packages/commit/572d5721248885a31ef470e1ead2d66907fc39ad))
* Take an optional owner to create the initialization nullifier ([#2647](https://github.com/AztecProtocol/aztec-packages/issues/2647)) ([fefc443](https://github.com/AztecProtocol/aztec-packages/commit/fefc4437f6bf1cda2ec48c6897df4d433eff0816))


### Bug Fixes

* Add missing properties to deployed packages ([#2678](https://github.com/AztecProtocol/aztec-packages/issues/2678)) ([343df30](https://github.com/AztecProtocol/aztec-packages/commit/343df30eb2482ba37e5aa1a264e5d38437b380ec))
* Include ignition data in package or save after 1st download ([#2591](https://github.com/AztecProtocol/aztec-packages/issues/2591)) ([d5e9f8b](https://github.com/AztecProtocol/aztec-packages/commit/d5e9f8be6bbcb8a88dfdec8fee8fe7cf439f6b19)), closes [#2445](https://github.com/AztecProtocol/aztec-packages/issues/2445)
* Make target architecture configurable, target westmere in GA. ([#2660](https://github.com/AztecProtocol/aztec-packages/issues/2660)) ([3cb9639](https://github.com/AztecProtocol/aztec-packages/commit/3cb9639ed1158e70b377aa49832eb650e5cd2930))
* Removal of setting private data root in kernel prover ([#2671](https://github.com/AztecProtocol/aztec-packages/issues/2671)) ([6a2cc28](https://github.com/AztecProtocol/aztec-packages/commit/6a2cc28c6230fedb24d8377a43cfe5d75c53ac8f)), closes [#778](https://github.com/AztecProtocol/aztec-packages/issues/778)


### Miscellaneous

* 1 deploy_dockerhub CI task ([#2670](https://github.com/AztecProtocol/aztec-packages/issues/2670)) ([dff396c](https://github.com/AztecProtocol/aztec-packages/commit/dff396ca03febf80ade82cf4683aaaab20192eb9))
* Check that portal address is saved ([#2641](https://github.com/AztecProtocol/aztec-packages/issues/2641)) ([9ebef6e](https://github.com/AztecProtocol/aztec-packages/commit/9ebef6e04d8ddd25649a325f5b3692b42699629e))
* Fixes in deploy scripts ([#2659](https://github.com/AztecProtocol/aztec-packages/issues/2659)) ([f44568b](https://github.com/AztecProtocol/aztec-packages/commit/f44568b8557aac15b4accf901b1ff72efaf2a1da))
* Measure circuit simulation times and input/output sizes ([#2663](https://github.com/AztecProtocol/aztec-packages/issues/2663)) ([027f7ec](https://github.com/AztecProtocol/aztec-packages/commit/027f7ec95f9d761189166936a7c42d08dacf55b7))
* Remove sandbox base image and force_deploy_build. Generalize in check_rebuild. ([#2645](https://github.com/AztecProtocol/aztec-packages/issues/2645)) ([805fe18](https://github.com/AztecProtocol/aztec-packages/commit/805fe18ec1bd207a713cf3438f6d241bf22317fa))

## [0.8.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.8.0...aztec-packages-v0.8.1) (2023-10-03)


### Bug Fixes

* Add missing ecc doubling gate into ultra plonk and ultra honk  ([#2610](https://github.com/AztecProtocol/aztec-packages/issues/2610)) ([7cb7c58](https://github.com/AztecProtocol/aztec-packages/commit/7cb7c58444a087d81684afc6d5c2fc254357035e))
* Benchmark script fixes for master branch ([#2638](https://github.com/AztecProtocol/aztec-packages/issues/2638)) ([0a161a4](https://github.com/AztecProtocol/aztec-packages/commit/0a161a4fc8a248865602e6729388bb610c2d2200))
* Redirect sunset instructions ([#2646](https://github.com/AztecProtocol/aztec-packages/issues/2646)) ([9253442](https://github.com/AztecProtocol/aztec-packages/commit/9253442144d7814005bcdea886f5d96faa4b1bc9))
* Remove -u from build_wasm script so that we can skip the build when SKIP_CPP_BUILD is unset ([#2649](https://github.com/AztecProtocol/aztec-packages/issues/2649)) ([84b8ff4](https://github.com/AztecProtocol/aztec-packages/commit/84b8ff4b46e1f542209c1f35a33b7cffdc083f04))


### Miscellaneous

* **benchmark:** Measure block sync time ([#2637](https://github.com/AztecProtocol/aztec-packages/issues/2637)) ([d11343f](https://github.com/AztecProtocol/aztec-packages/commit/d11343fb87653a8fc834e5afada2682309b75093))
* Update acir_tests script to point to master ([#2650](https://github.com/AztecProtocol/aztec-packages/issues/2650)) ([51d1e79](https://github.com/AztecProtocol/aztec-packages/commit/51d1e79c3463461864878d4d8f2e84d7e74b9c86))

## [0.8.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.10...aztec-packages-v0.8.0) (2023-10-03)


### ⚠ BREAKING CHANGES

* Gates command should always return 8 bytes ([#2631](https://github.com/AztecProtocol/aztec-packages/issues/2631))

### Features

* **1090:** Validate that some arrays are zero-padded on the right ([#2519](https://github.com/AztecProtocol/aztec-packages/issues/2519)) ([0327b54](https://github.com/AztecProtocol/aztec-packages/commit/0327b544f3763b0188981b8ad491770980e26bb9))
* Add --wait/--no-wait flags to certain cli commands ([#2378](https://github.com/AztecProtocol/aztec-packages/issues/2378)) ([57a2f10](https://github.com/AztecProtocol/aztec-packages/commit/57a2f104623da41a708d0b2f25c20b14a71c3d3c))
* Add boxes to CI ([#2456](https://github.com/AztecProtocol/aztec-packages/issues/2456)) ([a90a185](https://github.com/AztecProtocol/aztec-packages/commit/a90a185bb1d72658c7910366e593303607edf873))
* Add selector to call_context ([#2626](https://github.com/AztecProtocol/aztec-packages/issues/2626)) ([8e317be](https://github.com/AztecProtocol/aztec-packages/commit/8e317be9fafb1daa7bc0bdd08d603ce95d3be2f9))
* AddNote api ([#2535](https://github.com/AztecProtocol/aztec-packages/issues/2535)) ([bb004f4](https://github.com/AztecProtocol/aztec-packages/commit/bb004f4419ca9dba9d8216eaba2e65d3a4a994f8))
* **aztec_noir:** Abstract storage initialization ([#2406](https://github.com/AztecProtocol/aztec-packages/issues/2406)) ([974b037](https://github.com/AztecProtocol/aztec-packages/commit/974b037650e7fac6fbc3721359daf5f1891b5a2a))
* **aztec.js:** Support AddressLike parameters ([#2430](https://github.com/AztecProtocol/aztec-packages/issues/2430)) ([5b5f139](https://github.com/AztecProtocol/aztec-packages/commit/5b5f139af2eb8ceb71e807c49be6c2b54e6e435b))
* Barretenberg/crypto/blake3s supports compile-time hashing ([#2556](https://github.com/AztecProtocol/aztec-packages/issues/2556)) ([da05dd7](https://github.com/AztecProtocol/aztec-packages/commit/da05dd7ea41208aea42efe0aeb838e4d76e2d34a))
* **bb:** Add `bb --version` command ([#2482](https://github.com/AztecProtocol/aztec-packages/issues/2482)) ([530676f](https://github.com/AztecProtocol/aztec-packages/commit/530676f8ec53e63ba24f6fabc9097ae8f5db5fc6))
* **bb:** Avoid initializing CRS for `bb info` command ([#2425](https://github.com/AztecProtocol/aztec-packages/issues/2425)) ([d22c7b1](https://github.com/AztecProtocol/aztec-packages/commit/d22c7b1f69ea936c532fac68d19c6362f8a34be5))
* Benchmarks ([#2605](https://github.com/AztecProtocol/aztec-packages/issues/2605)) ([37d9f9c](https://github.com/AztecProtocol/aztec-packages/commit/37d9f9cc782d5923bf963c7e6b4cbc56f9f923ef))
* Bootstrap_docker skips build it can pull from ecr. ([#2545](https://github.com/AztecProtocol/aztec-packages/issues/2545)) ([466a517](https://github.com/AztecProtocol/aztec-packages/commit/466a517f5dfdd4eafd077d5bdd4dcce2339834ef))
* **cli:** Reenable CLI version check ([#2441](https://github.com/AztecProtocol/aztec-packages/issues/2441)) ([c6ddd23](https://github.com/AztecProtocol/aztec-packages/commit/c6ddd23747e38ea5f248951ed40b0224d167f5c7))
* Collapse interfaces for single implementation ([#2599](https://github.com/AztecProtocol/aztec-packages/issues/2599)) ([860f340](https://github.com/AztecProtocol/aztec-packages/commit/860f3403980f872d6361acc8f6cc31d95d26a635))
* Consistent pedersen hash (work in progress) ([#1945](https://github.com/AztecProtocol/aztec-packages/issues/1945)) ([b4ad8f3](https://github.com/AztecProtocol/aztec-packages/commit/b4ad8f38250d82531439d6db33c8f81387c42496))
* Deprecate assert_contains_and_remove ([#2594](https://github.com/AztecProtocol/aztec-packages/issues/2594)) ([d225d56](https://github.com/AztecProtocol/aztec-packages/commit/d225d56d2e48a84c0c8854fc033b6aad48a1f66e))
* **docs:** Allow raw code interpolation ([#2447](https://github.com/AztecProtocol/aztec-packages/issues/2447)) ([e078ff4](https://github.com/AztecProtocol/aztec-packages/commit/e078ff436b254d802d4ef9a2443fc29f0143dd1b))
* **docs:** Load current aztec version for aztec.nr dependencies in docs ([#2440](https://github.com/AztecProtocol/aztec-packages/issues/2440)) ([63cf415](https://github.com/AztecProtocol/aztec-packages/commit/63cf41586fe7d893329ec4c37483b5219508838b))
* **docs:** Reenable typedoc for aztec-rpc and aztec.js ([#2452](https://github.com/AztecProtocol/aztec-packages/issues/2452)) ([85e504c](https://github.com/AztecProtocol/aztec-packages/commit/85e504c95953cc8ebbb32e2c4ea2f66c7da7a889)), closes [#2045](https://github.com/AztecProtocol/aztec-packages/issues/2045) [#2415](https://github.com/AztecProtocol/aztec-packages/issues/2415)
* **docs:** Use preprocessor syntax for including versions ([#2462](https://github.com/AztecProtocol/aztec-packages/issues/2462)) ([7d315cd](https://github.com/AztecProtocol/aztec-packages/commit/7d315cdb32a8cef809e7622718f2ea72456cec97))
* **docs:** Use released version of code snippets in docs ([#2439](https://github.com/AztecProtocol/aztec-packages/issues/2439)) ([76fc2cf](https://github.com/AztecProtocol/aztec-packages/commit/76fc2cf175da41ca5531a37e65e8afea19a48ed6))
* **docs:** Warn if snippet is grabbed from master ([#2544](https://github.com/AztecProtocol/aztec-packages/issues/2544)) ([36896e7](https://github.com/AztecProtocol/aztec-packages/commit/36896e71960999445e2cf0b67123f5dad8a3721a))
* **e2e:** Public flow for uniswap ([#2596](https://github.com/AztecProtocol/aztec-packages/issues/2596)) ([2f871ee](https://github.com/AztecProtocol/aztec-packages/commit/2f871ee9f385dec026cdb965b3dbe374b291f4e6))
* Enforce that 0th nullifier is non-zero in private kernel circuit ([#2576](https://github.com/AztecProtocol/aztec-packages/issues/2576)) ([458a4fe](https://github.com/AztecProtocol/aztec-packages/commit/458a4fe852a70a1d6c55a0059eb5b6e1e54614a7)), closes [#1329](https://github.com/AztecProtocol/aztec-packages/issues/1329)
* Expose registry address in `getNodeInfo` ([#2478](https://github.com/AztecProtocol/aztec-packages/issues/2478)) ([652bb04](https://github.com/AztecProtocol/aztec-packages/commit/652bb0444deddfb72c323a07b5e88979a18c4f82))
* Expose transaction data from AztecRPC ([#2469](https://github.com/AztecProtocol/aztec-packages/issues/2469)) ([fc00553](https://github.com/AztecProtocol/aztec-packages/commit/fc00553dde3f792928d85094207271f78f5465ba))
* Extend function documentation ([#2408](https://github.com/AztecProtocol/aztec-packages/issues/2408)) ([6a75fd0](https://github.com/AztecProtocol/aztec-packages/commit/6a75fd050dc6d2482bd13b03edb1756c03e14f19))
* Goblin op queue transcript aggregation ([#2257](https://github.com/AztecProtocol/aztec-packages/issues/2257)) ([b7f627a](https://github.com/AztecProtocol/aztec-packages/commit/b7f627a5e472d3dc691b799a5e3df508b685a272))
* Json type by default in `JsonRpcServer` ([#2504](https://github.com/AztecProtocol/aztec-packages/issues/2504)) ([be38fcc](https://github.com/AztecProtocol/aztec-packages/commit/be38fcc9262bfd6fa91c82ab133c71a011b9dd73)), closes [#2479](https://github.com/AztecProtocol/aztec-packages/issues/2479)
* Listing expected args in CLI ([#2423](https://github.com/AztecProtocol/aztec-packages/issues/2423)) ([b2243ad](https://github.com/AztecProtocol/aztec-packages/commit/b2243ad1e46f6fdc961f0002e87842f7600f5bae))
* Log topic and contract address in unencrypted logs ([#2595](https://github.com/AztecProtocol/aztec-packages/issues/2595)) ([a5b763f](https://github.com/AztecProtocol/aztec-packages/commit/a5b763fb077b967f592ad4de9e391acf2790a094)), closes [#2580](https://github.com/AztecProtocol/aztec-packages/issues/2580) [#2581](https://github.com/AztecProtocol/aztec-packages/issues/2581) [#2586](https://github.com/AztecProtocol/aztec-packages/issues/2586) [#2587](https://github.com/AztecProtocol/aztec-packages/issues/2587)
* Parallelization update for polynomials ([#2311](https://github.com/AztecProtocol/aztec-packages/issues/2311)) ([922fc99](https://github.com/AztecProtocol/aztec-packages/commit/922fc9912a4a88a41eef42fe64ca2b59d859b5b1))
* Restore latest block number ([#2474](https://github.com/AztecProtocol/aztec-packages/issues/2474)) ([6dc2da7](https://github.com/AztecProtocol/aztec-packages/commit/6dc2da70584ed1f1f0f00b3dfeca11610e80cc5a))
* Serialize L2Block to JSON ([#2496](https://github.com/AztecProtocol/aztec-packages/issues/2496)) ([714c727](https://github.com/AztecProtocol/aztec-packages/commit/714c727a88d4c07b76e456e462ab1cf43bcaea75))
* Standalone Aztec Node and RPC Server ([#2522](https://github.com/AztecProtocol/aztec-packages/issues/2522)) ([8e355bc](https://github.com/AztecProtocol/aztec-packages/commit/8e355bc8c905d2992678d4a2a3b49d354dfa5bf6))
* Unbox empty box ([#2387](https://github.com/AztecProtocol/aztec-packages/issues/2387)) ([3e3930c](https://github.com/AztecProtocol/aztec-packages/commit/3e3930c6487c3b2a264c7a93bccb25473baf0b22))
* Uniswap private flow ([#2559](https://github.com/AztecProtocol/aztec-packages/issues/2559)) ([39f3a91](https://github.com/AztecProtocol/aztec-packages/commit/39f3a917a3bb88f29d8d17ee6c9e1b2294a45937))
* Update to protogalaxy interfaces ([#2498](https://github.com/AztecProtocol/aztec-packages/issues/2498)) ([9a3d265](https://github.com/AztecProtocol/aztec-packages/commit/9a3d2652d2614439017a6f47152efb9a177b7127))
* YML manifest. Simplify YBP. ([#2353](https://github.com/AztecProtocol/aztec-packages/issues/2353)) ([bf73bc3](https://github.com/AztecProtocol/aztec-packages/commit/bf73bc3e8fd0fd13193f9301073905682044a6c5))


### Bug Fixes

* Add aztec/overview redirect ([#2424](https://github.com/AztecProtocol/aztec-packages/issues/2424)) ([4e30dcd](https://github.com/AztecProtocol/aztec-packages/commit/4e30dcd579cff7bc202f82b710252dc5a61a3315))
* Add redirects from old docs site urls to new site urls ([#2429](https://github.com/AztecProtocol/aztec-packages/issues/2429)) ([18fe88a](https://github.com/AztecProtocol/aztec-packages/commit/18fe88aa63e9a93f4f62789e94047edf33620bfa))
* **barretenberg:** Brittle headers caused error compiling for clang-16 on mainframe ([#2547](https://github.com/AztecProtocol/aztec-packages/issues/2547)) ([cc909da](https://github.com/AztecProtocol/aztec-packages/commit/cc909da0464003aee6d2ff4036ba59c321a5b617))
* Bb rebuild patterns ([#2499](https://github.com/AztecProtocol/aztec-packages/issues/2499)) ([868cceb](https://github.com/AztecProtocol/aztec-packages/commit/868cceb98c7fd6a8edd6710eba4d76ef58a68664))
* Bootstrap.sh ([#2524](https://github.com/AztecProtocol/aztec-packages/issues/2524)) ([bb1fb90](https://github.com/AztecProtocol/aztec-packages/commit/bb1fb907c74894b2a4ed571fd60ea043020a79be))
* Box injected sandbox tag ([#2555](https://github.com/AztecProtocol/aztec-packages/issues/2555)) ([069bdc7](https://github.com/AztecProtocol/aztec-packages/commit/069bdc76775d75f958fd54b466425fdf03653499))
* **build-system:** Don't wait 30s+ always ([#2494](https://github.com/AztecProtocol/aztec-packages/issues/2494)) ([89d700d](https://github.com/AztecProtocol/aztec-packages/commit/89d700d3e161a02549e6eaabf5e11523fc7931f1))
* **build:** CI fixes from previous merges ([#2579](https://github.com/AztecProtocol/aztec-packages/issues/2579)) ([a9e5d05](https://github.com/AztecProtocol/aztec-packages/commit/a9e5d05e702acbc351bea183ac7a077a4e2dec85))
* Bump foundry version ([#2553](https://github.com/AztecProtocol/aztec-packages/issues/2553)) ([0dde3d5](https://github.com/AztecProtocol/aztec-packages/commit/0dde3d5dd0560fbd45e6663a28b68655784a1a6e))
* Call public fn in contract constructor ([#2549](https://github.com/AztecProtocol/aztec-packages/issues/2549)) ([14ab6d6](https://github.com/AztecProtocol/aztec-packages/commit/14ab6d6664c769132d4fedffb9bdd33e364505e8))
* Canary image build ([#2480](https://github.com/AztecProtocol/aztec-packages/issues/2480)) ([6366be5](https://github.com/AztecProtocol/aztec-packages/commit/6366be596f659e1ca4364bc3f0f95c104c8f5717))
* Cli type check arguments and options ([#2571](https://github.com/AztecProtocol/aztec-packages/issues/2571)) ([ecffc36](https://github.com/AztecProtocol/aztec-packages/commit/ecffc366e81cb945ebcde2529a62c627e9f60596))
* **cli:** Typos in cli output ([#2428](https://github.com/AztecProtocol/aztec-packages/issues/2428)) ([08acf90](https://github.com/AztecProtocol/aztec-packages/commit/08acf9090f039112fcf1c9ee0b3c4fb6f4025aba))
* Docs: Token tutorial, update links and add note on imports ([#2604](https://github.com/AztecProtocol/aztec-packages/issues/2604)) ([003d801](https://github.com/AztecProtocol/aztec-packages/commit/003d80117d145a67f7f32bd44ac126b981db6251))
* **docs:** 'command not found: export' ([#2443](https://github.com/AztecProtocol/aztec-packages/issues/2443)) ([f56aa02](https://github.com/AztecProtocol/aztec-packages/commit/f56aa02cea814d00bc39b9b49cefdc5519eb1575))
* **docs:** Docs correction ([#2437](https://github.com/AztecProtocol/aztec-packages/issues/2437)) ([6499248](https://github.com/AztecProtocol/aztec-packages/commit/64992486a3f902462311e7e36f3d9472ac147fe0))
* **docs:** Fix imports in token contract tutorial ([#2432](https://github.com/AztecProtocol/aztec-packages/issues/2432)) ([34ed663](https://github.com/AztecProtocol/aztec-packages/commit/34ed66360c93ba4cc61ff0d19aa37a13373b361d))
* Drop txs with duplicate nullifiers from the same block ([#2511](https://github.com/AztecProtocol/aztec-packages/issues/2511)) ([d9ca1d8](https://github.com/AztecProtocol/aztec-packages/commit/d9ca1d8bebf35659e8fb9cccbdc3c4fec7349514)), closes [#2502](https://github.com/AztecProtocol/aztec-packages/issues/2502)
* E2e browser tests ([#2531](https://github.com/AztecProtocol/aztec-packages/issues/2531)) ([adf2b1e](https://github.com/AztecProtocol/aztec-packages/commit/adf2b1e9f8dd27e463fbe34417fb995900e835f3)), closes [#2527](https://github.com/AztecProtocol/aztec-packages/issues/2527)
* Fix working dir bug causing stdlib-tests to not run. ([#2495](https://github.com/AztecProtocol/aztec-packages/issues/2495)) ([6b3402c](https://github.com/AztecProtocol/aztec-packages/commit/6b3402c552292068dcdf74a920c65b2aad635441))
* Foundry ([#2611](https://github.com/AztecProtocol/aztec-packages/issues/2611)) ([9830fbf](https://github.com/AztecProtocol/aztec-packages/commit/9830fbf7ab41070349c16dce53fa1487e65fb05d))
* Gates command should always return 8 bytes ([#2631](https://github.com/AztecProtocol/aztec-packages/issues/2631)) ([9668165](https://github.com/AztecProtocol/aztec-packages/commit/9668165372c4f5170aa7c4f161e031da0c845649))
* JSON-RPC server returns spec-compliant errors ([#2590](https://github.com/AztecProtocol/aztec-packages/issues/2590)) ([5eafa3d](https://github.com/AztecProtocol/aztec-packages/commit/5eafa3ddbe41e60486422225878d4e6f59021ab9))
* Loading salt into buffer in the cli ([#2467](https://github.com/AztecProtocol/aztec-packages/issues/2467)) ([753ac49](https://github.com/AztecProtocol/aztec-packages/commit/753ac4927ec08485d6673806fcd959b90bf46a3d))
* **master:** Remove secret_hash ref ([#2617](https://github.com/AztecProtocol/aztec-packages/issues/2617)) ([1073bcd](https://github.com/AztecProtocol/aztec-packages/commit/1073bcd742dda8be92f86a46bbab77df19704277))
* Nightly subrepo mirror ([#2520](https://github.com/AztecProtocol/aztec-packages/issues/2520)) ([bedc8c8](https://github.com/AztecProtocol/aztec-packages/commit/bedc8c88cfc24a51806690f225a128f973c5845f))
* Prevent race conditions around data pulled from L1 ([#2577](https://github.com/AztecProtocol/aztec-packages/issues/2577)) ([defea83](https://github.com/AztecProtocol/aztec-packages/commit/defea83088619a8d36cbc1e19f7cade5d45c76c3))
* Readd docs after ci refactor. ([#2514](https://github.com/AztecProtocol/aztec-packages/issues/2514)) ([1eb1a3c](https://github.com/AztecProtocol/aztec-packages/commit/1eb1a3ce45d229cc9ccca9681e5ff61515ad4434))
* Remove "standard" from references to token contracts ([#2533](https://github.com/AztecProtocol/aztec-packages/issues/2533)) ([f931d56](https://github.com/AztecProtocol/aztec-packages/commit/f931d56106cb8520318b02679292f8b29fe06f6a))
* Try fix boxes-blank ([#2539](https://github.com/AztecProtocol/aztec-packages/issues/2539)) ([87b8080](https://github.com/AztecProtocol/aztec-packages/commit/87b8080f5e61b357be96164f5b8d6948584f83c1))
* Try to fix publish bb ([#2529](https://github.com/AztecProtocol/aztec-packages/issues/2529)) ([7c623c4](https://github.com/AztecProtocol/aztec-packages/commit/7c623c44f5e46f41d6fd289fc985edaee721e793))
* Try to fix publish-bb.yml ([#2523](https://github.com/AztecProtocol/aztec-packages/issues/2523)) ([2f6e9bd](https://github.com/AztecProtocol/aztec-packages/commit/2f6e9bde6c9132cc1bc82d2e9df1515f5a3f44f8))
* Use #import_code in Token contract tutorial ([#2438](https://github.com/AztecProtocol/aztec-packages/issues/2438)) ([b58cfb5](https://github.com/AztecProtocol/aztec-packages/commit/b58cfb55c192d3942c3eacecb74d6db28326055d))


### Miscellaneous

* `computeContractAddress` as `computeCompleteAddress` ([#1876](https://github.com/AztecProtocol/aztec-packages/issues/1876)) ([4d95b44](https://github.com/AztecProtocol/aztec-packages/commit/4d95b4420e5a2bf9b5af121a3029d9b3e8a41fa0)), closes [#1873](https://github.com/AztecProtocol/aztec-packages/issues/1873)
* Add instructions on circleci session for debugging ([#2503](https://github.com/AztecProtocol/aztec-packages/issues/2503)) ([a4197e7](https://github.com/AztecProtocol/aztec-packages/commit/a4197e751e14dfe88f5791f1e213336751b9b32e))
* Add output saying how to get the right noir version ([#2622](https://github.com/AztecProtocol/aztec-packages/issues/2622)) ([10b30e0](https://github.com/AztecProtocol/aztec-packages/commit/10b30e061fccd974432f082347715dea6f052f5e))
* Aztec-node json-rpc ([#2444](https://github.com/AztecProtocol/aztec-packages/issues/2444)) ([04efee1](https://github.com/AztecProtocol/aztec-packages/commit/04efee1f5db83eebe5e4e9139ad8fc1a16a74c40))
* BI build tweaks ([#2487](https://github.com/AztecProtocol/aztec-packages/issues/2487)) ([f8b6548](https://github.com/AztecProtocol/aztec-packages/commit/f8b65481eec99876007e521beecd671b9a18f19a))
* Check tree roots in world state sync ([#2543](https://github.com/AztecProtocol/aztec-packages/issues/2543)) ([314e8a0](https://github.com/AztecProtocol/aztec-packages/commit/314e8a0030f93b6b94a17dfa2235e177066e6153))
* **circuits:** 2612 - add validation in native private kernel circuit of arrays in accumulated data ([#2614](https://github.com/AztecProtocol/aztec-packages/issues/2614)) ([f1fe059](https://github.com/AztecProtocol/aztec-packages/commit/f1fe05910ca70224f7334f45cb5b5df7de826b9b))
* **circuits:** Remove obsolete comments in native private kernel circuit ([#2570](https://github.com/AztecProtocol/aztec-packages/issues/2570)) ([a6b6c7b](https://github.com/AztecProtocol/aztec-packages/commit/a6b6c7b0e7b156b72462259b7ea8ead7f42f428b))
* **contract_deployment.md:** Don't require main edit ([#2449](https://github.com/AztecProtocol/aztec-packages/issues/2449)) ([16a3d9c](https://github.com/AztecProtocol/aztec-packages/commit/16a3d9cfb858527c4b59da71a457add1b7dd6d65))
* **deps:** Bump get-func-name from 2.0.0 to 2.0.2 in /yarn-project ([#2630](https://github.com/AztecProtocol/aztec-packages/issues/2630)) ([5cebf18](https://github.com/AztecProtocol/aztec-packages/commit/5cebf18527aec8cb1a41845d20869f2b339b54e2))
* **deps:** Bump ua-parser-js from 0.7.32 to 0.7.36 in /docs ([#2629](https://github.com/AztecProtocol/aztec-packages/issues/2629)) ([b2c87c2](https://github.com/AztecProtocol/aztec-packages/commit/b2c87c26158dea0677ce49ee3c4d5e2045e0d27f))
* Disable pushing/pulling for layer caching in build. ([#2517](https://github.com/AztecProtocol/aztec-packages/issues/2517)) ([51352ae](https://github.com/AztecProtocol/aztec-packages/commit/51352ae3973c937bbb6a4baee401aff52b54246d))
* **docs:** Fix tutorial in dapp development ([#2421](https://github.com/AztecProtocol/aztec-packages/issues/2421)) ([027530f](https://github.com/AztecProtocol/aztec-packages/commit/027530f1518232a372a7d78551fee6a2d2ee96b0))
* **docs:** Incorporate docs feedback ([#2434](https://github.com/AztecProtocol/aztec-packages/issues/2434)) ([4992d5b](https://github.com/AztecProtocol/aztec-packages/commit/4992d5b59bb73e8f14fc14963a89c9c97268f773))
* Embed yq in repo to avoid network hiccups. ([#2560](https://github.com/AztecProtocol/aztec-packages/issues/2560)) ([84f207f](https://github.com/AztecProtocol/aztec-packages/commit/84f207f629b2b0d5312c8d73e7b620ff255332e8))
* Fix box noir versioning ([#2578](https://github.com/AztecProtocol/aztec-packages/issues/2578)) ([6eaf0c7](https://github.com/AztecProtocol/aztec-packages/commit/6eaf0c7d32a394c36759853ba8d63dde90122f0a))
* Fixing foundry version ([#2528](https://github.com/AztecProtocol/aztec-packages/issues/2528)) ([3af0753](https://github.com/AztecProtocol/aztec-packages/commit/3af0753dfb932ec4a8ba68e55843149daa570268))
* Kill Turbo ([#2442](https://github.com/AztecProtocol/aztec-packages/issues/2442)) ([c832825](https://github.com/AztecProtocol/aztec-packages/commit/c83282582536421ae67bbd936b3059597d908253))
* Move hash utils to aztec-nr ([#2583](https://github.com/AztecProtocol/aztec-packages/issues/2583)) ([78bd1a3](https://github.com/AztecProtocol/aztec-packages/commit/78bd1a36805bd6508155a62bef06cf223bc67948))
* No private key account state ([#2491](https://github.com/AztecProtocol/aztec-packages/issues/2491)) ([5813fb3](https://github.com/AztecProtocol/aztec-packages/commit/5813fb365f63d4921dcfd53b205a15f14065e213))
* Provide cross compile to cjs. ([#2566](https://github.com/AztecProtocol/aztec-packages/issues/2566)) ([47d0d37](https://github.com/AztecProtocol/aztec-packages/commit/47d0d376727dfcb798af4ea019dfc23a9a57b6ca))
* Recursion todos ([#2516](https://github.com/AztecProtocol/aztec-packages/issues/2516)) ([2df107b](https://github.com/AztecProtocol/aztec-packages/commit/2df107b2da73217eb96d39c8ed880f76a2b3e4cd))
* Reenable some ultra honk composer tests ([#2417](https://github.com/AztecProtocol/aztec-packages/issues/2417)) ([31f4c32](https://github.com/AztecProtocol/aztec-packages/commit/31f4c32e2c4a3a91879e842ea2366eb167fdd510))
* Refactor e2e test teardown ([#2513](https://github.com/AztecProtocol/aztec-packages/issues/2513)) ([2e43248](https://github.com/AztecProtocol/aztec-packages/commit/2e432483170d873f15aa1a17ed105699f484add1))
* Remove `BarretenbergBinderSync` import from typescript bindgen file ([#2607](https://github.com/AztecProtocol/aztec-packages/issues/2607)) ([43af1a3](https://github.com/AztecProtocol/aztec-packages/commit/43af1a35c1bbe55cab102bef21375dd9986202ea))
* Remove build system tainting now we have ci cmds in comments. ([#2589](https://github.com/AztecProtocol/aztec-packages/issues/2589)) ([2040335](https://github.com/AztecProtocol/aztec-packages/commit/204033598f09c218aec5c9cc64ebf1c0f6dfbcd3))
* Remove composer keyword from stdlib ([#2418](https://github.com/AztecProtocol/aztec-packages/issues/2418)) ([f3e7d91](https://github.com/AztecProtocol/aztec-packages/commit/f3e7d914e3b8b7f98eacde0dff12a51a04dde93e))
* Remove debug log in world state sync ([#2613](https://github.com/AztecProtocol/aztec-packages/issues/2613)) ([177f468](https://github.com/AztecProtocol/aztec-packages/commit/177f468a8f68ec03bb297ea1fb70002fe58ba22c))
* Remove Standard Honk ([#2435](https://github.com/AztecProtocol/aztec-packages/issues/2435)) ([9b3ee45](https://github.com/AztecProtocol/aztec-packages/commit/9b3ee4579c0a13378eb27b5c24bf9b99a07de350))
* Remove unneeded dockerfiles. ([#2588](https://github.com/AztecProtocol/aztec-packages/issues/2588)) ([d6f903d](https://github.com/AztecProtocol/aztec-packages/commit/d6f903dfd7e7cf5b878d4f25686a1a01b24505ab))
* Rename all the occurrences of `Aztec RPC` ([#2552](https://github.com/AztecProtocol/aztec-packages/issues/2552)) ([8cc4f69](https://github.com/AztecProtocol/aztec-packages/commit/8cc4f694f93499e91026bd6c144a3f646d987588)), closes [#2451](https://github.com/AztecProtocol/aztec-packages/issues/2451)
* Renaming `@aztec/aztec-rpc` package as `@aztec/pxe` ([#2538](https://github.com/AztecProtocol/aztec-packages/issues/2538)) ([0dd70aa](https://github.com/AztecProtocol/aztec-packages/commit/0dd70aa6df929317c350cf1a3731fdd3cd3446d4))
* Resuscitate private kernel tests related to call stack item check ([#2558](https://github.com/AztecProtocol/aztec-packages/issues/2558)) ([9e938fc](https://github.com/AztecProtocol/aztec-packages/commit/9e938fca9654dc4997790806853551f1c5ffbf04))
* Run formatting:fix for box lint ([#2479](https://github.com/AztecProtocol/aztec-packages/issues/2479)) ([3995de9](https://github.com/AztecProtocol/aztec-packages/commit/3995de91ebfed185714b0b2045c1e1243386e778))
* Run quick-start guide in CI ([#2413](https://github.com/AztecProtocol/aztec-packages/issues/2413)) ([5f43715](https://github.com/AztecProtocol/aztec-packages/commit/5f437157fe26d8f2913d0c9199c149dcb2ad9a4f))
* Simulator ([#2534](https://github.com/AztecProtocol/aztec-packages/issues/2534)) ([a26198e](https://github.com/AztecProtocol/aztec-packages/commit/a26198efc5c7f1ae3678eb165173c52ee4cb1e60))
* Switch to upstream docusaurus-plugin-typedoc ([#2557](https://github.com/AztecProtocol/aztec-packages/issues/2557)) ([fdf5fce](https://github.com/AztecProtocol/aztec-packages/commit/fdf5fce49ebdc2013a924fb71d648578cd43806e))
* Token contract storage cleanup ([#2536](https://github.com/AztecProtocol/aztec-packages/issues/2536)) ([0b62207](https://github.com/AztecProtocol/aztec-packages/commit/0b62207aa9969849625a112668298fc294d60fb0))
* Typo ([#2546](https://github.com/AztecProtocol/aztec-packages/issues/2546)) ([8656a3b](https://github.com/AztecProtocol/aztec-packages/commit/8656a3b1f4fce63c3acaed6e81ae77632df05ef5))
* Unskip test and fix params ([#2454](https://github.com/AztecProtocol/aztec-packages/issues/2454)) ([e484c5f](https://github.com/AztecProtocol/aztec-packages/commit/e484c5f656fc1b10b9795727ffcb016d586aaf38))
* Update private token box ([#2385](https://github.com/AztecProtocol/aztec-packages/issues/2385)) ([b730196](https://github.com/AztecProtocol/aztec-packages/commit/b730196305597385e01e8d07c2173af8bf323624))
* Use US spelling ([#2475](https://github.com/AztecProtocol/aztec-packages/issues/2475)) ([2fe8f5e](https://github.com/AztecProtocol/aztec-packages/commit/2fe8f5e41879cc72bd208eb77fb5a2e3261d1cf8)), closes [#1934](https://github.com/AztecProtocol/aztec-packages/issues/1934)


### Documentation

* Capitalizing x in pxe ([#2564](https://github.com/AztecProtocol/aztec-packages/issues/2564)) ([2927cf1](https://github.com/AztecProtocol/aztec-packages/commit/2927cf14e9035d05a39627d46af5063771bc5e0e))
* Common contract errors ([#2471](https://github.com/AztecProtocol/aztec-packages/issues/2471)) ([a8aec70](https://github.com/AztecProtocol/aztec-packages/commit/a8aec70dac829c42874b89119767e4eb5689d4aa)), closes [#2468](https://github.com/AztecProtocol/aztec-packages/issues/2468)
* Fixed original minus underflow test ([#2472](https://github.com/AztecProtocol/aztec-packages/issues/2472)) ([0cf4bdc](https://github.com/AztecProtocol/aztec-packages/commit/0cf4bdc853d864fd4cf73d5af7e261ee2515c0d0))
* Including sandbox diagrams in the sandbox section ([#2573](https://github.com/AztecProtocol/aztec-packages/issues/2573)) ([2fa143e](https://github.com/AztecProtocol/aztec-packages/commit/2fa143e4d88b3089ebbe2a9e53645edf66157dc8))
* Initial storage cleanup ([#2433](https://github.com/AztecProtocol/aztec-packages/issues/2433)) ([d833483](https://github.com/AztecProtocol/aztec-packages/commit/d833483ac51296c3bbb7eedfb6a1f1435a725903))
* Misc docs changes ([#2416](https://github.com/AztecProtocol/aztec-packages/issues/2416)) ([0e789c7](https://github.com/AztecProtocol/aztec-packages/commit/0e789c7c31a6272ec5b063f4583fb2d59e6ba73f))
* More `RPC Server` --&gt; `PXE` naming fixes ([#2574](https://github.com/AztecProtocol/aztec-packages/issues/2574)) ([b33eea5](https://github.com/AztecProtocol/aztec-packages/commit/b33eea595ff9c01d993fd9727e6924e403517d6e))
* Portal messaging ([#2419](https://github.com/AztecProtocol/aztec-packages/issues/2419)) ([7979bb9](https://github.com/AztecProtocol/aztec-packages/commit/7979bb9c356b9e3dd8796eb964d2fd2490fa4295))
* Update instructions ([#2297](https://github.com/AztecProtocol/aztec-packages/issues/2297)) ([ab612df](https://github.com/AztecProtocol/aztec-packages/commit/ab612dff85aa2dec28aefd680764a8477efd86e6)), closes [#1827](https://github.com/AztecProtocol/aztec-packages/issues/1827)
* Update sidebar + embed youtube video ([#2470](https://github.com/AztecProtocol/aztec-packages/issues/2470)) ([a779d11](https://github.com/AztecProtocol/aztec-packages/commit/a779d114584742e41e5489ce36821d8554772ea0))

## [0.7.10](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.9...aztec-packages-v0.7.10) (2023-09-20)


### Features

* Aztec-cli unbox "really empty box" ([#2388](https://github.com/AztecProtocol/aztec-packages/issues/2388)) ([b57182d](https://github.com/AztecProtocol/aztec-packages/commit/b57182d6dff9fc27d5fb555a4c4cb948d9e5cc55))
* **docs:** Document noir macros  ([#2016](https://github.com/AztecProtocol/aztec-packages/issues/2016)) ([1f1a17f](https://github.com/AztecProtocol/aztec-packages/commit/1f1a17fe056d8898c4c065fb6244e53da04800cb))
* **docs:** Include aztec rpc interface typedoc output in docs ([#2255](https://github.com/AztecProtocol/aztec-packages/issues/2255)) ([62c9e9b](https://github.com/AztecProtocol/aztec-packages/commit/62c9e9bfdc9535ccfc6bd76782971e22478a7784))
* **token portal standard:** Create a token portal standard ([#2351](https://github.com/AztecProtocol/aztec-packages/issues/2351)) ([426a3ea](https://github.com/AztecProtocol/aztec-packages/commit/426a3ea6a5c3529b4edaea94affaece97d39a35b))


### Bug Fixes

* **build:** Fix build system post deployment tests ([#2420](https://github.com/AztecProtocol/aztec-packages/issues/2420)) ([d509dc3](https://github.com/AztecProtocol/aztec-packages/commit/d509dc359c4cd9dc37492a434a1eb3813c002839))
* CLI encoding for arrays and structs ([#2407](https://github.com/AztecProtocol/aztec-packages/issues/2407)) ([85283bd](https://github.com/AztecProtocol/aztec-packages/commit/85283bdd5b0916c207dca11ad17338f524ae18f6))
* Correct sandbox addresses in up-quick-start test ([#2412](https://github.com/AztecProtocol/aztec-packages/issues/2412)) ([974d859](https://github.com/AztecProtocol/aztec-packages/commit/974d85922fc11734c543e7ce9fe7edaad527bd69))
* **docs:** Revert include aztec rpc interface typedoc output in docs ([#2255](https://github.com/AztecProtocol/aztec-packages/issues/2255)) ([f852432](https://github.com/AztecProtocol/aztec-packages/commit/f85243298ef2a5c01764e592c6f6ea50d835bf07))
* Handle falsy bigints in json-rpc ([#2403](https://github.com/AztecProtocol/aztec-packages/issues/2403)) ([d100650](https://github.com/AztecProtocol/aztec-packages/commit/d100650d107b6685e17fcdbbf68363505c5ed0ed)), closes [#2402](https://github.com/AztecProtocol/aztec-packages/issues/2402)
* **nargo_check.sh:** UNIX standard grep ([#2396](https://github.com/AztecProtocol/aztec-packages/issues/2396)) ([02e788a](https://github.com/AztecProtocol/aztec-packages/commit/02e788a8e39c7fcb5c75a6aaf4ceb705a8ebaa4a))


### Miscellaneous

* **docs:** Note getter options ([#2411](https://github.com/AztecProtocol/aztec-packages/issues/2411)) ([8a95d8c](https://github.com/AztecProtocol/aztec-packages/commit/8a95d8cb6287689b90149b44968cab4ba3e13e28))
* Update docs url in config  ([#2386](https://github.com/AztecProtocol/aztec-packages/issues/2386)) ([e44066d](https://github.com/AztecProtocol/aztec-packages/commit/e44066da3cca70a8494c6822b3bc231679acaf16))

## [0.7.9](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.8...aztec-packages-v0.7.9) (2023-09-19)


### Bug Fixes

* Don't cache contract witnesses ([#2398](https://github.com/AztecProtocol/aztec-packages/issues/2398)) ([1092060](https://github.com/AztecProtocol/aztec-packages/commit/1092060ec88e4d9d48b9bbaf1345cf058003cc82)), closes [#2397](https://github.com/AztecProtocol/aztec-packages/issues/2397)

## [0.7.8](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.7...aztec-packages-v0.7.8) (2023-09-19)


### Features

* `NodeInfo` cleanup ([#2370](https://github.com/AztecProtocol/aztec-packages/issues/2370)) ([89fe978](https://github.com/AztecProtocol/aztec-packages/commit/89fe978b12919b7b4838b95438635f8489d27229))
* Allow custom ports in sandbox ([#2393](https://github.com/AztecProtocol/aztec-packages/issues/2393)) ([41ef378](https://github.com/AztecProtocol/aztec-packages/commit/41ef378a01db6a87a8620d7fb676784222a3b7f4))
* Allow tracing build system with [debug ci] ([#2389](https://github.com/AztecProtocol/aztec-packages/issues/2389)) ([ce311a9](https://github.com/AztecProtocol/aztec-packages/commit/ce311a9b44a8f0327235ccd3bb8f9a8fca97443e))
* **docs:** Show current noir version for aztec in docs ([#2379](https://github.com/AztecProtocol/aztec-packages/issues/2379)) ([5c7b2ab](https://github.com/AztecProtocol/aztec-packages/commit/5c7b2ab566fc6ce870c7d6f121f86b721bf3e660))


### Bug Fixes

* Build script exiting on failed grep ([#2384](https://github.com/AztecProtocol/aztec-packages/issues/2384)) ([e70a781](https://github.com/AztecProtocol/aztec-packages/commit/e70a781765a83cbe45e78e3a560bd6191fd9211e))
* Bump e2e_sandbox_example.test.ts timeout ([#2391](https://github.com/AztecProtocol/aztec-packages/issues/2391)) ([9a1bb62](https://github.com/AztecProtocol/aztec-packages/commit/9a1bb6282b8df4d4b1eb7d2df8e4197e29032ba2))
* Compile script for the unboxed project ([#2380](https://github.com/AztecProtocol/aztec-packages/issues/2380)) ([2801da2](https://github.com/AztecProtocol/aztec-packages/commit/2801da2c5a307f8bc5691f73f9273391345acf59))
* **Docs:** Nargo.toml docs fix ([#2334](https://github.com/AztecProtocol/aztec-packages/issues/2334)) ([af24b5a](https://github.com/AztecProtocol/aztec-packages/commit/af24b5a12f04ff97333a7631fe634f1440c9df35))
* Force_deploy_build error ([#2375](https://github.com/AztecProtocol/aztec-packages/issues/2375)) ([4d1cbf9](https://github.com/AztecProtocol/aztec-packages/commit/4d1cbf9742cb2a39e936a971e2f954d362f8f08b))
* Propagate [debug ci] thru spot ([#2395](https://github.com/AztecProtocol/aztec-packages/issues/2395)) ([fe5eedd](https://github.com/AztecProtocol/aztec-packages/commit/fe5eedd202ab26d3e27a195f482ea3e75df74d9b))
* Remove non-npm packages from end-to-end during canary flow ([#2394](https://github.com/AztecProtocol/aztec-packages/issues/2394)) ([e3f97f2](https://github.com/AztecProtocol/aztec-packages/commit/e3f97f26f016025353851327e24adbb8e752301f))
* Update aztec sandbox getting started markdown ([#2374](https://github.com/AztecProtocol/aztec-packages/issues/2374)) ([a3c6bcf](https://github.com/AztecProtocol/aztec-packages/commit/a3c6bcf88ea1003b3f134cbd29ae1a39680f5b9f))


### Miscellaneous

* Adds on-brand design to private token project ([#2355](https://github.com/AztecProtocol/aztec-packages/issues/2355)) ([072e313](https://github.com/AztecProtocol/aztec-packages/commit/072e313515871b54473ee7662f5bdd4bfa21e3e7))
* Docs restructure ([#2322](https://github.com/AztecProtocol/aztec-packages/issues/2322)) ([1368b55](https://github.com/AztecProtocol/aztec-packages/commit/1368b55d0a9bc9ea61e29bb095ca62aa6902645f))


### Documentation

* Updated noirup command ([#2339](https://github.com/AztecProtocol/aztec-packages/issues/2339)) ([5308c21](https://github.com/AztecProtocol/aztec-packages/commit/5308c21d4dc225233af8ae584c471e7bed5d9381))

## [0.7.7](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.6...aztec-packages-v0.7.7) (2023-09-18)


### Bug Fixes

* Deploy_npm script variable ([#2372](https://github.com/AztecProtocol/aztec-packages/issues/2372)) ([b46e06d](https://github.com/AztecProtocol/aztec-packages/commit/b46e06d4e3caedb0584b1e7e28ac035ed264f682))

## [0.7.6](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.5...aztec-packages-v0.7.6) (2023-09-18)


### Features

* New api to get note nonces ([#2327](https://github.com/AztecProtocol/aztec-packages/issues/2327)) ([8f5eb28](https://github.com/AztecProtocol/aztec-packages/commit/8f5eb28fae2cdcd549241a85b8b2343c7b661aac))
* Replace private token in testing ([#2304](https://github.com/AztecProtocol/aztec-packages/issues/2304)) ([934ba96](https://github.com/AztecProtocol/aztec-packages/commit/934ba96ad6a6843edfed81aef179d826ce6c4cea))


### Bug Fixes

* Exit with error log when COMMIT_TAG is not set properly on canary ([#2371](https://github.com/AztecProtocol/aztec-packages/issues/2371)) ([68fe053](https://github.com/AztecProtocol/aztec-packages/commit/68fe053f8bf7830659a98a9aae8a7c3fbdfe664c))
* Preserve public function call ordering in account entrypoint ([#2348](https://github.com/AztecProtocol/aztec-packages/issues/2348)) ([5b2cf75](https://github.com/AztecProtocol/aztec-packages/commit/5b2cf758b54a810693cb296bb5a2985c5d882dee))
* Return output-debug flag ([#2364](https://github.com/AztecProtocol/aztec-packages/issues/2364)) ([af86580](https://github.com/AztecProtocol/aztec-packages/commit/af86580814d6f63f15a9ae2476f91c58c835bf82))
* Revert "fix: strip leading 'v' from dockerhub tags" ([#2367](https://github.com/AztecProtocol/aztec-packages/issues/2367)) ([53bc041](https://github.com/AztecProtocol/aztec-packages/commit/53bc041af5d7f7ad66baf5076130cf627f8f65d5))
* Stale CLI docs ([#2336](https://github.com/AztecProtocol/aztec-packages/issues/2336)) ([f38873b](https://github.com/AztecProtocol/aztec-packages/commit/f38873b1751a7604cba3aed888323c7cd106b689))
* Strip leading 'v' from dockerhub tags ([#2360](https://github.com/AztecProtocol/aztec-packages/issues/2360)) ([a4bb05c](https://github.com/AztecProtocol/aztec-packages/commit/a4bb05ca2bda0f4e5ccd15c01bf1faadaa664354))


### Miscellaneous

* Added docs for artifact files ([#2362](https://github.com/AztecProtocol/aztec-packages/issues/2362)) ([6d3ba3f](https://github.com/AztecProtocol/aztec-packages/commit/6d3ba3fc833aa4f103c6b84065bb2dd0bea0f6b4)), closes [#2190](https://github.com/AztecProtocol/aztec-packages/issues/2190)
* **aztec_noir:** Remove inputs from consume l1 to l2 message  ([#2354](https://github.com/AztecProtocol/aztec-packages/issues/2354)) ([2235f7c](https://github.com/AztecProtocol/aztec-packages/commit/2235f7cd0cdf5dfdd3188d3f606673e94e25c47d))
* Remove "as unknown" casts for ABIs where possible ([#2331](https://github.com/AztecProtocol/aztec-packages/issues/2331)) ([bf2651e](https://github.com/AztecProtocol/aztec-packages/commit/bf2651e714e148cdd0a625a435fe1ee64d762ffb))
* Script to extract tag version ([#2368](https://github.com/AztecProtocol/aztec-packages/issues/2368)) ([4b686b0](https://github.com/AztecProtocol/aztec-packages/commit/4b686b0d17f5a0811bfeef6bbe50d29b44cd7753))
* Share e2e code with canary ([#2299](https://github.com/AztecProtocol/aztec-packages/issues/2299)) ([21224de](https://github.com/AztecProtocol/aztec-packages/commit/21224dea64318a5956a705d0b413dd0e7bcf795c))

## [0.7.5](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.4...aztec-packages-v0.7.5) (2023-09-15)


### Features

* Protogalaxy interfaces ([#2125](https://github.com/AztecProtocol/aztec-packages/issues/2125)) ([b45dd26](https://github.com/AztecProtocol/aztec-packages/commit/b45dd26214119f0c52c2c4f48ff11f650912fef9))
* Renamed `nargoVersion` as `compatibleNargoVersion` ([#2338](https://github.com/AztecProtocol/aztec-packages/issues/2338)) ([6f9e0f1](https://github.com/AztecProtocol/aztec-packages/commit/6f9e0f1bbb721f72b9951caed64921f311a4a30b))


### Bug Fixes

* Add retry around docker login and revive spot_run_test_script ([#2346](https://github.com/AztecProtocol/aztec-packages/issues/2346)) ([79e5f05](https://github.com/AztecProtocol/aztec-packages/commit/79e5f05c70cdc4bfb1bd6635d900b593dc8ada6b))
* Unbox command. ([#2337](https://github.com/AztecProtocol/aztec-packages/issues/2337)) ([e9bc9c6](https://github.com/AztecProtocol/aztec-packages/commit/e9bc9c60fd1f79592ffe828a59618320ff26327b))


### Miscellaneous

* Increase guides-dapp-testing test timeout ([#2343](https://github.com/AztecProtocol/aztec-packages/issues/2343)) ([1cebe2c](https://github.com/AztecProtocol/aztec-packages/commit/1cebe2c22a93686f36c952a912540bb129768ee4))
* Use retries by default on rpc client fetch ([#2342](https://github.com/AztecProtocol/aztec-packages/issues/2342)) ([f4ffd68](https://github.com/AztecProtocol/aztec-packages/commit/f4ffd68f1c4fe75a53caa1bec32f246aa5f0c818))

## [0.7.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.3...aztec-packages-v0.7.4) (2023-09-15)


### Features

* Elliptic Curve Virtual Machine Circuit ([#1268](https://github.com/AztecProtocol/aztec-packages/issues/1268)) ([f85ecd9](https://github.com/AztecProtocol/aztec-packages/commit/f85ecd921271ec94b551992bcfe16c2b56f72d2e))
* Exposing nargo version via `NodeInfo` ([#2333](https://github.com/AztecProtocol/aztec-packages/issues/2333)) ([1c2669c](https://github.com/AztecProtocol/aztec-packages/commit/1c2669c4b442c022f9f501f0b8caf102a08e0b0c)), closes [#2332](https://github.com/AztecProtocol/aztec-packages/issues/2332)
* Migrate accounts to auth witness ([#2281](https://github.com/AztecProtocol/aztec-packages/issues/2281)) ([91152af](https://github.com/AztecProtocol/aztec-packages/commit/91152afbdde0313972007d265230276c6160eb2c)), closes [#2043](https://github.com/AztecProtocol/aztec-packages/issues/2043)


### Bug Fixes

* Aztec-nr mirror url ([#2321](https://github.com/AztecProtocol/aztec-packages/issues/2321)) ([aaf7f67](https://github.com/AztecProtocol/aztec-packages/commit/aaf7f67fcb0e226f9094feeff6795957dfd9d67e))
* **build:** Fixed paths on s3 deployments ([#2335](https://github.com/AztecProtocol/aztec-packages/issues/2335)) ([38c7979](https://github.com/AztecProtocol/aztec-packages/commit/38c7979c03f7e1c5ffbaf8537cd91ed1574e0c95))


### Miscellaneous

* Do not format boxes with global format ([#2326](https://github.com/AztecProtocol/aztec-packages/issues/2326)) ([2fe845f](https://github.com/AztecProtocol/aztec-packages/commit/2fe845f2f0cb46c8940826045a703de333b8b0f5))
* Remove native token ([#2280](https://github.com/AztecProtocol/aztec-packages/issues/2280)) ([4032d01](https://github.com/AztecProtocol/aztec-packages/commit/4032d014c29a2a1eddb13881d6e469b35177f207))
* Rename getAccounts to getRegisteredAccounts ([#2330](https://github.com/AztecProtocol/aztec-packages/issues/2330)) ([c7f3776](https://github.com/AztecProtocol/aztec-packages/commit/c7f37769df6584a8c3f0a970d8694a2b455f00d3))

## [0.7.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.2...aztec-packages-v0.7.3) (2023-09-15)


### Features

* Constraining note owner ([#1896](https://github.com/AztecProtocol/aztec-packages/issues/1896)) ([cb25bc9](https://github.com/AztecProtocol/aztec-packages/commit/cb25bc9b679e7d559357a7ed9be5c8cf4ebc69d3)), closes [#1817](https://github.com/AztecProtocol/aztec-packages/issues/1817)


### Bug Fixes

* **build:** Navigate to correct directory for publishing ([#2318](https://github.com/AztecProtocol/aztec-packages/issues/2318)) ([f555356](https://github.com/AztecProtocol/aztec-packages/commit/f555356a78c68660b0a324c45a6dce29fb8df518))
* Use bool for set_minter ([#2313](https://github.com/AztecProtocol/aztec-packages/issues/2313)) ([5b18f9e](https://github.com/AztecProtocol/aztec-packages/commit/5b18f9e697404a5ad7d2dbe4f8f3875edcf8c58c))

## [0.7.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.1...aztec-packages-v0.7.2) (2023-09-14)


### Features

* ASAN build ([#2307](https://github.com/AztecProtocol/aztec-packages/issues/2307)) ([274c89f](https://github.com/AztecProtocol/aztec-packages/commit/274c89f1916d8af2054d9773dc632f87bb3bf2fc))


### Bug Fixes

* **build:** Attempt to fix deployments ([#2309](https://github.com/AztecProtocol/aztec-packages/issues/2309)) ([39f16f9](https://github.com/AztecProtocol/aztec-packages/commit/39f16f9f642ff348920e1cd4511df9d0f72bacf9))

## [0.7.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.7.0...aztec-packages-v0.7.1) (2023-09-14)


### Features

* Build system handles dynamic deps first class. ([#2283](https://github.com/AztecProtocol/aztec-packages/issues/2283)) ([f66077a](https://github.com/AztecProtocol/aztec-packages/commit/f66077a6f7bfd446eec81dd4f09723322fc0c980))
* Build_manifest default tweaks. ([#2287](https://github.com/AztecProtocol/aztec-packages/issues/2287)) ([c8a5cfb](https://github.com/AztecProtocol/aztec-packages/commit/c8a5cfb375b498475503c12cc83fcdba39f2ec5f))
* **build:** Build multi-architecture docker images for aztec-sandbox ([#2305](https://github.com/AztecProtocol/aztec-packages/issues/2305)) ([8ee61b8](https://github.com/AztecProtocol/aztec-packages/commit/8ee61b85f682fec0c03eb831f417ba2938658310))
* Cli "unbox" command ([#2029](https://github.com/AztecProtocol/aztec-packages/issues/2029)) ([26ab88f](https://github.com/AztecProtocol/aztec-packages/commit/26ab88fd5b8d5be7f20cd6f6e4335d344f2219c7))
* Creating an SMT verification module ([#1932](https://github.com/AztecProtocol/aztec-packages/issues/1932)) ([4642b61](https://github.com/AztecProtocol/aztec-packages/commit/4642b61a60534daeec8edd9541f283058d0d66bd))
* Token standard ([#2069](https://github.com/AztecProtocol/aztec-packages/issues/2069)) ([5e8fbf2](https://github.com/AztecProtocol/aztec-packages/commit/5e8fbf2d387aeb0ae0cb1432525c39f82eb7baa1))


### Bug Fixes

* Ensure_note_hash_exists ([#2256](https://github.com/AztecProtocol/aztec-packages/issues/2256)) ([271b060](https://github.com/AztecProtocol/aztec-packages/commit/271b060f2642570f58e38881cbb3477745b84ddf))
* Msgpack stack blowups on schema gen ([#2259](https://github.com/AztecProtocol/aztec-packages/issues/2259)) ([1afc566](https://github.com/AztecProtocol/aztec-packages/commit/1afc566df942e82f70d2e82e33c0e39539714ad5))
* Noir bootstrap ([#2274](https://github.com/AztecProtocol/aztec-packages/issues/2274)) ([f85db49](https://github.com/AztecProtocol/aztec-packages/commit/f85db4972411c863585e968fe2535e68c467b028))
* Workaround sequencer timeout ([#2269](https://github.com/AztecProtocol/aztec-packages/issues/2269)) ([9fc3f3d](https://github.com/AztecProtocol/aztec-packages/commit/9fc3f3d6652e592d674a9f5f2a55bd1994b7060d))


### Miscellaneous

* Bump nargo to 0.11.1-aztec.0 ([#2298](https://github.com/AztecProtocol/aztec-packages/issues/2298)) ([8b76a12](https://github.com/AztecProtocol/aztec-packages/commit/8b76a124390102574efcc8078bc9bc47c8e7ba35))
* **ci:** Mirror Aztec-nr ([#2270](https://github.com/AztecProtocol/aztec-packages/issues/2270)) ([c57f027](https://github.com/AztecProtocol/aztec-packages/commit/c57f027af9a9796ddef970db24e56be954215760))
* **circuits:** Base rollup cbind msgpack ([#2263](https://github.com/AztecProtocol/aztec-packages/issues/2263)) ([0d4c707](https://github.com/AztecProtocol/aztec-packages/commit/0d4c707079ff1ff4212fc3345066b0deded98449))
* **circuits:** Clean up of some superfluous header includes ([#2302](https://github.com/AztecProtocol/aztec-packages/issues/2302)) ([5e53345](https://github.com/AztecProtocol/aztec-packages/commit/5e53345270873a3af2b47f6f078e3b4f1cc973d0))
* **circuits:** Removing assertMemberLength on Tuple objects ([#2296](https://github.com/AztecProtocol/aztec-packages/issues/2296)) ([0247b85](https://github.com/AztecProtocol/aztec-packages/commit/0247b859d88781740fa990801a24881c09c5ca3c))
* Consolidate mirror repos on a nightly schedule ([#1994](https://github.com/AztecProtocol/aztec-packages/issues/1994)) ([1a586c4](https://github.com/AztecProtocol/aztec-packages/commit/1a586c4197f2e093521e921e7ef21599be71e5b5))
* **docs:** Rename to aztec.nr ([#1943](https://github.com/AztecProtocol/aztec-packages/issues/1943)) ([a91db48](https://github.com/AztecProtocol/aztec-packages/commit/a91db48d1943fdc2e39535a153216b7aaca40de4))
* Move barretenberg to top of repo. Make circuits build off barretenberg build. ([#2221](https://github.com/AztecProtocol/aztec-packages/issues/2221)) ([404ec34](https://github.com/AztecProtocol/aztec-packages/commit/404ec34d38e1a9c3fbe7a3cdb6e88c28f62f72e4))
* Replace native token in lending contract ([#2276](https://github.com/AztecProtocol/aztec-packages/issues/2276)) ([c46b3c8](https://github.com/AztecProtocol/aztec-packages/commit/c46b3c8f848e7ff240eb466fa2f3f8aad96dc328))
* **subrepo:** Push aztec-nr, update default branches  ([#2300](https://github.com/AztecProtocol/aztec-packages/issues/2300)) ([80c9b77](https://github.com/AztecProtocol/aztec-packages/commit/80c9b77c3e6adc755ec80f02a7f8261a7e8581c4))
* Updated `acvm_js` ([#2272](https://github.com/AztecProtocol/aztec-packages/issues/2272)) ([9f1a3a5](https://github.com/AztecProtocol/aztec-packages/commit/9f1a3a5e4b72506489645f8be8c8aa5129a2e179))

## [0.7.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.7...aztec-packages-v0.7.0) (2023-09-13)


### ⚠ BREAKING CHANGES

* **aztec-noir:** rename noir-aztec to aztec-noir ([#2071](https://github.com/AztecProtocol/aztec-packages/issues/2071))

### Features

* **build:** Use LTS version of ubuntu ([#2239](https://github.com/AztecProtocol/aztec-packages/issues/2239)) ([ce6671e](https://github.com/AztecProtocol/aztec-packages/commit/ce6671e6ab72fcdc8114df5b6a45f81c0086b19d))
* **ci:** Run nargo test in ci on all packages  ([#2197](https://github.com/AztecProtocol/aztec-packages/issues/2197)) ([cca55f2](https://github.com/AztecProtocol/aztec-packages/commit/cca55f225f7277cfb44b44e4d1f50d5527afdb8b))
* **cli:** Add commands for generating ts and nr interfaces ([#2241](https://github.com/AztecProtocol/aztec-packages/issues/2241)) ([c11b70d](https://github.com/AztecProtocol/aztec-packages/commit/c11b70d8186ef7ab9a9d4ab1a09589e7c47e91bb)), closes [#2183](https://github.com/AztecProtocol/aztec-packages/issues/2183)
* **cli:** Inspect contract command ([#2248](https://github.com/AztecProtocol/aztec-packages/issues/2248)) ([381706e](https://github.com/AztecProtocol/aztec-packages/commit/381706eaaad7054d620855f7b986e2df3cf62a91)), closes [#2180](https://github.com/AztecProtocol/aztec-packages/issues/2180)
* Define specific Sandbox version when running docker-compose up ([#2238](https://github.com/AztecProtocol/aztec-packages/issues/2238)) ([71da236](https://github.com/AztecProtocol/aztec-packages/commit/71da2360986e5b57f211ca095b95ade2617f4eb8))
* **docs:** Updated docs explaining Sandbox accounts ([#2235](https://github.com/AztecProtocol/aztec-packages/issues/2235)) ([f560066](https://github.com/AztecProtocol/aztec-packages/commit/f560066394c3fc9725be18f320597794e29dc077))
* Optimize sandbox startup time by only initializing the BB solver once. ([#2240](https://github.com/AztecProtocol/aztec-packages/issues/2240)) ([e9cac9c](https://github.com/AztecProtocol/aztec-packages/commit/e9cac9ced3604fdef1d6b298091639fc510cb4fb))
* Remove entrypoint collection ([#2148](https://github.com/AztecProtocol/aztec-packages/issues/2148)) ([e97c94d](https://github.com/AztecProtocol/aztec-packages/commit/e97c94d8bc0659a95f457ba63369fca0dfba47c8))
* Validate nargo version against expected one ([#2254](https://github.com/AztecProtocol/aztec-packages/issues/2254)) ([011c0b7](https://github.com/AztecProtocol/aztec-packages/commit/011c0b7c070f004fcc1c6f9ce8936830c9f496f6))


### Bug Fixes

* Add cjs-entry to bbjs package files ([#2237](https://github.com/AztecProtocol/aztec-packages/issues/2237)) ([ae16193](https://github.com/AztecProtocol/aztec-packages/commit/ae16193b3cdb2da3d57a1c74f7e71f139ced54d1))
* Add link to example contracts in the monorepo ([#2219](https://github.com/AztecProtocol/aztec-packages/issues/2219)) ([8aede54](https://github.com/AztecProtocol/aztec-packages/commit/8aede5470d8e7d88227bf807c3c6cb5dec77a93d))
* **build:** Update ubuntu version used in Docker builds ([#2236](https://github.com/AztecProtocol/aztec-packages/issues/2236)) ([dbe80b7](https://github.com/AztecProtocol/aztec-packages/commit/dbe80b739e97474b29e6a4125ac0d2f16e248b32))
* **docs:** Use code snippet macros in bridge docs ([#2205](https://github.com/AztecProtocol/aztec-packages/issues/2205)) ([0c3a627](https://github.com/AztecProtocol/aztec-packages/commit/0c3a6271a1d90fa95a0163606e49f432573e66da))
* Format barretenberg ([#2209](https://github.com/AztecProtocol/aztec-packages/issues/2209)) ([0801372](https://github.com/AztecProtocol/aztec-packages/commit/08013725091c7e80c1e83145ffbf3983cf1e7fe3))
* Msgpack blowup with bigger objects ([#2207](https://github.com/AztecProtocol/aztec-packages/issues/2207)) ([b909937](https://github.com/AztecProtocol/aztec-packages/commit/b909937ba53b896e11e6b65db08b8f2bb83218d5))
* Refactor constraints in scalar mul to use the high limb ([#2161](https://github.com/AztecProtocol/aztec-packages/issues/2161)) ([1d0e25d](https://github.com/AztecProtocol/aztec-packages/commit/1d0e25d9fad69aebccacf9f646e3291ea89716ca))
* Reinstate v stripping in build ([#2220](https://github.com/AztecProtocol/aztec-packages/issues/2220)) ([13d34f5](https://github.com/AztecProtocol/aztec-packages/commit/13d34f56855bf5c86f04eec15c70b06ded7c955e))
* Return partial witnesses based on the content of read requests. ([#2164](https://github.com/AztecProtocol/aztec-packages/issues/2164)) ([a2125f7](https://github.com/AztecProtocol/aztec-packages/commit/a2125f7611ad9ab3f479b806cbcc7ff1f97db57e))
* Try e2e cli timeout bump ([#2210](https://github.com/AztecProtocol/aztec-packages/issues/2210)) ([a039fdd](https://github.com/AztecProtocol/aztec-packages/commit/a039fdd5d39a57eb25119e990acf309e3447b244))
* Try workaround sample dapp ci timeout ([#2208](https://github.com/AztecProtocol/aztec-packages/issues/2208)) ([e39f6bf](https://github.com/AztecProtocol/aztec-packages/commit/e39f6bf3be2e577e9dffa2d4815b11eb442b5152))


### Miscellaneous

* Add a Nargo workspace in `noir-contracts` ([#2083](https://github.com/AztecProtocol/aztec-packages/issues/2083)) ([728a79c](https://github.com/AztecProtocol/aztec-packages/commit/728a79ca16c962462090b25959d1eab0f1e9f47f))
* Add debugging to run_tests ([#2212](https://github.com/AztecProtocol/aztec-packages/issues/2212)) ([1c5e78a](https://github.com/AztecProtocol/aztec-packages/commit/1c5e78a4ac01bee4b785857447efdb02d8d9cb35))
* **aztec-noir:** Rename noir-aztec to aztec-noir ([#2071](https://github.com/AztecProtocol/aztec-packages/issues/2071)) ([e1e14d2](https://github.com/AztecProtocol/aztec-packages/commit/e1e14d2c7fb44d56b9a10a645676d3551830bb10))
* **circuits:** Merge and root rollup cbind msgpack ([#2192](https://github.com/AztecProtocol/aztec-packages/issues/2192)) ([4f3ecee](https://github.com/AztecProtocol/aztec-packages/commit/4f3eceefe1914dcd1ae3a9c7ae2d91861c25f1d3))
* **noir-contracts:** 1655 - rename functions to make hack clearer for publicly created notes ([#2230](https://github.com/AztecProtocol/aztec-packages/issues/2230)) ([707bc09](https://github.com/AztecProtocol/aztec-packages/commit/707bc09a3c4b5a6460154931db55ee48842ee041))
* Run the test for zero division with mul div up ([#2206](https://github.com/AztecProtocol/aztec-packages/issues/2206)) ([747de6a](https://github.com/AztecProtocol/aztec-packages/commit/747de6aa4b7da488d0f4bc7c545c7e0f4eed4ca9))
* Update url for acir artifacts ([#2231](https://github.com/AztecProtocol/aztec-packages/issues/2231)) ([5e0abd3](https://github.com/AztecProtocol/aztec-packages/commit/5e0abd35dec449a665760e5ee51eeff89c76532c))
* Use workspace build with `nargo compile --workspace` ([#2266](https://github.com/AztecProtocol/aztec-packages/issues/2266)) ([9ab66a0](https://github.com/AztecProtocol/aztec-packages/commit/9ab66a05993cebfd7e126fad4b3cdc6bb1e37faa))


### Documentation

* Dapp tutorial ([#2109](https://github.com/AztecProtocol/aztec-packages/issues/2109)) ([573dbc2](https://github.com/AztecProtocol/aztec-packages/commit/573dbc20a2b5ebae0e967e320da75febd5361eaf)), closes [#2051](https://github.com/AztecProtocol/aztec-packages/issues/2051)
* Minor fixes to dapp tutorial ([#2203](https://github.com/AztecProtocol/aztec-packages/issues/2203)) ([dcc927c](https://github.com/AztecProtocol/aztec-packages/commit/dcc927c9aa347cd305cecd260cfedfb5cda0454f))

## [0.6.7](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.6...aztec-packages-v0.6.7) (2023-09-11)


### Features

* Testing commands in CLI docs ([#2119](https://github.com/AztecProtocol/aztec-packages/issues/2119)) ([73328db](https://github.com/AztecProtocol/aztec-packages/commit/73328dbe4e509235329e32ff88f823d849a2b673))


### Bug Fixes

* Add homepage url to aztec.js package.json ([#2196](https://github.com/AztecProtocol/aztec-packages/issues/2196)) ([7361302](https://github.com/AztecProtocol/aztec-packages/commit/7361302b0b06bc218d287da56cabd7f567cd6aa3))
* **ci:** Add install backend step in noir rebuild ([#2182](https://github.com/AztecProtocol/aztec-packages/issues/2182)) ([27b8bed](https://github.com/AztecProtocol/aztec-packages/commit/27b8bed05fea4f44f36894739613b07cdb8089ac))
* Use Github Bot token for dispatch workflow ([#2171](https://github.com/AztecProtocol/aztec-packages/issues/2171)) ([e6af616](https://github.com/AztecProtocol/aztec-packages/commit/e6af6164095a706109a6f61ef7e1196de67716dc))

## [0.6.6](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.5...aztec-packages-v0.6.6) (2023-09-11)


### Features

* **noir:** Introduce context union to simplify storage declarations ([#2143](https://github.com/AztecProtocol/aztec-packages/issues/2143)) ([2288e44](https://github.com/AztecProtocol/aztec-packages/commit/2288e44a5b817076c9d51db5f99905deeeffc418)), closes [#2012](https://github.com/AztecProtocol/aztec-packages/issues/2012)


### Bug Fixes

* **test:** Fix regex in canary test ([#2165](https://github.com/AztecProtocol/aztec-packages/issues/2165)) ([e5f50df](https://github.com/AztecProtocol/aztec-packages/commit/e5f50df55e68f6c94b602fc16b33abbcea15674e))

## [0.6.5](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.4...aztec-packages-v0.6.5) (2023-09-08)


### Bug Fixes

* Revert "fix: commit tags and rebuilds" ([#2159](https://github.com/AztecProtocol/aztec-packages/issues/2159)) ([50396a0](https://github.com/AztecProtocol/aztec-packages/commit/50396a068f11216947eac0137baa198424da9b81))

## [0.6.4](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.3...aztec-packages-v0.6.4) (2023-09-08)


### Bug Fixes

* Commit tags and rebuilds ([#2156](https://github.com/AztecProtocol/aztec-packages/issues/2156)) ([7669b43](https://github.com/AztecProtocol/aztec-packages/commit/7669b43253f8c2633e96f483ec12c75478dcf539))

## [0.6.3](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.2...aztec-packages-v0.6.3) (2023-09-08)


### Bug Fixes

* Revert bad spot-ification ([#2153](https://github.com/AztecProtocol/aztec-packages/issues/2153)) ([d993d47](https://github.com/AztecProtocol/aztec-packages/commit/d993d47b4df93544c9d0128460eefea286212d77))

## [0.6.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.6.1...aztec-packages-v0.6.2) (2023-09-08)


### Bug Fixes

* Setup_env commit check ([#2149](https://github.com/AztecProtocol/aztec-packages/issues/2149)) ([08ade47](https://github.com/AztecProtocol/aztec-packages/commit/08ade4706e250945be3764587b6863b824092fdd))

## [0.6.1](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.5.2...aztec-packages-v0.6.1) (2023-09-08)


### Features

* Example card game ([#2135](https://github.com/AztecProtocol/aztec-packages/issues/2135)) ([9084b89](https://github.com/AztecProtocol/aztec-packages/commit/9084b89da80953cb781913ba526f77a9a3b12714))


### Bug Fixes

* Retry with git checkout ([#2147](https://github.com/AztecProtocol/aztec-packages/issues/2147)) ([9df0431](https://github.com/AztecProtocol/aztec-packages/commit/9df04312d4d5b4d824725bebd5739e56243c0dce))
* **tests:** Increase test timeout ([#2144](https://github.com/AztecProtocol/aztec-packages/issues/2144)) ([7da9615](https://github.com/AztecProtocol/aztec-packages/commit/7da96152ccc65594e4d7cf80e1931fe5eadfd684))
* Work around intermittent wasm webkit issue ([#2140](https://github.com/AztecProtocol/aztec-packages/issues/2140)) ([a9b0934](https://github.com/AztecProtocol/aztec-packages/commit/a9b09344c80d8628f95f859d4e2d455d61f9e7c6))


### Miscellaneous

* **build:** Updated release please config ([#2142](https://github.com/AztecProtocol/aztec-packages/issues/2142)) ([e119c4f](https://github.com/AztecProtocol/aztec-packages/commit/e119c4f7af0b0f8007abf43c0cad9c0ac6f4e6ac))
* **build:** Updated version check ([#2145](https://github.com/AztecProtocol/aztec-packages/issues/2145)) ([4ed5f05](https://github.com/AztecProtocol/aztec-packages/commit/4ed5f0548cf7e8a9c65f176f469103363a42bc5f))
* **master:** Release 0.5.2 ([#2141](https://github.com/AztecProtocol/aztec-packages/issues/2141)) ([451aad6](https://github.com/AztecProtocol/aztec-packages/commit/451aad6ea92ebced9839ca14baae10cee327be35))
* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))
* Release 0.6.1 ([1bd1a79](https://github.com/AztecProtocol/aztec-packages/commit/1bd1a79b0cefcd90306133aab141d992e8ea5fc3))

## [0.5.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.5.2...aztec-packages-v0.5.2) (2023-09-08)


### Features

* Example card game ([#2135](https://github.com/AztecProtocol/aztec-packages/issues/2135)) ([9084b89](https://github.com/AztecProtocol/aztec-packages/commit/9084b89da80953cb781913ba526f77a9a3b12714))


### Bug Fixes

* **tests:** Increase test timeout ([#2144](https://github.com/AztecProtocol/aztec-packages/issues/2144)) ([7da9615](https://github.com/AztecProtocol/aztec-packages/commit/7da96152ccc65594e4d7cf80e1931fe5eadfd684))
* Work around intermittent wasm webkit issue ([#2140](https://github.com/AztecProtocol/aztec-packages/issues/2140)) ([a9b0934](https://github.com/AztecProtocol/aztec-packages/commit/a9b09344c80d8628f95f859d4e2d455d61f9e7c6))


### Miscellaneous

* **build:** Updated release please config ([#2142](https://github.com/AztecProtocol/aztec-packages/issues/2142)) ([e119c4f](https://github.com/AztecProtocol/aztec-packages/commit/e119c4f7af0b0f8007abf43c0cad9c0ac6f4e6ac))
* **build:** Updated version check ([#2145](https://github.com/AztecProtocol/aztec-packages/issues/2145)) ([4ed5f05](https://github.com/AztecProtocol/aztec-packages/commit/4ed5f0548cf7e8a9c65f176f469103363a42bc5f))
* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))

## [0.5.2](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.5.1...aztec-packages-v0.5.2) (2023-09-08)


### Bug Fixes

* **build:** Config fixes for release please ([#2123](https://github.com/AztecProtocol/aztec-packages/issues/2123)) ([7b4f30d](https://github.com/AztecProtocol/aztec-packages/commit/7b4f30dbdf29a907b7474e5f308849ca068f056e))
* **build:** Don't include component in tag ([#2128](https://github.com/AztecProtocol/aztec-packages/issues/2128)) ([b588e3a](https://github.com/AztecProtocol/aztec-packages/commit/b588e3aad944c7a7f555718794609a9b2559e18a))
* **build:** Updated version file ([#2131](https://github.com/AztecProtocol/aztec-packages/issues/2131)) ([30f9935](https://github.com/AztecProtocol/aztec-packages/commit/30f993502eb6b312fa57f0e995d359ed55b037f5))
* Canary browser test transfer method ([#2126](https://github.com/AztecProtocol/aztec-packages/issues/2126)) ([a23b037](https://github.com/AztecProtocol/aztec-packages/commit/a23b0370ae9395ca51ed8f94a1d71b57d35916a0))
* File reference to canary docker-compose file ([#2124](https://github.com/AztecProtocol/aztec-packages/issues/2124)) ([13d3f16](https://github.com/AztecProtocol/aztec-packages/commit/13d3f161cc2ee6b49e4448ae9e8d33dd7f6ce7d2))
* Retry with -eu was suspect in retrospect ([#2110](https://github.com/AztecProtocol/aztec-packages/issues/2110)) ([7265c2a](https://github.com/AztecProtocol/aztec-packages/commit/7265c2adc64445ae635779351683e479d345fcaf))


### Miscellaneous

* **build:** Enabled components in tags ([#2139](https://github.com/AztecProtocol/aztec-packages/issues/2139)) ([ccb38fb](https://github.com/AztecProtocol/aztec-packages/commit/ccb38fbab252f1a6ed4fb5b974e0255b2b7556b2))
* **build:** Fixed manifest ([#2122](https://github.com/AztecProtocol/aztec-packages/issues/2122)) ([91faa66](https://github.com/AztecProtocol/aztec-packages/commit/91faa668650b98306813e64e9ebe3064bd7a221e))
* **build:** Force a rebuild ([#2136](https://github.com/AztecProtocol/aztec-packages/issues/2136)) ([f26c9a0](https://github.com/AztecProtocol/aztec-packages/commit/f26c9a0df2889c1e1bfbc706199abed67ed3efc4))
* **build:** Reset version back ([#2132](https://github.com/AztecProtocol/aztec-packages/issues/2132)) ([750a757](https://github.com/AztecProtocol/aztec-packages/commit/750a7570c91262e3320bc786fc2944c097427d94))
* **build:** Unify barretenberg releases with aztec-packages ([#2120](https://github.com/AztecProtocol/aztec-packages/issues/2120)) ([82823d8](https://github.com/AztecProtocol/aztec-packages/commit/82823d8cd6882b191a7b363aa40344f66dfd7af7))
* Delete broken bb Dockerfile.arm64-linux-gcc ([#2138](https://github.com/AztecProtocol/aztec-packages/issues/2138)) ([0f988b7](https://github.com/AztecProtocol/aztec-packages/commit/0f988b77ed4c1d0b01916763433344b54765e65a))
* **documentation:** Document noteCommitment vs noteHash ([#2127](https://github.com/AztecProtocol/aztec-packages/issues/2127)) ([73b484f](https://github.com/AztecProtocol/aztec-packages/commit/73b484f474a16b53920fa1dc4f71cbe1ff2bf9ce)), closes [#1679](https://github.com/AztecProtocol/aztec-packages/issues/1679)
* **master:** Release 0.6.0 ([#2121](https://github.com/AztecProtocol/aztec-packages/issues/2121)) ([9bc8e11](https://github.com/AztecProtocol/aztec-packages/commit/9bc8e11ec4598c54d2c8f37c9f1a38ad90148f12))

## [0.6.0](https://github.com/AztecProtocol/aztec-packages/compare/aztec-packages-v0.5.1...aztec-packages-v0.6.0) (2023-09-08)


### ⚠ BREAKING CHANGES

* update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/aztec-packages/issues/1925))
* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/aztec-packages/issues/1618))

### Features

* `CompleteAddress` type and overall AztecRPC refactor ([#1524](https://github.com/AztecProtocol/aztec-packages/issues/1524)) ([aa2c74c](https://github.com/AztecProtocol/aztec-packages/commit/aa2c74c8503469630611b7004c4052b80b5fe815))
* `FunctionSelector` type ([#1518](https://github.com/AztecProtocol/aztec-packages/issues/1518)) ([942f705](https://github.com/AztecProtocol/aztec-packages/commit/942f7058adc706924ff26d2490bec7f7d57d7149)), closes [#1424](https://github.com/AztecProtocol/aztec-packages/issues/1424)
* `GrumpkinScalar` type ([#1919](https://github.com/AztecProtocol/aztec-packages/issues/1919)) ([3a9238a](https://github.com/AztecProtocol/aztec-packages/commit/3a9238a99a32259d8d6b85df6335a002c7bab354))
* **892:** Add hints for matching transient read requests with correspondi… ([#1995](https://github.com/AztecProtocol/aztec-packages/issues/1995)) ([0955bb7](https://github.com/AztecProtocol/aztec-packages/commit/0955bb7b0903b12c4f041096d51a1dbb48f6359d))
* Add `info` command to bb ([#2010](https://github.com/AztecProtocol/aztec-packages/issues/2010)) ([1fd8196](https://github.com/AztecProtocol/aztec-packages/commit/1fd8196f302ee49f540dea54ce5df4c450592d05))
* Add ARM build for Mac + cleanup artifacts ([#1837](https://github.com/AztecProtocol/aztec-packages/issues/1837)) ([270a4ae](https://github.com/AztecProtocol/aztec-packages/commit/270a4ae4d1998149735251e2c3c1be73a9029f61))
* Add msgpack defs to remaining circuit types ([#1538](https://github.com/AztecProtocol/aztec-packages/issues/1538)) ([22037d8](https://github.com/AztecProtocol/aztec-packages/commit/22037d89cc45c718bb0dc1a49e76d78cd6ba90dd))
* Add support for assert messages & runtime call stacks  ([#1997](https://github.com/AztecProtocol/aztec-packages/issues/1997)) ([ac68837](https://github.com/AztecProtocol/aztec-packages/commit/ac68837677a80897538d7a0790af8d04410c4446))
* Add workflow to output to dev-bb.js ([#1299](https://github.com/AztecProtocol/aztec-packages/issues/1299)) ([624ffaf](https://github.com/AztecProtocol/aztec-packages/commit/624ffaf1c920d29a12458eb045c8fec7ce978a1a))
* **aztec-js:** Account class ([#1429](https://github.com/AztecProtocol/aztec-packages/issues/1429)) ([e788745](https://github.com/AztecProtocol/aztec-packages/commit/e788745b73a5b7632a3e346e2a698dbbb2314ed7))
* **aztec-js:** Remove sender from execution request and add batching ([#1415](https://github.com/AztecProtocol/aztec-packages/issues/1415)) ([05b6e86](https://github.com/AztecProtocol/aztec-packages/commit/05b6e869d89e9313f6e60580a3eef21f88f55446))
* **aztec-js:** Return contract instance when awaiting deploy tx ([#1360](https://github.com/AztecProtocol/aztec-packages/issues/1360)) ([e9c945c](https://github.com/AztecProtocol/aztec-packages/commit/e9c945cc680974383023737d299bc35645771e85))
* **aztec-js:** Tx.wait waits for rpc to be synced ([#1381](https://github.com/AztecProtocol/aztec-packages/issues/1381)) ([261032e](https://github.com/AztecProtocol/aztec-packages/commit/261032ee3d8244a12850add3e75e9aeddd68456b)), closes [#1340](https://github.com/AztecProtocol/aztec-packages/issues/1340)
* **aztec-noir:** Align public and private execution patterns ([#1515](https://github.com/AztecProtocol/aztec-packages/issues/1515)) ([35a81c3](https://github.com/AztecProtocol/aztec-packages/commit/35a81c38f0738d2121b57e2dbfc1c4a85c20d6b8))
* **Aztec.nr:** Kernel return types abstraction ([#1924](https://github.com/AztecProtocol/aztec-packages/issues/1924)) ([3a8e702](https://github.com/AztecProtocol/aztec-packages/commit/3a8e7026ea10aa8564bdcc127efd4213ebd526de))
* **bb:** Use an environment variable to set the transcript URL ([#1750](https://github.com/AztecProtocol/aztec-packages/issues/1750)) ([31488c1](https://github.com/AztecProtocol/aztec-packages/commit/31488c19acfdfd5ff0c3e7f242f94dc0aa049158))
* **blocks_tree:** Compute block hashes within root rollup circuit ([#1214](https://github.com/AztecProtocol/aztec-packages/issues/1214)) ([71dc039](https://github.com/AztecProtocol/aztec-packages/commit/71dc03973455c320ad4edb1a21d059bdf417445a))
* Broadcasting 'public key' and 'partial address' as L1 calldata ([#1801](https://github.com/AztecProtocol/aztec-packages/issues/1801)) ([78d6444](https://github.com/AztecProtocol/aztec-packages/commit/78d6444e82903fe3d0d17318cd38b1b262e81391)), closes [#1778](https://github.com/AztecProtocol/aztec-packages/issues/1778)
* CDP/Lending example contract ([#1554](https://github.com/AztecProtocol/aztec-packages/issues/1554)) ([ecf6df2](https://github.com/AztecProtocol/aztec-packages/commit/ecf6df201047dcaa61c270cdb512cdc62086b356))
* Celer benchmark ([#1369](https://github.com/AztecProtocol/aztec-packages/issues/1369)) ([7ec6b32](https://github.com/AztecProtocol/aztec-packages/commit/7ec6b32620c851d73e133e939f888047474ebc71))
* Check sandbox version matches CLI's ([#1849](https://github.com/AztecProtocol/aztec-packages/issues/1849)) ([7279730](https://github.com/AztecProtocol/aztec-packages/commit/72797305ac9ce8639abb09334cf2471f0932ca88))
* Checking if origin is registered ([#1393](https://github.com/AztecProtocol/aztec-packages/issues/1393)) ([8b3a064](https://github.com/AztecProtocol/aztec-packages/commit/8b3a0641a5fc78c5906d88267d3c8f0e2753025d)), closes [#1230](https://github.com/AztecProtocol/aztec-packages/issues/1230)
* **ci:** Initial release please config ([#1769](https://github.com/AztecProtocol/aztec-packages/issues/1769)) ([4207559](https://github.com/AztecProtocol/aztec-packages/commit/42075590058b21f38b5e745af54b2062371f9ebe))
* **circuits:** Hints nullifier transient commitments ([#2056](https://github.com/AztecProtocol/aztec-packages/issues/2056)) ([725b550](https://github.com/AztecProtocol/aztec-packages/commit/725b550a368494abd15a38e95b15b1379bc926bc))
* **ci:** Use content hash in build system, restrict docs build to *.ts or *.cpp ([#1953](https://github.com/AztecProtocol/aztec-packages/issues/1953)) ([0036e07](https://github.com/AztecProtocol/aztec-packages/commit/0036e0742a67dfa8aa1fcdb498b89caca6441508))
* **cli:** Noir contract compiler CLI ([#1561](https://github.com/AztecProtocol/aztec-packages/issues/1561)) ([4af4845](https://github.com/AztecProtocol/aztec-packages/commit/4af4845f80b1be548efa1ca79f5588bb1c7f1423)), closes [#1457](https://github.com/AztecProtocol/aztec-packages/issues/1457)
* **cli:** Retry on http errors ([#1606](https://github.com/AztecProtocol/aztec-packages/issues/1606)) ([7af5994](https://github.com/AztecProtocol/aztec-packages/commit/7af59942e8691fa49b834f036b58f5de26821171))
* **cli:** Use options instead of args in get-logs ([#1559](https://github.com/AztecProtocol/aztec-packages/issues/1559)) ([9f40ef8](https://github.com/AztecProtocol/aztec-packages/commit/9f40ef80d7180bab42685453d51cfce8d770dfb0))
* Compress debug symbols ([#1760](https://github.com/AztecProtocol/aztec-packages/issues/1760)) ([9464b25](https://github.com/AztecProtocol/aztec-packages/commit/9464b25c1a2a809db559ddc4e2d4ee5ade1fa65a))
* Do not allow slot 0 in `noir-libs` ([#1884](https://github.com/AztecProtocol/aztec-packages/issues/1884)) ([54094b4](https://github.com/AztecProtocol/aztec-packages/commit/54094b464a4dc7aebf157ca54145cffce822bc6f)), closes [#1692](https://github.com/AztecProtocol/aztec-packages/issues/1692)
* **docs:** Add tabs for deploying contract with cli and aztec.js ([#1703](https://github.com/AztecProtocol/aztec-packages/issues/1703)) ([d2a284d](https://github.com/AztecProtocol/aztec-packages/commit/d2a284dabd30e05ec771e719f9d0c9963438d4af))
* **docs:** Adding some nitpick suggestions before sandbox release ([#1859](https://github.com/AztecProtocol/aztec-packages/issues/1859)) ([c1144f7](https://github.com/AztecProtocol/aztec-packages/commit/c1144f7bcfe8ebe222b840b0edd3d901ca30bdaf))
* **docs:** Cheatcode docs ([#1585](https://github.com/AztecProtocol/aztec-packages/issues/1585)) ([b1a2f8f](https://github.com/AztecProtocol/aztec-packages/commit/b1a2f8fa6b38a1c03a62c25428932c8d2a9a4fdb))
* **docs:** Set up noir contracts in getting-started ([#1770](https://github.com/AztecProtocol/aztec-packages/issues/1770)) ([33eb99d](https://github.com/AztecProtocol/aztec-packages/commit/33eb99d4a00831f340b1b0de0352fc272cb66d14))
* **docs:** Testing guide and getPrivateStorage method ([#1992](https://github.com/AztecProtocol/aztec-packages/issues/1992)) ([5a8c571](https://github.com/AztecProtocol/aztec-packages/commit/5a8c5719753549f71ceeec9114d69b8d1d640376))
* Generate public context contract interfaces ([#1860](https://github.com/AztecProtocol/aztec-packages/issues/1860)) ([2f4045e](https://github.com/AztecProtocol/aztec-packages/commit/2f4045e22dbea0e316103da20c6ba8a667826777)), closes [#1782](https://github.com/AztecProtocol/aztec-packages/issues/1782)
* Goblin recursive verifier ([#1822](https://github.com/AztecProtocol/aztec-packages/issues/1822)) ([f962cb6](https://github.com/AztecProtocol/aztec-packages/commit/f962cb68f46d25047bf67a1ad2e7407a176ffc53))
* Honk recursive verifier Pt. 1 ([#1488](https://github.com/AztecProtocol/aztec-packages/issues/1488)) ([4669555](https://github.com/AztecProtocol/aztec-packages/commit/466955559750bce4b4d81149ca81c02742b9246c))
* Initial `is_valid` eip1271 style wallet + minimal test changes ([#1935](https://github.com/AztecProtocol/aztec-packages/issues/1935)) ([f264c54](https://github.com/AztecProtocol/aztec-packages/commit/f264c5421424bf58d983fe104aaf7c7126259e01))
* Initial cheatcode `loadPublic` ([#1353](https://github.com/AztecProtocol/aztec-packages/issues/1353)) ([75c35a7](https://github.com/AztecProtocol/aztec-packages/commit/75c35a7506bcc5a9ae1afee90c70cfb95b08b347))
* Initial portal docs + minor cleanups ([#1469](https://github.com/AztecProtocol/aztec-packages/issues/1469)) ([37316f4](https://github.com/AztecProtocol/aztec-packages/commit/37316f4fb484c7c03bd44e9a14cd576714f092c5))
* Initial trazability of ACIR ([#1701](https://github.com/AztecProtocol/aztec-packages/issues/1701)) ([89e4e1a](https://github.com/AztecProtocol/aztec-packages/commit/89e4e1ac5e90905aa475ba2f8b6afb7b77dc772a))
* Minimal barretenberg .circleci ([#1352](https://github.com/AztecProtocol/aztec-packages/issues/1352)) ([36e4239](https://github.com/AztecProtocol/aztec-packages/commit/36e4239eccf00bc009e42ec218d0922b5d1138da))
* More reliable getTxReceipt api. ([#1793](https://github.com/AztecProtocol/aztec-packages/issues/1793)) ([ad16b22](https://github.com/AztecProtocol/aztec-packages/commit/ad16b2219bff44dfbc3482b81c86e29bf0d60fc5))
* New NoteProcessor works through all blocks ([#1404](https://github.com/AztecProtocol/aztec-packages/issues/1404)) ([c8e7d53](https://github.com/AztecProtocol/aztec-packages/commit/c8e7d539b7a3f4d7b4eee7e4eef1499715711109))
* New stdlib Transcript  ([#1219](https://github.com/AztecProtocol/aztec-packages/issues/1219)) ([2f66de1](https://github.com/AztecProtocol/aztec-packages/commit/2f66de15212a5b6eb398e0919ae3ad4ec572fde0))
* No unencrypted logs in private functions ([#1780](https://github.com/AztecProtocol/aztec-packages/issues/1780)) ([4d8002e](https://github.com/AztecProtocol/aztec-packages/commit/4d8002e0d101a14c465929d92ea05d0be6e8d99a)), closes [#1689](https://github.com/AztecProtocol/aztec-packages/issues/1689)
* No unlimited retries by default in aztec.js ([#1723](https://github.com/AztecProtocol/aztec-packages/issues/1723)) ([95d1350](https://github.com/AztecProtocol/aztec-packages/commit/95d1350b23b6205ff2a7d3de41a37e0bc9ee7640))
* **noir-contracts:** `Option&lt;T&gt;` for get_notes ([#1272](https://github.com/AztecProtocol/aztec-packages/issues/1272)) ([584b70f](https://github.com/AztecProtocol/aztec-packages/commit/584b70f11d9cfd95201462f61b154ed2abdb685c))
* **noir:** Autogenerate contract interface for calling from external contracts ([#1487](https://github.com/AztecProtocol/aztec-packages/issues/1487)) ([e9d0e6b](https://github.com/AztecProtocol/aztec-packages/commit/e9d0e6bbe6645c6f9a303f99c9952fc2ce7bcb03))
* **noir:** Better NoteGetterOptions. ([#1695](https://github.com/AztecProtocol/aztec-packages/issues/1695)) ([2f78293](https://github.com/AztecProtocol/aztec-packages/commit/2f78293643186232d4f2013acdf56b263b89bf56))
* **noir:** Use `#[aztec(private)]` and `#[aztec(public)` attributes ([#1735](https://github.com/AztecProtocol/aztec-packages/issues/1735)) ([89756fa](https://github.com/AztecProtocol/aztec-packages/commit/89756fae7d562274a84c60024beff5fae032f297))
* Not retrying unrecoverable errors ([#1752](https://github.com/AztecProtocol/aztec-packages/issues/1752)) ([c0f2820](https://github.com/AztecProtocol/aztec-packages/commit/c0f28204f53152c941704ece66287eddfe13c047))
* **oracle:** Add oracle to get portal contract address ([#1474](https://github.com/AztecProtocol/aztec-packages/issues/1474)) ([5cce848](https://github.com/AztecProtocol/aztec-packages/commit/5cce848fc776abe4fcf54fb39e1b1ed740fd3583))
* Pin noir commit to aztec tag ([#1461](https://github.com/AztecProtocol/aztec-packages/issues/1461)) ([afe601a](https://github.com/AztecProtocol/aztec-packages/commit/afe601afa0f58c09c421a6d559645472d4b42ed3))
* Public view functions (unconstrained can read public storage) ([#1421](https://github.com/AztecProtocol/aztec-packages/issues/1421)) ([912c1b4](https://github.com/AztecProtocol/aztec-packages/commit/912c1b44b83a87ce6da7e9c5a99b9d5d3ba8aaf4))
* Recursive fn calls to spend more notes. ([#1779](https://github.com/AztecProtocol/aztec-packages/issues/1779)) ([94053e4](https://github.com/AztecProtocol/aztec-packages/commit/94053e44f4d2a702fe9066bfff3cdd35e6d1b645))
* Register-public-key & CLI update to use options instead of args ([#1397](https://github.com/AztecProtocol/aztec-packages/issues/1397)) ([d142181](https://github.com/AztecProtocol/aztec-packages/commit/d14218184478a22cca1a011763801d2f82a40f65))
* Simulate enqueued public functions and locate failing constraints on them ([#1853](https://github.com/AztecProtocol/aztec-packages/issues/1853)) ([a065fd5](https://github.com/AztecProtocol/aztec-packages/commit/a065fd53dde48a1f28616ebe130222dd39d07b11))
* Throw when creating an instance of non-existent contract ([#1300](https://github.com/AztecProtocol/aztec-packages/issues/1300)) ([5353ed0](https://github.com/AztecProtocol/aztec-packages/commit/5353ed0ae5ecfd227fac36b8f2305c3d91d1c855)), closes [#1225](https://github.com/AztecProtocol/aztec-packages/issues/1225)
* Throwing when submitting a duplicate tx of a settled one ([#1880](https://github.com/AztecProtocol/aztec-packages/issues/1880)) ([9ad768f](https://github.com/AztecProtocol/aztec-packages/commit/9ad768f1af5344dc079a74e80ec601e062558fd5)), closes [#1810](https://github.com/AztecProtocol/aztec-packages/issues/1810)
* Timing in build system ([#1411](https://github.com/AztecProtocol/aztec-packages/issues/1411)) ([b30f43f](https://github.com/AztecProtocol/aztec-packages/commit/b30f43fa9ffd5d62b20ffd843c0deeef5e132e4f))
* Typos, using Tx.clone functionality, better naming ([#1976](https://github.com/AztecProtocol/aztec-packages/issues/1976)) ([00bca67](https://github.com/AztecProtocol/aztec-packages/commit/00bca675cf7984052c960c3d1797c5b017f07f57))
* Update safe_math and move to libraries ([#1803](https://github.com/AztecProtocol/aztec-packages/issues/1803)) ([b10656d](https://github.com/AztecProtocol/aztec-packages/commit/b10656d30622366dcbbe5adb5b3948b0702a06e7))
* Updated noir version ([#1581](https://github.com/AztecProtocol/aztec-packages/issues/1581)) ([91f9047](https://github.com/AztecProtocol/aztec-packages/commit/91f9047da8489404718441ba498b9424c9d7000e))
* Write debug-level log to local file in Sandbox ([#1846](https://github.com/AztecProtocol/aztec-packages/issues/1846)) ([0317e93](https://github.com/AztecProtocol/aztec-packages/commit/0317e93d3dffb3b66a926863e7fe8b8c15f61536)), closes [#1605](https://github.com/AztecProtocol/aztec-packages/issues/1605)
* **yarn:** Run workspace commands in parallel ([#1543](https://github.com/AztecProtocol/aztec-packages/issues/1543)) ([791f1cc](https://github.com/AztecProtocol/aztec-packages/commit/791f1ccecc4fa20eb48d0069061483c6a68b6d28))


### Bug Fixes

* Accidental git marker ([#2039](https://github.com/AztecProtocol/aztec-packages/issues/2039)) ([2be9908](https://github.com/AztecProtocol/aztec-packages/commit/2be990861ca25ec206f6bd02b604b73b30710ca8))
* **acir:** When retrying failed ACIR tests it should not use the default CLI argument ([#1673](https://github.com/AztecProtocol/aztec-packages/issues/1673)) ([910b103](https://github.com/AztecProtocol/aztec-packages/commit/910b10392a9bb7472948bec5cc634eebea137288))
* Add noir clean command & clean noir artifacts when building ([#1482](https://github.com/AztecProtocol/aztec-packages/issues/1482)) ([8e722c3](https://github.com/AztecProtocol/aztec-packages/commit/8e722c3a4deaab2794506092dae7dff4f977db04))
* Add retry to tag and docker actions ([#2099](https://github.com/AztecProtocol/aztec-packages/issues/2099)) ([9f741f4](https://github.com/AztecProtocol/aztec-packages/commit/9f741f4e181120edcb63c28fa6c50b5b5e2e26c9))
* Add retry_10 around ensure_repo ([#1963](https://github.com/AztecProtocol/aztec-packages/issues/1963)) ([0afde39](https://github.com/AztecProtocol/aztec-packages/commit/0afde390ac63d132b0ba85440500da3375fd2e22))
* Adds Mac cross compile flags into barretenberg ([#1954](https://github.com/AztecProtocol/aztec-packages/issues/1954)) ([3aaf91e](https://github.com/AztecProtocol/aztec-packages/commit/3aaf91e03fc01f0cb12249f22dbcb007023f69d4))
* Align bbmalloc implementations ([#1513](https://github.com/AztecProtocol/aztec-packages/issues/1513)) ([c512fcd](https://github.com/AztecProtocol/aztec-packages/commit/c512fcd23b43090f5e01819a2ead29747e7517ad))
* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/aztec-packages/issues/1618)) ([4bc551e](https://github.com/AztecProtocol/aztec-packages/commit/4bc551ef086c1e3d966f8ece5f5930405d8f5b11))
* Bb meta-data ([#1960](https://github.com/AztecProtocol/aztec-packages/issues/1960)) ([712e0a0](https://github.com/AztecProtocol/aztec-packages/commit/712e0a088bff9ae2f49489901fab2a3fe0fb6d4b))
* Bb sync take 2 ([#1669](https://github.com/AztecProtocol/aztec-packages/issues/1669)) ([fd09bc2](https://github.com/AztecProtocol/aztec-packages/commit/fd09bc26780dc08214d0ceca3d04ed10db23fead))
* **bb.js:** (breaking change) bundles bb.js properly so that it works in the browser and in node ([#1855](https://github.com/AztecProtocol/aztec-packages/issues/1855)) ([1aa6f59](https://github.com/AztecProtocol/aztec-packages/commit/1aa6f5934cd97dd32d81e490013f8ef7d1e14ec7))
* **bb:** Fix Typo ([#1709](https://github.com/AztecProtocol/aztec-packages/issues/1709)) ([287f5ae](https://github.com/AztecProtocol/aztec-packages/commit/287f5ae2cc556c1664d4240928baecadf92627e5))
* Benchmark git repo ([#2041](https://github.com/AztecProtocol/aztec-packages/issues/2041)) ([3c696bb](https://github.com/AztecProtocol/aztec-packages/commit/3c696bba1ca4bd69c8e3f5bc004d1a07adb23cf1))
* Benchmark preset uses clang16 ([#1902](https://github.com/AztecProtocol/aztec-packages/issues/1902)) ([4f7eeea](https://github.com/AztecProtocol/aztec-packages/commit/4f7eeea6c79604aea88433790dfc542a356aa898))
* **breaking change:** Change embedded curve scalar mul to use two limbs to properly encode the scalar field ([#2105](https://github.com/AztecProtocol/aztec-packages/issues/2105)) ([070cc4c](https://github.com/AztecProtocol/aztec-packages/commit/070cc4cb31ada29e42846e16df1ec191100214a5))
* Broken bootstrap.sh after renaming `aztec-cli` dir as `cli` ([#2097](https://github.com/AztecProtocol/aztec-packages/issues/2097)) ([2386781](https://github.com/AztecProtocol/aztec-packages/commit/2386781fd1fed9f552559961b4e9f60406095067))
* Browser test in canary flow ([#2102](https://github.com/AztecProtocol/aztec-packages/issues/2102)) ([d52af6c](https://github.com/AztecProtocol/aztec-packages/commit/d52af6c0e2c5ed268747237e65603368645c9966)), closes [#2086](https://github.com/AztecProtocol/aztec-packages/issues/2086)
* Build ([#1906](https://github.com/AztecProtocol/aztec-packages/issues/1906)) ([8223be1](https://github.com/AztecProtocol/aztec-packages/commit/8223be18d98ebb4edb7700310b2fda5201bd04b9))
* Build script ([#2017](https://github.com/AztecProtocol/aztec-packages/issues/2017)) ([23fce27](https://github.com/AztecProtocol/aztec-packages/commit/23fce277c44a06777ea168085ac498d62016b36e))
* Build-system spot request cancellation ([#1339](https://github.com/AztecProtocol/aztec-packages/issues/1339)) ([0c8ce7d](https://github.com/AztecProtocol/aztec-packages/commit/0c8ce7d33483b6df5f747c7ad0aa8376b4f392a1))
* **build-system:** Undefined IMAGE_TAG and ARG_TAG ([#2030](https://github.com/AztecProtocol/aztec-packages/issues/2030)) ([dfdba4b](https://github.com/AztecProtocol/aztec-packages/commit/dfdba4b5c6fb0c75f7f463e0b5eb71e6e7d1b667))
* **build:** Config fixes for release please ([#2123](https://github.com/AztecProtocol/aztec-packages/issues/2123)) ([7b4f30d](https://github.com/AztecProtocol/aztec-packages/commit/7b4f30dbdf29a907b7474e5f308849ca068f056e))
* **build:** Use semver version in docker version tag ([#2065](https://github.com/AztecProtocol/aztec-packages/issues/2065)) ([b3db0d0](https://github.com/AztecProtocol/aztec-packages/commit/b3db0d0ae6d6b7d8a6d7338a556e2b9507e2631a))
* Canary browser test transfer method ([#2126](https://github.com/AztecProtocol/aztec-packages/issues/2126)) ([a23b037](https://github.com/AztecProtocol/aztec-packages/commit/a23b0370ae9395ca51ed8f94a1d71b57d35916a0))
* Check a note is read before nullifying it. ([#2076](https://github.com/AztecProtocol/aztec-packages/issues/2076)) ([aabfb13](https://github.com/AztecProtocol/aztec-packages/commit/aabfb1383033364df9c045573098a4f13ca3a452)), closes [#1899](https://github.com/AztecProtocol/aztec-packages/issues/1899)
* **ci:** Incorrect content hash in some build targets ([#1973](https://github.com/AztecProtocol/aztec-packages/issues/1973)) ([0a2a515](https://github.com/AztecProtocol/aztec-packages/commit/0a2a515ecf52849cce1e45a7b39f44d420b43f34))
* **ci:** Publish missing sandbox dependency ([#1599](https://github.com/AztecProtocol/aztec-packages/issues/1599)) ([52c7966](https://github.com/AztecProtocol/aztec-packages/commit/52c7966a118fdbe90bc739c006b9a116bc4c4dc0))
* Circuits issues when building with gcc ([#2107](https://github.com/AztecProtocol/aztec-packages/issues/2107)) ([4f5c4fe](https://github.com/AztecProtocol/aztec-packages/commit/4f5c4fe24f012988169d8a0a3d8ae5245e24d3ee))
* Circuits should not link openmp with -DMULTITHREADING ([#1929](https://github.com/AztecProtocol/aztec-packages/issues/1929)) ([cd1a685](https://github.com/AztecProtocol/aztec-packages/commit/cd1a685a3ecdd571d83cd2ad0844bd1d143fd9af))
* Clang version in README and subrepo edge case ([#1730](https://github.com/AztecProtocol/aztec-packages/issues/1730)) ([26d836d](https://github.com/AztecProtocol/aztec-packages/commit/26d836d6453c2bc7fd9a1a091bdd63aefc4ed1dd))
* Cli canary & deployment ([#2053](https://github.com/AztecProtocol/aztec-packages/issues/2053)) ([1ddd24a](https://github.com/AztecProtocol/aztec-packages/commit/1ddd24ad2f8702fd3d3c48ed015a652b3326bfd9))
* **cli:** Fixes in get-logs and deploy commands ([#1572](https://github.com/AztecProtocol/aztec-packages/issues/1572)) ([493405b](https://github.com/AztecProtocol/aztec-packages/commit/493405b3d882706c592bf42142e1072aba650dbd))
* COMMIT_TAG arg value in canary Dockerfile ([#2118](https://github.com/AztecProtocol/aztec-packages/issues/2118)) ([a3d6459](https://github.com/AztecProtocol/aztec-packages/commit/a3d645978a6ccef279870498979733682f63e206))
* Compilation on homebrew clang 16.06 ([#1937](https://github.com/AztecProtocol/aztec-packages/issues/1937)) ([c611582](https://github.com/AztecProtocol/aztec-packages/commit/c611582239a057717410f0a6c0fd8202844a564e))
* Complete JS call stacks across ACVM wasm boundaries ([#2013](https://github.com/AztecProtocol/aztec-packages/issues/2013)) ([8e84e46](https://github.com/AztecProtocol/aztec-packages/commit/8e84e460899f11eaf7f383863e20dc5395e45c6e))
* Conditionally compile base64 command for bb binary ([#1851](https://github.com/AztecProtocol/aztec-packages/issues/1851)) ([be97185](https://github.com/AztecProtocol/aztec-packages/commit/be9718505c7e387bb46183299c9db855e6d7f91c))
* Default color to light mode ([#1847](https://github.com/AztecProtocol/aztec-packages/issues/1847)) ([4fc8d39](https://github.com/AztecProtocol/aztec-packages/commit/4fc8d39041d437940bb18815e14f506b2ebe259e))
* Deploy_ecr calculating CONTENT_HASH ([#2024](https://github.com/AztecProtocol/aztec-packages/issues/2024)) ([edee198](https://github.com/AztecProtocol/aztec-packages/commit/edee1981d8d795aef64bd6de738f09ea9a1a2547))
* Disable uniswap until [#1367](https://github.com/AztecProtocol/aztec-packages/issues/1367) ([#1368](https://github.com/AztecProtocol/aztec-packages/issues/1368)) ([7a1c4f7](https://github.com/AztecProtocol/aztec-packages/commit/7a1c4f7901788f127e903d275d4efa2316eab848))
* Disallow unregistered classes in JSON RPC interface and match by name ([#1820](https://github.com/AztecProtocol/aztec-packages/issues/1820)) ([35b8170](https://github.com/AztecProtocol/aztec-packages/commit/35b817055e1fe848e6d87d445a7881c5c128ad35))
* Do not warn on mismatched cli/sandbox version ([#1894](https://github.com/AztecProtocol/aztec-packages/issues/1894)) ([a44a0f6](https://github.com/AztecProtocol/aztec-packages/commit/a44a0f6489b8ea7d648d1b9babf49fae8d593b7b))
* Docs preprocessor line numbers and errors ([#1883](https://github.com/AztecProtocol/aztec-packages/issues/1883)) ([4e7e290](https://github.com/AztecProtocol/aztec-packages/commit/4e7e290478ae4ca9c128c0b6b4b26529965cc2a2))
* **docs:** Fix code snippet preprocessor ([#1485](https://github.com/AztecProtocol/aztec-packages/issues/1485)) ([db0cc14](https://github.com/AztecProtocol/aztec-packages/commit/db0cc1414978b04518218c85e04cba424b64b942))
* Don't include SRS in sandbox docker img ([#1704](https://github.com/AztecProtocol/aztec-packages/issues/1704)) ([aa7f662](https://github.com/AztecProtocol/aztec-packages/commit/aa7f662d3fe3a3c3833c594947c637790442477d))
* Dont assume safety of nvm ([#2079](https://github.com/AztecProtocol/aztec-packages/issues/2079)) ([a4167e7](https://github.com/AztecProtocol/aztec-packages/commit/a4167e7e5ef55c9780c786959d078fe854093656))
* Download SRS using one canonical URL across the codebase ([#1748](https://github.com/AztecProtocol/aztec-packages/issues/1748)) ([899b055](https://github.com/AztecProtocol/aztec-packages/commit/899b05557365a5bf97e64793dd563a1b4bfa0f3f))
* End-to-end aztec cli dependency issue ([#2092](https://github.com/AztecProtocol/aztec-packages/issues/2092)) ([16ee3e5](https://github.com/AztecProtocol/aztec-packages/commit/16ee3e530bd99c2a47b8bcda53f0a13f67df2ac6))
* Ensure CLI command doesn't fail due to missing client version ([#1895](https://github.com/AztecProtocol/aztec-packages/issues/1895)) ([88086e4](https://github.com/AztecProtocol/aztec-packages/commit/88086e4a80d7841d28188366a469800afa281693))
* Ensure noir clean doesnt error ([#1613](https://github.com/AztecProtocol/aztec-packages/issues/1613)) ([ee00df5](https://github.com/AztecProtocol/aztec-packages/commit/ee00df5794b1d8e0ec4776fab8ec7d957d692fa5))
* Ensure_repo undefined-safe ([#2025](https://github.com/AztecProtocol/aztec-packages/issues/2025)) ([e36fb6b](https://github.com/AztecProtocol/aztec-packages/commit/e36fb6bb8a1ee9a3d405c3e5340ffa4e589656e2))
* Error handling in acir simulator ([#1907](https://github.com/AztecProtocol/aztec-packages/issues/1907)) ([165008e](https://github.com/AztecProtocol/aztec-packages/commit/165008ec3027d8f2f76256c37f63e5d7a669b5dd))
* File reference to canary docker-compose file ([#2124](https://github.com/AztecProtocol/aztec-packages/issues/2124)) ([13d3f16](https://github.com/AztecProtocol/aztec-packages/commit/13d3f161cc2ee6b49e4448ae9e8d33dd7f6ce7d2))
* Fix off by one in circuits.js when fetching points from transcript ([#1993](https://github.com/AztecProtocol/aztec-packages/issues/1993)) ([cec901f](https://github.com/AztecProtocol/aztec-packages/commit/cec901f3df440ebc0e3bdcfb2567b70fd9bde9dd))
* Fix paths in `barretenberg` bootstrap.sh script ([#1662](https://github.com/AztecProtocol/aztec-packages/issues/1662)) ([24bbfd4](https://github.com/AztecProtocol/aztec-packages/commit/24bbfd446bf1f2b7fec8313dc010cd5094df0e71))
* Fix race condition between RPC Server and Aztec Node ([#1700](https://github.com/AztecProtocol/aztec-packages/issues/1700)) ([4c89941](https://github.com/AztecProtocol/aztec-packages/commit/4c89941d0c3803ce72b86e76eead95a23d80d810))
* Fixed a failing test and added a small fuzzer ([#1384](https://github.com/AztecProtocol/aztec-packages/issues/1384)) ([f258e08](https://github.com/AztecProtocol/aztec-packages/commit/f258e08aaa2e02c7a39d8d6b83a7037c0a5d36ea))
* Fixing fuzzing build after composer splitting ([#1317](https://github.com/AztecProtocol/aztec-packages/issues/1317)) ([6b2e759](https://github.com/AztecProtocol/aztec-packages/commit/6b2e75940026e0133f9fa56080a4c424172172f0))
* Format.sh issues ([#1946](https://github.com/AztecProtocol/aztec-packages/issues/1946)) ([f24814b](https://github.com/AztecProtocol/aztec-packages/commit/f24814b328c45316fa584cad1d9aa4784b6a0b2e))
* Hack an ordering index for enqueued public calls ([#1639](https://github.com/AztecProtocol/aztec-packages/issues/1639)) ([87712e8](https://github.com/AztecProtocol/aztec-packages/commit/87712e82a504d8c09d2df5f8b8f57a03d88fae93)), closes [#1624](https://github.com/AztecProtocol/aztec-packages/issues/1624)
* Increment time by 1 for previous rollup was warped ([#1594](https://github.com/AztecProtocol/aztec-packages/issues/1594)) ([2a52107](https://github.com/AztecProtocol/aztec-packages/commit/2a521070397b6d1915e55b4ec702d4778563e683))
* Master ([#1981](https://github.com/AztecProtocol/aztec-packages/issues/1981)) ([6bfb053](https://github.com/AztecProtocol/aztec-packages/commit/6bfb053fb2c4053a72a8daa18a241261380ee311))
* Minor annoyances ([#2115](https://github.com/AztecProtocol/aztec-packages/issues/2115)) ([a147582](https://github.com/AztecProtocol/aztec-packages/commit/a1475822b20c360d19a88f6205a4a35d987fc2f5))
* Mirror after direct bb merge ([#1651](https://github.com/AztecProtocol/aztec-packages/issues/1651)) ([5f08fff](https://github.com/AztecProtocol/aztec-packages/commit/5f08fff8355671e883bef0380bf06313429d3e1d))
* More accurate c++ build pattern ([#1962](https://github.com/AztecProtocol/aztec-packages/issues/1962)) ([21c2f8e](https://github.com/AztecProtocol/aztec-packages/commit/21c2f8edd110da8749a0039c900c25aff8baa7a4))
* Noir contract artifacts generation in CI ([#1366](https://github.com/AztecProtocol/aztec-packages/issues/1366)) ([f715a55](https://github.com/AztecProtocol/aztec-packages/commit/f715a55c8b66ddd6133e6cec70b82c4083575233))
* **noir-ci:** Reinstate artifact builds ([#1396](https://github.com/AztecProtocol/aztec-packages/issues/1396)) ([2c43878](https://github.com/AztecProtocol/aztec-packages/commit/2c43878a72d9ce43e212416c1901bad40a0a763a))
* Noir-contracts build ([#1362](https://github.com/AztecProtocol/aztec-packages/issues/1362)) ([71384b0](https://github.com/AztecProtocol/aztec-packages/commit/71384b098b0f81190329d6a685ddfc6c34536473))
* **noir:** Add workaround for latest noir in account contracts ([#1781](https://github.com/AztecProtocol/aztec-packages/issues/1781)) ([eb8a052](https://github.com/AztecProtocol/aztec-packages/commit/eb8a052ad4e19394f096cc3a0f533c2560a7f5cc))
* Option to fail silently when retrying ([#2015](https://github.com/AztecProtocol/aztec-packages/issues/2015)) ([453c9c1](https://github.com/AztecProtocol/aztec-packages/commit/453c9c1b234213fff4d63e117f2bc6c827040125))
* Padded printing for e2e-cli ([#2106](https://github.com/AztecProtocol/aztec-packages/issues/2106)) ([5988014](https://github.com/AztecProtocol/aztec-packages/commit/5988014330c929e1fcb52c4fbba5a755fa013c16))
* Polyfill by bundling fileURLToPath ([#1949](https://github.com/AztecProtocol/aztec-packages/issues/1949)) ([1b2de01](https://github.com/AztecProtocol/aztec-packages/commit/1b2de01df69a16f442c348cc302ade1392e74519))
* Post bb merge sync ([#1697](https://github.com/AztecProtocol/aztec-packages/issues/1697)) ([d27a026](https://github.com/AztecProtocol/aztec-packages/commit/d27a026cdab57dbba12b162e2df75aab142130c9))
* Proving fails when circuit has size &gt; ~500K ([#1739](https://github.com/AztecProtocol/aztec-packages/issues/1739)) ([708b05c](https://github.com/AztecProtocol/aztec-packages/commit/708b05ca6638dc0d6ca7cb34fb8de76665a43b58))
* Race condition ([#1427](https://github.com/AztecProtocol/aztec-packages/issues/1427)) ([cd78ec9](https://github.com/AztecProtocol/aztec-packages/commit/cd78ec9afa887b1e9ac0b446b110603fad29e7e2))
* Remaining refs to clang15 ([#2077](https://github.com/AztecProtocol/aztec-packages/issues/2077)) ([2c16547](https://github.com/AztecProtocol/aztec-packages/commit/2c16547c450ac7591d5be7c734962be86be4310e))
* Remove automatic update to `AztecProtocol/dev-bb.js` ([#1712](https://github.com/AztecProtocol/aztec-packages/issues/1712)) ([6969f6d](https://github.com/AztecProtocol/aztec-packages/commit/6969f6d41febcda0c884d9ea19fb0875f788f425))
* Remove extra transfer arg in CLI Guide ([#1887](https://github.com/AztecProtocol/aztec-packages/issues/1887)) ([55728b8](https://github.com/AztecProtocol/aztec-packages/commit/55728b850c19403ba8b2aaefe89181640acbd9fd))
* Reset keccak var inputs to 0 ([#1881](https://github.com/AztecProtocol/aztec-packages/issues/1881)) ([382f07e](https://github.com/AztecProtocol/aztec-packages/commit/382f07e3032c5ad3cf15e62e38bb5f0583ab46dd))
* Retry git submodule fetch ([#1371](https://github.com/AztecProtocol/aztec-packages/issues/1371)) ([5cf9c20](https://github.com/AztecProtocol/aztec-packages/commit/5cf9c203e126b7613bf80960063d86cb9ee97954))
* Return DecodedReturn instead of any[] ([#1540](https://github.com/AztecProtocol/aztec-packages/issues/1540)) ([2e344e1](https://github.com/AztecProtocol/aztec-packages/commit/2e344e13eaf628e3f380de625da6a526af4a6b0f))
* Revert clang check bootstrap.sh ([#1734](https://github.com/AztecProtocol/aztec-packages/issues/1734)) ([a931e07](https://github.com/AztecProtocol/aztec-packages/commit/a931e077f2efac2aaa50c5336ead87a0e87a813e))
* **rpc:** Fix bigint serialisation in API responses ([#1644](https://github.com/AztecProtocol/aztec-packages/issues/1644)) ([d1ce814](https://github.com/AztecProtocol/aztec-packages/commit/d1ce81478e8993e68257722df1fce6c9e8e0f9e8))
* **rpc:** Fixes getNodeInfo serialisation ([#1991](https://github.com/AztecProtocol/aztec-packages/issues/1991)) ([0a29fa8](https://github.com/AztecProtocol/aztec-packages/commit/0a29fa8dd95b37e490c18df2db90a7324ebe762c))
* **rpc:** Validate accounts registered in the rpc server are sound ([#1431](https://github.com/AztecProtocol/aztec-packages/issues/1431)) ([77b096b](https://github.com/AztecProtocol/aztec-packages/commit/77b096b716fa5454d23c0acbb51cc84640a464ff))
* Run e2e tests without spot ([#2081](https://github.com/AztecProtocol/aztec-packages/issues/2081)) ([f0aa3ca](https://github.com/AztecProtocol/aztec-packages/commit/f0aa3ca0de995f58ea5a18e64c18ee437b520675))
* **sandbox:** Build script for tagged commits ([#2057](https://github.com/AztecProtocol/aztec-packages/issues/2057)) ([c9d9722](https://github.com/AztecProtocol/aztec-packages/commit/c9d9722151de1e6f9a49a4cc6310e5646593ec01))
* Selector name regression ([#1800](https://github.com/AztecProtocol/aztec-packages/issues/1800)) ([a5be8bb](https://github.com/AztecProtocol/aztec-packages/commit/a5be8bb92f858d266cf96671c46343b6e1ff400a))
* Set correct version of RPC & Sandbox when deploying tagged commit ([#1914](https://github.com/AztecProtocol/aztec-packages/issues/1914)) ([898c50d](https://github.com/AztecProtocol/aztec-packages/commit/898c50d594b7515f6ca3b904d31ccf724b683ade))
* Set side effect counter on contract reads ([#1870](https://github.com/AztecProtocol/aztec-packages/issues/1870)) ([1d8881e](https://github.com/AztecProtocol/aztec-packages/commit/1d8881e4872b39195ace523432c0e34bc9081f8d)), closes [#1588](https://github.com/AztecProtocol/aztec-packages/issues/1588)
* **simulator:** Use nullifier.value in client's `pendingNullifier` set so `set.has()` works ([#1534](https://github.com/AztecProtocol/aztec-packages/issues/1534)) ([a78daf7](https://github.com/AztecProtocol/aztec-packages/commit/a78daf75e3171d9cfafecba5507d5ae215fdd0ef))
* **synchronizer:** Store most recent globals hash in the synchronizer, rather than fetching from the latest block ([#1539](https://github.com/AztecProtocol/aztec-packages/issues/1539)) ([1dd6225](https://github.com/AztecProtocol/aztec-packages/commit/1dd62256cc323831418808689496f0506d402fc4))
* **sync:** Sync latest globals within merkle tree ops ([#1612](https://github.com/AztecProtocol/aztec-packages/issues/1612)) ([03b4cf6](https://github.com/AztecProtocol/aztec-packages/commit/03b4cf67cbd4c1629c2937dfae1ea714248d6d3b))
* Truncate SRS size to the amount of points that we have downloaded ([#1862](https://github.com/AztecProtocol/aztec-packages/issues/1862)) ([0a7058c](https://github.com/AztecProtocol/aztec-packages/commit/0a7058cbda228c9baf378d69c906596e204d804f))
* Try to catch last undefined safety issues ([#2027](https://github.com/AztecProtocol/aztec-packages/issues/2027)) ([12e7486](https://github.com/AztecProtocol/aztec-packages/commit/12e7486c0750f648f51d2b43317df843a3c52bec))
* Typescript lookup of aztec.js types ([#1948](https://github.com/AztecProtocol/aztec-packages/issues/1948)) ([22901ae](https://github.com/AztecProtocol/aztec-packages/commit/22901ae8fa63b61ba1fbf4885f3940dc839b555c))
* Undefined safety in master part 5 ([#2034](https://github.com/AztecProtocol/aztec-packages/issues/2034)) ([41eccaa](https://github.com/AztecProtocol/aztec-packages/commit/41eccaa516200bd65847e1b7b736c2f2cf858960))
* Unify base64 interface between mac and linux (cherry-picked) ([#1968](https://github.com/AztecProtocol/aztec-packages/issues/1968)) ([ee24b52](https://github.com/AztecProtocol/aztec-packages/commit/ee24b52234956744d2b35b0eb0d3b5c2dcf7ed82))
* Update barretenberg bootstrap.sh for mac ([#1732](https://github.com/AztecProtocol/aztec-packages/issues/1732)) ([83a212a](https://github.com/AztecProtocol/aztec-packages/commit/83a212a6f64cca5281411bdd3c0a844b1aca38aa))
* Update bootstrap compilation order ([#1398](https://github.com/AztecProtocol/aztec-packages/issues/1398)) ([c03a6fa](https://github.com/AztecProtocol/aztec-packages/commit/c03a6faaa255b73ebe6f1a3e744df4804ad9d475))
* Update decoder block specification comment ([#1690](https://github.com/AztecProtocol/aztec-packages/issues/1690)) ([5a0a4c4](https://github.com/AztecProtocol/aztec-packages/commit/5a0a4c4cc9dcfb7d8df93746f068b36c4a4db6ae))
* Update docs search config ([#1920](https://github.com/AztecProtocol/aztec-packages/issues/1920)) ([c8764e6](https://github.com/AztecProtocol/aztec-packages/commit/c8764e6150b7d372c34ddc008be9925e5f5f6dfb))
* Update docs search keys ([#1931](https://github.com/AztecProtocol/aztec-packages/issues/1931)) ([03b200c](https://github.com/AztecProtocol/aztec-packages/commit/03b200c10da71bd4b6fa3902edb254f9f625bf8b))
* Updated CLI readme ([#2098](https://github.com/AztecProtocol/aztec-packages/issues/2098)) ([2226091](https://github.com/AztecProtocol/aztec-packages/commit/2226091e21d0aa0dbfa3bea4f95a0ea2a31a4c43)), closes [#1784](https://github.com/AztecProtocol/aztec-packages/issues/1784)
* Use COMMIT_TAG_VERSION properly in deploy_dockerhub ([#2033](https://github.com/AztecProtocol/aztec-packages/issues/2033)) ([064ddc3](https://github.com/AztecProtocol/aztec-packages/commit/064ddc3b345ac445fc9fe2385c8aee78b8fb6e47))
* Use exit, not return in retry_10 ([#1468](https://github.com/AztecProtocol/aztec-packages/issues/1468)) ([a65727a](https://github.com/AztecProtocol/aztec-packages/commit/a65727a4e67ecf2ec61b4b5370d359c114ec55ef))
* Use WARN or ERROR "tags" for warnings and errors ([#1589](https://github.com/AztecProtocol/aztec-packages/issues/1589)) ([fb80522](https://github.com/AztecProtocol/aztec-packages/commit/fb80522c45e49112797d53e3b475a58101cca131)), closes [#1607](https://github.com/AztecProtocol/aztec-packages/issues/1607)
* Used dumped state instead of fork ([#1399](https://github.com/AztecProtocol/aztec-packages/issues/1399)) ([c265e73](https://github.com/AztecProtocol/aztec-packages/commit/c265e73db0539919df6b3124ea03fef566bcc606))
* Yarn install in canary ([#1454](https://github.com/AztecProtocol/aztec-packages/issues/1454)) ([9bbe79e](https://github.com/AztecProtocol/aztec-packages/commit/9bbe79e2a4d8d7f60a3eba46bbd2e287ee568d17))


### Miscellaneous

* `AztecRPC` API using sandbox ([#1568](https://github.com/AztecProtocol/aztec-packages/issues/1568)) ([b2662db](https://github.com/AztecProtocol/aztec-packages/commit/b2662dbc45b0149b380ae3c88d058b70174266cb))
* **1074:** Remove read request data from final private kernel circuit public inputs ([#1840](https://github.com/AztecProtocol/aztec-packages/issues/1840)) ([c61557a](https://github.com/AztecProtocol/aztec-packages/commit/c61557ae926f89cead7306368197fdbe8f23dd6d))
* **1407:** Remove forwarding witnesses ([#1930](https://github.com/AztecProtocol/aztec-packages/issues/1930)) ([cc8bc8f](https://github.com/AztecProtocol/aztec-packages/commit/cc8bc8f48b175479e1c4dfbcf9b92159f096c2cf)), closes [#1407](https://github.com/AztecProtocol/aztec-packages/issues/1407)
* **1879:** Add use of PrivateKernelPublicInputs in TS whenever relevant ([#1911](https://github.com/AztecProtocol/aztec-packages/issues/1911)) ([8d5f548](https://github.com/AztecProtocol/aztec-packages/commit/8d5f548e42d627da1685820f99fc28ff5f47abbe))
* Acir tests are no longer base64 encoded ([#1854](https://github.com/AztecProtocol/aztec-packages/issues/1854)) ([7fffd16](https://github.com/AztecProtocol/aztec-packages/commit/7fffd1680d6246f64ee4d4ca965b9764c6c0ebb3))
* Add back double verify proof to test suite ([#1986](https://github.com/AztecProtocol/aztec-packages/issues/1986)) ([f8688d7](https://github.com/AztecProtocol/aztec-packages/commit/f8688d7df05abcb6c650aafb130dedb707931950))
* Add browser test to canary flow ([#1808](https://github.com/AztecProtocol/aztec-packages/issues/1808)) ([7f4fa43](https://github.com/AztecProtocol/aztec-packages/commit/7f4fa438bf2f4966338e3e53ece7c1d01e8dd054))
* Add CLI test to canary flow ([#1918](https://github.com/AztecProtocol/aztec-packages/issues/1918)) ([cc68958](https://github.com/AztecProtocol/aztec-packages/commit/cc689585a845ce3c20ea9714ca744f4aa8837462)), closes [#1903](https://github.com/AztecProtocol/aztec-packages/issues/1903)
* Add FunctionData.fromAbi for QoL  ([#1333](https://github.com/AztecProtocol/aztec-packages/issues/1333)) ([6f5fc3b](https://github.com/AztecProtocol/aztec-packages/commit/6f5fc3bbd54f633582a69d8104327bd405b1e3c4))
* Add rebuild pattern for bb-bin-tests to rebuild when ts folder is changed and add target folder for bb-bin-test ([#1640](https://github.com/AztecProtocol/aztec-packages/issues/1640)) ([b3ee3d9](https://github.com/AztecProtocol/aztec-packages/commit/b3ee3d979172c9d4eae3f9090d0fbbc05fc5a613))
* Add safemath noir testing ([#1967](https://github.com/AztecProtocol/aztec-packages/issues/1967)) ([cb1f1ec](https://github.com/AztecProtocol/aztec-packages/commit/cb1f1ece1fd050b00ad8cbe9086e76383f9e6377))
* Add tests that check ordering of public state updates ([#1661](https://github.com/AztecProtocol/aztec-packages/issues/1661)) ([5b9aedd](https://github.com/AztecProtocol/aztec-packages/commit/5b9aeddd4a1bffcf9015786819dd3f6c1ff66fb4))
* Add todo for using generator indices in note commitment and nullifier computation. ([#1762](https://github.com/AztecProtocol/aztec-packages/issues/1762)) ([2db6728](https://github.com/AztecProtocol/aztec-packages/commit/2db6728fcaf75ce8c98d821b65695543bb0c82a2))
* Another pedantic change to public state naming ([#1359](https://github.com/AztecProtocol/aztec-packages/issues/1359)) ([cb77440](https://github.com/AztecProtocol/aztec-packages/commit/cb774405e89c71a622e32b51032aa761cd767959))
* Aztec RPC interface cleanup ([#1423](https://github.com/AztecProtocol/aztec-packages/issues/1423)) ([1a6168a](https://github.com/AztecProtocol/aztec-packages/commit/1a6168abc9cdc092cf7c9843191194c9b90adae7))
* **Aztec.nr:** Remove implicit imports ([#1901](https://github.com/AztecProtocol/aztec-packages/issues/1901)) ([c7d5190](https://github.com/AztecProtocol/aztec-packages/commit/c7d5190e48771c334bfa7062c361bcd623faa318))
* **Aztec.nr:** Remove the open keyword from public functions ([#1917](https://github.com/AztecProtocol/aztec-packages/issues/1917)) ([4db8603](https://github.com/AztecProtocol/aztec-packages/commit/4db8603a4ee293c64a67be5ba74072bd654c7ec5))
* **bb:** Refactor bb CLI interface ([#1672](https://github.com/AztecProtocol/aztec-packages/issues/1672)) ([a5bf6e0](https://github.com/AztecProtocol/aztec-packages/commit/a5bf6e008b19127bf15c8b12a5a699182b7ff4e7)), closes [#1671](https://github.com/AztecProtocol/aztec-packages/issues/1671)
* **bb:** Upgrade to clang16 for Linux builds ([#1705](https://github.com/AztecProtocol/aztec-packages/issues/1705)) ([feb53aa](https://github.com/AztecProtocol/aztec-packages/commit/feb53aa396f03e49c95f07b9e9635498a89d5807))
* **blocks tree:** Remove historic roots trees ([#1355](https://github.com/AztecProtocol/aztec-packages/issues/1355)) ([ac935e1](https://github.com/AztecProtocol/aztec-packages/commit/ac935e1ea17f89c1dc6ca7d11a332a82bdc85d97))
* Build-system submodule=&gt;subrepo ([#1378](https://github.com/AztecProtocol/aztec-packages/issues/1378)) ([29ab491](https://github.com/AztecProtocol/aztec-packages/commit/29ab49130812918c51852b32b207f3e7cf633d66))
* **build:** Fixed manifest ([#2122](https://github.com/AztecProtocol/aztec-packages/issues/2122)) ([91faa66](https://github.com/AztecProtocol/aztec-packages/commit/91faa668650b98306813e64e9ebe3064bd7a221e))
* **build:** Unify barretenberg releases with aztec-packages ([#2120](https://github.com/AztecProtocol/aztec-packages/issues/2120)) ([82823d8](https://github.com/AztecProtocol/aztec-packages/commit/82823d8cd6882b191a7b363aa40344f66dfd7af7))
* **ci:** Build docs on every pr ([#1955](https://github.com/AztecProtocol/aztec-packages/issues/1955)) ([c200bc5](https://github.com/AztecProtocol/aztec-packages/commit/c200bc5337da9d6122a2545fceeada98a28d7077))
* **ci:** Clean up stale image tags ([#1818](https://github.com/AztecProtocol/aztec-packages/issues/1818)) ([3c8b7b8](https://github.com/AztecProtocol/aztec-packages/commit/3c8b7b84efe938e32c938bbcd744a335ffc50f49))
* **ci:** Deploy sandbox dependencies to npm ([#1593](https://github.com/AztecProtocol/aztec-packages/issues/1593)) ([d90c460](https://github.com/AztecProtocol/aztec-packages/commit/d90c460d898724d742dbbf8a98def8de9db10ace)), closes [#1536](https://github.com/AztecProtocol/aztec-packages/issues/1536)
* **ci:** Fix output name in release please workflow ([#1858](https://github.com/AztecProtocol/aztec-packages/issues/1858)) ([857821f](https://github.com/AztecProtocol/aztec-packages/commit/857821fa1923aa013fe9470f12067208d5c494d1))
* **circuits:** - remove dead code from cbind of private kernel circuit ([#2088](https://github.com/AztecProtocol/aztec-packages/issues/2088)) ([43dc9d7](https://github.com/AztecProtocol/aztec-packages/commit/43dc9d7500fa3d11a0b557b8fc82da4495c4e605))
* **circuits:** - use msgpack for cbind routines of native private kernel circuits ([#1938](https://github.com/AztecProtocol/aztec-packages/issues/1938)) ([3dc5c07](https://github.com/AztecProtocol/aztec-packages/commit/3dc5c07358d99786df8809f46638fdb04b33a6c2))
* **circuits:** Remove dead code in cbind.cpp for public kernel ([#2094](https://github.com/AztecProtocol/aztec-packages/issues/2094)) ([861f960](https://github.com/AztecProtocol/aztec-packages/commit/861f960524436796263d9f79fa06a38d0e62ae84))
* **circuits:** Rename function to validate private call hash in PKC (it pops too) ([#1418](https://github.com/AztecProtocol/aztec-packages/issues/1418)) ([a76496f](https://github.com/AztecProtocol/aztec-packages/commit/a76496facb87d62f5032759cf930c885df1d5cc7))
* **ci:** Set up nightly barretenberg releases ([#1761](https://github.com/AztecProtocol/aztec-packages/issues/1761)) ([e0078da](https://github.com/AztecProtocol/aztec-packages/commit/e0078dabfcd9e006c2a489c7142ab141d5d81b80))
* **ci:** Update acir tests to reflect compilation based off of package name ([#1405](https://github.com/AztecProtocol/aztec-packages/issues/1405)) ([bb38c7a](https://github.com/AztecProtocol/aztec-packages/commit/bb38c7aef6f630aa34d3abb81c6fd1dc8e4f9884))
* **ci:** Update build artifacts for recursion bin-test and enable bin-test ([#1326](https://github.com/AztecProtocol/aztec-packages/issues/1326)) ([48aa541](https://github.com/AztecProtocol/aztec-packages/commit/48aa5414c9b2c99175b304f4258d0d08ffbd8c7c))
* **ci:** Updated release please config ([#1775](https://github.com/AztecProtocol/aztec-packages/issues/1775)) ([0085e8b](https://github.com/AztecProtocol/aztec-packages/commit/0085e8b17efc36256974f82525530c39ed182639))
* **ci:** Updated release please configuration ([#1787](https://github.com/AztecProtocol/aztec-packages/issues/1787)) ([6eb2f7a](https://github.com/AztecProtocol/aztec-packages/commit/6eb2f7abc40bae88ebeec546ad9f8f2c7d810a24))
* CLI tests ([#1786](https://github.com/AztecProtocol/aztec-packages/issues/1786)) ([2987065](https://github.com/AztecProtocol/aztec-packages/commit/298706557a8f2b73a87dfb10c81626ebf127cadb)), closes [#1450](https://github.com/AztecProtocol/aztec-packages/issues/1450)
* Compile minimal WASM binary needed for blackbox functions ([#1824](https://github.com/AztecProtocol/aztec-packages/issues/1824)) ([76a30b8](https://github.com/AztecProtocol/aztec-packages/commit/76a30b8b5b5e765a14fe7d896d8890897cad7756))
* **compiler:** Remove wasm option from noir compiler ([#1628](https://github.com/AztecProtocol/aztec-packages/issues/1628)) ([c552322](https://github.com/AztecProtocol/aztec-packages/commit/c552322c1669b53016bea66beab02aded9c7c29c))
* Conservatively raise the minimum supported clang version in CMakeList ([#2023](https://github.com/AztecProtocol/aztec-packages/issues/2023)) ([f49c416](https://github.com/AztecProtocol/aztec-packages/commit/f49c4164387d307f8a86e93faff3eb96d7c99e36))
* Consistent block number method naming ([#1751](https://github.com/AztecProtocol/aztec-packages/issues/1751)) ([df1afe2](https://github.com/AztecProtocol/aztec-packages/commit/df1afe255d3095a9b2851b47480801c06d116eed))
* **constants:** Bump number of private reads and writes ([#2062](https://github.com/AztecProtocol/aztec-packages/issues/2062)) ([ab6c6b1](https://github.com/AztecProtocol/aztec-packages/commit/ab6c6b1cefdc1dd1da6e1198f99a211b31e73d85))
* **contracts:** Rename Schnorr multi key account to just Schnorr account ([#1447](https://github.com/AztecProtocol/aztec-packages/issues/1447)) ([3afd853](https://github.com/AztecProtocol/aztec-packages/commit/3afd853074be02ebf0a8d1f6187e49505513017e))
* **contracts:** Use autogenerated Noir interfaces where possible ([#2073](https://github.com/AztecProtocol/aztec-packages/issues/2073)) ([bd6368b](https://github.com/AztecProtocol/aztec-packages/commit/bd6368bd16159aad88906496cb9d6270e483a26e)), closes [#1604](https://github.com/AztecProtocol/aztec-packages/issues/1604)
* Create fixtures folder in E2E ([#1419](https://github.com/AztecProtocol/aztec-packages/issues/1419)) ([b8972b4](https://github.com/AztecProtocol/aztec-packages/commit/b8972b4838df02004e8c2b40da446a484e1c0df4))
* **deps:** Remove deprecated multiaddr dependency ([#1631](https://github.com/AztecProtocol/aztec-packages/issues/1631)) ([e72d226](https://github.com/AztecProtocol/aztec-packages/commit/e72d2261a5cbea536c591304d7e3feeed33c5612))
* Disable fft functions for polynomials instantiated on Grumpkin ([#1471](https://github.com/AztecProtocol/aztec-packages/issues/1471)) ([f09909a](https://github.com/AztecProtocol/aztec-packages/commit/f09909ad13d77b21654d90894c018e1b39896105))
* **docs:** API docs stucture ([#2014](https://github.com/AztecProtocol/aztec-packages/issues/2014)) ([9aab9dd](https://github.com/AztecProtocol/aztec-packages/commit/9aab9ddefac63d35ebc356afed573af268896b35))
* **e2e:** Initial e2e test for CLI ([#1576](https://github.com/AztecProtocol/aztec-packages/issues/1576)) ([c2c30da](https://github.com/AztecProtocol/aztec-packages/commit/c2c30da82233a9e8eaae364d19711e4f3596d7d2))
* **e2e:** Trigger public call stack ordering error ([#1637](https://github.com/AztecProtocol/aztec-packages/issues/1637)) ([5ef2a83](https://github.com/AztecProtocol/aztec-packages/commit/5ef2a830b33875bacebe7b4edb269cd15522879f)), closes [#1615](https://github.com/AztecProtocol/aztec-packages/issues/1615)
* Enable project-specific releases for dockerhub too ([#1721](https://github.com/AztecProtocol/aztec-packages/issues/1721)) ([5d2c082](https://github.com/AztecProtocol/aztec-packages/commit/5d2c0824eedb748ca3e2beaa8589410a21ba6e57))
* Enable project-specific tagged releases ([#1425](https://github.com/AztecProtocol/aztec-packages/issues/1425)) ([28cbe7b](https://github.com/AztecProtocol/aztec-packages/commit/28cbe7b30cd5654b2e03d3288f70cfb8a4935fc3))
* Enforce PR titles follow conventional commit specification ([#1706](https://github.com/AztecProtocol/aztec-packages/issues/1706)) ([eeb38ac](https://github.com/AztecProtocol/aztec-packages/commit/eeb38ac700048b9e760e02ca17d8963d2828944c))
* Fix acir-tests ([#1435](https://github.com/AztecProtocol/aztec-packages/issues/1435)) ([4b9b3fe](https://github.com/AztecProtocol/aztec-packages/commit/4b9b3fea10671fee38a55852d283d8489d7965a6))
* Fix dirty merge ([#1574](https://github.com/AztecProtocol/aztec-packages/issues/1574)) ([58dc9bf](https://github.com/AztecProtocol/aztec-packages/commit/58dc9bffa6c8f225640b7f2a2e7c18105cac8592))
* Fix typo ([#1681](https://github.com/AztecProtocol/aztec-packages/issues/1681)) ([7ac25ea](https://github.com/AztecProtocol/aztec-packages/commit/7ac25ea060bdbf7b04ab5ff9defd4f24835f11df))
* Fixed linter errors for `ecc`, `numeric` and `common` modules ([#1714](https://github.com/AztecProtocol/aztec-packages/issues/1714)) ([026273b](https://github.com/AztecProtocol/aztec-packages/commit/026273b42d8c41de9bc4a86f898162cbbb3ad35f))
* Make stdlib bn254 naming match native version ([#1560](https://github.com/AztecProtocol/aztec-packages/issues/1560)) ([347a38a](https://github.com/AztecProtocol/aztec-packages/commit/347a38a54e0ea7f6da1b45a8640b8506c3712bb1))
* Manually resolves barretenberg conflicts ([#1455](https://github.com/AztecProtocol/aztec-packages/issues/1455)) ([b137f85](https://github.com/AztecProtocol/aztec-packages/commit/b137f85689ee941d8efe04c1d9e596d8465fc7e1))
* **master:** Release 0.1.0-alpha45 ([#1774](https://github.com/AztecProtocol/aztec-packages/issues/1774)) ([e910929](https://github.com/AztecProtocol/aztec-packages/commit/e9109297eb801d5e0bb1ee5ca8251af01988ce44))
* **master:** Release 0.1.0-alpha46 ([#1777](https://github.com/AztecProtocol/aztec-packages/issues/1777)) ([13ab91d](https://github.com/AztecProtocol/aztec-packages/commit/13ab91d82214646ff8acee6c0ac8ab83ea5a219b))
* **master:** Release 0.1.0-alpha47 ([#1788](https://github.com/AztecProtocol/aztec-packages/issues/1788)) ([1970651](https://github.com/AztecProtocol/aztec-packages/commit/1970651e641a323c1747d0dc64a81f5ac677c840))
* **master:** Release 0.1.0-alpha48 ([#1804](https://github.com/AztecProtocol/aztec-packages/issues/1804)) ([e89cd26](https://github.com/AztecProtocol/aztec-packages/commit/e89cd267d2cf2c0919a602ec4dc5d5456f95d5d4))
* **master:** Release 0.1.0-alpha49 ([#1882](https://github.com/AztecProtocol/aztec-packages/issues/1882)) ([685e3a9](https://github.com/AztecProtocol/aztec-packages/commit/685e3a95fc1054c76342119d7ec27053edf038d1))
* **master:** Release 0.1.0-alpha50 ([#1900](https://github.com/AztecProtocol/aztec-packages/issues/1900)) ([8135fee](https://github.com/AztecProtocol/aztec-packages/commit/8135feef4ed2f394ec56461f8e2bd2ee77f97cc0))
* **master:** Release 0.1.0-alpha51 ([#2018](https://github.com/AztecProtocol/aztec-packages/issues/2018)) ([c5d95c8](https://github.com/AztecProtocol/aztec-packages/commit/c5d95c8ee5b5fb1f0d5b2c88ea8fcf24fdb466b8))
* **master:** Release 0.1.0-alpha52 ([#2020](https://github.com/AztecProtocol/aztec-packages/issues/2020)) ([0c6dd60](https://github.com/AztecProtocol/aztec-packages/commit/0c6dd60f62f0ebc425c36af5631a6905aeeeaf47))
* **master:** Release 0.1.0-alpha53 ([#2026](https://github.com/AztecProtocol/aztec-packages/issues/2026)) ([1990779](https://github.com/AztecProtocol/aztec-packages/commit/1990779a7ea30b7f90569fcb7b00a4a7b5a1d088))
* **master:** Release 0.1.0-alpha54 ([#2028](https://github.com/AztecProtocol/aztec-packages/issues/2028)) ([a0ccd4a](https://github.com/AztecProtocol/aztec-packages/commit/a0ccd4a1cec87121ff24e3b4e50c15030fedd5ff))
* **master:** Release 0.1.0-alpha55 ([#2031](https://github.com/AztecProtocol/aztec-packages/issues/2031)) ([4c9a438](https://github.com/AztecProtocol/aztec-packages/commit/4c9a438f5a062a32198bad3a008a1ea03555b1a8))
* **master:** Release 0.1.0-alpha56 ([#2032](https://github.com/AztecProtocol/aztec-packages/issues/2032)) ([7cac648](https://github.com/AztecProtocol/aztec-packages/commit/7cac64887994d7873704e2cf27e098a013884014))
* **master:** Release 0.1.0-alpha57 ([#2035](https://github.com/AztecProtocol/aztec-packages/issues/2035)) ([6b93483](https://github.com/AztecProtocol/aztec-packages/commit/6b93483312a1a65ddc941579c9322732c2774175))
* **master:** Release 0.1.0-alpha58 ([#2037](https://github.com/AztecProtocol/aztec-packages/issues/2037)) ([b652ca4](https://github.com/AztecProtocol/aztec-packages/commit/b652ca48f5438546ead9c7f5c9f612574e922fe3))
* **master:** Release 0.1.0-alpha59 ([#2038](https://github.com/AztecProtocol/aztec-packages/issues/2038)) ([3f833c7](https://github.com/AztecProtocol/aztec-packages/commit/3f833c7e2bbb1c121d05d56ba4aebc3e700fc291))
* **master:** Release 0.1.0-alpha60 ([#2040](https://github.com/AztecProtocol/aztec-packages/issues/2040)) ([fbd8b67](https://github.com/AztecProtocol/aztec-packages/commit/fbd8b672dbbdb46c5c484e5d06f2ac955b5db97f))
* **master:** Release 0.1.0-alpha61 ([#2059](https://github.com/AztecProtocol/aztec-packages/issues/2059)) ([5324750](https://github.com/AztecProtocol/aztec-packages/commit/5324750404cf4fc37fd656009577ae80f75d58bb))
* **master:** Release 0.1.0-alpha62 ([#2060](https://github.com/AztecProtocol/aztec-packages/issues/2060)) ([28a877b](https://github.com/AztecProtocol/aztec-packages/commit/28a877bc31012a748a0ab923fa6367271f5b6a75))
* **master:** Release 0.1.0-alpha63 ([#2078](https://github.com/AztecProtocol/aztec-packages/issues/2078)) ([a5f2852](https://github.com/AztecProtocol/aztec-packages/commit/a5f2852966457b9e11012118f9772118682b12e1))
* Merge bb release-please ([#2080](https://github.com/AztecProtocol/aztec-packages/issues/2080)) ([e89b043](https://github.com/AztecProtocol/aztec-packages/commit/e89b04358acbf6f43b72c346406cd97c0fa26af2))
* Move jsdocs to interfaces ([#1356](https://github.com/AztecProtocol/aztec-packages/issues/1356)) ([7f7519d](https://github.com/AztecProtocol/aztec-packages/commit/7f7519d131409d87dfc8fce55a73e882bcf5f015))
* Move storage into main.nr. ([#2068](https://github.com/AztecProtocol/aztec-packages/issues/2068)) ([2c2d72b](https://github.com/AztecProtocol/aztec-packages/commit/2c2d72b7799b24273e498805ecf4c36d69f08d7d))
* **noir-lib:** Add unit tests for context utility functions ([#1481](https://github.com/AztecProtocol/aztec-packages/issues/1481)) ([1d2c5d4](https://github.com/AztecProtocol/aztec-packages/commit/1d2c5d46174548bac715298e26598f126d1a02c2))
* **noir-libs:** TransparentNote rework ([#1412](https://github.com/AztecProtocol/aztec-packages/issues/1412)) ([22fb8fe](https://github.com/AztecProtocol/aztec-packages/commit/22fb8fe0281379bf23836e1be33766b4f38a1813))
* **noir:** Silence warnings ([#1544](https://github.com/AztecProtocol/aztec-packages/issues/1544)) ([ac1dc4b](https://github.com/AztecProtocol/aztec-packages/commit/ac1dc4b6ca39c15a1846fb011116810b39e4fa4a))
* Not breaking note processing on missing hash and nullifier func ([#1364](https://github.com/AztecProtocol/aztec-packages/issues/1364)) ([861db2a](https://github.com/AztecProtocol/aztec-packages/commit/861db2a6bdb0b94d8722539b1159bb8b903b7d97))
* **p2p:** Updated libp2p dependencies ([#1792](https://github.com/AztecProtocol/aztec-packages/issues/1792)) ([79df831](https://github.com/AztecProtocol/aztec-packages/commit/79df83134e15655dc3a5ed9dae00dc52a3d40681))
* Protogalaxy relations ([#1897](https://github.com/AztecProtocol/aztec-packages/issues/1897)) ([35407e2](https://github.com/AztecProtocol/aztec-packages/commit/35407e25081744702ec35efe3f95aa0137fe0ebb))
* Re-enabling pubkey check ([#1720](https://github.com/AztecProtocol/aztec-packages/issues/1720)) ([5385b18](https://github.com/AztecProtocol/aztec-packages/commit/5385b1894aed030448a8d6d3e317072bf9924538))
* Reduce max circuit size in bb binary ([#1942](https://github.com/AztecProtocol/aztec-packages/issues/1942)) ([c61439b](https://github.com/AztecProtocol/aztec-packages/commit/c61439b316829563c93bbfcb78b799bdc105ff71))
* Reenable and refactor nested calls e2e tests ([#1868](https://github.com/AztecProtocol/aztec-packages/issues/1868)) ([570de80](https://github.com/AztecProtocol/aztec-packages/commit/570de803376de4af6a1824b7a3c95129c98e2fa0)), closes [#1587](https://github.com/AztecProtocol/aztec-packages/issues/1587)
* Refactor Cli interface to be more unix-like ([#1833](https://github.com/AztecProtocol/aztec-packages/issues/1833)) ([28d722e](https://github.com/AztecProtocol/aztec-packages/commit/28d722ef965d907b7b7820ccdd7ee0afc97c88fa))
* Refactor hash.hpp to use const& ([#1578](https://github.com/AztecProtocol/aztec-packages/issues/1578)) ([4c329af](https://github.com/AztecProtocol/aztec-packages/commit/4c329af59e5665ce15d8e0465165c3993c4801bc))
* Reference noir master for acir tests ([#1969](https://github.com/AztecProtocol/aztec-packages/issues/1969)) ([86b72e1](https://github.com/AztecProtocol/aztec-packages/commit/86b72e1e8da29a0335e40c6de4c46538d8138f2f))
* Remove debug output from `run_acir_tests` script ([#1970](https://github.com/AztecProtocol/aztec-packages/issues/1970)) ([74c83c5](https://github.com/AztecProtocol/aztec-packages/commit/74c83c5e1436f391eef435926c2da1d508d67713))
* Remove individual historic roots from privateCircuitPublicInputs ([#1571](https://github.com/AztecProtocol/aztec-packages/issues/1571)) ([088cbe5](https://github.com/AztecProtocol/aztec-packages/commit/088cbe5190d3f1a547844a12d4492c901c7b1116))
* Remove Params concept ([#1541](https://github.com/AztecProtocol/aztec-packages/issues/1541)) ([f4bd85e](https://github.com/AztecProtocol/aztec-packages/commit/f4bd85efc286825b6d39b140630ded408e7b1eda))
* Removed `getPreimagesAt` ([#1517](https://github.com/AztecProtocol/aztec-packages/issues/1517)) ([7e14e7b](https://github.com/AztecProtocol/aztec-packages/commit/7e14e7bbea7d092242ac2e6ae03086fe5b9a9ebf)), closes [#1502](https://github.com/AztecProtocol/aztec-packages/issues/1502)
* Rename public state serialisation interface structs for clarity ([#1338](https://github.com/AztecProtocol/aztec-packages/issues/1338)) ([cb2d210](https://github.com/AztecProtocol/aztec-packages/commit/cb2d210b6b8d065b2468cf678bb4fb53f883f14d))
* Renamed take to limit ([#1361](https://github.com/AztecProtocol/aztec-packages/issues/1361)) ([ba9d00b](https://github.com/AztecProtocol/aztec-packages/commit/ba9d00b12f231722b5053b5641a949a825f0a4a8)), closes [#1231](https://github.com/AztecProtocol/aztec-packages/issues/1231)
* Renaming storage getters ([#1348](https://github.com/AztecProtocol/aztec-packages/issues/1348)) ([cb5ce9e](https://github.com/AztecProtocol/aztec-packages/commit/cb5ce9e1295e7d7b6572a052f4fe39f0b5d29631))
* Required option in aztec-cli ([#1584](https://github.com/AztecProtocol/aztec-packages/issues/1584)) ([f287416](https://github.com/AztecProtocol/aztec-packages/commit/f2874165d0748e4c78e5057482907e483bb13cad))
* Restructure documentation ([#1437](https://github.com/AztecProtocol/aztec-packages/issues/1437)) ([da74f58](https://github.com/AztecProtocol/aztec-packages/commit/da74f580e83e7b220573354203e93d756175353d))
* Rework nonces ([#1210](https://github.com/AztecProtocol/aztec-packages/issues/1210)) ([#1331](https://github.com/AztecProtocol/aztec-packages/issues/1331)) ([665cb75](https://github.com/AztecProtocol/aztec-packages/commit/665cb753f50f003ccd21935755aa1f08bfb78deb))
* Sandbox logging tweaks ([#1797](https://github.com/AztecProtocol/aztec-packages/issues/1797)) ([0e3914e](https://github.com/AztecProtocol/aztec-packages/commit/0e3914ed6ad63062add1cc08f6ea85646c068f8a))
* **scripts:** Convenience script to update local generated artifacts ([#1349](https://github.com/AztecProtocol/aztec-packages/issues/1349)) ([317981a](https://github.com/AztecProtocol/aztec-packages/commit/317981a13c9faf791a2760a07e9808a8474ecae2))
* Simplified AztecRpc.registerAccount function ([#1729](https://github.com/AztecProtocol/aztec-packages/issues/1729)) ([8e5f828](https://github.com/AztecProtocol/aztec-packages/commit/8e5f828c0aff0602c49575139883c8abc3cb6e91))
* **simulator:** Initialize ACVM's SimulatedBackend separately (setup pedersen init only happens once) ([#1596](https://github.com/AztecProtocol/aztec-packages/issues/1596)) ([1a260ed](https://github.com/AztecProtocol/aztec-packages/commit/1a260ede0729b1f70b90e06c2e6588bcb5eb9fc3))
* Split out yarn-project bootstrap.sh ([#1790](https://github.com/AztecProtocol/aztec-packages/issues/1790)) ([1788fe6](https://github.com/AztecProtocol/aztec-packages/commit/1788fe6259f5e7fd191929b27996a7342e3f13e5))
* Split SumcheckRound into Prover/Verifier classes ([#1373](https://github.com/AztecProtocol/aztec-packages/issues/1373)) ([8b1d48a](https://github.com/AztecProtocol/aztec-packages/commit/8b1d48a52c41f4f6cf436b481823f59582611b81))
* Start sandbox as a bin from npm package ([#1595](https://github.com/AztecProtocol/aztec-packages/issues/1595)) ([3f793b9](https://github.com/AztecProtocol/aztec-packages/commit/3f793b96674a677472241259d92d352d00f8a6ef))
* Storing `&mut context` in state vars ([#1926](https://github.com/AztecProtocol/aztec-packages/issues/1926)) ([89a7a3f](https://github.com/AztecProtocol/aztec-packages/commit/89a7a3ff22ebc469fe1b58d929af5ef162514c17)), closes [#1805](https://github.com/AztecProtocol/aztec-packages/issues/1805)
* Sync bb master ([#1710](https://github.com/AztecProtocol/aztec-packages/issues/1710)) ([0039c4f](https://github.com/AztecProtocol/aztec-packages/commit/0039c4fdf7c713d9f375d6abda15353325e38d56))
* Sync bb master ([#1713](https://github.com/AztecProtocol/aztec-packages/issues/1713)) ([ec5241c](https://github.com/AztecProtocol/aztec-packages/commit/ec5241c34a9c1214ff66a20133ad6cc5e4081d77))
* Sync bb master ([#1776](https://github.com/AztecProtocol/aztec-packages/issues/1776)) ([7c6fb15](https://github.com/AztecProtocol/aztec-packages/commit/7c6fb15979b48d4d4d5eb5a1ea83d3c0d0ee3b5e))
* Sync bb master ([#1842](https://github.com/AztecProtocol/aztec-packages/issues/1842)) ([2c1ff72](https://github.com/AztecProtocol/aztec-packages/commit/2c1ff729fd1994270644a96da5a954ce2ec72382))
* Sync bb master ([#1852](https://github.com/AztecProtocol/aztec-packages/issues/1852)) ([f979878](https://github.com/AztecProtocol/aztec-packages/commit/f979878cb84dd1b0506cedd59e9df1bb65a99b0a))
* Sync bb master ([#1866](https://github.com/AztecProtocol/aztec-packages/issues/1866)) ([e681a49](https://github.com/AztecProtocol/aztec-packages/commit/e681a4901ee51cdd133c126d299881be6fad3680))
* Sync bb master ([#1947](https://github.com/AztecProtocol/aztec-packages/issues/1947)) ([eed58e1](https://github.com/AztecProtocol/aztec-packages/commit/eed58e157c2740043ad6f53c76b13ba9924c5d93))
* **tests:** Use account class for e2e browser tests ([#1446](https://github.com/AztecProtocol/aztec-packages/issues/1446)) ([ff7ad30](https://github.com/AztecProtocol/aztec-packages/commit/ff7ad3030cc786ceb8525fec488555d42343a02f))
* **tests:** Use new account class in e2e tests ([#1433](https://github.com/AztecProtocol/aztec-packages/issues/1433)) ([fe41757](https://github.com/AztecProtocol/aztec-packages/commit/fe4175759b4c311982026cd4c22ecce699f385a5))
* Typescript script names should be consistent ([#1843](https://github.com/AztecProtocol/aztec-packages/issues/1843)) ([eff8fe7](https://github.com/AztecProtocol/aztec-packages/commit/eff8fe7ea9f2674383b7b8ea1232be49626fc595))
* Update bootstrap.sh in Barretenberg to check for clang 16 ([#1717](https://github.com/AztecProtocol/aztec-packages/issues/1717)) ([87815d7](https://github.com/AztecProtocol/aztec-packages/commit/87815d7e0c6182973d98155c23d2f60b7c66314c))
* Update formatting ([#1874](https://github.com/AztecProtocol/aztec-packages/issues/1874)) ([fb973ca](https://github.com/AztecProtocol/aztec-packages/commit/fb973caeabc2d10daaf052046987e54f563b7e4b))
* Update function selector computation ([#2001](https://github.com/AztecProtocol/aztec-packages/issues/2001)) ([e07ea1a](https://github.com/AztecProtocol/aztec-packages/commit/e07ea1a887484f3a1a2ba8b5328af5abf6ccc6a2))
* Update noir readme alter noir bootstrap to always install tagged version ([#1563](https://github.com/AztecProtocol/aztec-packages/issues/1563)) ([bfc79c2](https://github.com/AztecProtocol/aztec-packages/commit/bfc79c268ff26fec20997e6f7227625b60dc12bd))
* Update pull request template ([#1379](https://github.com/AztecProtocol/aztec-packages/issues/1379)) ([a463dff](https://github.com/AztecProtocol/aztec-packages/commit/a463dffbc2df23dbdbeacc14a47f839906d4b29b))
* Update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/aztec-packages/issues/1925)) ([e728304](https://github.com/AztecProtocol/aztec-packages/commit/e72830468362f2ea26b3f830b7e056b096f56d6a))
* Update to acvm 0.24.1 ([#1978](https://github.com/AztecProtocol/aztec-packages/issues/1978)) ([31c0a02](https://github.com/AztecProtocol/aztec-packages/commit/31c0a0219330bce94a16dea9833fd900e61d93b4))
* Updating docs to clang16 ([#1875](https://github.com/AztecProtocol/aztec-packages/issues/1875)) ([a248dae](https://github.com/AztecProtocol/aztec-packages/commit/a248dae54af9cb7ca64b2a7780a4b90e3848a69b))
* Use 2^19 as `MAX_CIRCUIT_SIZE` for NodeJS CLI ([#1834](https://github.com/AztecProtocol/aztec-packages/issues/1834)) ([c573282](https://github.com/AztecProtocol/aztec-packages/commit/c573282fd59e44df70ae125f68281ebb67b7453d))
* Use context instead of custom oracles for public functions ([#1754](https://github.com/AztecProtocol/aztec-packages/issues/1754)) ([46de77a](https://github.com/AztecProtocol/aztec-packages/commit/46de77ad3e5e91b9276146410381c69ccba1ae2b))


### Documentation

* Account contract tutorial ([#1772](https://github.com/AztecProtocol/aztec-packages/issues/1772)) ([0faefba](https://github.com/AztecProtocol/aztec-packages/commit/0faefba283a7c654c0771ba8f15d5bb6346282ab))
* Compile guide ([#1575](https://github.com/AztecProtocol/aztec-packages/issues/1575)) ([d93fa96](https://github.com/AztecProtocol/aztec-packages/commit/d93fa96e6c1229a7c1f3dbb583f49d27378d8603)), closes [#1569](https://github.com/AztecProtocol/aztec-packages/issues/1569)
* Convert quick start guides into e2e tests ([#1726](https://github.com/AztecProtocol/aztec-packages/issues/1726)) ([802a678](https://github.com/AztecProtocol/aztec-packages/commit/802a678e3dd19339cd88b105a0ce341026b58054)), closes [#1564](https://github.com/AztecProtocol/aztec-packages/issues/1564)
* Deploy command fix ([#1634](https://github.com/AztecProtocol/aztec-packages/issues/1634)) ([a0a43d6](https://github.com/AztecProtocol/aztec-packages/commit/a0a43d68189896b8d05ca92f1cecc77adc5ad6be))
* Deploying contracs fixes ([#1633](https://github.com/AztecProtocol/aztec-packages/issues/1633)) ([5036b31](https://github.com/AztecProtocol/aztec-packages/commit/5036b3140bb9d2dc5cc130c3760049ff40bb987f))
* Deploying contracts using `aztec-cli` ([#1592](https://github.com/AztecProtocol/aztec-packages/issues/1592)) ([b43d7a0](https://github.com/AztecProtocol/aztec-packages/commit/b43d7a008dd6672df67090390432893b597bcd62))
* Derivation is not yet implemented for keys ([#1632](https://github.com/AztecProtocol/aztec-packages/issues/1632)) ([881bc71](https://github.com/AztecProtocol/aztec-packages/commit/881bc715d3d69de03bb6413d671f8a4e1cc7a5d6))
* Developer/wallet-providers/keys ([#1271](https://github.com/AztecProtocol/aztec-packages/issues/1271)) ([d70c45b](https://github.com/AztecProtocol/aztec-packages/commit/d70c45b9a27189258daf767f2860bfc3894783a1))
* Events ([#1768](https://github.com/AztecProtocol/aztec-packages/issues/1768)) ([5a38cea](https://github.com/AztecProtocol/aztec-packages/commit/5a38cea3f7c1567a8eea3d6c2c58cad6f79b05f2)), closes [#1756](https://github.com/AztecProtocol/aztec-packages/issues/1756)
* Including "real" code in keys docs ([#1767](https://github.com/AztecProtocol/aztec-packages/issues/1767)) ([cd9cadb](https://github.com/AztecProtocol/aztec-packages/commit/cd9cadbfb6b0311c381586799588a5f64df98f29))
* **keys:** Complete addresses are now broadcast ([#1975](https://github.com/AztecProtocol/aztec-packages/issues/1975)) ([92068ad](https://github.com/AztecProtocol/aztec-packages/commit/92068ad4249b2a20a4c83d82b82517ccdcbfe7f9)), closes [#1936](https://github.com/AztecProtocol/aztec-packages/issues/1936)
* Limitations, privacy, roadmap ([#1759](https://github.com/AztecProtocol/aztec-packages/issues/1759)) ([0cdb27a](https://github.com/AztecProtocol/aztec-packages/commit/0cdb27af8359b61b4a1f51a829ddfc4995ec1d30))
* **limitations:** Limitations on ordering and logs of chopped notes ([#2085](https://github.com/AztecProtocol/aztec-packages/issues/2085)) ([315ad3d](https://github.com/AztecProtocol/aztec-packages/commit/315ad3d58eeb467361848a1e70fd32f3074b35d5)), closes [#1652](https://github.com/AztecProtocol/aztec-packages/issues/1652)
* Link to local ethereum nodes in testing guide ([#2061](https://github.com/AztecProtocol/aztec-packages/issues/2061)) ([e29148b](https://github.com/AztecProtocol/aztec-packages/commit/e29148b379a435a9fefd846cc5fe78af6be7021d))
* Lists of questions to be addressed ([#1414](https://github.com/AztecProtocol/aztec-packages/issues/1414)) ([64bf57b](https://github.com/AztecProtocol/aztec-packages/commit/64bf57b0788f5db78f74b1aa1fb93c50ff72271b))
* Put dev docs before spec ([#1944](https://github.com/AztecProtocol/aztec-packages/issues/1944)) ([f1b29cd](https://github.com/AztecProtocol/aztec-packages/commit/f1b29cd7c7bc0ace2cef55d54f647077e94facad))
* Quick start guide for up page ([#1573](https://github.com/AztecProtocol/aztec-packages/issues/1573)) ([b102517](https://github.com/AztecProtocol/aztec-packages/commit/b102517c24cb4ed5fa05d0078a3eddd2bcc7cb77))
* Some initial noir contract docs ([#1449](https://github.com/AztecProtocol/aztec-packages/issues/1449)) ([a3514c3](https://github.com/AztecProtocol/aztec-packages/commit/a3514c30438c7ef5c1aa9eb4640d228649ee4410))
* Storage and state variables ([#1725](https://github.com/AztecProtocol/aztec-packages/issues/1725)) ([fc72f84](https://github.com/AztecProtocol/aztec-packages/commit/fc72f84a5bf21f083eddf3b8c59a00321dce26fd))
* Use the pre-processor code snippet import method ([#1719](https://github.com/AztecProtocol/aztec-packages/issues/1719)) ([54f6410](https://github.com/AztecProtocol/aztec-packages/commit/54f641081c141e035097a39935952de6be3090fc))
* Wallet dev docs ([#1746](https://github.com/AztecProtocol/aztec-packages/issues/1746)) ([9b4281d](https://github.com/AztecProtocol/aztec-packages/commit/9b4281dab16868cdda86a8f59d6d62aaaa8a90d6)), closes [#1744](https://github.com/AztecProtocol/aztec-packages/issues/1744)

## [0.1.0-alpha63](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha62...v0.1.0-alpha63) (2023-09-08)


### Features

* `GrumpkinScalar` type ([#1919](https://github.com/AztecProtocol/aztec-packages/issues/1919)) ([3a9238a](https://github.com/AztecProtocol/aztec-packages/commit/3a9238a99a32259d8d6b85df6335a002c7bab354))


### Bug Fixes

* add retry to tag and docker actions ([#2099](https://github.com/AztecProtocol/aztec-packages/issues/2099)) ([9f741f4](https://github.com/AztecProtocol/aztec-packages/commit/9f741f4e181120edcb63c28fa6c50b5b5e2e26c9))
* **breaking change:** change embedded curve scalar mul to use two limbs to properly encode the scalar field ([#2105](https://github.com/AztecProtocol/aztec-packages/issues/2105)) ([070cc4c](https://github.com/AztecProtocol/aztec-packages/commit/070cc4cb31ada29e42846e16df1ec191100214a5))
* broken bootstrap.sh after renaming `aztec-cli` dir as `cli` ([#2097](https://github.com/AztecProtocol/aztec-packages/issues/2097)) ([2386781](https://github.com/AztecProtocol/aztec-packages/commit/2386781fd1fed9f552559961b4e9f60406095067))
* browser test in canary flow ([#2102](https://github.com/AztecProtocol/aztec-packages/issues/2102)) ([d52af6c](https://github.com/AztecProtocol/aztec-packages/commit/d52af6c0e2c5ed268747237e65603368645c9966)), closes [#2086](https://github.com/AztecProtocol/aztec-packages/issues/2086)
* check a note is read before nullifying it. ([#2076](https://github.com/AztecProtocol/aztec-packages/issues/2076)) ([aabfb13](https://github.com/AztecProtocol/aztec-packages/commit/aabfb1383033364df9c045573098a4f13ca3a452)), closes [#1899](https://github.com/AztecProtocol/aztec-packages/issues/1899)
* circuits issues when building with gcc ([#2107](https://github.com/AztecProtocol/aztec-packages/issues/2107)) ([4f5c4fe](https://github.com/AztecProtocol/aztec-packages/commit/4f5c4fe24f012988169d8a0a3d8ae5245e24d3ee))
* COMMIT_TAG arg value in canary Dockerfile ([#2118](https://github.com/AztecProtocol/aztec-packages/issues/2118)) ([a3d6459](https://github.com/AztecProtocol/aztec-packages/commit/a3d645978a6ccef279870498979733682f63e206))
* dont assume safety of nvm ([#2079](https://github.com/AztecProtocol/aztec-packages/issues/2079)) ([a4167e7](https://github.com/AztecProtocol/aztec-packages/commit/a4167e7e5ef55c9780c786959d078fe854093656))
* end-to-end aztec cli dependency issue ([#2092](https://github.com/AztecProtocol/aztec-packages/issues/2092)) ([16ee3e5](https://github.com/AztecProtocol/aztec-packages/commit/16ee3e530bd99c2a47b8bcda53f0a13f67df2ac6))
* minor annoyances ([#2115](https://github.com/AztecProtocol/aztec-packages/issues/2115)) ([a147582](https://github.com/AztecProtocol/aztec-packages/commit/a1475822b20c360d19a88f6205a4a35d987fc2f5))
* padded printing for e2e-cli ([#2106](https://github.com/AztecProtocol/aztec-packages/issues/2106)) ([5988014](https://github.com/AztecProtocol/aztec-packages/commit/5988014330c929e1fcb52c4fbba5a755fa013c16))
* remaining refs to clang15 ([#2077](https://github.com/AztecProtocol/aztec-packages/issues/2077)) ([2c16547](https://github.com/AztecProtocol/aztec-packages/commit/2c16547c450ac7591d5be7c734962be86be4310e))
* run e2e tests without spot ([#2081](https://github.com/AztecProtocol/aztec-packages/issues/2081)) ([f0aa3ca](https://github.com/AztecProtocol/aztec-packages/commit/f0aa3ca0de995f58ea5a18e64c18ee437b520675))
* updated CLI readme ([#2098](https://github.com/AztecProtocol/aztec-packages/issues/2098)) ([2226091](https://github.com/AztecProtocol/aztec-packages/commit/2226091e21d0aa0dbfa3bea4f95a0ea2a31a4c43)), closes [#1784](https://github.com/AztecProtocol/aztec-packages/issues/1784)


### Miscellaneous

* **circuits:** - remove dead code from cbind of private kernel circuit ([#2088](https://github.com/AztecProtocol/aztec-packages/issues/2088)) ([43dc9d7](https://github.com/AztecProtocol/aztec-packages/commit/43dc9d7500fa3d11a0b557b8fc82da4495c4e605))
* **circuits:** remove dead code in cbind.cpp for public kernel ([#2094](https://github.com/AztecProtocol/aztec-packages/issues/2094)) ([861f960](https://github.com/AztecProtocol/aztec-packages/commit/861f960524436796263d9f79fa06a38d0e62ae84))
* Conservatively raise the minimum supported clang version in CMakeList ([#2023](https://github.com/AztecProtocol/aztec-packages/issues/2023)) ([f49c416](https://github.com/AztecProtocol/aztec-packages/commit/f49c4164387d307f8a86e93faff3eb96d7c99e36))
* **constants:** bump number of private reads and writes ([#2062](https://github.com/AztecProtocol/aztec-packages/issues/2062)) ([ab6c6b1](https://github.com/AztecProtocol/aztec-packages/commit/ab6c6b1cefdc1dd1da6e1198f99a211b31e73d85))
* **contracts:** Use autogenerated Noir interfaces where possible ([#2073](https://github.com/AztecProtocol/aztec-packages/issues/2073)) ([bd6368b](https://github.com/AztecProtocol/aztec-packages/commit/bd6368bd16159aad88906496cb9d6270e483a26e)), closes [#1604](https://github.com/AztecProtocol/aztec-packages/issues/1604)
* merge bb release-please ([#2080](https://github.com/AztecProtocol/aztec-packages/issues/2080)) ([e89b043](https://github.com/AztecProtocol/aztec-packages/commit/e89b04358acbf6f43b72c346406cd97c0fa26af2))
* move storage into main.nr. ([#2068](https://github.com/AztecProtocol/aztec-packages/issues/2068)) ([2c2d72b](https://github.com/AztecProtocol/aztec-packages/commit/2c2d72b7799b24273e498805ecf4c36d69f08d7d))
* protogalaxy relations ([#1897](https://github.com/AztecProtocol/aztec-packages/issues/1897)) ([35407e2](https://github.com/AztecProtocol/aztec-packages/commit/35407e25081744702ec35efe3f95aa0137fe0ebb))


### Documentation

* **limitations:** limitations on ordering and logs of chopped notes ([#2085](https://github.com/AztecProtocol/aztec-packages/issues/2085)) ([315ad3d](https://github.com/AztecProtocol/aztec-packages/commit/315ad3d58eeb467361848a1e70fd32f3074b35d5)), closes [#1652](https://github.com/AztecProtocol/aztec-packages/issues/1652)

## [0.1.0-alpha62](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha61...v0.1.0-alpha62) (2023-09-06)


### Features

* **circuits:** hints nullifier transient commitments ([#2056](https://github.com/AztecProtocol/aztec-packages/issues/2056)) ([725b550](https://github.com/AztecProtocol/aztec-packages/commit/725b550a368494abd15a38e95b15b1379bc926bc))
* **docs:** Testing guide and getPrivateStorage method ([#1992](https://github.com/AztecProtocol/aztec-packages/issues/1992)) ([5a8c571](https://github.com/AztecProtocol/aztec-packages/commit/5a8c5719753549f71ceeec9114d69b8d1d640376))


### Bug Fixes

* **build:** Use semver version in docker version tag ([#2065](https://github.com/AztecProtocol/aztec-packages/issues/2065)) ([b3db0d0](https://github.com/AztecProtocol/aztec-packages/commit/b3db0d0ae6d6b7d8a6d7338a556e2b9507e2631a))


### Documentation

* Link to local ethereum nodes in testing guide ([#2061](https://github.com/AztecProtocol/aztec-packages/issues/2061)) ([e29148b](https://github.com/AztecProtocol/aztec-packages/commit/e29148b379a435a9fefd846cc5fe78af6be7021d))

## [0.1.0-alpha61](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha60...v0.1.0-alpha61) (2023-09-06)


### Bug Fixes

* **sandbox:** build script for tagged commits ([#2057](https://github.com/AztecProtocol/aztec-packages/issues/2057)) ([c9d9722](https://github.com/AztecProtocol/aztec-packages/commit/c9d9722151de1e6f9a49a4cc6310e5646593ec01))

## [0.1.0-alpha60](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha59...v0.1.0-alpha60) (2023-09-06)


### Features

* Goblin recursive verifier ([#1822](https://github.com/AztecProtocol/aztec-packages/issues/1822)) ([f962cb6](https://github.com/AztecProtocol/aztec-packages/commit/f962cb68f46d25047bf67a1ad2e7407a176ffc53))
* initial `is_valid` eip1271 style wallet + minimal test changes ([#1935](https://github.com/AztecProtocol/aztec-packages/issues/1935)) ([f264c54](https://github.com/AztecProtocol/aztec-packages/commit/f264c5421424bf58d983fe104aaf7c7126259e01))


### Bug Fixes

* benchmark git repo ([#2041](https://github.com/AztecProtocol/aztec-packages/issues/2041)) ([3c696bb](https://github.com/AztecProtocol/aztec-packages/commit/3c696bba1ca4bd69c8e3f5bc004d1a07adb23cf1))
* cli canary & deployment ([#2053](https://github.com/AztecProtocol/aztec-packages/issues/2053)) ([1ddd24a](https://github.com/AztecProtocol/aztec-packages/commit/1ddd24ad2f8702fd3d3c48ed015a652b3326bfd9))
* **rpc:** Fixes getNodeInfo serialisation ([#1991](https://github.com/AztecProtocol/aztec-packages/issues/1991)) ([0a29fa8](https://github.com/AztecProtocol/aztec-packages/commit/0a29fa8dd95b37e490c18df2db90a7324ebe762c))


### Miscellaneous

* **circuits:** - use msgpack for cbind routines of native private kernel circuits ([#1938](https://github.com/AztecProtocol/aztec-packages/issues/1938)) ([3dc5c07](https://github.com/AztecProtocol/aztec-packages/commit/3dc5c07358d99786df8809f46638fdb04b33a6c2))
* **docs:** API docs stucture ([#2014](https://github.com/AztecProtocol/aztec-packages/issues/2014)) ([9aab9dd](https://github.com/AztecProtocol/aztec-packages/commit/9aab9ddefac63d35ebc356afed573af268896b35))
* Update function selector computation ([#2001](https://github.com/AztecProtocol/aztec-packages/issues/2001)) ([e07ea1a](https://github.com/AztecProtocol/aztec-packages/commit/e07ea1a887484f3a1a2ba8b5328af5abf6ccc6a2))

## [0.1.0-alpha59](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha58...v0.1.0-alpha59) (2023-09-05)


### Features

* Add `info` command to bb ([#2010](https://github.com/AztecProtocol/aztec-packages/issues/2010)) ([1fd8196](https://github.com/AztecProtocol/aztec-packages/commit/1fd8196f302ee49f540dea54ce5df4c450592d05))


### Bug Fixes

* accidental git marker ([#2039](https://github.com/AztecProtocol/aztec-packages/issues/2039)) ([2be9908](https://github.com/AztecProtocol/aztec-packages/commit/2be990861ca25ec206f6bd02b604b73b30710ca8))

## [0.1.0-alpha58](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha57...v0.1.0-alpha58) (2023-09-05)


### Miscellaneous

* **ci:** Clean up stale image tags ([#1818](https://github.com/AztecProtocol/aztec-packages/issues/1818)) ([3c8b7b8](https://github.com/AztecProtocol/aztec-packages/commit/3c8b7b84efe938e32c938bbcd744a335ffc50f49))

## [0.1.0-alpha57](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha56...v0.1.0-alpha57) (2023-09-05)


### Bug Fixes

* undefined safety in master part 5 ([#2034](https://github.com/AztecProtocol/aztec-packages/issues/2034)) ([41eccaa](https://github.com/AztecProtocol/aztec-packages/commit/41eccaa516200bd65847e1b7b736c2f2cf858960))

## [0.1.0-alpha56](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha55...v0.1.0-alpha56) (2023-09-05)


### Bug Fixes

* use COMMIT_TAG_VERSION properly in deploy_dockerhub ([#2033](https://github.com/AztecProtocol/aztec-packages/issues/2033)) ([064ddc3](https://github.com/AztecProtocol/aztec-packages/commit/064ddc3b345ac445fc9fe2385c8aee78b8fb6e47))

## [0.1.0-alpha55](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha54...v0.1.0-alpha55) (2023-09-05)


### Bug Fixes

* **build-system:** undefined IMAGE_TAG and ARG_TAG ([#2030](https://github.com/AztecProtocol/aztec-packages/issues/2030)) ([dfdba4b](https://github.com/AztecProtocol/aztec-packages/commit/dfdba4b5c6fb0c75f7f463e0b5eb71e6e7d1b667))

## [0.1.0-alpha54](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha53...v0.1.0-alpha54) (2023-09-05)


### Bug Fixes

* try to catch last undefined safety issues ([#2027](https://github.com/AztecProtocol/aztec-packages/issues/2027)) ([12e7486](https://github.com/AztecProtocol/aztec-packages/commit/12e7486c0750f648f51d2b43317df843a3c52bec))

## [0.1.0-alpha53](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha52...v0.1.0-alpha53) (2023-09-05)


### Bug Fixes

* ensure_repo undefined-safe ([#2025](https://github.com/AztecProtocol/aztec-packages/issues/2025)) ([e36fb6b](https://github.com/AztecProtocol/aztec-packages/commit/e36fb6bb8a1ee9a3d405c3e5340ffa4e589656e2))

## [0.1.0-alpha52](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha51...v0.1.0-alpha52) (2023-09-05)


### Features

* **docs:** set up noir contracts in getting-started ([#1770](https://github.com/AztecProtocol/aztec-packages/issues/1770)) ([33eb99d](https://github.com/AztecProtocol/aztec-packages/commit/33eb99d4a00831f340b1b0de0352fc272cb66d14))


### Bug Fixes

* Complete JS call stacks across ACVM wasm boundaries ([#2013](https://github.com/AztecProtocol/aztec-packages/issues/2013)) ([8e84e46](https://github.com/AztecProtocol/aztec-packages/commit/8e84e460899f11eaf7f383863e20dc5395e45c6e))
* deploy_ecr calculating CONTENT_HASH ([#2024](https://github.com/AztecProtocol/aztec-packages/issues/2024)) ([edee198](https://github.com/AztecProtocol/aztec-packages/commit/edee1981d8d795aef64bd6de738f09ea9a1a2547))
* Option to fail silently when retrying ([#2015](https://github.com/AztecProtocol/aztec-packages/issues/2015)) ([453c9c1](https://github.com/AztecProtocol/aztec-packages/commit/453c9c1b234213fff4d63e117f2bc6c827040125))

## [0.1.0-alpha51](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha50...v0.1.0-alpha51) (2023-09-05)


### Bug Fixes

* build script ([#2017](https://github.com/AztecProtocol/aztec-packages/issues/2017)) ([23fce27](https://github.com/AztecProtocol/aztec-packages/commit/23fce277c44a06777ea168085ac498d62016b36e))

## [0.1.0-alpha50](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha49...v0.1.0-alpha50) (2023-09-05)


### ⚠ BREAKING CHANGES

* update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/aztec-packages/issues/1925))

### Features

* **892:** add hints for matching transient read requests with correspondi… ([#1995](https://github.com/AztecProtocol/aztec-packages/issues/1995)) ([0955bb7](https://github.com/AztecProtocol/aztec-packages/commit/0955bb7b0903b12c4f041096d51a1dbb48f6359d))
* Add support for assert messages & runtime call stacks  ([#1997](https://github.com/AztecProtocol/aztec-packages/issues/1997)) ([ac68837](https://github.com/AztecProtocol/aztec-packages/commit/ac68837677a80897538d7a0790af8d04410c4446))
* **Aztec.nr:** Kernel return types abstraction ([#1924](https://github.com/AztecProtocol/aztec-packages/issues/1924)) ([3a8e702](https://github.com/AztecProtocol/aztec-packages/commit/3a8e7026ea10aa8564bdcc127efd4213ebd526de))
* **ci:** use content hash in build system, restrict docs build to *.ts or *.cpp ([#1953](https://github.com/AztecProtocol/aztec-packages/issues/1953)) ([0036e07](https://github.com/AztecProtocol/aztec-packages/commit/0036e0742a67dfa8aa1fcdb498b89caca6441508))
* do not allow slot 0 in `noir-libs` ([#1884](https://github.com/AztecProtocol/aztec-packages/issues/1884)) ([54094b4](https://github.com/AztecProtocol/aztec-packages/commit/54094b464a4dc7aebf157ca54145cffce822bc6f)), closes [#1692](https://github.com/AztecProtocol/aztec-packages/issues/1692)
* throwing when submitting a duplicate tx of a settled one ([#1880](https://github.com/AztecProtocol/aztec-packages/issues/1880)) ([9ad768f](https://github.com/AztecProtocol/aztec-packages/commit/9ad768f1af5344dc079a74e80ec601e062558fd5)), closes [#1810](https://github.com/AztecProtocol/aztec-packages/issues/1810)
* typos, using Tx.clone functionality, better naming ([#1976](https://github.com/AztecProtocol/aztec-packages/issues/1976)) ([00bca67](https://github.com/AztecProtocol/aztec-packages/commit/00bca675cf7984052c960c3d1797c5b017f07f57))


### Bug Fixes

* add retry_10 around ensure_repo ([#1963](https://github.com/AztecProtocol/aztec-packages/issues/1963)) ([0afde39](https://github.com/AztecProtocol/aztec-packages/commit/0afde390ac63d132b0ba85440500da3375fd2e22))
* Adds Mac cross compile flags into barretenberg ([#1954](https://github.com/AztecProtocol/aztec-packages/issues/1954)) ([3aaf91e](https://github.com/AztecProtocol/aztec-packages/commit/3aaf91e03fc01f0cb12249f22dbcb007023f69d4))
* bb meta-data ([#1960](https://github.com/AztecProtocol/aztec-packages/issues/1960)) ([712e0a0](https://github.com/AztecProtocol/aztec-packages/commit/712e0a088bff9ae2f49489901fab2a3fe0fb6d4b))
* **bb.js:** (breaking change) bundles bb.js properly so that it works in the browser and in node ([#1855](https://github.com/AztecProtocol/aztec-packages/issues/1855)) ([1aa6f59](https://github.com/AztecProtocol/aztec-packages/commit/1aa6f5934cd97dd32d81e490013f8ef7d1e14ec7))
* Benchmark preset uses clang16 ([#1902](https://github.com/AztecProtocol/aztec-packages/issues/1902)) ([4f7eeea](https://github.com/AztecProtocol/aztec-packages/commit/4f7eeea6c79604aea88433790dfc542a356aa898))
* build ([#1906](https://github.com/AztecProtocol/aztec-packages/issues/1906)) ([8223be1](https://github.com/AztecProtocol/aztec-packages/commit/8223be18d98ebb4edb7700310b2fda5201bd04b9))
* **ci:** Incorrect content hash in some build targets ([#1973](https://github.com/AztecProtocol/aztec-packages/issues/1973)) ([0a2a515](https://github.com/AztecProtocol/aztec-packages/commit/0a2a515ecf52849cce1e45a7b39f44d420b43f34))
* circuits should not link openmp with -DMULTITHREADING ([#1929](https://github.com/AztecProtocol/aztec-packages/issues/1929)) ([cd1a685](https://github.com/AztecProtocol/aztec-packages/commit/cd1a685a3ecdd571d83cd2ad0844bd1d143fd9af))
* compilation on homebrew clang 16.06 ([#1937](https://github.com/AztecProtocol/aztec-packages/issues/1937)) ([c611582](https://github.com/AztecProtocol/aztec-packages/commit/c611582239a057717410f0a6c0fd8202844a564e))
* docs preprocessor line numbers and errors ([#1883](https://github.com/AztecProtocol/aztec-packages/issues/1883)) ([4e7e290](https://github.com/AztecProtocol/aztec-packages/commit/4e7e290478ae4ca9c128c0b6b4b26529965cc2a2))
* ensure CLI command doesn't fail due to missing client version ([#1895](https://github.com/AztecProtocol/aztec-packages/issues/1895)) ([88086e4](https://github.com/AztecProtocol/aztec-packages/commit/88086e4a80d7841d28188366a469800afa281693))
* error handling in acir simulator ([#1907](https://github.com/AztecProtocol/aztec-packages/issues/1907)) ([165008e](https://github.com/AztecProtocol/aztec-packages/commit/165008ec3027d8f2f76256c37f63e5d7a669b5dd))
* Fix off by one in circuits.js when fetching points from transcript ([#1993](https://github.com/AztecProtocol/aztec-packages/issues/1993)) ([cec901f](https://github.com/AztecProtocol/aztec-packages/commit/cec901f3df440ebc0e3bdcfb2567b70fd9bde9dd))
* format.sh issues ([#1946](https://github.com/AztecProtocol/aztec-packages/issues/1946)) ([f24814b](https://github.com/AztecProtocol/aztec-packages/commit/f24814b328c45316fa584cad1d9aa4784b6a0b2e))
* master ([#1981](https://github.com/AztecProtocol/aztec-packages/issues/1981)) ([6bfb053](https://github.com/AztecProtocol/aztec-packages/commit/6bfb053fb2c4053a72a8daa18a241261380ee311))
* More accurate c++ build pattern ([#1962](https://github.com/AztecProtocol/aztec-packages/issues/1962)) ([21c2f8e](https://github.com/AztecProtocol/aztec-packages/commit/21c2f8edd110da8749a0039c900c25aff8baa7a4))
* polyfill by bundling fileURLToPath ([#1949](https://github.com/AztecProtocol/aztec-packages/issues/1949)) ([1b2de01](https://github.com/AztecProtocol/aztec-packages/commit/1b2de01df69a16f442c348cc302ade1392e74519))
* Set correct version of RPC & Sandbox when deploying tagged commit ([#1914](https://github.com/AztecProtocol/aztec-packages/issues/1914)) ([898c50d](https://github.com/AztecProtocol/aztec-packages/commit/898c50d594b7515f6ca3b904d31ccf724b683ade))
* typescript lookup of aztec.js types ([#1948](https://github.com/AztecProtocol/aztec-packages/issues/1948)) ([22901ae](https://github.com/AztecProtocol/aztec-packages/commit/22901ae8fa63b61ba1fbf4885f3940dc839b555c))
* unify base64 interface between mac and linux (cherry-picked) ([#1968](https://github.com/AztecProtocol/aztec-packages/issues/1968)) ([ee24b52](https://github.com/AztecProtocol/aztec-packages/commit/ee24b52234956744d2b35b0eb0d3b5c2dcf7ed82))
* Update docs search config ([#1920](https://github.com/AztecProtocol/aztec-packages/issues/1920)) ([c8764e6](https://github.com/AztecProtocol/aztec-packages/commit/c8764e6150b7d372c34ddc008be9925e5f5f6dfb))
* update docs search keys ([#1931](https://github.com/AztecProtocol/aztec-packages/issues/1931)) ([03b200c](https://github.com/AztecProtocol/aztec-packages/commit/03b200c10da71bd4b6fa3902edb254f9f625bf8b))


### Miscellaneous

* **1407:** remove forwarding witnesses ([#1930](https://github.com/AztecProtocol/aztec-packages/issues/1930)) ([cc8bc8f](https://github.com/AztecProtocol/aztec-packages/commit/cc8bc8f48b175479e1c4dfbcf9b92159f096c2cf)), closes [#1407](https://github.com/AztecProtocol/aztec-packages/issues/1407)
* **1879:** add use of PrivateKernelPublicInputs in TS whenever relevant ([#1911](https://github.com/AztecProtocol/aztec-packages/issues/1911)) ([8d5f548](https://github.com/AztecProtocol/aztec-packages/commit/8d5f548e42d627da1685820f99fc28ff5f47abbe))
* acir tests are no longer base64 encoded ([#1854](https://github.com/AztecProtocol/aztec-packages/issues/1854)) ([7fffd16](https://github.com/AztecProtocol/aztec-packages/commit/7fffd1680d6246f64ee4d4ca965b9764c6c0ebb3))
* Add back double verify proof to test suite ([#1986](https://github.com/AztecProtocol/aztec-packages/issues/1986)) ([f8688d7](https://github.com/AztecProtocol/aztec-packages/commit/f8688d7df05abcb6c650aafb130dedb707931950))
* add CLI test to canary flow ([#1918](https://github.com/AztecProtocol/aztec-packages/issues/1918)) ([cc68958](https://github.com/AztecProtocol/aztec-packages/commit/cc689585a845ce3c20ea9714ca744f4aa8837462)), closes [#1903](https://github.com/AztecProtocol/aztec-packages/issues/1903)
* Add safemath noir testing ([#1967](https://github.com/AztecProtocol/aztec-packages/issues/1967)) ([cb1f1ec](https://github.com/AztecProtocol/aztec-packages/commit/cb1f1ece1fd050b00ad8cbe9086e76383f9e6377))
* **Aztec.nr:** remove implicit imports ([#1901](https://github.com/AztecProtocol/aztec-packages/issues/1901)) ([c7d5190](https://github.com/AztecProtocol/aztec-packages/commit/c7d5190e48771c334bfa7062c361bcd623faa318))
* **Aztec.nr:** Remove the open keyword from public functions ([#1917](https://github.com/AztecProtocol/aztec-packages/issues/1917)) ([4db8603](https://github.com/AztecProtocol/aztec-packages/commit/4db8603a4ee293c64a67be5ba74072bd654c7ec5))
* **ci:** build docs on every pr ([#1955](https://github.com/AztecProtocol/aztec-packages/issues/1955)) ([c200bc5](https://github.com/AztecProtocol/aztec-packages/commit/c200bc5337da9d6122a2545fceeada98a28d7077))
* Enable project-specific releases for dockerhub too ([#1721](https://github.com/AztecProtocol/aztec-packages/issues/1721)) ([5d2c082](https://github.com/AztecProtocol/aztec-packages/commit/5d2c0824eedb748ca3e2beaa8589410a21ba6e57))
* reduce max circuit size in bb binary ([#1942](https://github.com/AztecProtocol/aztec-packages/issues/1942)) ([c61439b](https://github.com/AztecProtocol/aztec-packages/commit/c61439b316829563c93bbfcb78b799bdc105ff71))
* Reference noir master for acir tests ([#1969](https://github.com/AztecProtocol/aztec-packages/issues/1969)) ([86b72e1](https://github.com/AztecProtocol/aztec-packages/commit/86b72e1e8da29a0335e40c6de4c46538d8138f2f))
* remove debug output from `run_acir_tests` script ([#1970](https://github.com/AztecProtocol/aztec-packages/issues/1970)) ([74c83c5](https://github.com/AztecProtocol/aztec-packages/commit/74c83c5e1436f391eef435926c2da1d508d67713))
* storing `&mut context` in state vars ([#1926](https://github.com/AztecProtocol/aztec-packages/issues/1926)) ([89a7a3f](https://github.com/AztecProtocol/aztec-packages/commit/89a7a3ff22ebc469fe1b58d929af5ef162514c17)), closes [#1805](https://github.com/AztecProtocol/aztec-packages/issues/1805)
* sync bb master ([#1947](https://github.com/AztecProtocol/aztec-packages/issues/1947)) ([eed58e1](https://github.com/AztecProtocol/aztec-packages/commit/eed58e157c2740043ad6f53c76b13ba9924c5d93))
* update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/aztec-packages/issues/1925)) ([e728304](https://github.com/AztecProtocol/aztec-packages/commit/e72830468362f2ea26b3f830b7e056b096f56d6a))
* Update to acvm 0.24.1 ([#1978](https://github.com/AztecProtocol/aztec-packages/issues/1978)) ([31c0a02](https://github.com/AztecProtocol/aztec-packages/commit/31c0a0219330bce94a16dea9833fd900e61d93b4))
* updating docs to clang16 ([#1875](https://github.com/AztecProtocol/aztec-packages/issues/1875)) ([a248dae](https://github.com/AztecProtocol/aztec-packages/commit/a248dae54af9cb7ca64b2a7780a4b90e3848a69b))


### Documentation

* **keys:** Complete addresses are now broadcast ([#1975](https://github.com/AztecProtocol/aztec-packages/issues/1975)) ([92068ad](https://github.com/AztecProtocol/aztec-packages/commit/92068ad4249b2a20a4c83d82b82517ccdcbfe7f9)), closes [#1936](https://github.com/AztecProtocol/aztec-packages/issues/1936)
* limitations, privacy, roadmap ([#1759](https://github.com/AztecProtocol/aztec-packages/issues/1759)) ([0cdb27a](https://github.com/AztecProtocol/aztec-packages/commit/0cdb27af8359b61b4a1f51a829ddfc4995ec1d30))
* put dev docs before spec ([#1944](https://github.com/AztecProtocol/aztec-packages/issues/1944)) ([f1b29cd](https://github.com/AztecProtocol/aztec-packages/commit/f1b29cd7c7bc0ace2cef55d54f647077e94facad))
* storage and state variables ([#1725](https://github.com/AztecProtocol/aztec-packages/issues/1725)) ([fc72f84](https://github.com/AztecProtocol/aztec-packages/commit/fc72f84a5bf21f083eddf3b8c59a00321dce26fd))

## [0.1.0-alpha49](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha48...v0.1.0-alpha49) (2023-08-30)


### Features

* Generate public context contract interfaces ([#1860](https://github.com/AztecProtocol/aztec-packages/issues/1860)) ([2f4045e](https://github.com/AztecProtocol/aztec-packages/commit/2f4045e22dbea0e316103da20c6ba8a667826777)), closes [#1782](https://github.com/AztecProtocol/aztec-packages/issues/1782)


### Bug Fixes

* Do not warn on mismatched cli/sandbox version ([#1894](https://github.com/AztecProtocol/aztec-packages/issues/1894)) ([a44a0f6](https://github.com/AztecProtocol/aztec-packages/commit/a44a0f6489b8ea7d648d1b9babf49fae8d593b7b))
* remove extra transfer arg in CLI Guide ([#1887](https://github.com/AztecProtocol/aztec-packages/issues/1887)) ([55728b8](https://github.com/AztecProtocol/aztec-packages/commit/55728b850c19403ba8b2aaefe89181640acbd9fd))
* Reset keccak var inputs to 0 ([#1881](https://github.com/AztecProtocol/aztec-packages/issues/1881)) ([382f07e](https://github.com/AztecProtocol/aztec-packages/commit/382f07e3032c5ad3cf15e62e38bb5f0583ab46dd))


### Miscellaneous

* **1074:** remove read request data from final private kernel circuit public inputs ([#1840](https://github.com/AztecProtocol/aztec-packages/issues/1840)) ([c61557a](https://github.com/AztecProtocol/aztec-packages/commit/c61557ae926f89cead7306368197fdbe8f23dd6d))
* Reenable and refactor nested calls e2e tests ([#1868](https://github.com/AztecProtocol/aztec-packages/issues/1868)) ([570de80](https://github.com/AztecProtocol/aztec-packages/commit/570de803376de4af6a1824b7a3c95129c98e2fa0)), closes [#1587](https://github.com/AztecProtocol/aztec-packages/issues/1587)
* Update formatting ([#1874](https://github.com/AztecProtocol/aztec-packages/issues/1874)) ([fb973ca](https://github.com/AztecProtocol/aztec-packages/commit/fb973caeabc2d10daaf052046987e54f563b7e4b))

## [0.1.0-alpha48](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha47...v0.1.0-alpha48) (2023-08-30)


### Features

* Add ARM build for Mac + cleanup artifacts ([#1837](https://github.com/AztecProtocol/aztec-packages/issues/1837)) ([270a4ae](https://github.com/AztecProtocol/aztec-packages/commit/270a4ae4d1998149735251e2c3c1be73a9029f61))
* broadcasting 'public key' and 'partial address' as L1 calldata ([#1801](https://github.com/AztecProtocol/aztec-packages/issues/1801)) ([78d6444](https://github.com/AztecProtocol/aztec-packages/commit/78d6444e82903fe3d0d17318cd38b1b262e81391)), closes [#1778](https://github.com/AztecProtocol/aztec-packages/issues/1778)
* Check sandbox version matches CLI's ([#1849](https://github.com/AztecProtocol/aztec-packages/issues/1849)) ([7279730](https://github.com/AztecProtocol/aztec-packages/commit/72797305ac9ce8639abb09334cf2471f0932ca88))
* **docs:** adding some nitpick suggestions before sandbox release ([#1859](https://github.com/AztecProtocol/aztec-packages/issues/1859)) ([c1144f7](https://github.com/AztecProtocol/aztec-packages/commit/c1144f7bcfe8ebe222b840b0edd3d901ca30bdaf))
* More reliable getTxReceipt api. ([#1793](https://github.com/AztecProtocol/aztec-packages/issues/1793)) ([ad16b22](https://github.com/AztecProtocol/aztec-packages/commit/ad16b2219bff44dfbc3482b81c86e29bf0d60fc5))
* **noir:** use `#[aztec(private)]` and `#[aztec(public)` attributes ([#1735](https://github.com/AztecProtocol/aztec-packages/issues/1735)) ([89756fa](https://github.com/AztecProtocol/aztec-packages/commit/89756fae7d562274a84c60024beff5fae032f297))
* Recursive fn calls to spend more notes. ([#1779](https://github.com/AztecProtocol/aztec-packages/issues/1779)) ([94053e4](https://github.com/AztecProtocol/aztec-packages/commit/94053e44f4d2a702fe9066bfff3cdd35e6d1b645))
* Simulate enqueued public functions and locate failing constraints on them ([#1853](https://github.com/AztecProtocol/aztec-packages/issues/1853)) ([a065fd5](https://github.com/AztecProtocol/aztec-packages/commit/a065fd53dde48a1f28616ebe130222dd39d07b11))
* Update safe_math and move to libraries ([#1803](https://github.com/AztecProtocol/aztec-packages/issues/1803)) ([b10656d](https://github.com/AztecProtocol/aztec-packages/commit/b10656d30622366dcbbe5adb5b3948b0702a06e7))
* Write debug-level log to local file in Sandbox ([#1846](https://github.com/AztecProtocol/aztec-packages/issues/1846)) ([0317e93](https://github.com/AztecProtocol/aztec-packages/commit/0317e93d3dffb3b66a926863e7fe8b8c15f61536)), closes [#1605](https://github.com/AztecProtocol/aztec-packages/issues/1605)


### Bug Fixes

* Conditionally compile base64 command for bb binary ([#1851](https://github.com/AztecProtocol/aztec-packages/issues/1851)) ([be97185](https://github.com/AztecProtocol/aztec-packages/commit/be9718505c7e387bb46183299c9db855e6d7f91c))
* default color to light mode ([#1847](https://github.com/AztecProtocol/aztec-packages/issues/1847)) ([4fc8d39](https://github.com/AztecProtocol/aztec-packages/commit/4fc8d39041d437940bb18815e14f506b2ebe259e))
* Disallow unregistered classes in JSON RPC interface and match by name ([#1820](https://github.com/AztecProtocol/aztec-packages/issues/1820)) ([35b8170](https://github.com/AztecProtocol/aztec-packages/commit/35b817055e1fe848e6d87d445a7881c5c128ad35))
* Set side effect counter on contract reads ([#1870](https://github.com/AztecProtocol/aztec-packages/issues/1870)) ([1d8881e](https://github.com/AztecProtocol/aztec-packages/commit/1d8881e4872b39195ace523432c0e34bc9081f8d)), closes [#1588](https://github.com/AztecProtocol/aztec-packages/issues/1588)
* Truncate SRS size to the amount of points that we have downloaded ([#1862](https://github.com/AztecProtocol/aztec-packages/issues/1862)) ([0a7058c](https://github.com/AztecProtocol/aztec-packages/commit/0a7058cbda228c9baf378d69c906596e204d804f))


### Miscellaneous

* add browser test to canary flow ([#1808](https://github.com/AztecProtocol/aztec-packages/issues/1808)) ([7f4fa43](https://github.com/AztecProtocol/aztec-packages/commit/7f4fa438bf2f4966338e3e53ece7c1d01e8dd054))
* **ci:** fix output name in release please workflow ([#1858](https://github.com/AztecProtocol/aztec-packages/issues/1858)) ([857821f](https://github.com/AztecProtocol/aztec-packages/commit/857821fa1923aa013fe9470f12067208d5c494d1))
* CLI tests ([#1786](https://github.com/AztecProtocol/aztec-packages/issues/1786)) ([2987065](https://github.com/AztecProtocol/aztec-packages/commit/298706557a8f2b73a87dfb10c81626ebf127cadb)), closes [#1450](https://github.com/AztecProtocol/aztec-packages/issues/1450)
* compile minimal WASM binary needed for blackbox functions ([#1824](https://github.com/AztecProtocol/aztec-packages/issues/1824)) ([76a30b8](https://github.com/AztecProtocol/aztec-packages/commit/76a30b8b5b5e765a14fe7d896d8890897cad7756))
* fixed linter errors for `ecc`, `numeric` and `common` modules ([#1714](https://github.com/AztecProtocol/aztec-packages/issues/1714)) ([026273b](https://github.com/AztecProtocol/aztec-packages/commit/026273b42d8c41de9bc4a86f898162cbbb3ad35f))
* Refactor Cli interface to be more unix-like ([#1833](https://github.com/AztecProtocol/aztec-packages/issues/1833)) ([28d722e](https://github.com/AztecProtocol/aztec-packages/commit/28d722ef965d907b7b7820ccdd7ee0afc97c88fa))
* sync bb master ([#1842](https://github.com/AztecProtocol/aztec-packages/issues/1842)) ([2c1ff72](https://github.com/AztecProtocol/aztec-packages/commit/2c1ff729fd1994270644a96da5a954ce2ec72382))
* sync bb master ([#1852](https://github.com/AztecProtocol/aztec-packages/issues/1852)) ([f979878](https://github.com/AztecProtocol/aztec-packages/commit/f979878cb84dd1b0506cedd59e9df1bb65a99b0a))
* sync bb master ([#1866](https://github.com/AztecProtocol/aztec-packages/issues/1866)) ([e681a49](https://github.com/AztecProtocol/aztec-packages/commit/e681a4901ee51cdd133c126d299881be6fad3680))
* typescript script names should be consistent ([#1843](https://github.com/AztecProtocol/aztec-packages/issues/1843)) ([eff8fe7](https://github.com/AztecProtocol/aztec-packages/commit/eff8fe7ea9f2674383b7b8ea1232be49626fc595))
* use 2^19 as `MAX_CIRCUIT_SIZE` for NodeJS CLI ([#1834](https://github.com/AztecProtocol/aztec-packages/issues/1834)) ([c573282](https://github.com/AztecProtocol/aztec-packages/commit/c573282fd59e44df70ae125f68281ebb67b7453d))


### Documentation

* Account contract tutorial ([#1772](https://github.com/AztecProtocol/aztec-packages/issues/1772)) ([0faefba](https://github.com/AztecProtocol/aztec-packages/commit/0faefba283a7c654c0771ba8f15d5bb6346282ab))
* Wallet dev docs ([#1746](https://github.com/AztecProtocol/aztec-packages/issues/1746)) ([9b4281d](https://github.com/AztecProtocol/aztec-packages/commit/9b4281dab16868cdda86a8f59d6d62aaaa8a90d6)), closes [#1744](https://github.com/AztecProtocol/aztec-packages/issues/1744)

## [0.1.0-alpha47](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha46...v0.1.0-alpha47) (2023-08-25)


### Features

* `FunctionSelector` type ([#1518](https://github.com/AztecProtocol/aztec-packages/issues/1518)) ([942f705](https://github.com/AztecProtocol/aztec-packages/commit/942f7058adc706924ff26d2490bec7f7d57d7149)), closes [#1424](https://github.com/AztecProtocol/aztec-packages/issues/1424)


### Bug Fixes

* increment time by 1 for previous rollup was warped ([#1594](https://github.com/AztecProtocol/aztec-packages/issues/1594)) ([2a52107](https://github.com/AztecProtocol/aztec-packages/commit/2a521070397b6d1915e55b4ec702d4778563e683))
* **noir:** Add workaround for latest noir in account contracts ([#1781](https://github.com/AztecProtocol/aztec-packages/issues/1781)) ([eb8a052](https://github.com/AztecProtocol/aztec-packages/commit/eb8a052ad4e19394f096cc3a0f533c2560a7f5cc))
* selector name regression ([#1800](https://github.com/AztecProtocol/aztec-packages/issues/1800)) ([a5be8bb](https://github.com/AztecProtocol/aztec-packages/commit/a5be8bb92f858d266cf96671c46343b6e1ff400a))


### Miscellaneous

* Add todo for using generator indices in note commitment and nullifier computation. ([#1762](https://github.com/AztecProtocol/aztec-packages/issues/1762)) ([2db6728](https://github.com/AztecProtocol/aztec-packages/commit/2db6728fcaf75ce8c98d821b65695543bb0c82a2))
* **p2p:** Updated libp2p dependencies ([#1792](https://github.com/AztecProtocol/aztec-packages/issues/1792)) ([79df831](https://github.com/AztecProtocol/aztec-packages/commit/79df83134e15655dc3a5ed9dae00dc52a3d40681))
* Sandbox logging tweaks ([#1797](https://github.com/AztecProtocol/aztec-packages/issues/1797)) ([0e3914e](https://github.com/AztecProtocol/aztec-packages/commit/0e3914ed6ad63062add1cc08f6ea85646c068f8a))
* split out yarn-project bootstrap.sh ([#1790](https://github.com/AztecProtocol/aztec-packages/issues/1790)) ([1788fe6](https://github.com/AztecProtocol/aztec-packages/commit/1788fe6259f5e7fd191929b27996a7342e3f13e5))

## [0.1.0-alpha46](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha45...v0.1.0-alpha46) (2023-08-24)


### Features

* CDP/Lending example contract ([#1554](https://github.com/AztecProtocol/aztec-packages/issues/1554)) ([ecf6df2](https://github.com/AztecProtocol/aztec-packages/commit/ecf6df201047dcaa61c270cdb512cdc62086b356))
* no unencrypted logs in private functions ([#1780](https://github.com/AztecProtocol/aztec-packages/issues/1780)) ([4d8002e](https://github.com/AztecProtocol/aztec-packages/commit/4d8002e0d101a14c465929d92ea05d0be6e8d99a)), closes [#1689](https://github.com/AztecProtocol/aztec-packages/issues/1689)


### Miscellaneous

* **ci:** Updated release please configuration ([#1787](https://github.com/AztecProtocol/aztec-packages/issues/1787)) ([6eb2f7a](https://github.com/AztecProtocol/aztec-packages/commit/6eb2f7abc40bae88ebeec546ad9f8f2c7d810a24))
* sync bb master ([#1776](https://github.com/AztecProtocol/aztec-packages/issues/1776)) ([7c6fb15](https://github.com/AztecProtocol/aztec-packages/commit/7c6fb15979b48d4d4d5eb5a1ea83d3c0d0ee3b5e))


### Documentation

* events ([#1768](https://github.com/AztecProtocol/aztec-packages/issues/1768)) ([5a38cea](https://github.com/AztecProtocol/aztec-packages/commit/5a38cea3f7c1567a8eea3d6c2c58cad6f79b05f2)), closes [#1756](https://github.com/AztecProtocol/aztec-packages/issues/1756)

## [0.1.0-alpha45](https://github.com/AztecProtocol/aztec-packages/compare/v0.1.0-alpha44...v0.1.0-alpha45) (2023-08-23)


### Features

* **bb:** Use an environment variable to set the transcript URL ([#1750](https://github.com/AztecProtocol/aztec-packages/issues/1750)) ([31488c1](https://github.com/AztecProtocol/aztec-packages/commit/31488c19acfdfd5ff0c3e7f242f94dc0aa049158))
* **ci:** Initial release please config ([#1769](https://github.com/AztecProtocol/aztec-packages/issues/1769)) ([4207559](https://github.com/AztecProtocol/aztec-packages/commit/42075590058b21f38b5e745af54b2062371f9ebe))
* compress debug symbols ([#1760](https://github.com/AztecProtocol/aztec-packages/issues/1760)) ([9464b25](https://github.com/AztecProtocol/aztec-packages/commit/9464b25c1a2a809db559ddc4e2d4ee5ade1fa65a))
* not retrying unrecoverable errors ([#1752](https://github.com/AztecProtocol/aztec-packages/issues/1752)) ([c0f2820](https://github.com/AztecProtocol/aztec-packages/commit/c0f28204f53152c941704ece66287eddfe13c047))


### Bug Fixes

* Download SRS using one canonical URL across the codebase ([#1748](https://github.com/AztecProtocol/aztec-packages/issues/1748)) ([899b055](https://github.com/AztecProtocol/aztec-packages/commit/899b05557365a5bf97e64793dd563a1b4bfa0f3f))
* proving fails when circuit has size &gt; ~500K ([#1739](https://github.com/AztecProtocol/aztec-packages/issues/1739)) ([708b05c](https://github.com/AztecProtocol/aztec-packages/commit/708b05ca6638dc0d6ca7cb34fb8de76665a43b58))


### Miscellaneous

* **ci:** set up nightly barretenberg releases ([#1761](https://github.com/AztecProtocol/aztec-packages/issues/1761)) ([e0078da](https://github.com/AztecProtocol/aztec-packages/commit/e0078dabfcd9e006c2a489c7142ab141d5d81b80))
* **ci:** Updated release please config ([#1775](https://github.com/AztecProtocol/aztec-packages/issues/1775)) ([0085e8b](https://github.com/AztecProtocol/aztec-packages/commit/0085e8b17efc36256974f82525530c39ed182639))
* consistent block number method naming ([#1751](https://github.com/AztecProtocol/aztec-packages/issues/1751)) ([df1afe2](https://github.com/AztecProtocol/aztec-packages/commit/df1afe255d3095a9b2851b47480801c06d116eed))
* Use context instead of custom oracles for public functions ([#1754](https://github.com/AztecProtocol/aztec-packages/issues/1754)) ([46de77a](https://github.com/AztecProtocol/aztec-packages/commit/46de77ad3e5e91b9276146410381c69ccba1ae2b))


### Documentation

* convert quick start guides into e2e tests ([#1726](https://github.com/AztecProtocol/aztec-packages/issues/1726)) ([802a678](https://github.com/AztecProtocol/aztec-packages/commit/802a678e3dd19339cd88b105a0ce341026b58054)), closes [#1564](https://github.com/AztecProtocol/aztec-packages/issues/1564)
* including "real" code in keys docs ([#1767](https://github.com/AztecProtocol/aztec-packages/issues/1767)) ([cd9cadb](https://github.com/AztecProtocol/aztec-packages/commit/cd9cadbfb6b0311c381586799588a5f64df98f29))
