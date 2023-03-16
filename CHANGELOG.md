# Changelog

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
