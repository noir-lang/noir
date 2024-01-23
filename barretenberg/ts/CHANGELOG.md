# Changelog

## [0.20.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.19.0...barretenberg.js-v0.20.0) (2024-01-22)


### Features

* Goblin acir composer ([#4112](https://github.com/AztecProtocol/aztec-packages/issues/4112)) ([5e85b92](https://github.com/AztecProtocol/aztec-packages/commit/5e85b92f48bc31fe55315de9f45c4907e417cb6a))

## [0.19.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.18.0...barretenberg.js-v0.19.0) (2024-01-17)


### Miscellaneous

* Barretenberg =&gt; bb namespace shortening ([#4066](https://github.com/AztecProtocol/aztec-packages/issues/4066)) ([e6b66b8](https://github.com/AztecProtocol/aztec-packages/commit/e6b66b856db498e6fc465212f3645cf2c196c31a))

## [0.18.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.17.0...barretenberg.js-v0.18.0) (2024-01-16)


### Features

* Bootstrap cache v2 ([#3876](https://github.com/AztecProtocol/aztec-packages/issues/3876)) ([331598d](https://github.com/AztecProtocol/aztec-packages/commit/331598d369ab9bb91dcc48d50bdd8df0684f0b79))


### Bug Fixes

* Dont spam logs with yarn install ([#4027](https://github.com/AztecProtocol/aztec-packages/issues/4027)) ([949c5ab](https://github.com/AztecProtocol/aztec-packages/commit/949c5abf1df399f691f17c19fab64f0e36476219))

## [0.17.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.9...barretenberg.js-v0.17.0) (2024-01-09)


### ⚠ BREAKING CHANGES

* return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735))

### Features

* Adding option to set initial and max memory ([#3265](https://github.com/AztecProtocol/aztec-packages/issues/3265)) ([0ad75fe](https://github.com/AztecProtocol/aztec-packages/commit/0ad75fe745099119726976f964a92d1587f32fbf))
* Bb uses goblin ([#3636](https://github.com/AztecProtocol/aztec-packages/issues/3636)) ([d093266](https://github.com/AztecProtocol/aztec-packages/commit/d09326636140dbd68d3efb8bc4ec2b6948e2bfe1))
* Correct circuit construction from acir ([#3757](https://github.com/AztecProtocol/aztec-packages/issues/3757)) ([a876ab8](https://github.com/AztecProtocol/aztec-packages/commit/a876ab8a61108be06bd5d884d727058e7e54a383))
* Return full verification contract from `AcirComposer::get_solidity_verifier` ([#3735](https://github.com/AztecProtocol/aztec-packages/issues/3735)) ([bd5614c](https://github.com/AztecProtocol/aztec-packages/commit/bd5614c2ee04065e149d3df48f1ace9c0ce3858f))


### Miscellaneous

* Remove HashToField128Security ACIR opcode ([#3631](https://github.com/AztecProtocol/aztec-packages/issues/3631)) ([1d6d3c9](https://github.com/AztecProtocol/aztec-packages/commit/1d6d3c94f327de1f20ef7d78302d3957db70019e))
* Use simple "flat" CRS. ([#3748](https://github.com/AztecProtocol/aztec-packages/issues/3748)) ([5c6c2ca](https://github.com/AztecProtocol/aztec-packages/commit/5c6c2caf212fb22856df41fd15464dda37e10dab))

## [0.16.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.8...barretenberg.js-v0.16.9) (2023-12-13)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.7...barretenberg.js-v0.16.8) (2023-12-13)


### Bug Fixes

* Aztec sandbox compose fixes ([#3634](https://github.com/AztecProtocol/aztec-packages/issues/3634)) ([765a19c](https://github.com/AztecProtocol/aztec-packages/commit/765a19c3aad3a2793a764b970b7cc8a819094da7))
* Top level init bb.js, but better scoped imports to not incur cost too early ([#3629](https://github.com/AztecProtocol/aztec-packages/issues/3629)) ([cea862d](https://github.com/AztecProtocol/aztec-packages/commit/cea862dd7feec714a34eba6a3cf7a2a174a59a1b))

## [0.16.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.6...barretenberg.js-v0.16.7) (2023-12-06)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.5...barretenberg.js-v0.16.6) (2023-12-06)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.4...barretenberg.js-v0.16.5) (2023-12-06)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.3...barretenberg.js-v0.16.4) (2023-12-05)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.2...barretenberg.js-v0.16.3) (2023-12-05)


### Miscellaneous

* CLI's startup time was pushing almost 2s. This gets the basic 'help' down to 0.16. ([#3529](https://github.com/AztecProtocol/aztec-packages/issues/3529)) ([396df13](https://github.com/AztecProtocol/aztec-packages/commit/396df13389cdcb8b8b0d5a92a4b3d1c2bffcb7a7))

## [0.16.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.1...barretenberg.js-v0.16.2) (2023-12-05)


### Miscellaneous

* Optimise bb.js package size and sandox/cli dockerfiles to unbloat final containers. ([#3462](https://github.com/AztecProtocol/aztec-packages/issues/3462)) ([cb3db5d](https://github.com/AztecProtocol/aztec-packages/commit/cb3db5d0f1f8912f1a97258e5043eb0f69eff551))
* Pin node version in docker base images and bump nvmrc ([#3537](https://github.com/AztecProtocol/aztec-packages/issues/3537)) ([5d3895a](https://github.com/AztecProtocol/aztec-packages/commit/5d3895aefb7812eb6bd8017baf43533959ad69b4))

## [0.16.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.16.0...barretenberg.js-v0.16.1) (2023-11-28)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.16.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.15.1...barretenberg.js-v0.16.0) (2023-11-27)


### Miscellaneous

* Plumbs noir subrepo into yarn-project. ([#3420](https://github.com/AztecProtocol/aztec-packages/issues/3420)) ([63173c4](https://github.com/AztecProtocol/aztec-packages/commit/63173c45db127288bc4b079229239a650fc5d4be))

## [0.15.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.15.0...barretenberg.js-v0.15.1) (2023-11-21)


### Features

* **bb:** Add ability to write pk to file or stdout ([#3335](https://github.com/AztecProtocol/aztec-packages/issues/3335)) ([c99862c](https://github.com/AztecProtocol/aztec-packages/commit/c99862c9602d7d37f7fef348e9f014fb137adab1))


### Miscellaneous

* All hashes in ts ([#3333](https://github.com/AztecProtocol/aztec-packages/issues/3333)) ([6307e12](https://github.com/AztecProtocol/aztec-packages/commit/6307e129770af7791dc5a477859b75ebb112a653))

## [0.15.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.14.2...barretenberg.js-v0.15.0) (2023-11-16)


### ⚠ BREAKING CHANGES

* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114))

### Bug Fixes

* Fix block constraint key divergence bug. ([#3256](https://github.com/AztecProtocol/aztec-packages/issues/3256)) ([1c71a0c](https://github.com/AztecProtocol/aztec-packages/commit/1c71a0cf38cf463efe1964126a6a5741c27bd2eb))


### Miscellaneous

* Replace computing hashes in circuits wasm, with computing them in ts via bb.js pedersen call. ([#3114](https://github.com/AztecProtocol/aztec-packages/issues/3114)) ([87eeb71](https://github.com/AztecProtocol/aztec-packages/commit/87eeb715014996ec329de969df85684083b18f83))
* Typo fixes based on cspell ([#3319](https://github.com/AztecProtocol/aztec-packages/issues/3319)) ([8ae44dd](https://github.com/AztecProtocol/aztec-packages/commit/8ae44dd702987db524ab5e3edd6545881614f56b))

## [0.14.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.14.1...barretenberg.js-v0.14.2) (2023-11-07)


### Features

* Run solidity tests for all acir artifacts ([#3161](https://github.com/AztecProtocol/aztec-packages/issues/3161)) ([d09f667](https://github.com/AztecProtocol/aztec-packages/commit/d09f66748fcbb7739b17940a36806abb72091ee1))

## [0.14.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.14.0...barretenberg.js-v0.14.1) (2023-11-07)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.14.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.13.1...barretenberg.js-v0.14.0) (2023-11-07)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.13.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.13.0...barretenberg.js-v0.13.1) (2023-10-31)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.13.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.12.0...barretenberg.js-v0.13.0) (2023-10-31)


### Features

* New script to output table of benchmarks for README pasting. ([#2780](https://github.com/AztecProtocol/aztec-packages/issues/2780)) ([6c20b45](https://github.com/AztecProtocol/aztec-packages/commit/6c20b45993ee9cbd319ab8351e2722e0c912f427))


### Miscellaneous

* Automatic c_binds for commit should return a point instead of an Fr element ([#3072](https://github.com/AztecProtocol/aztec-packages/issues/3072)) ([2e289a5](https://github.com/AztecProtocol/aztec-packages/commit/2e289a5d11d28496ac47220bede03268065e0cb7))
* Remove unecessary calls to `pedersen__init` ([#3079](https://github.com/AztecProtocol/aztec-packages/issues/3079)) ([84f8db2](https://github.com/AztecProtocol/aztec-packages/commit/84f8db20f482242ac29a23eb4c8876f14f060b4c))
* Remove unused pedersen c_binds ([#3058](https://github.com/AztecProtocol/aztec-packages/issues/3058)) ([e71e5f9](https://github.com/AztecProtocol/aztec-packages/commit/e71e5f94ba920208e7cc9b2b1b9d62678b699812))

## [0.12.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.11.1...barretenberg.js-v0.12.0) (2023-10-26)


### ⚠ BREAKING CHANGES

* remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033))

### Miscellaneous

* Proxy redundant `hash` methods ([#3046](https://github.com/AztecProtocol/aztec-packages/issues/3046)) ([df389b5](https://github.com/AztecProtocol/aztec-packages/commit/df389b5f593a202bc644479a6c3dff884b7d3652))
* Remove `pedersen_buffer_to_field` from c_bind ([#3045](https://github.com/AztecProtocol/aztec-packages/issues/3045)) ([de7e63b](https://github.com/AztecProtocol/aztec-packages/commit/de7e63bf7e1184333c1eaadf2387fef6bf163871))
* Remove plookup pedersen methods from c_bind namespace ([#3033](https://github.com/AztecProtocol/aztec-packages/issues/3033)) ([a8ea391](https://github.com/AztecProtocol/aztec-packages/commit/a8ea391c95a9fe4fa26a3fa987f52114a40c664a))
* Rename pedersen typescript methods to be called `hash` instead of compress ([#3047](https://github.com/AztecProtocol/aztec-packages/issues/3047)) ([2f7cc5f](https://github.com/AztecProtocol/aztec-packages/commit/2f7cc5fd3242b04fa996b71dbd7282444e82e903))

## [0.11.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.11.0...barretenberg.js-v0.11.1) (2023-10-24)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.11.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.10.1...barretenberg.js-v0.11.0) (2023-10-24)


### Features

* Pedersen hash in acir format ([#2990](https://github.com/AztecProtocol/aztec-packages/issues/2990)) ([2a4c548](https://github.com/AztecProtocol/aztec-packages/commit/2a4c548bc816a5f379ee841e26bb30411deef56b))

## [0.10.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.10.0...barretenberg.js-v0.10.1) (2023-10-24)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.10.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.9.0...barretenberg.js-v0.10.0) (2023-10-24)


### Features

* Refactor pedersen hash standard ([#2592](https://github.com/AztecProtocol/aztec-packages/issues/2592)) ([3085676](https://github.com/AztecProtocol/aztec-packages/commit/3085676dd8a68ac43abc3e5c7843ff437df91d7d))

## [0.9.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.14...barretenberg.js-v0.9.0) (2023-10-17)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.14](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.13...barretenberg.js-v0.8.14) (2023-10-13)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.13](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.12...barretenberg.js-v0.8.13) (2023-10-13)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.12](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.11...barretenberg.js-v0.8.12) (2023-10-13)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.11](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.10...barretenberg.js-v0.8.11) (2023-10-13)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.10](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.9...barretenberg.js-v0.8.10) (2023-10-11)


### Features

* Adding Fr back as a BB export (ts) ([#2770](https://github.com/AztecProtocol/aztec-packages/issues/2770)) ([d9ac808](https://github.com/AztecProtocol/aztec-packages/commit/d9ac8080a5525b9792b7b3f10c40583536bb256c))

## [0.8.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.8...barretenberg.js-v0.8.9) (2023-10-10)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.7...barretenberg.js-v0.8.8) (2023-10-09)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.6...barretenberg.js-v0.8.7) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.5...barretenberg.js-v0.8.6) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.4...barretenberg.js-v0.8.5) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.3...barretenberg.js-v0.8.4) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.2...barretenberg.js-v0.8.3) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.1...barretenberg.js-v0.8.2) (2023-10-04)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.8.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.8.0...barretenberg.js-v0.8.1) (2023-10-03)


### Bug Fixes

* Remove -u from build_wasm script so that we can skip the build when SKIP_CPP_BUILD is unset ([#2649](https://github.com/AztecProtocol/aztec-packages/issues/2649)) ([84b8ff4](https://github.com/AztecProtocol/aztec-packages/commit/84b8ff4b46e1f542209c1f35a33b7cffdc083f04))

## [0.8.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.10...barretenberg.js-v0.8.0) (2023-10-03)


### ⚠ BREAKING CHANGES

* Gates command should always return 8 bytes ([#2631](https://github.com/AztecProtocol/aztec-packages/issues/2631))

### Bug Fixes

* Gates command should always return 8 bytes ([#2631](https://github.com/AztecProtocol/aztec-packages/issues/2631)) ([9668165](https://github.com/AztecProtocol/aztec-packages/commit/9668165372c4f5170aa7c4f161e031da0c845649))


### Miscellaneous

* Provide cross compile to cjs. ([#2566](https://github.com/AztecProtocol/aztec-packages/issues/2566)) ([47d0d37](https://github.com/AztecProtocol/aztec-packages/commit/47d0d376727dfcb798af4ea019dfc23a9a57b6ca))
* Remove `BarretenbergBinderSync` import from typescript bindgen file ([#2607](https://github.com/AztecProtocol/aztec-packages/issues/2607)) ([43af1a3](https://github.com/AztecProtocol/aztec-packages/commit/43af1a35c1bbe55cab102bef21375dd9986202ea))
* Typo ([#2546](https://github.com/AztecProtocol/aztec-packages/issues/2546)) ([8656a3b](https://github.com/AztecProtocol/aztec-packages/commit/8656a3b1f4fce63c3acaed6e81ae77632df05ef5))

## [0.7.10](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.9...barretenberg.js-v0.7.10) (2023-09-20)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.9](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.8...barretenberg.js-v0.7.9) (2023-09-19)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.8](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.7...barretenberg.js-v0.7.8) (2023-09-19)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.6...barretenberg.js-v0.7.7) (2023-09-18)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.5...barretenberg.js-v0.7.6) (2023-09-18)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.4...barretenberg.js-v0.7.5) (2023-09-15)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.3...barretenberg.js-v0.7.4) (2023-09-15)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.2...barretenberg.js-v0.7.3) (2023-09-15)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.1...barretenberg.js-v0.7.2) (2023-09-14)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.7.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.7.0...barretenberg.js-v0.7.1) (2023-09-14)


### Miscellaneous

* Move barretenberg to top of repo. Make circuits build off barretenberg build. ([#2221](https://github.com/AztecProtocol/aztec-packages/issues/2221)) ([404ec34](https://github.com/AztecProtocol/aztec-packages/commit/404ec34d38e1a9c3fbe7a3cdb6e88c28f62f72e4))

## [0.7.0](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.7...barretenberg.js-v0.7.0) (2023-09-13)


### Bug Fixes

* Add cjs-entry to bbjs package files ([#2237](https://github.com/AztecProtocol/aztec-packages/issues/2237)) ([ae16193](https://github.com/AztecProtocol/aztec-packages/commit/ae16193b3cdb2da3d57a1c74f7e71f139ced54d1))


### Miscellaneous

* Add debugging to run_tests ([#2212](https://github.com/AztecProtocol/aztec-packages/issues/2212)) ([1c5e78a](https://github.com/AztecProtocol/aztec-packages/commit/1c5e78a4ac01bee4b785857447efdb02d8d9cb35))

## [0.6.7](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.6...barretenberg.js-v0.6.7) (2023-09-11)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.6](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.5...barretenberg.js-v0.6.6) (2023-09-11)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.5](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.4...barretenberg.js-v0.6.5) (2023-09-08)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.4](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.3...barretenberg.js-v0.6.4) (2023-09-08)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.3](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.2...barretenberg.js-v0.6.3) (2023-09-08)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.6.1...barretenberg.js-v0.6.2) (2023-09-08)


### Miscellaneous

* **barretenberg.js:** Synchronize aztec-packages versions

## [0.6.1](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.5.2...barretenberg.js-v0.6.1) (2023-09-08)


### Miscellaneous

* **master:** Release 0.5.2 ([#2141](https://github.com/AztecProtocol/aztec-packages/issues/2141)) ([451aad6](https://github.com/AztecProtocol/aztec-packages/commit/451aad6ea92ebced9839ca14baae10cee327be35))
* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))
* Release 0.6.1 ([1bd1a79](https://github.com/AztecProtocol/aztec-packages/commit/1bd1a79b0cefcd90306133aab141d992e8ea5fc3))

## [0.5.2](https://github.com/AztecProtocol/aztec-packages/compare/barretenberg.js-v0.5.2...barretenberg.js-v0.5.2) (2023-09-08)


### Miscellaneous

* Release 0.5.2 ([f76b53c](https://github.com/AztecProtocol/aztec-packages/commit/f76b53c985116ac131a9b11b2a255feb7d0f8f13))

## [0.5.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.5.0...barretenberg.js-v0.5.1) (2023-09-05)


### Features

* Add `info` command to bb ([#2010](https://github.com/AztecProtocol/barretenberg/issues/2010)) ([2882d97](https://github.com/AztecProtocol/barretenberg/commit/2882d97f5165239badb328be80568e7d683c0465))
* **ci:** Use content hash in build system, restrict docs build to *.ts or *.cpp ([#1953](https://github.com/AztecProtocol/barretenberg/issues/1953)) ([297a20d](https://github.com/AztecProtocol/barretenberg/commit/297a20d7878a4aabab1cabf2cc5d2d67f9e969c5))


### Bug Fixes

* **bb.js:** (breaking change) bundles bb.js properly so that it works in the browser and in node ([#1855](https://github.com/AztecProtocol/barretenberg/issues/1855)) ([bc93a5f](https://github.com/AztecProtocol/barretenberg/commit/bc93a5f8510d0dc600343e7e613ab84380d3c225))
* **ci:** Incorrect content hash in some build targets ([#1973](https://github.com/AztecProtocol/barretenberg/issues/1973)) ([c6c469a](https://github.com/AztecProtocol/barretenberg/commit/c6c469aa5da7c6973f656ddf8af4fb20c3e8e4f6))
* Master ([#1981](https://github.com/AztecProtocol/barretenberg/issues/1981)) ([59a454e](https://github.com/AztecProtocol/barretenberg/commit/59a454ecf1611424893e1cb093774a23dde39310))

## [0.5.0](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.6...barretenberg.js-v0.5.0) (2023-09-01)


### ⚠ BREAKING CHANGES

* update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/barretenberg/issues/1925))

### Miscellaneous Chores

* Update to acvm 0.24.0 ([#1925](https://github.com/AztecProtocol/barretenberg/issues/1925)) ([5d8db8e](https://github.com/AztecProtocol/barretenberg/commit/5d8db8eb993334b43e24a51efba9c59e123320ab))

## [0.4.6](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.5...barretenberg.js-v0.4.6) (2023-08-29)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.4.5](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.4...barretenberg.js-v0.4.5) (2023-08-28)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.4.4](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.3...barretenberg.js-v0.4.4) (2023-08-28)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.4.3](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.2...barretenberg.js-v0.4.3) (2023-08-23)


### Bug Fixes

* Download SRS using one canonical URL across the codebase ([#1748](https://github.com/AztecProtocol/barretenberg/issues/1748)) ([5c91de7](https://github.com/AztecProtocol/barretenberg/commit/5c91de7296e054f6d5ac3dca94ca85e06d496048))
* Proving fails when circuit has size &gt; ~500K ([#1739](https://github.com/AztecProtocol/barretenberg/issues/1739)) ([6d32383](https://github.com/AztecProtocol/barretenberg/commit/6d323838a525190618d608598357ee4608c46699))

## [0.4.2](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.1...barretenberg.js-v0.4.2) (2023-08-21)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.4.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.4.0...barretenberg.js-v0.4.1) (2023-08-21)


### Bug Fixes

* **bb:** Fix Typo ([#1709](https://github.com/AztecProtocol/barretenberg/issues/1709)) ([286d64e](https://github.com/AztecProtocol/barretenberg/commit/286d64e6036336314114f1d2a25273f4dabe36f4))

## [0.4.0](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.6...barretenberg.js-v0.4.0) (2023-08-21)


### ⚠ BREAKING CHANGES

* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/barretenberg/issues/1618))

### Bug Fixes

* Barretenberg binaries now take in the encoded circuit instead of a json file ([#1618](https://github.com/AztecProtocol/barretenberg/issues/1618)) ([180cdc9](https://github.com/AztecProtocol/barretenberg/commit/180cdc9ac7cf9aa793d9774dc866ceb4e6ec3fbc))
* Bin reference when installing package ([#678](https://github.com/AztecProtocol/barretenberg/issues/678)) ([c734295](https://github.com/AztecProtocol/barretenberg/commit/c734295a10d2c40ede773519664170880f28b2b7))
* Sync aztec master ([#680](https://github.com/AztecProtocol/barretenberg/issues/680)) ([3afc243](https://github.com/AztecProtocol/barretenberg/commit/3afc2438053f530e49fbebbdbadd8db8a630bb8c))

## [0.3.6](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.5...barretenberg.js-v0.3.6) (2023-08-08)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.3.5](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.4...barretenberg.js-v0.3.5) (2023-08-07)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.3.4](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.3...barretenberg.js-v0.3.4) (2023-07-25)


### Features

* Modify bb.js to be compatible with next.js ([#544](https://github.com/AztecProtocol/barretenberg/issues/544)) ([d384089](https://github.com/AztecProtocol/barretenberg/commit/d384089f60d1a6d5baeb0d3459556a310b790366))

## [0.3.3](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.2...barretenberg.js-v0.3.3) (2023-07-17)


### Features

* Bb and bb.js directly parse nargo bincode format. ([#610](https://github.com/AztecProtocol/barretenberg/issues/610)) ([d25e37a](https://github.com/AztecProtocol/barretenberg/commit/d25e37ad74b88dc45337b2a529ede3136dd4a699))

## [0.3.2](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.1...barretenberg.js-v0.3.2) (2023-07-12)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## [0.3.1](https://github.com/AztecProtocol/barretenberg/compare/barretenberg.js-v0.3.0...barretenberg.js-v0.3.1) (2023-07-11)


### Miscellaneous Chores

* **barretenberg.js:** Synchronize barretenberg versions

## 0.3.0 (2023-07-11)


### Features

* **bb.js:** initial API ([#232](https://github.com/AztecProtocol/barretenberg/issues/232)) ([c860b02](https://github.com/AztecProtocol/barretenberg/commit/c860b02d80425de161af50acf33e94d94eb0659c))
* **dsl:** Add ECDSA secp256r1 verification ([#582](https://github.com/AztecProtocol/barretenberg/issues/582)) ([adc4c7b](https://github.com/AztecProtocol/barretenberg/commit/adc4c7b4eb634eae28dd28e25b94b93a5b49c80e))
* Initial native version of bb binary. ([#524](https://github.com/AztecProtocol/barretenberg/issues/524)) ([4a1b532](https://github.com/AztecProtocol/barretenberg/commit/4a1b5322dc78921d253e6a374eba0b616ab788df))
* Optimize memory consumption of pedersen generators ([#413](https://github.com/AztecProtocol/barretenberg/issues/413)) ([d60b16a](https://github.com/AztecProtocol/barretenberg/commit/d60b16a14219fd4bd130ce4537c3e94bfa10128f))
* **ts:** allow passing srs via env functions ([#260](https://github.com/AztecProtocol/barretenberg/issues/260)) ([ac78353](https://github.com/AztecProtocol/barretenberg/commit/ac7835304f4524039abf0a0df9ae85d905f55c86))


### Bug Fixes

* **build:** git add -f .yalc ([#265](https://github.com/AztecProtocol/barretenberg/issues/265)) ([7671192](https://github.com/AztecProtocol/barretenberg/commit/7671192c8a60ff0bc0f8ad3e14ac299ff780cc25))
* bump timeout on common test. ([c9bc87d](https://github.com/AztecProtocol/barretenberg/commit/c9bc87d29fa1325162cb1e7bf2db7cc85747fd9e))
* Trigger release-please ([#594](https://github.com/AztecProtocol/barretenberg/issues/594)) ([5042861](https://github.com/AztecProtocol/barretenberg/commit/5042861405df6b5659c0c32418720d8bdea81081))
