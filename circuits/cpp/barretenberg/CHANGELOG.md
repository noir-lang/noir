# Changelog

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
