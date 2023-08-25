# Changelog

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
