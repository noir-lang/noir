# Changelog

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
* properly initialise `Evaluator` in test ([#863](https://github.com/noir-lang/noir/issues/863)) ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))
* properly initialise Evaluator in test ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))
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
* **ci:** Add concurreny group for rust workflow ([1b80f55](https://github.com/noir-lang/noir/commit/1b80f559599c2a7d7b8697f42f63db8e59d318c5))
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
