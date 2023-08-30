# Changelog

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
