# Changelog

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
