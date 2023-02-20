# Changelog

## [0.3.0](https://github.com/noir-lang/noir/compare/v0.2.0...v0.3.0) (2023-02-20)


### ⚠ BREAKING CHANGES

* prevent inconsistent language usage in `Driver` ([#881](https://github.com/noir-lang/noir/issues/881))
* **abi:** add explicit return type field to ABI. ([#865](https://github.com/noir-lang/noir/issues/865))
* **abi:** merge both abi encoding/decoding methods ([#862](https://github.com/noir-lang/noir/issues/862))
* **abi:** add an explicit mapping from ABI params to witness indices ([#851](https://github.com/noir-lang/noir/issues/851))
* Allow impls on primitive types ([#847](https://github.com/noir-lang/noir/issues/847))

### Features

* **abi:** add an explicit mapping from ABI params to witness indices ([#851](https://github.com/noir-lang/noir/issues/851)) ([5bd4bd5](https://github.com/noir-lang/noir/commit/5bd4bd5047e4bc9a67bd79ab2a2519dc0c92da42))
* **abi:** add explicit return type field to ABI. ([#865](https://github.com/noir-lang/noir/issues/865)) ([8ca5676](https://github.com/noir-lang/noir/commit/8ca5676ba68403fff8bd953fe7c2d2f7c8e62a09))
* **abi:** merge both abi encoding/decoding methods ([#862](https://github.com/noir-lang/noir/issues/862)) ([fecd32c](https://github.com/noir-lang/noir/commit/fecd32cc27b552eb47681618ba44894c635c7f8c))
* Allow impls on primitive types ([#847](https://github.com/noir-lang/noir/issues/847)) ([479da0e](https://github.com/noir-lang/noir/commit/479da0e724dc34667baaabd8e37ce143193bf97e))
* **ci:** Publish noir_wasm when we cut a release ([#871](https://github.com/noir-lang/noir/issues/871)) ([5186ab9](https://github.com/noir-lang/noir/commit/5186ab97a0fc087413f6d217b87c77f693c574ac))


### Bug Fixes

* Optimize parser ([#869](https://github.com/noir-lang/noir/issues/869)) ([e927a39](https://github.com/noir-lang/noir/commit/e927a39dc3d6517f233509b8349dfd9c7f79471d))
* prevent inconsistent language usage in `Driver` ([48cda7a](https://github.com/noir-lang/noir/commit/48cda7a08b22afdde9f904632b502c53fb491ee6))
* prevent inconsistent language usage in `Driver` ([#881](https://github.com/noir-lang/noir/issues/881)) ([48cda7a](https://github.com/noir-lang/noir/commit/48cda7a08b22afdde9f904632b502c53fb491ee6))
* properly initialise `Evaluator` in test ([#863](https://github.com/noir-lang/noir/issues/863)) ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))
* properly initialise Evaluator in test ([bbb70bd](https://github.com/noir-lang/noir/commit/bbb70bdcc78041f5db9b74657cdcc92ad34c035b))

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
