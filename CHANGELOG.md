# Changelog

## [0.10.3](https://github.com/noir-lang/noir/compare/v0.10.2...v0.10.3) (2023-08-16)


### Features

* Allow calling higher-order functions with closures ([#2335](https://github.com/noir-lang/noir/issues/2335)) ([75fd3e0](https://github.com/noir-lang/noir/commit/75fd3e0e27f16fb0aa5f8b01cefe78e04f867726))
* **lsp:** Add `Compile` code lens for `main` function and contracts ([#2309](https://github.com/noir-lang/noir/issues/2309)) ([5fe69c6](https://github.com/noir-lang/noir/commit/5fe69c6eeef0b7ed2e4df9c3a80627f54c75a355))
* **lsp:** Add `Execute` code lens for `main` functions ([#2330](https://github.com/noir-lang/noir/issues/2330)) ([5aa59e0](https://github.com/noir-lang/noir/commit/5aa59e0f3c4b3e6e14330d1f0e45ec912f562892))


### Bug Fixes

* Display warning if last expression of block is unused ([#2314](https://github.com/noir-lang/noir/issues/2314)) ([8110136](https://github.com/noir-lang/noir/commit/81101362ccba787a44c6d48c0378696cb16f0acb))

## [0.10.2](https://github.com/noir-lang/noir/compare/v0.10.1...v0.10.2) (2023-08-16)


### Bug Fixes

* Prevent dead instruction elimination of brillig functions which may contain side-effects ([#2340](https://github.com/noir-lang/noir/issues/2340)) ([ba8ffd8](https://github.com/noir-lang/noir/commit/ba8ffd84c19b3516334c126bc2f25c725985baa3))

## [0.10.1](https://github.com/noir-lang/noir/compare/v0.10.0...v0.10.1) (2023-08-15)


### Features

* Add full call stacks to runtime errors ([#2310](https://github.com/noir-lang/noir/issues/2310)) ([9004181](https://github.com/noir-lang/noir/commit/900418192216dc2657d6ffe48f85ac82411fb054))
* Improved error message for unexpected return type ([#2302](https://github.com/noir-lang/noir/issues/2302)) ([d7e1e65](https://github.com/noir-lang/noir/commit/d7e1e658fe09443ae37f34e3fc6a8cb7765cf1d9))
* **ssa:** Perform dead instruction elimination on intrinsic functions ([#2276](https://github.com/noir-lang/noir/issues/2276)) ([3fe3f8c](https://github.com/noir-lang/noir/commit/3fe3f8ca11d646a054f36e6939211a22f79f10f1))
* **ssa:** Switch mem2reg pass to be per function rather than per block ([#2243](https://github.com/noir-lang/noir/issues/2243)) ([0d548b9](https://github.com/noir-lang/noir/commit/0d548b9b27710de231759c34e1a198c9991d33ef))
* **stdlib:** Implement `str` `as_bytes` and `into_bytes` function ([#2298](https://github.com/noir-lang/noir/issues/2298)) ([92549d4](https://github.com/noir-lang/noir/commit/92549d432470244ff7e8581fbe02c744c88942d9))

## [0.10.0](https://github.com/noir-lang/noir/compare/v0.9.0...v0.10.0) (2023-08-15)


### ⚠ BREAKING CHANGES

* **nargo:** Remove `-p` short flag from the `--program-dir` flag ([#2300](https://github.com/noir-lang/noir/issues/2300))
* **nargo:** Replace `--contracts` flag with `contract` package type ([#2204](https://github.com/noir-lang/noir/issues/2204))
* **nargo:** remove `flat_witness` feature flag ([#2208](https://github.com/noir-lang/noir/issues/2208))
* **nargo:** Require package `type` be specified in Nargo.toml ([#2134](https://github.com/noir-lang/noir/issues/2134))
* Allow specifying new package name with `--name` flag ([#2144](https://github.com/noir-lang/noir/issues/2144))
* **nargo:** Remove unused flags on LSP command ([#2170](https://github.com/noir-lang/noir/issues/2170))
* Support workspaces and package selection on every nargo command ([#1992](https://github.com/noir-lang/noir/issues/1992))
* **nargo:** Require package names in Nargo.toml files ([#2056](https://github.com/noir-lang/noir/issues/2056))
* Update to ACVM 0.21.0 ([#2051](https://github.com/noir-lang/noir/issues/2051))
* Drop support for the legacy SSA ([#2049](https://github.com/noir-lang/noir/issues/2049))
* **nargo:** Rename nargo gates to nargo info ([#2038](https://github.com/noir-lang/noir/issues/2038))
* **nargo:** Default to new SSA code for compilation

### Features

* **acir_gen:** RecursiveAggregation opcode and updates to black box func call generation ([#2097](https://github.com/noir-lang/noir/issues/2097)) ([5cb8166](https://github.com/noir-lang/noir/commit/5cb816664e03992a766ba9dcb2650e9596fbb291))
* Add `assert_constant` ([#2242](https://github.com/noir-lang/noir/issues/2242)) ([a72daa4](https://github.com/noir-lang/noir/commit/a72daa4764310078ab0c70720f16c85b2c0375bb))
* Add `deprecated` attribute ([#2041](https://github.com/noir-lang/noir/issues/2041)) ([9e2cf6f](https://github.com/noir-lang/noir/commit/9e2cf6f25f775d927b67c12aba1698c5635242e3))
* Add `Option&lt;T&gt;` to noir stdlib ([#1781](https://github.com/noir-lang/noir/issues/1781)) ([920a900](https://github.com/noir-lang/noir/commit/920a900818b31285c9bf2f5dd5b84c2799610a7c))
* Add basic benchmarking ([#2213](https://github.com/noir-lang/noir/issues/2213)) ([c8fe617](https://github.com/noir-lang/noir/commit/c8fe6175fa69abdfa29d7a9e1c5c653af5f15f1d))
* Add slice append ([#2241](https://github.com/noir-lang/noir/issues/2241)) ([90c5d18](https://github.com/noir-lang/noir/commit/90c5d182b578b6d512e4b5dc2c07810e6815f31e))
* Add support for bitshifts by distances known at runtime ([#2072](https://github.com/noir-lang/noir/issues/2072)) ([b0fbc53](https://github.com/noir-lang/noir/commit/b0fbc536dc432ba8d3ab6c12462758b11c2c21c4))
* Add support for slices of structs and nested slices in brillig ([#2084](https://github.com/noir-lang/noir/issues/2084)) ([620517f](https://github.com/noir-lang/noir/commit/620517f6527a7b5173dccc16e200ce349f489998))
* allow returning nested arrays from brillig ([#2047](https://github.com/noir-lang/noir/issues/2047)) ([4378bb8](https://github.com/noir-lang/noir/commit/4378bb85bf2900e7ab13856cadc764fd4a80bff5))
* Allow specifying new package name with `--name` flag ([#2144](https://github.com/noir-lang/noir/issues/2144)) ([e932599](https://github.com/noir-lang/noir/commit/e932599b1187fbf426b73c364626d1b17348a55e))
* Drop support for the legacy SSA ([#2049](https://github.com/noir-lang/noir/issues/2049)) ([3f33e44](https://github.com/noir-lang/noir/commit/3f33e447fbd6f1b94ff9935e21905c68c1dc9c83))
* Execute brillig opcodes with constant inputs at compile-time ([#2190](https://github.com/noir-lang/noir/issues/2190)) ([79af8e6](https://github.com/noir-lang/noir/commit/79af8e6fd359723716395913b23057beddcbdb83))
* Format strings for prints  ([#1952](https://github.com/noir-lang/noir/issues/1952)) ([3c82721](https://github.com/noir-lang/noir/commit/3c827217900d19a710ee8a49d782ed3d43a6336c))
* Implement traits - parser support [#2094](https://github.com/noir-lang/noir/issues/2094) ([#2230](https://github.com/noir-lang/noir/issues/2230)) ([589f173](https://github.com/noir-lang/noir/commit/589f173a85dabb68ada248e5c44fc0e13c6eb0f3))
* Implement type aliases ([#2112](https://github.com/noir-lang/noir/issues/2112)) ([ce94cb4](https://github.com/noir-lang/noir/commit/ce94cb4f9f9fccf504de9d0b12b8760fc8fab75c))
* Include struct names in ABIs ([#2266](https://github.com/noir-lang/noir/issues/2266)) ([9824ca5](https://github.com/noir-lang/noir/commit/9824ca567c6c151c0cc5469be402ffba2cbaa9cc))
* Issue warning for signed integers ([#2185](https://github.com/noir-lang/noir/issues/2185)) ([1be1bcc](https://github.com/noir-lang/noir/commit/1be1bcc5b40374c76f0b8e4d717e9292e15457f5))
* Make arrays and slices polymorphic over each other ([#2070](https://github.com/noir-lang/noir/issues/2070)) ([ef91286](https://github.com/noir-lang/noir/commit/ef91286b920fb3e17c7368839a93ccad2441edc8))
* **nargo:** Add `--exact` flag to `nargo test` ([#2272](https://github.com/noir-lang/noir/issues/2272)) ([1ad9199](https://github.com/noir-lang/noir/commit/1ad9199bcfbc5dd52166038a25ddfc7b03d90981))
* **nargo:** Add `--workspace` flag to run commands in every package ([#2313](https://github.com/noir-lang/noir/issues/2313)) ([d6deb0c](https://github.com/noir-lang/noir/commit/d6deb0c96bf8a12e21881bf10bbd139bc6531796))
* **nargo:** Add support for contracts in `nargo check` ([#2267](https://github.com/noir-lang/noir/issues/2267)) ([3d1b252](https://github.com/noir-lang/noir/commit/3d1b2522c8d9a0acebff102f9f877c44178c5db5))
* **nargo:** Default to new SSA code for compilation ([ce37718](https://github.com/noir-lang/noir/commit/ce377186a1d6afa025bd88d7436f61319c30cc33))
* **nargo:** Replace `--contracts` flag with `contract` package type ([#2204](https://github.com/noir-lang/noir/issues/2204)) ([968e12c](https://github.com/noir-lang/noir/commit/968e12c896f3fc910f36063b5afa542756ea95db))
* **nargo:** Require package `type` be specified in Nargo.toml ([#2134](https://github.com/noir-lang/noir/issues/2134)) ([1c991d0](https://github.com/noir-lang/noir/commit/1c991d0f0eac9270eb218b9ad672e36e8af74bc9))
* **nargo:** Support custom entry points specified in TOML ([#2158](https://github.com/noir-lang/noir/issues/2158)) ([effb02a](https://github.com/noir-lang/noir/commit/effb02afc78f379d023719a0d869f42e7109b05f))
* Only create new witnesses for distinctiveness when duplicates exist ([#2191](https://github.com/noir-lang/noir/issues/2191)) ([14cbdbc](https://github.com/noir-lang/noir/commit/14cbdbc1055ce7efe5d31bb02707c9a601ee7745))
* open functions are unconstrained ([be44c7b](https://github.com/noir-lang/noir/commit/be44c7be172b93ebaf74719b870fc9cc3bc24105))
* Optimize `x &lt; 0` for unsigned `x` to false ([#2206](https://github.com/noir-lang/noir/issues/2206)) ([25bc969](https://github.com/noir-lang/noir/commit/25bc9698efee601f5d8d4531a1bece8e5dc293ab))
* Optimize away constant calls to black box functions ([#1981](https://github.com/noir-lang/noir/issues/1981)) ([47b372c](https://github.com/noir-lang/noir/commit/47b372c1762ed1184bf2ed9b90d7dc3e2c161880))
* Optimize equality checks between a boolean and constant ([#2201](https://github.com/noir-lang/noir/issues/2201)) ([478c026](https://github.com/noir-lang/noir/commit/478c0266cc267b942f7ff10d32fffdeb6affa140))
* Optionally output a debug artifact on compile ([#2260](https://github.com/noir-lang/noir/issues/2260)) ([edded24](https://github.com/noir-lang/noir/commit/edded24d2256a074e8e390285e123e39f926551d))
* Perform input validation on user's package names ([#2293](https://github.com/noir-lang/noir/issues/2293)) ([87174ac](https://github.com/noir-lang/noir/commit/87174ac4927c4e237a2d0dbd6290da309e9f70c0))
* Perform sorting of constant arrays at compile time ([#2195](https://github.com/noir-lang/noir/issues/2195)) ([c46d7a0](https://github.com/noir-lang/noir/commit/c46d7a01ca49bb47548df6f3b2aa25d35aa43360))
* Remove `comptime` and warn upon usage ([#2178](https://github.com/noir-lang/noir/issues/2178)) ([98d0de3](https://github.com/noir-lang/noir/commit/98d0de3814eb228f38c2985be99095e1db564065))
* Remove an unnecessary witness in `mul_with_witness` ([#2078](https://github.com/noir-lang/noir/issues/2078)) ([9f3198e](https://github.com/noir-lang/noir/commit/9f3198efc77c308028f761175da4fe3659f70579))
* replace boolean `AND`s with multiplication ([#1954](https://github.com/noir-lang/noir/issues/1954)) ([435ab35](https://github.com/noir-lang/noir/commit/435ab3520d06b6b4f898d41a5ad403c5ddbd7771))
* **ssa:** Add additional BinaryOp simplifications ([#2124](https://github.com/noir-lang/noir/issues/2124)) ([50b2816](https://github.com/noir-lang/noir/commit/50b2816099a021e4b8cb44a9017fb849abf014e6))
* Support `contract` package type in `nargo info` command ([#2249](https://github.com/noir-lang/noir/issues/2249)) ([d309cc0](https://github.com/noir-lang/noir/commit/d309cc0086df4c2a5697269ef9618cf026d323ff))
* Support workspaces and package selection on every nargo command ([#1992](https://github.com/noir-lang/noir/issues/1992)) ([940b189](https://github.com/noir-lang/noir/commit/940b189d4fd47dad8cc9f2650162da9e99c5024c))
* Update to ACVM 0.21.0 ([#2051](https://github.com/noir-lang/noir/issues/2051)) ([ad118eb](https://github.com/noir-lang/noir/commit/ad118eb8165ef83402e25b3001dfe27cf3a358b1))


### Bug Fixes

* Add foreign impl error ([#2216](https://github.com/noir-lang/noir/issues/2216)) ([a53f5ed](https://github.com/noir-lang/noir/commit/a53f5ed86ad9a372ecad8a0367f7af3a843aae56))
* Avoid non-determinism in defunctionalization ([#2069](https://github.com/noir-lang/noir/issues/2069)) ([898a9fa](https://github.com/noir-lang/noir/commit/898a9fa3328b24334e5fac1a8ae8d43570652599))
* avoid non-determinism in defunctionalize ([898a9fa](https://github.com/noir-lang/noir/commit/898a9fa3328b24334e5fac1a8ae8d43570652599))
* avoid potential panic in `two_complement` ([#2081](https://github.com/noir-lang/noir/issues/2081)) ([63c4da0](https://github.com/noir-lang/noir/commit/63c4da0586e2575d6d14a3e537ccb64863a13f78))
* Fix 3 parser test cases in parsing ([#2284](https://github.com/noir-lang/noir/issues/2284)) ([094aef1](https://github.com/noir-lang/noir/commit/094aef191a3eafeccba714823e43d8e73ede8f50))
* fix an ICE happening when we call a closure result from if/else ([#2146](https://github.com/noir-lang/noir/issues/2146)) ([928b3ad](https://github.com/noir-lang/noir/commit/928b3ad5d93943960cc6f480b28bce25f29b3271))
* Fix an ICE when reassigning a mutable lambda variable to one with a different environment type ([#2172](https://github.com/noir-lang/noir/issues/2172)) ([a56db3e](https://github.com/noir-lang/noir/commit/a56db3ec9b20de587735e2f002be5c355c6b6b83))
* Fix assignment when both `mut` and `&mut` are used ([#2264](https://github.com/noir-lang/noir/issues/2264)) ([b07a7ff](https://github.com/noir-lang/noir/commit/b07a7ff90445afa1f173934367ffaecd0878777c))
* Fix methods not mutating fields ([#2087](https://github.com/noir-lang/noir/issues/2087)) ([6acc242](https://github.com/noir-lang/noir/commit/6acc242bae48aee7e1de013ceadb6587dc900296))
* flattening pass no longer overwrites previously mapped condition values ([#2117](https://github.com/noir-lang/noir/issues/2117)) ([f7742ab](https://github.com/noir-lang/noir/commit/f7742ab026092f129bd4ec4f122bcd3249100529))
* **globals:** Accurately filter literals for resolving globals ([#2126](https://github.com/noir-lang/noir/issues/2126)) ([1c21d0c](https://github.com/noir-lang/noir/commit/1c21d0caf1e3b3a92266b4b8238f3e6e6c394d05))
* Implement `.len()` in Acir-Gen ([#2077](https://github.com/noir-lang/noir/issues/2077)) ([ab61e3a](https://github.com/noir-lang/noir/commit/ab61e3ab70aa0f7a037e0ad4a430975f50266097))
* Implement slices of structs ([#2150](https://github.com/noir-lang/noir/issues/2150)) ([6abcb79](https://github.com/noir-lang/noir/commit/6abcb792e510454896d032cea5017bd43ef8cfc3))
* Initialize numeric generics' type to a polymorphic integer when used in an expression ([#2179](https://github.com/noir-lang/noir/issues/2179)) ([c74b228](https://github.com/noir-lang/noir/commit/c74b22850ef0a530d0a3327c2bb3a8a05bd43bbb))
* **lsp:** Ensure lsp does not crawl past the root specified ([#2322](https://github.com/noir-lang/noir/issues/2322)) ([d69e372](https://github.com/noir-lang/noir/commit/d69e3728a22a31a7d170bf383ac9e65cc4cf61cc))
* **lsp:** Improve dependency resolution in context of `Nargo.toml` ([#2226](https://github.com/noir-lang/noir/issues/2226)) ([8846bf2](https://github.com/noir-lang/noir/commit/8846bf23364b6fdcb4e79171dffedddad5df91b6))
* **lsp:** Pass `--program-dir` to test command from codelens ([#2292](https://github.com/noir-lang/noir/issues/2292)) ([92e1802](https://github.com/noir-lang/noir/commit/92e1802979e5713ec4287d8932e4675c95439861))
* Mutating a variable no longer mutates its copy ([#2057](https://github.com/noir-lang/noir/issues/2057)) ([e85e485](https://github.com/noir-lang/noir/commit/e85e4850546552b7240466031e770c2667280444))
* **nargo:** Allow `--program-dir` flag anywhere in a command ([#2290](https://github.com/noir-lang/noir/issues/2290)) ([7834fce](https://github.com/noir-lang/noir/commit/7834fcee0bda8f72d97a65964605fd82742ea75f))
* **nargo:** Indicate which TOML file is missing package name ([#2177](https://github.com/noir-lang/noir/issues/2177)) ([9529157](https://github.com/noir-lang/noir/commit/9529157bd759d1ce1f632b732d76a58417ddfb51))
* **nargo:** Make dependencies section optional in TOML ([#2161](https://github.com/noir-lang/noir/issues/2161)) ([099f4d4](https://github.com/noir-lang/noir/commit/099f4d421e86c471343693d29e77beb1fb912a33))
* **nargo:** Remove `-p` short flag from the `--program-dir` flag ([#2300](https://github.com/noir-lang/noir/issues/2300)) ([cc2af74](https://github.com/noir-lang/noir/commit/cc2af74e586bbbba0c45aa0b7c9f9a9e6480f851))
* Open contract functions are unconstrained ([#2052](https://github.com/noir-lang/noir/issues/2052)) ([be44c7b](https://github.com/noir-lang/noir/commit/be44c7be172b93ebaf74719b870fc9cc3bc24105))
* optimize contracts built by `nargo info` ([b30b3f4](https://github.com/noir-lang/noir/commit/b30b3f438e8ed6953f2fec9c610619ac4fb17553))
* Optimize contracts built by `nargo info` ([#2259](https://github.com/noir-lang/noir/issues/2259)) ([b30b3f4](https://github.com/noir-lang/noir/commit/b30b3f438e8ed6953f2fec9c610619ac4fb17553))
* Overflowing assignment will result in an error ([#2321](https://github.com/noir-lang/noir/issues/2321)) ([bc645fc](https://github.com/noir-lang/noir/commit/bc645fcebb42858984ee0e7df560e40b56438512))
* Prevent panic when passing relative paths to `--program-dir` ([#2324](https://github.com/noir-lang/noir/issues/2324)) ([9eb45da](https://github.com/noir-lang/noir/commit/9eb45dafc7bef8e1714235e95d9e703c2b8c3c3b))
* properly capture lvalues in closure environments ([#2120](https://github.com/noir-lang/noir/issues/2120)) ([#2257](https://github.com/noir-lang/noir/issues/2257)) ([ed5273c](https://github.com/noir-lang/noir/commit/ed5273c827c5556f1b92e5ed8b628a0be77870be))
* remove duplicated `name` option in `nargo new` ([#2183](https://github.com/noir-lang/noir/issues/2183)) ([68f5887](https://github.com/noir-lang/noir/commit/68f5887f9083e8194a9252d09ee0af363ffffa03))
* Remove last vestige of array of structs to struct of arrays conversion ([#2217](https://github.com/noir-lang/noir/issues/2217)) ([34be264](https://github.com/noir-lang/noir/commit/34be264c0c112e9d0139654eabe4840f35535c1e))
* Rename `Option::value` to `Option::_value` ([#2127](https://github.com/noir-lang/noir/issues/2127)) ([8a1ace7](https://github.com/noir-lang/noir/commit/8a1ace792c4550ab1ce8c6044794abdb39d02872))
* Require package names to be non-empty ([#2293](https://github.com/noir-lang/noir/issues/2293)) ([87174ac](https://github.com/noir-lang/noir/commit/87174ac4927c4e237a2d0dbd6290da309e9f70c0))
* Set location before cast instructions in SSA ([#2202](https://github.com/noir-lang/noir/issues/2202)) ([a72cc96](https://github.com/noir-lang/noir/commit/a72cc96e7535f3b85db005f2b09014488933b4df))
* simplification of overflowing integer operations ([#2153](https://github.com/noir-lang/noir/issues/2153)) ([4a5d2de](https://github.com/noir-lang/noir/commit/4a5d2de23af112b9cb794a2e86caf313f860f8d3))
* **stdlib:** correct `tecurve::contains` formula ([#1821](https://github.com/noir-lang/noir/issues/1821)) ([6a10ecf](https://github.com/noir-lang/noir/commit/6a10ecf829a5c228b1e1e8a3e9ded886e53cad48))


### Miscellaneous Chores

* **nargo:** remove `flat_witness` feature flag ([#2208](https://github.com/noir-lang/noir/issues/2208)) ([32d52d3](https://github.com/noir-lang/noir/commit/32d52d36052b954b777e918d2cd67d056dd04232))
* **nargo:** Remove unused flags on LSP command ([#2170](https://github.com/noir-lang/noir/issues/2170)) ([ccba78e](https://github.com/noir-lang/noir/commit/ccba78e330463ea9eee00f745e0b489379059bd9))
* **nargo:** Rename nargo gates to nargo info ([#2038](https://github.com/noir-lang/noir/issues/2038)) ([5907e96](https://github.com/noir-lang/noir/commit/5907e96b8dded6eb3a68d5b9e167b055f65bf783))
* **nargo:** Require package names in Nargo.toml files ([#2056](https://github.com/noir-lang/noir/issues/2056)) ([bb28223](https://github.com/noir-lang/noir/commit/bb282232aec7b0b9dae08a062b586e4564036123))

## [0.9.0](https://github.com/noir-lang/noir/compare/v0.8.0...v0.9.0) (2023-07-25)


### ⚠ BREAKING CHANGES

* ACIR bytecode encoding with Base64 ([#1935](https://github.com/noir-lang/noir/issues/1935))
* Update to ACVM 0.18.1 and implement missing brillig blackboxes ([#1914](https://github.com/noir-lang/noir/issues/1914))

### Features

* ACIR bytecode encoding with Base64 ([#1935](https://github.com/noir-lang/noir/issues/1935)) ([347cfc4](https://github.com/noir-lang/noir/commit/347cfc4ce2ed463b457fce9a2530cad4b06516e0))
* Add `nargo build` as alias for `nargo compile` ([#1940](https://github.com/noir-lang/noir/issues/1940)) ([13618d4](https://github.com/noir-lang/noir/commit/13618d4bcc89079155a9fcadc3cbe2c07d2aa972))
* add `nargo init` command ([#1859](https://github.com/noir-lang/noir/issues/1859)) ([2d87c87](https://github.com/noir-lang/noir/commit/2d87c873a286b21741736ad61fbef546b6d42b21))
* Add ability to create a proof for a workspace member using `nargo prove -p {crate_name}` ([#1930](https://github.com/noir-lang/noir/issues/1930)) ([266126f](https://github.com/noir-lang/noir/commit/266126f89935ffe9abcecac709b7b06af36a5c95))
* Add Acir debug information ([#1864](https://github.com/noir-lang/noir/issues/1864)) ([5ff8b53](https://github.com/noir-lang/noir/commit/5ff8b53bbb4720241768acfcb76e9866214d43c2))
* Add multi-line comments ([#1936](https://github.com/noir-lang/noir/issues/1936)) ([cfb1765](https://github.com/noir-lang/noir/commit/cfb176562736207e5844ac16f0f941b4ee4e12d0))
* Add support for nested arrays on brillig gen ([#2029](https://github.com/noir-lang/noir/issues/2029)) ([8adc57c](https://github.com/noir-lang/noir/commit/8adc57c77ad0012d329684781d9cbee882d0b100))
* Add to_radix and to_bits support to brillig gen ([#2012](https://github.com/noir-lang/noir/issues/2012)) ([3eef41c](https://github.com/noir-lang/noir/commit/3eef41c752fabd1d0c989084f12cd82f81a6fa4c))
* Add unit literals and unit types to parser ([#1960](https://github.com/noir-lang/noir/issues/1960)) ([ea80de5](https://github.com/noir-lang/noir/commit/ea80de57a57a92533b3fb545a0920bca2d74e109))
* Adding internal keyword ([#1873](https://github.com/noir-lang/noir/issues/1873)) ([7a85493](https://github.com/noir-lang/noir/commit/7a854937ca5a300ae05f335612d2ff72ce88b4b1))
* Allow arrays of arbitrary types in the program ABI ([#1651](https://github.com/noir-lang/noir/issues/1651)) ([811ede1](https://github.com/noir-lang/noir/commit/811ede19f2160d809904deffc09a51799448d8d6))
* Allow shadowing by default ([#2000](https://github.com/noir-lang/noir/issues/2000)) ([88a4f74](https://github.com/noir-lang/noir/commit/88a4f74a36704137d7de94e3791c2e6bea9319b5))
* avoid unnecessary witness assignments in euclidian division / bound constraint  ([#1989](https://github.com/noir-lang/noir/issues/1989)) ([c23257d](https://github.com/noir-lang/noir/commit/c23257d4bdd8d93b9219fd767de6d806e237ccea))
* **brillig_gen:** Return slices from foreign calls ([#1909](https://github.com/noir-lang/noir/issues/1909)) ([6fa3144](https://github.com/noir-lang/noir/commit/6fa3144b30ef908a350273fbfd950d5a247104b2))
* compile to brillig reachable acir fns ([#1919](https://github.com/noir-lang/noir/issues/1919)) ([2b4237d](https://github.com/noir-lang/noir/commit/2b4237d7ffc2a0246cdaa1b7d85cc1ef7d7b3eb1))
* dynamic arrays for experimental-ssa ([#1969](https://github.com/noir-lang/noir/issues/1969)) ([08d199a](https://github.com/noir-lang/noir/commit/08d199aa4daa2038ca01f5ad23376fec27950f9a))
* Implement parsing of traits ([#1886](https://github.com/noir-lang/noir/issues/1886)) ([3ba1e72](https://github.com/noir-lang/noir/commit/3ba1e72408b5f15cc623a4b2ca9f5c2e4b9652ae))
* Implement references in brillig ([#1901](https://github.com/noir-lang/noir/issues/1901)) ([3a078fb](https://github.com/noir-lang/noir/commit/3a078fb9c5c5c256a767c8bd7f1312d07c8db93c))
* initial implementation of slices in brillig ([#1932](https://github.com/noir-lang/noir/issues/1932)) ([ea47936](https://github.com/noir-lang/noir/commit/ea47936cfea201aa634432c972b71e6b89cdb513))
* Refactor Logging to use Brillig foreign calls ([#1917](https://github.com/noir-lang/noir/issues/1917)) ([c15f9aa](https://github.com/noir-lang/noir/commit/c15f9aa8a7d21ec44e9b63e90cc83290ac96cd9c))
* **stdlib:** Add multiple slice builtins ([#1888](https://github.com/noir-lang/noir/issues/1888)) ([008a16b](https://github.com/noir-lang/noir/commit/008a16b799f494115f028e523f9daa54fd8f476f))
* **stdlib:** Add secp256r1 builtin function   ([#1858](https://github.com/noir-lang/noir/issues/1858)) ([f3800c5](https://github.com/noir-lang/noir/commit/f3800c52c81a27d3b52cfe23f45e764234b1c268))
* **stdlib:** Vec type ([#1905](https://github.com/noir-lang/noir/issues/1905)) ([3734e25](https://github.com/noir-lang/noir/commit/3734e2554661567a77e7a18d91134b2d521a5c06))
* Update to ACVM 0.18.1 and implement missing brillig blackboxes ([#1914](https://github.com/noir-lang/noir/issues/1914)) ([2bc7d25](https://github.com/noir-lang/noir/commit/2bc7d25271ca8c375a54d00116f507857b4b79ae))


### Bug Fixes

* `9_conditional` end to end test ([#1951](https://github.com/noir-lang/noir/issues/1951)) ([2f6741f](https://github.com/noir-lang/noir/commit/2f6741f4f3eaa892bd970ffbb19703546c4254c4))
* `regression` end to end test ([#1965](https://github.com/noir-lang/noir/issues/1965)) ([59f92e3](https://github.com/noir-lang/noir/commit/59f92e303a7d3279af779946e216860082567de3))
* Account for missing indices in flattened witness map ([#1907](https://github.com/noir-lang/noir/issues/1907)) ([3972410](https://github.com/noir-lang/noir/commit/39724108a428015cfade4c4ef032af8941bc9a93))
* Add missing `follow_bindings` when checking if a type can be casted ([#2022](https://github.com/noir-lang/noir/issues/2022)) ([537c2bd](https://github.com/noir-lang/noir/commit/537c2bd7844dea85c9d7136b09a5d2ccd33c3108))
* Add Result to acir gen ([#1927](https://github.com/noir-lang/noir/issues/1927)) ([1f8fd51](https://github.com/noir-lang/noir/commit/1f8fd51fb28b62e05f4b0c0829d446e43e8b85cc))
* Assignment to arrays of structs ([#1998](https://github.com/noir-lang/noir/issues/1998)) ([2c3d976](https://github.com/noir-lang/noir/commit/2c3d976ded4d98529a76b7f24284209f58bc04b9))
* **brillig_gen:** Pass correct size of complex types input for brillig foreign calls ([#1922](https://github.com/noir-lang/noir/issues/1922)) ([04c89d2](https://github.com/noir-lang/noir/commit/04c89d2581f3f73073bb0ab83d37a853c638959f))
* Create `FileManager` with a root and normalize filenames against it ([#1881](https://github.com/noir-lang/noir/issues/1881)) ([50c1648](https://github.com/noir-lang/noir/commit/50c16489173f847dc466e2f82738a5e441445407))
* Differentiate stdlib `CrateId` from others ([#1895](https://github.com/noir-lang/noir/issues/1895)) ([211e251](https://github.com/noir-lang/noir/commit/211e2512861566f21c5bfec4b47eb6964211f4c0))
* Don't panic when checking if an undeclared variable is mutable ([#1987](https://github.com/noir-lang/noir/issues/1987)) ([0449518](https://github.com/noir-lang/noir/commit/0449518a430d1148b4edccb819af072cf029a83d))
* emit `Opcode`s in correct order from `GeneratedAcir::radix_le_decompose` ([#1903](https://github.com/noir-lang/noir/issues/1903)) ([e5fe839](https://github.com/noir-lang/noir/commit/e5fe839876210a208f68fd4672b4b1e86d3c0073))
* emit opcode in correct order from `GeneratedAcir::radix_le_decompose` ([e5fe839](https://github.com/noir-lang/noir/commit/e5fe839876210a208f68fd4672b4b1e86d3c0073))
* emit opcodes for sorting variables in order of execution ([c43efab](https://github.com/noir-lang/noir/commit/c43efab06065c32fa83f8b09afca9605ba82da45))
* Emit opcodes for sorting variables in order of execution ([#1941](https://github.com/noir-lang/noir/issues/1941)) ([c43efab](https://github.com/noir-lang/noir/commit/c43efab06065c32fa83f8b09afca9605ba82da45))
* Fix auto-deref operations assigning the wrong result type ([#1904](https://github.com/noir-lang/noir/issues/1904)) ([827f78c](https://github.com/noir-lang/noir/commit/827f78c8d3cd478b7917deb2fcd3c854540116cb))
* **lsp:** Avoid storing Context until recompiles are possible ([#1891](https://github.com/noir-lang/noir/issues/1891)) ([fb5f20b](https://github.com/noir-lang/noir/commit/fb5f20b1b4d911de565faebfc9baa498cd5e2128))
* method resolution when calling an `&mut` method with an `&mut` object type ([#1947](https://github.com/noir-lang/noir/issues/1947)) ([73c2e94](https://github.com/noir-lang/noir/commit/73c2e9416c2c77cc384b9cfe76b594cd1764c586))
* Only flatten main ([#1984](https://github.com/noir-lang/noir/issues/1984)) ([ac865b1](https://github.com/noir-lang/noir/commit/ac865b1b83952015d89cc8fde4702148c5eac3c6))
* Parse an if followed by a tuple as a block ([#1924](https://github.com/noir-lang/noir/issues/1924)) ([8df4f05](https://github.com/noir-lang/noir/commit/8df4f05d3ae467c74c409287ad6202c5778b073d))
* Perform `occurs` check before binding function types ([#2027](https://github.com/noir-lang/noir/issues/2027)) ([1544786](https://github.com/noir-lang/noir/commit/154478698db4192d56050e57091991ffd25da36b))
* Prevent `if` and `for` from parsing constructor expressions ([#1916](https://github.com/noir-lang/noir/issues/1916)) ([6d3029a](https://github.com/noir-lang/noir/commit/6d3029a10fdcb4f839f624e2011f32b3774dbeea))
* Rebuild tests plus script to include secp256r1 change ([#1908](https://github.com/noir-lang/noir/issues/1908)) ([ca68666](https://github.com/noir-lang/noir/commit/ca68666d959fb63dbddd449691f43305460c1a9d))
* Switch from HashMap to BTreeMap in merge_stores ([#2035](https://github.com/noir-lang/noir/issues/2035)) ([4d179e3](https://github.com/noir-lang/noir/commit/4d179e3862a63d3d924215e75e31199369c6f3e8))
* update int division optimization ([#1928](https://github.com/noir-lang/noir/issues/1928)) ([fb872c6](https://github.com/noir-lang/noir/commit/fb872c624cb04a66b99f121b4e4a478998d96271))
* Various fixes for defunctionalization & brillig gen ([#1973](https://github.com/noir-lang/noir/issues/1973)) ([f99f4bf](https://github.com/noir-lang/noir/commit/f99f4bf94053918f1baee7d985bca92d64977a3e))
* workaround for LSP dependency resolution ([#1865](https://github.com/noir-lang/noir/issues/1865)) ([a8ac338](https://github.com/noir-lang/noir/commit/a8ac338758e4afd1cd459803658f011e04666177))

## [0.8.0](https://github.com/noir-lang/noir/compare/v0.7.1...v0.8.0) (2023-07-07)


### ⚠ BREAKING CHANGES

* **ssa_refactor:** Add Slices ([#1728](https://github.com/noir-lang/noir/issues/1728))
* **nargo:** Make proving and verification keys optional ([#1880](https://github.com/noir-lang/noir/issues/1880))
* update to ACVM 0.16.0 ([#1863](https://github.com/noir-lang/noir/issues/1863))

### Features

* add signed division ([#1831](https://github.com/noir-lang/noir/issues/1831)) ([d0894ad](https://github.com/noir-lang/noir/commit/d0894ada1d292f2910ebb38858b9439066f012d8))
* allow main to be a brillig function ([#1861](https://github.com/noir-lang/noir/issues/1861)) ([1330a2a](https://github.com/noir-lang/noir/commit/1330a2aabeb227146d2ea7d2850d1e8fd05beffe))
* **brillig:** implemented blackbox functions ([#1788](https://github.com/noir-lang/noir/issues/1788)) ([f9f38de](https://github.com/noir-lang/noir/commit/f9f38ded4f0491ad56402a0820cbd355913e6361))
* **brillig:** wrap brillig fns to be top level ([1330a2a](https://github.com/noir-lang/noir/commit/1330a2aabeb227146d2ea7d2850d1e8fd05beffe))
* defunctionalization pass for ssa refactor ([#1870](https://github.com/noir-lang/noir/issues/1870)) ([1d5d84d](https://github.com/noir-lang/noir/commit/1d5d84dd6db650aa9c136d3e9746a6544cf13945))
* **driver:** Remove `Driver` struct and refactor functions to take `Context` ([#1867](https://github.com/noir-lang/noir/issues/1867)) ([8895853](https://github.com/noir-lang/noir/commit/8895853a688b8e3a9d6ffb727dc1435f5687a4b3))
* **driver:** Remove Driver struct and refactor functions to take context ([8895853](https://github.com/noir-lang/noir/commit/8895853a688b8e3a9d6ffb727dc1435f5687a4b3))
* **lsp:** Add a codelens that runs test when clicked ([#1835](https://github.com/noir-lang/noir/issues/1835)) ([5d64f8a](https://github.com/noir-lang/noir/commit/5d64f8a175ea087ed980e20041dc525eb799ba95))
* make array indexes polymophic integers ([#1877](https://github.com/noir-lang/noir/issues/1877)) ([0fc93fa](https://github.com/noir-lang/noir/commit/0fc93fa4d9b2224bef5c5a27e362e88d8996164f))
* make use of type information when serialising inputs ([#1655](https://github.com/noir-lang/noir/issues/1655)) ([310368d](https://github.com/noir-lang/noir/commit/310368d30db3b312117f988c48fca1e22fbb4c03))
* recursion working in brillig ([#1854](https://github.com/noir-lang/noir/issues/1854)) ([e55b5a8](https://github.com/noir-lang/noir/commit/e55b5a8804648511b176f8002209152b3cc8aaaa))
* **ssa refactor:** Implement first-class references ([#1849](https://github.com/noir-lang/noir/issues/1849)) ([e5773e4](https://github.com/noir-lang/noir/commit/e5773e47c212c7c8fa1a7d7456893b508cdb400c))
* **ssa_refactor:** Add Slices ([#1728](https://github.com/noir-lang/noir/issues/1728)) ([4bee979](https://github.com/noir-lang/noir/commit/4bee9794a84f386cbab8f85b9eebe76c8fe90bd0))
* update to ACVM 0.16.0 ([#1863](https://github.com/noir-lang/noir/issues/1863)) ([9c89def](https://github.com/noir-lang/noir/commit/9c89def172a36327e4b75aa510b34f8cca0f998a))


### Bug Fixes

* **crates:** do not process relative dependencies twice ([#1856](https://github.com/noir-lang/noir/issues/1856)) ([b2e71bb](https://github.com/noir-lang/noir/commit/b2e71bb64ecff7d951eb00e7fcea8e316dca9bd5))
* **lsp:** Ensure stdlib is always added before the `check_crate` phase ([#1840](https://github.com/noir-lang/noir/issues/1840)) ([cb607f5](https://github.com/noir-lang/noir/commit/cb607f5787f76856a3b9907151c3de44045bc9c7))
* **lsp:** Ensure that stdlib is always added to the driver during the check_crate phase ([cb607f5](https://github.com/noir-lang/noir/commit/cb607f5787f76856a3b9907151c3de44045bc9c7))
* Prevent comparisons from being used on `Field`s ([#1860](https://github.com/noir-lang/noir/issues/1860)) ([c8858fd](https://github.com/noir-lang/noir/commit/c8858fdaccfd205a69dd918fd262902db92516f3))
* **ssa refactor:** Add missed call to resolve ([#1817](https://github.com/noir-lang/noir/issues/1817)) ([fa9be1d](https://github.com/noir-lang/noir/commit/fa9be1d255cb10fafcf81c92bd02488f366eaf23))
* **ssa refactor:** Fix recursive call to `create_value_from_type` ([#1815](https://github.com/noir-lang/noir/issues/1815)) ([890a63b](https://github.com/noir-lang/noir/commit/890a63be4839520d1fb13ec62e21e36086ae8003))
* **ssa refactor:** Prevent stores in 'then' branch from affecting the 'else' branch ([#1827](https://github.com/noir-lang/noir/issues/1827)) ([e068fd4](https://github.com/noir-lang/noir/commit/e068fd416c0cc6db671be770c30643fe9e2b59fe))


### Miscellaneous Chores

* **nargo:** Make proving and verification keys optional ([#1880](https://github.com/noir-lang/noir/issues/1880)) ([be36c1e](https://github.com/noir-lang/noir/commit/be36c1e816e685f4882538eb3dec4b8e81f61bc2))

## [0.7.1](https://github.com/noir-lang/noir/compare/v0.7.0...v0.7.1) (2023-06-23)


### Features

* **brillig:** foreign calls with dynamic-size objects ([#1705](https://github.com/noir-lang/noir/issues/1705)) ([fe7bb99](https://github.com/noir-lang/noir/commit/fe7bb99045abdd3052614f3a25a5ad7be3bd62a3))
* update acvm to 0.15.1 ([#1764](https://github.com/noir-lang/noir/issues/1764)) ([b52f25d](https://github.com/noir-lang/noir/commit/b52f25da9ddca31fd24a9c0077821a7b31a605c7))


### Bug Fixes

* **lsp:** Ensure LSP can compile on Windows ([#1794](https://github.com/noir-lang/noir/issues/1794)) ([2992915](https://github.com/noir-lang/noir/commit/2992915da7582b0aae2198579c7f928953f3befc))
* Methods called after being passed through a generic type were not being detected ([#1785](https://github.com/noir-lang/noir/issues/1785)) ([e560cd2](https://github.com/noir-lang/noir/commit/e560cd2f56f78486d5add12bc6fce16b6b1d36f6))
* **nargo:** Update acvm-backend-barretenberg to allow wasm backend compilation ([#1771](https://github.com/noir-lang/noir/issues/1771)) ([97da745](https://github.com/noir-lang/noir/commit/97da74572b9eceac3cc819b7ebb39cd6ff632768))
* **old ssa:** fix to_be_bits ([#1765](https://github.com/noir-lang/noir/issues/1765)) ([2541fbd](https://github.com/noir-lang/noir/commit/2541fbd8c62be80caf0e1cea19cd36c5e0d1e62b))
* **ssa refactor:** ACIR gen NOT integer ([#1749](https://github.com/noir-lang/noir/issues/1749)) ([af749a0](https://github.com/noir-lang/noir/commit/af749a0941cbba567c857da964a4fa57b4626004))
* **ssa refactor:** allow simplified call inserts & fix const radix arg handling ([#1774](https://github.com/noir-lang/noir/issues/1774)) ([46facce](https://github.com/noir-lang/noir/commit/46faccefc6e60846143485d5c8320dbb4e7a937c))
* **ssa refactor:** Fix flattening pass inserting loads before stores occur ([#1783](https://github.com/noir-lang/noir/issues/1783)) ([4293b15](https://github.com/noir-lang/noir/commit/4293b15639b58eb27703acffdc034b8219391018))
* **ssa refactor:** Fix panic in acir-gen from multiplying values of different types ([#1769](https://github.com/noir-lang/noir/issues/1769)) ([1f9a132](https://github.com/noir-lang/noir/commit/1f9a132acec8a442df5c9b36976f9ee1688ecc8a))
* **ssa refactor:** function inlining orphans calls ([#1747](https://github.com/noir-lang/noir/issues/1747)) ([f30a90f](https://github.com/noir-lang/noir/commit/f30a90f4eb6c2512eab7ec7f43c9dd287e6080b2))
* **ssa refactor:** Ignore array out of bounds errors when enable_side_effects is false ([#1797](https://github.com/noir-lang/noir/issues/1797)) ([7b7682a](https://github.com/noir-lang/noir/commit/7b7682a575d01a6d798e52ad2f28dde22e60b549))
* **ssa refactor:** Implement merging of array values during flattening pass ([#1767](https://github.com/noir-lang/noir/issues/1767)) ([8f24751](https://github.com/noir-lang/noir/commit/8f24751ec4f49aa46a02d3b45f4dad1323e933d1))
* **ssa refactor:** recursion_level decrement ([#1745](https://github.com/noir-lang/noir/issues/1745)) ([e449b92](https://github.com/noir-lang/noir/commit/e449b924e0baf2f6e34b36d182da3979cd1276ee))
* **ssa refactor:** recursive branch analysis ([#1759](https://github.com/noir-lang/noir/issues/1759)) ([635b574](https://github.com/noir-lang/noir/commit/635b574b14cead36c9e45b5807921885deaa4b61))
* **ssa refactor:** Reset condition value during flattening pass ([#1811](https://github.com/noir-lang/noir/issues/1811)) ([2e330e0](https://github.com/noir-lang/noir/commit/2e330e091c1a4daab25dfb7d9bc829cbc7063ddd))
* **ssa refactor:** Speedup acir-gen ([#1793](https://github.com/noir-lang/noir/issues/1793)) ([1e75f0e](https://github.com/noir-lang/noir/commit/1e75f0e0fea48fa240abf18ab2f5c8dafb458f80))
* **ssa refactor:** Speedup find-branch-ends ([#1786](https://github.com/noir-lang/noir/issues/1786)) ([861e42c](https://github.com/noir-lang/noir/commit/861e42c3ded473522332032cb7124a82dcc2c80c))
* Update array type when processing staged memory ([#1751](https://github.com/noir-lang/noir/issues/1751)) ([27eb748](https://github.com/noir-lang/noir/commit/27eb74885d5e3bddc4a8ef07f5c2f958dee20839))

## [0.7.0](https://github.com/noir-lang/noir/compare/root-v0.6.0...root-v0.7.0) (2023-06-19)


### ⚠ BREAKING CHANGES

* Update to acvm 0.14.0 ([#1594](https://github.com/noir-lang/noir/issues/1594))
* update to ACVM 0.13.0 ([#1393](https://github.com/noir-lang/noir/issues/1393))
* **stdlib:** remove unnecessary merkle functions from stdlib ([#1424](https://github.com/noir-lang/noir/issues/1424))
* **stdlib:** return update verification functions to return `bool`
* **stdlib:** update stdlib functions to return `bool` where appropriate ([#1409](https://github.com/noir-lang/noir/issues/1409))
* Change serialization of struct field order to match the user defined order ([#1166](https://github.com/noir-lang/noir/issues/1166))
* Update to ACVM 0.12.0 ([#1339](https://github.com/noir-lang/noir/issues/1339))
* remove concept of noir fallbacks for foreign functions ([#1371](https://github.com/noir-lang/noir/issues/1371))
* **nargo:** retire print-acir in favour of flag ([#1328](https://github.com/noir-lang/noir/issues/1328))

### Features

* Allow warnings by default ([#1383](https://github.com/noir-lang/noir/issues/1383)) ([e7a0d5c](https://github.com/noir-lang/noir/commit/e7a0d5c7b3b86587861401533d4e6784d0353404))
* **ci:** update noir to build wasm with a nix flake file ([#1208](https://github.com/noir-lang/noir/issues/1208)) ([2209369](https://github.com/noir-lang/noir/commit/22093699a1a9c0c654c57fcce683fb42808db3e4))
* **lsp:** Publish diagnostics on file save ([#1676](https://github.com/noir-lang/noir/issues/1676)) ([c53bfc8](https://github.com/noir-lang/noir/commit/c53bfc8c2207b64ac7e4a8d732dc4bc431b1990e))
* Make for-loop range be a polymorphic integer instead of just Field in unconstrained functions ([#1583](https://github.com/noir-lang/noir/issues/1583)) ([77fba56](https://github.com/noir-lang/noir/commit/77fba5677f9d1466d9d08c2eddc57149f9010db4))
* multiple item imports in use statement ([#1466](https://github.com/noir-lang/noir/issues/1466)) ([1dcd2ee](https://github.com/noir-lang/noir/commit/1dcd2ee9dd42c7867d9abcd528b763dd0a05bdd3))
* **nargo:** Add `lsp` command to start server that reports no capabilities ([#1560](https://github.com/noir-lang/noir/issues/1560)) ([e28529d](https://github.com/noir-lang/noir/commit/e28529d49f82300292e2b4d564f42a1c6bcaab59))
* **nargo:** Allow user-specified file for prover inputs instead of `Prover.toml` ([#1531](https://github.com/noir-lang/noir/issues/1531)) ([91cbec6](https://github.com/noir-lang/noir/commit/91cbec6cff1dabf6fd73a0eeff84006c2aa14080))
* **nargo:** retire print-acir in favour of flag ([#1328](https://github.com/noir-lang/noir/issues/1328)) ([dffa3c5](https://github.com/noir-lang/noir/commit/dffa3c50337ec0f71a62377d985ebdc8eefe490e))
* pass in closure to `Driver` to signal backend opcode support ([#1349](https://github.com/noir-lang/noir/issues/1349)) ([1e958c2](https://github.com/noir-lang/noir/commit/1e958c2aef89328e5354457c2a1e8697486e2978))
* remove concept of noir fallbacks for foreign functions ([#1371](https://github.com/noir-lang/noir/issues/1371)) ([dbec6f2](https://github.com/noir-lang/noir/commit/dbec6f284e17c7d656d8ffcf9534bd370eee9756))
* **ssa refactor:** mem2reg opt pass ([#1363](https://github.com/noir-lang/noir/issues/1363)) ([5d1efd5](https://github.com/noir-lang/noir/commit/5d1efd51dc3cc762ae8b75032bc71705845f30ff))
* **stdlib:** EdDSA sig verification ([#1313](https://github.com/noir-lang/noir/issues/1313)) ([04a15e0](https://github.com/noir-lang/noir/commit/04a15e00331077410a74c91934e7eb64aa165d9e))
* **stdlib:** return update verification functions to return `bool` ([2b2be1e](https://github.com/noir-lang/noir/commit/2b2be1e7fbfbfcb00cfd15587cbc9df083b91055))
* **stdlib:** update stdlib functions to return `bool` where appropriate ([#1409](https://github.com/noir-lang/noir/issues/1409)) ([2b2be1e](https://github.com/noir-lang/noir/commit/2b2be1e7fbfbfcb00cfd15587cbc9df083b91055))
* Update to acvm 0.14.0 ([#1594](https://github.com/noir-lang/noir/issues/1594)) ([f2d6b7b](https://github.com/noir-lang/noir/commit/f2d6b7bd8c909cbe85c8b5ff760ac2a4607ab56e))
* update to ACVM 0.15.0 ([#1616](https://github.com/noir-lang/noir/issues/1616)) ([3109239](https://github.com/noir-lang/noir/commit/3109239f2c0a7ad4767a3cd1bcc4436a367a8860))
* use RAM/ROM opcode when supported by the backend ([#1282](https://github.com/noir-lang/noir/issues/1282)) ([242f07b](https://github.com/noir-lang/noir/commit/242f07b513c0f7141c0c661e6c7913db04eeccef))


### Bug Fixes

* Change serialization of struct field order to match the user defined order ([#1166](https://github.com/noir-lang/noir/issues/1166)) ([809aa3a](https://github.com/noir-lang/noir/commit/809aa3a071ab3eb5143747f5ee8e03597afe7719))
* Fix modulo operator for comptime values ([#1361](https://github.com/noir-lang/noir/issues/1361)) ([ba15d6d](https://github.com/noir-lang/noir/commit/ba15d6d654739cc710e147dc08d94dcfe9dedb2a))
* Fix nargo not showing compiler errors or warnings ([#1694](https://github.com/noir-lang/noir/issues/1694)) ([4233068](https://github.com/noir-lang/noir/commit/4233068e790e6b2544b61571183fdfe8dbaa7c57))
* **frontend:** Avoid panic if dependency cannot be resolved ([#1719](https://github.com/noir-lang/noir/issues/1719)) ([f35b346](https://github.com/noir-lang/noir/commit/f35b3468ee0fe928b472a47a13b2dd0dcf37bb46))
* **nargo:** prevent -p arg clash ([#1605](https://github.com/noir-lang/noir/issues/1605)) ([4867f4e](https://github.com/noir-lang/noir/commit/4867f4ec9d00160640a7665cf64c65bd6982cf77))
* **noirc_driver:** Move error printing into nargo ([#1598](https://github.com/noir-lang/noir/issues/1598)) ([561cd63](https://github.com/noir-lang/noir/commit/561cd63debc24d96fa95d3eced72d8b2f8122f49))
* **ssa refactor:** Add missing calls to resolve in Instruction::simplify ([#1678](https://github.com/noir-lang/noir/issues/1678)) ([07b07d0](https://github.com/noir-lang/noir/commit/07b07d06cf8d73a85f2bde64d10cfbc677d9d3b1))
* **ssa refactor:** BigUint for radix ([#1715](https://github.com/noir-lang/noir/issues/1715)) ([00cf462](https://github.com/noir-lang/noir/commit/00cf462bbe277d7658ca9c6824165b7f3c26514e))
* **ssa refactor:** Change the result of simplifying Eq and Lt to bool ([#1672](https://github.com/noir-lang/noir/issues/1672)) ([1d48929](https://github.com/noir-lang/noir/commit/1d48929ecf20b1d2f9ab07ecf233c2565679ecec))
* **ssa refactor:** Do not remove enable_side_effects instructions in die pass ([#1673](https://github.com/noir-lang/noir/issues/1673)) ([cbee4c0](https://github.com/noir-lang/noir/commit/cbee4c0ad3606b2607fb4fdc88d1caa90a7c3462))
* **ssa refactor:** euclidean division for unsigned ([#1721](https://github.com/noir-lang/noir/issues/1721)) ([a1596bc](https://github.com/noir-lang/noir/commit/a1596bca1794af822a7804c22789ea6598f11edb))
* **ssa refactor:** filter unreachable blocks from cfg ([#1523](https://github.com/noir-lang/noir/issues/1523)) ([202c345](https://github.com/noir-lang/noir/commit/202c34548515bbc542c28a1225882590cfa086eb))
* **ssa refactor:** fix array element propagation through constant folding and DIE ([#1674](https://github.com/noir-lang/noir/issues/1674)) ([301e244](https://github.com/noir-lang/noir/commit/301e24476975a74d21270df5957c7b27f08706aa))
* **ssa refactor:** Fix array elements not being mapped to new values ([#1717](https://github.com/noir-lang/noir/issues/1717)) ([4ebcbeb](https://github.com/noir-lang/noir/commit/4ebcbeba166fb19f277c1a3508e618f989fa75b0)), closes [#1688](https://github.com/noir-lang/noir/issues/1688)
* **ssa refactor:** fix bad constant type caching ([#1593](https://github.com/noir-lang/noir/issues/1593)) ([37c0be6](https://github.com/noir-lang/noir/commit/37c0be65f0a06e6535169193547ed9b9bceb1ff9))
* **ssa refactor:** Fix constant folding looping forever ([#1611](https://github.com/noir-lang/noir/issues/1611)) ([afe58cc](https://github.com/noir-lang/noir/commit/afe58ccc8b80aecadb8c5ab8564d5e87f5d1094c))
* **ssa refactor:** Fix failed_to_inline_a_function being set for intrinsics ([#1675](https://github.com/noir-lang/noir/issues/1675)) ([377ac5c](https://github.com/noir-lang/noir/commit/377ac5c2d2faf38078f682f8428f0de165a7ca68))
* **ssa refactor:** Fix flatten_cfg for ifs with no else ([#1671](https://github.com/noir-lang/noir/issues/1671)) ([7ce8cce](https://github.com/noir-lang/noir/commit/7ce8cce6da5f668ac00dbbdeefda5b3b0f61815f))
* **ssa refactor:** Fix mem2reg pass not always removing unused stores ([#1677](https://github.com/noir-lang/noir/issues/1677)) ([8310544](https://github.com/noir-lang/noir/commit/8310544382d11fcf46a87b6e51f1a7d5f4cbbefc))
* **ssa refactor:** Fix ssa-gen of nested ifs ([#1406](https://github.com/noir-lang/noir/issues/1406)) ([5fd976e](https://github.com/noir-lang/noir/commit/5fd976e03e8034e521840621035c99ea840b13ba))
* **ssa refactor:** Fix stack overflow during loop unrolling ([#1666](https://github.com/noir-lang/noir/issues/1666)) ([c7a7216](https://github.com/noir-lang/noir/commit/c7a7216c9b01bf89aa4493330a71e825378f631e))
* **ssa refactor:** Implement array equality in SSA-gen ([#1704](https://github.com/noir-lang/noir/issues/1704)) ([0d31d83](https://github.com/noir-lang/noir/commit/0d31d831e29016c892bfb21ccc71159591b02519))
* **ssa refactor:** more comprehensive instruction simplification ([#1735](https://github.com/noir-lang/noir/issues/1735)) ([97d6747](https://github.com/noir-lang/noir/commit/97d674728e6c6174d97b096077e13940c20c2eee))
* **ssa refactor:** pad radix result ([#1730](https://github.com/noir-lang/noir/issues/1730)) ([8e9b612](https://github.com/noir-lang/noir/commit/8e9b6122532079ecf71aafe448265797828b69cf))
* **ssa refactor:** resolve replaced value ids for printing ([#1535](https://github.com/noir-lang/noir/issues/1535)) ([08ca847](https://github.com/noir-lang/noir/commit/08ca847d764fdd0eff357d199d0a9d9eac44e5de))
* **ssa refactor:** safe to query cfg for single block programs ([#1401](https://github.com/noir-lang/noir/issues/1401)) ([e2a23b3](https://github.com/noir-lang/noir/commit/e2a23b3d933824f09d8a8f0e2535531e6dcf76cf))
* **ssa refactor:** schnorr signature handling ([#1727](https://github.com/noir-lang/noir/issues/1727)) ([98ecf93](https://github.com/noir-lang/noir/commit/98ecf9315431afa67e7239fed6b3f4da9cced294))
* **ssa refactor:** Translate strings as arrays of characters ([#1669](https://github.com/noir-lang/noir/issues/1669)) ([2ba2ef6](https://github.com/noir-lang/noir/commit/2ba2ef632cd6ffdd9f162e87108b19833973450f))
* **ssa refactor:** truncate when simplifying constant casts ([#1714](https://github.com/noir-lang/noir/issues/1714)) ([a2108d7](https://github.com/noir-lang/noir/commit/a2108d7931bbd65a32aa56c2f5c36900cf706fd9))
* **ssa:** conditionalise array indexes under IF statements ([#1395](https://github.com/noir-lang/noir/issues/1395)) ([ddca3b4](https://github.com/noir-lang/noir/commit/ddca3b4fd1902275f7094251bba88c3eba4d3854))
* **stdlib:** Workaround for Field comparison error in EdDSA signature verification ([#1372](https://github.com/noir-lang/noir/issues/1372)) ([e790c9f](https://github.com/noir-lang/noir/commit/e790c9f5da784f7617a0b578623b470af7e01116))


### Miscellaneous Chores

* **stdlib:** remove unnecessary merkle functions from stdlib ([#1424](https://github.com/noir-lang/noir/issues/1424)) ([50fcb3c](https://github.com/noir-lang/noir/commit/50fcb3cded8cf37403a2dc3839bf99b7df4261b5))
* Update to ACVM 0.12.0 ([#1339](https://github.com/noir-lang/noir/issues/1339)) ([b938c7e](https://github.com/noir-lang/noir/commit/b938c7eeaa5ee493b28cad5451e7d5b7921ad934))
* update to ACVM 0.13.0 ([#1393](https://github.com/noir-lang/noir/issues/1393)) ([22dee75](https://github.com/noir-lang/noir/commit/22dee75464d3d02af17109d9065d37342fbbcddb))

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
