# Changelog

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
