window.BENCHMARK_DATA = {
  "lastUpdate": 1761252706488,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d5d7c2a9407ba25b399a43fd3266b3560f9528a7",
          "message": "chore: bump bb version (#10181)",
          "timestamp": "2025-10-14T14:03:04+01:00",
          "tree_id": "e2bf36cee7bfce5abc0d8575d355faf7abe1f041",
          "url": "https://github.com/noir-lang/noir/commit/d5d7c2a9407ba25b399a43fd3266b3560f9528a7"
        },
        "date": 1760448554487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.47,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449907816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.42,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455108718,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.42,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458323923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461216894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.51,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528308699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.47,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542381037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.43,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546220607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.45,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554866661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.39,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560825544,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.53,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616857220,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.45,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618829545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.43,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619945316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620880870,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.42,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620936964,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.52,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621200007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624665245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.41,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626320128,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.51,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626903020,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.42,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633906288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.56,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634570324,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760639077662,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.55,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702442033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703911081,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.44,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.65,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713249557,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.43,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716558384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721286586,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814845088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817425920,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.51,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951885375,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760962183223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.43,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760965166437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.5,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760966536391,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975671204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989283393,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.53,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037920122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056628043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.49,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057519959,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.47,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761061011016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.45,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062665002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.5,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072406582,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.52,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078371436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761127151845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761139189120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149475320,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.43,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163452666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.49,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215600815,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761236309858,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 265.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 247.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 340.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 6810,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 344.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1050,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.64,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.52,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761245161706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 5910,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 5920,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.59,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250491596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.37,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 5910,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 5920,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 2890,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.59,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.18,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.48,
            "unit": "MB"
          }
        ]
      }
    ],
    "Compilation Time": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d5d7c2a9407ba25b399a43fd3266b3560f9528a7",
          "message": "chore: bump bb version (#10181)",
          "timestamp": "2025-10-14T14:03:04+01:00",
          "tree_id": "e2bf36cee7bfce5abc0d8575d355faf7abe1f041",
          "url": "https://github.com/noir-lang/noir/commit/d5d7c2a9407ba25b399a43fd3266b3560f9528a7"
        },
        "date": 1760448484679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.678,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.902,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.667,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449909890,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.852,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.13,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.544,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.821,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.576,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455040002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.784,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.938,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 127.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.822,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.691,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458340027,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.684,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.338,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 125.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.586,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461313091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.792,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.814,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 133.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.743,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528342568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.888,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.314,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.676,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 125.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.344,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.639,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542418666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.58,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.642,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 127.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.798,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.61,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546219770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 123.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.73,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554909886,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.94,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.018,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 221,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.548,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 131.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.777,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.857,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560822734,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.83,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.466,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 127.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.798,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.818,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616918455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.722,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.12,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 266,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.524,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.854,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.815,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618728936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.962,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.97,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 129.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.786,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.655,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619744423,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.972,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 130.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.76,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620827056,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.818,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.432,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.644,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 132.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.881,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620975004,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.704,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.952,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 130.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.388,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.814,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.677,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621134334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.798,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.074,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 130.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.545,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624667257,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.414,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.634,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 130.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.604,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626014380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.81,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.092,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.546,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.825,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.534,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626793696,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.696,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.08,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 128.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.668,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633952175,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.736,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.768,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.524,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.825,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.727,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634567750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.62,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.556,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 133.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.554,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638864536,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.824,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.518,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 24.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 132.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.797,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.531,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702434713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.802,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.794,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.786,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.871,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703740744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 131.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.769,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.621,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713217406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.204,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 133.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.804,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.638,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716525249,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.872,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.156,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 132.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.552,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721309835,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.798,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.528,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 135.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.727,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814704719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.606,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.326,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 139.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.816,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.61,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817447374,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.796,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.802,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.574,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 130.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.759,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951310537,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.886,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.819,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.222,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760961586361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.779,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.405,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760964507986,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.718,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.004,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.775,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.164,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760965925439,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.838,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.108,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.79,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.166,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975071340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.704,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.82,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.344,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.638,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.799,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.235,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989007195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.86,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.774,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.556,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.725,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037736802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.718,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.848,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.338,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.736,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056430356,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.018,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.799,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.694,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057246238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.972,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.586,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.769,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060749035,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.778,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.002,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.566,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.66,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.388,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.806,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.583,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062424286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.814,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.432,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.812,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.685,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072182341,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.754,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.328,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.61,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.836,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.519,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078144564,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.762,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.322,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.59,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.794,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.665,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126905432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.9,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.762,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.628,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138987024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.234,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.806,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.648,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149252168,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.784,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.308,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.518,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 222,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.514,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.08,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.863,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.529,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163228954,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.722,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.86,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.304,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.653,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215420082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.98,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.388,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.58,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235986910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.956,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.526,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.26,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.657,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244935800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.68,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.694,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.344,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.826,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.754,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250455318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.924,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.596,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.816,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.543,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449903522,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.056,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455043384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.156,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.064,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458350118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.152,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.061,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461313292,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.152,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.245,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.061,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528344210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.061,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542416149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.304,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.068,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546219132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.071,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554911229,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.236,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.062,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560835689,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.075,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616946467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.304,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.065,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618736610,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.239,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.062,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619688291,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.064,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620822516,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.239,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.062,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620976648,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.068,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621132001,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.06,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624667864,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.066,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626017994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.06,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626788632,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.067,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633954228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.057,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634565809,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.306,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.064,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638857041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.053,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702428319,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.06,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703746133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.242,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.058,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713209793,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.246,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.049,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716547501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.067,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721311198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.243,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.05,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814707385,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.152,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.251,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.053,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817448534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951311768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.243,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.05,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760961586211,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.301,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.246,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760964504912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.245,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.055,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760965927543,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.304,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.246,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.067,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975070712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.153,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.303,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.242,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989006799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.155,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.302,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.243,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.091,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037745371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.078,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056425114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.235,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.053,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057273818,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.149,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.057,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060748943,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062433130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.243,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.051,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072179848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.244,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.084,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078145363,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.241,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.051,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126908607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.238,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138987926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.242,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.053,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149244073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 10.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.239,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.05,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163209328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.245,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.052,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215419031,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.082,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235987223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.05,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244936259,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.305,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.248,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.075,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250455467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.307,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.239,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.077,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f43a65f84b8c1bcba66fa1507abea6d992b2a549",
          "message": "chore(frontend): Split out trait impl setup when defining function meta data  (#10271)",
          "timestamp": "2025-10-23T20:16:36Z",
          "tree_id": "cbe9edf59b2585db1f78548d5f78f18a3c7508e7",
          "url": "https://github.com/noir-lang/noir/commit/f43a65f84b8c1bcba66fa1507abea6d992b2a549"
        },
        "date": 1761252679570,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 10.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.243,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.048,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Memory": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d5d7c2a9407ba25b399a43fd3266b3560f9528a7",
          "message": "chore: bump bb version (#10181)",
          "timestamp": "2025-10-14T14:03:04+01:00",
          "tree_id": "e2bf36cee7bfce5abc0d8575d355faf7abe1f041",
          "url": "https://github.com/noir-lang/noir/commit/d5d7c2a9407ba25b399a43fd3266b3560f9528a7"
        },
        "date": 1760448561860,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449908551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455109601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458325966,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461217379,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528304154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542384018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546221308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554867915,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560814896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616865252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618759445,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619945715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620878600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620939525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621201152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624665129,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626327303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626913976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633909961,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634605360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760639091483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702441705,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703903227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.69,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713252930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716565438,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721288156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814846290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817423308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951888046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760962196341,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760965164177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760966522989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975672448,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989284960,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037920417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056629949,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057522730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761061011844,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062664455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072411655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078363803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761127153784,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761139189120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149475268,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163453871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215599638,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761236299686,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761245163981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250491778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1020,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.87,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.7,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.96,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "86565a22ac8f611a07285a215070fb70aa9bc2bb",
          "message": "chore(ssa): Restore the use of unchecked index operations (#10110)",
          "timestamp": "2025-10-13T17:57:48Z",
          "tree_id": "00c25b704a7f71ddc961a3e2964464ee5bdf59c1",
          "url": "https://github.com/noir-lang/noir/commit/86565a22ac8f611a07285a215070fb70aa9bc2bb"
        },
        "date": 1760380267609,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 273,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 328,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d5d7c2a9407ba25b399a43fd3266b3560f9528a7",
          "message": "chore: bump bb version (#10181)",
          "timestamp": "2025-10-14T14:03:04+01:00",
          "tree_id": "e2bf36cee7bfce5abc0d8575d355faf7abe1f041",
          "url": "https://github.com/noir-lang/noir/commit/d5d7c2a9407ba25b399a43fd3266b3560f9528a7"
        },
        "date": 1760448283280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 320,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 324,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449823361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 283,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 366,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 0,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760454834941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 301,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 147,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 347,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458033841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 138,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 233,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 147,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 356,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760460975662,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 266,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 328,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760527940348,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 270,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 333,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542119866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 310,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760545928110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 286,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 148,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 323,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554563204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 140,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 277,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 319,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560486062,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 286,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 232,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 144,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 344,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616703563,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 275,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 244,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 358,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618561984,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 233,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 365,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619581513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 299,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760620901941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 330,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 440,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624258905,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 258,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 225,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 326,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760625747617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 182,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 327,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626475487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 273,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 233,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 354,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633360994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 0,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634261074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 387,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638775403,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 293,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 326,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760701908603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 225,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 18,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703347164,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 328,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760712859925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 306,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716157645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 291,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 232,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 335,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721031969,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 361,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 418,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814300066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 236,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 351,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817118960,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 282,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 341,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760950446188,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 319,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 256,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 221,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 413,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760960809518,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 273,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 443,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760963664494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 288,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 408,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760965197974,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 269,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 249,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 445,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760974280189,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 285,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 423,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760988940473,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 335,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 374,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037750662,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 264,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056398940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 264,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057181369,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 269,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 359,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060662901,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 299,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 324,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062406557,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 241,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072149657,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 326,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078073169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 321,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 338,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126843886,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 281,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 225,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 377,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138868097,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 301,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 291,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 348,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149168387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 278,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 326,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163108382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 313,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 257,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 328,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215444358,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 233,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 407,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235894642,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 140,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 252,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 352,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244871504,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 230,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 325,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250239420,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 375,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 337,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f43a65f84b8c1bcba66fa1507abea6d992b2a549",
          "message": "chore(frontend): Split out trait impl setup when defining function meta data  (#10271)",
          "timestamp": "2025-10-23T20:16:36Z",
          "tree_id": "cbe9edf59b2585db1f78548d5f78f18a3c7508e7",
          "url": "https://github.com/noir-lang/noir/commit/f43a65f84b8c1bcba66fa1507abea6d992b2a549"
        },
        "date": 1761252630898,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 240,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 345,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 3,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir_rsa_",
            "value": 2,
            "unit": "s"
          }
        ]
      }
    ],
    "ACVM Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449007907,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262673,
            "range": "± 5549",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228737,
            "range": "± 5426",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785308,
            "range": "± 7001",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760454286922,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267121,
            "range": "± 1519",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234995,
            "range": "± 4248",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785310,
            "range": "± 2176",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760457582274,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262162,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229233,
            "range": "± 5585",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784606,
            "range": "± 1671",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760460486009,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262838,
            "range": "± 2294",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229991,
            "range": "± 8865",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783742,
            "range": "± 2359",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760527510348,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265639,
            "range": "± 1083",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231161,
            "range": "± 4797",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786419,
            "range": "± 1342",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760541587551,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262625,
            "range": "± 5092",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228530,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2787204,
            "range": "± 3645",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760545481519,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261999,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229882,
            "range": "± 3096",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785511,
            "range": "± 3715",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554141360,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267533,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231477,
            "range": "± 10202",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785008,
            "range": "± 7215",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560043226,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264765,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230895,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786052,
            "range": "± 6235",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616058902,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264422,
            "range": "± 2285",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230680,
            "range": "± 2097",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2788271,
            "range": "± 27648",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760617658834,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268476,
            "range": "± 952",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235042,
            "range": "± 11725",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786519,
            "range": "± 1475",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760618733158,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262652,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229568,
            "range": "± 4791",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784047,
            "range": "± 1115",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620004986,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264798,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229470,
            "range": "± 3381",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2787267,
            "range": "± 1313",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620165989,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265680,
            "range": "± 3595",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230742,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785317,
            "range": "± 18209",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760620344571,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264597,
            "range": "± 930",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229609,
            "range": "± 8944",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784430,
            "range": "± 2415",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760623829500,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259582,
            "range": "± 2062",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227362,
            "range": "± 5248",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784991,
            "range": "± 140758",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760625080554,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262554,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229903,
            "range": "± 5870",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786145,
            "range": "± 14771",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626017807,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262858,
            "range": "± 605",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229968,
            "range": "± 5634",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785376,
            "range": "± 7846",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633151505,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264742,
            "range": "± 3665",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230196,
            "range": "± 8022",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786665,
            "range": "± 13476",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760633470649,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261951,
            "range": "± 1813",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229459,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786517,
            "range": "± 5446",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638065216,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268193,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231993,
            "range": "± 2903",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785310,
            "range": "± 1354",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760701378565,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 275607,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236570,
            "range": "± 1070",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785406,
            "range": "± 33581",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760702922282,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264940,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231415,
            "range": "± 4959",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783411,
            "range": "± 1039",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760712406625,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267485,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235813,
            "range": "± 5001",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2788424,
            "range": "± 2128",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760715710280,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256408,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229083,
            "range": "± 9124",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797329,
            "range": "± 12535",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760720492983,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259969,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232059,
            "range": "± 4354",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794894,
            "range": "± 1397",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760813769336,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258111,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229649,
            "range": "± 5205",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796738,
            "range": "± 1936",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760816679609,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258772,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231930,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799844,
            "range": "± 3800",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760949792108,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258781,
            "range": "± 2780",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229070,
            "range": "± 4065",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794044,
            "range": "± 17980",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760960314491,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255603,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227976,
            "range": "± 5368",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799147,
            "range": "± 3184",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760963150044,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258161,
            "range": "± 960",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228847,
            "range": "± 4217",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798732,
            "range": "± 10580",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760964608046,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256733,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229062,
            "range": "± 3310",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796401,
            "range": "± 1438",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760973757019,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255650,
            "range": "± 1497",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227403,
            "range": "± 3410",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792856,
            "range": "± 116926",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760988457380,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259405,
            "range": "± 941",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229338,
            "range": "± 3083",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799647,
            "range": "± 2697",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037204986,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264623,
            "range": "± 891",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235123,
            "range": "± 2699",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798745,
            "range": "± 2981",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761055731843,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267702,
            "range": "± 845",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236361,
            "range": "± 1670",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798299,
            "range": "± 4454",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761056728926,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267354,
            "range": "± 3697",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236775,
            "range": "± 7572",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2802837,
            "range": "± 25876",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060230706,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266652,
            "range": "± 1161",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236671,
            "range": "± 4350",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2800871,
            "range": "± 2762",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761061847012,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265416,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237019,
            "range": "± 5587",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799030,
            "range": "± 2987",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761071686223,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264009,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232908,
            "range": "± 4843",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793860,
            "range": "± 1898",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761077621285,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263301,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231628,
            "range": "± 7433",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796442,
            "range": "± 2539",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126352773,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266411,
            "range": "± 1777",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235628,
            "range": "± 2745",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793416,
            "range": "± 1553",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138403739,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261297,
            "range": "± 2347",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228859,
            "range": "± 5067",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2749224,
            "range": "± 27571",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761148733123,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263382,
            "range": "± 1075",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233195,
            "range": "± 7559",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797707,
            "range": "± 1912",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761162595750,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264214,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232209,
            "range": "± 4285",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794574,
            "range": "± 19826",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761214867964,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265479,
            "range": "± 2424",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 238232,
            "range": "± 2810",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798161,
            "range": "± 2665",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235446105,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263341,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232921,
            "range": "± 4071",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794668,
            "range": "± 5721",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244441487,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267680,
            "range": "± 1094",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236901,
            "range": "± 2580",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2818975,
            "range": "± 18511",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761249750633,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264443,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234923,
            "range": "± 3453",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797145,
            "range": "± 12460",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f43a65f84b8c1bcba66fa1507abea6d992b2a549",
          "message": "chore(frontend): Split out trait impl setup when defining function meta data  (#10271)",
          "timestamp": "2025-10-23T20:16:36Z",
          "tree_id": "cbe9edf59b2585db1f78548d5f78f18a3c7508e7",
          "url": "https://github.com/noir-lang/noir/commit/f43a65f84b8c1bcba66fa1507abea6d992b2a549"
        },
        "date": 1761252165940,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262270,
            "range": "± 960",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232909,
            "range": "± 4952",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798406,
            "range": "± 4313",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Artifact Size": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d5d7c2a9407ba25b399a43fd3266b3560f9528a7",
          "message": "chore: bump bb version (#10181)",
          "timestamp": "2025-10-14T14:03:04+01:00",
          "tree_id": "e2bf36cee7bfce5abc0d8575d355faf7abe1f041",
          "url": "https://github.com/noir-lang/noir/commit/d5d7c2a9407ba25b399a43fd3266b3560f9528a7"
        },
        "date": 1760448523017,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449903376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455040340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458346230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461314489,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528343605,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542421077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546218138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554907544,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560822655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616925317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618731107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619763012,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620821380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620975939,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621144306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624668126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626016866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626790355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633952695,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634550462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638866378,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702443896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703740149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 258.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27686.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713230003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716530777,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721309980,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814708440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817445604,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951310127,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760961585130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760964503837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760965928051,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975069494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989006166,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037743689,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056426694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057247687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060752683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062423265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072183012,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078147245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126905994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138988381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149251747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163208603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215417641,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235988149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244936245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250452997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1864,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 257.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 370.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27640.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27685.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4909.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4555.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.1,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.3,
            "unit": "KB"
          }
        ]
      }
    ],
    "Opcode count": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "96fb8193436323f4469e5e6f6c7090a0be99be8f",
          "message": "fix: emit error on oracle functions with function bodies (#10132)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-14T12:55:48Z",
          "tree_id": "1a9280c16f08cdf18ebffc903628915e18da1fa6",
          "url": "https://github.com/noir-lang/noir/commit/96fb8193436323f4469e5e6f6c7090a0be99be8f"
        },
        "date": 1760449901981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14545,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70421,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11682,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245188,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f8b6e72a31836f824f11a44d2ba8754af8d990a1",
          "message": "fix(ssa-interpreter): Ignore index overflow when side effects are disabled (#10183)",
          "timestamp": "2025-10-14T14:40:39Z",
          "tree_id": "0e8a40a550deb6b5f83ced62e224216c6d22bcfa",
          "url": "https://github.com/noir-lang/noir/commit/f8b6e72a31836f824f11a44d2ba8754af8d990a1"
        },
        "date": 1760455040005,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14545,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70421,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11682,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245188,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1261133954f604d0b12edf1ab858da7e7ac20adf",
          "message": "chore: add unit test for keep_last_store (#10177)",
          "timestamp": "2025-10-14T15:34:48Z",
          "tree_id": "4ba6180da15b78b994f8b3bf61c05bd28ec212c8",
          "url": "https://github.com/noir-lang/noir/commit/1261133954f604d0b12edf1ab858da7e7ac20adf"
        },
        "date": 1760458340016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14545,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70421,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11682,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245188,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82ec52a8c755d30ce655a2005834186a4acfa0c7",
          "message": "feat(ACIR): exact element_type_sizes_array (#10188)",
          "timestamp": "2025-10-14T16:24:52Z",
          "tree_id": "c99c311995cb61ba60ca4b2fb0412c37b31de6b0",
          "url": "https://github.com/noir-lang/noir/commit/82ec52a8c755d30ce655a2005834186a4acfa0c7"
        },
        "date": 1760461314369,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b0181a89c3ed0eacaf6646d569ae9f13dfdba39",
          "message": "chore: ensure that `useful_instructions` cannot overflow (#10173)",
          "timestamp": "2025-10-15T11:01:22Z",
          "tree_id": "c5382d239d3d3376dd54b10fd1f0f67b912483a2",
          "url": "https://github.com/noir-lang/noir/commit/3b0181a89c3ed0eacaf6646d569ae9f13dfdba39"
        },
        "date": 1760528345372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0cf97e746aee1a8843053618e0b1f2f3de67a695",
          "message": "chore: add incremental mutation testing (#10196)",
          "timestamp": "2025-10-15T16:16:12+01:00",
          "tree_id": "f2586c178a0c549ca75a80f2f4f450787a88ad5a",
          "url": "https://github.com/noir-lang/noir/commit/0cf97e746aee1a8843053618e0b1f2f3de67a695"
        },
        "date": 1760542415918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "724ac92e39d16efccde39bc06904b1e9456c5294",
          "message": "chore(ssa_fuzzer): allow brillig fuzz target to work in multi threads (#10100)",
          "timestamp": "2025-10-15T16:00:30Z",
          "tree_id": "8a1058f8e4a7e06cf880f74bc1ad35c617af7787",
          "url": "https://github.com/noir-lang/noir/commit/724ac92e39d16efccde39bc06904b1e9456c5294"
        },
        "date": 1760546217541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "816edd602b6fb20cab2ec852b6f89c8cd72272bd",
          "message": "chore: remove if-condition from `array_set_optimization_pre_check` (#10193)",
          "timestamp": "2025-10-15T18:27:04Z",
          "tree_id": "1d9bc0e274d9a84e95edc35a0ed04b757abec16a",
          "url": "https://github.com/noir-lang/noir/commit/816edd602b6fb20cab2ec852b6f89c8cd72272bd"
        },
        "date": 1760554908909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3f4b2849da02163ed82936f8917767b7a43b1c3c",
          "message": "chore(ACIR): prefer displaying `ASSERT return_value = ...` (#10195)",
          "timestamp": "2025-10-15T20:02:59Z",
          "tree_id": "57e1102ec5f5a5313601677e6db8f7e7d25e499a",
          "url": "https://github.com/noir-lang/noir/commit/3f4b2849da02163ed82936f8917767b7a43b1c3c"
        },
        "date": 1760560824708,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba",
          "message": "fix: address off-by-one error when removing casts before constraining to constant (#10194)",
          "timestamp": "2025-10-16T11:36:35Z",
          "tree_id": "1f111d56653129fb257245a4a1dae1aa81c0f729",
          "url": "https://github.com/noir-lang/noir/commit/64d592629184fab1bd938ad3c3cf4c3a1fbcf3ba"
        },
        "date": 1760616888674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32",
          "message": "chore: address clippy warnings (#10207)",
          "timestamp": "2025-10-16T13:20:22+01:00",
          "tree_id": "8d246a93d17eaa98e25420957816c06762f6c5a7",
          "url": "https://github.com/noir-lang/noir/commit/8d89dab4f5b5d3552a5f9a6f3f0a70bb65cf1c32"
        },
        "date": 1760618730091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eac078091d68fd38408138b817f13f36f2067fe",
          "message": "chore: add minimal reproductions for Cantina issues + typo fixes (#10120)",
          "timestamp": "2025-10-16T13:42:59+01:00",
          "tree_id": "5a7dff63df259d8be905e6f3c37d965cee6c64be",
          "url": "https://github.com/noir-lang/noir/commit/4eac078091d68fd38408138b817f13f36f2067fe"
        },
        "date": 1760619761687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d503bb73ff273f20fc34017e07c24e37cece45e8",
          "message": "chore(ACIR): optimize slice_insert (#10164)",
          "timestamp": "2025-10-16T12:25:57Z",
          "tree_id": "54867783a52a623512c7ede5353b8ab6dea7fefc",
          "url": "https://github.com/noir-lang/noir/commit/d503bb73ff273f20fc34017e07c24e37cece45e8"
        },
        "date": 1760620824431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2063f1c95a414605f3465122dab46e7a6bf1a85c",
          "message": "chore: assume Intrinsic::ArrayLen never reaches ACIR (#10201)",
          "timestamp": "2025-10-16T12:36:05Z",
          "tree_id": "9d9383848bc0f59dc53302aef0faa97f850323cc",
          "url": "https://github.com/noir-lang/noir/commit/2063f1c95a414605f3465122dab46e7a6bf1a85c"
        },
        "date": 1760620973167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f",
          "message": "chore(ACIR): better display/parse for blackbox calls (#10157)",
          "timestamp": "2025-10-16T12:45:36Z",
          "tree_id": "7bf48eafeb3cf51d70f722f3ec8e4b744f04ebf3",
          "url": "https://github.com/noir-lang/noir/commit/2ae3fbebe6b8004d5691a18b21953fc6aa2cbe8f"
        },
        "date": 1760621132314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f78a15d0d96f010539105cef295df888f7b0a2af",
          "message": "chore: document precondition to unrolling SSA pass (#10208)",
          "timestamp": "2025-10-16T13:47:20Z",
          "tree_id": "2898b3ecf76ecf93f2fc2ad584447d00a0aa6faf",
          "url": "https://github.com/noir-lang/noir/commit/f78a15d0d96f010539105cef295df888f7b0a2af"
        },
        "date": 1760624668922,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1152e995ffd23429f51a5a4234a71a196a87bc9f",
          "message": "chore(ACIR): make it clear that modulo is only for signed integers (#10209)",
          "timestamp": "2025-10-16T14:04:44Z",
          "tree_id": "96862f83ff644704a1d0fb4f1ef1514f0d99f4e5",
          "url": "https://github.com/noir-lang/noir/commit/1152e995ffd23429f51a5a4234a71a196a87bc9f"
        },
        "date": 1760626026983,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4ba5c86fcaee4675c75a2801e3df9c835cadc364",
          "message": "chore(ACIR): no need to return types in `flatten` (#10210)",
          "timestamp": "2025-10-16T14:22:40Z",
          "tree_id": "36e32f58f879203d35e2b66f24ffffc7b170e17a",
          "url": "https://github.com/noir-lang/noir/commit/4ba5c86fcaee4675c75a2801e3df9c835cadc364"
        },
        "date": 1760626790169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9d70064a07a388dd62fcb513ca5f262029889bfe",
          "message": "chore(ACIR): display/parse memory arrays as b0, b1, etc. (#10211)",
          "timestamp": "2025-10-16T16:20:13Z",
          "tree_id": "b49d7ed02fe291d3de9602a8479bb1c5ccdcefd7",
          "url": "https://github.com/noir-lang/noir/commit/9d70064a07a388dd62fcb513ca5f262029889bfe"
        },
        "date": 1760633953760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d293d1c97b5dad9a7ada3fe194f717379c62bdd3",
          "message": "chore: remove incremental mutation tests (#10212)",
          "timestamp": "2025-10-16T17:48:43+01:00",
          "tree_id": "53d6d482b6e4ff291d357da9d2699e68b03903ab",
          "url": "https://github.com/noir-lang/noir/commit/d293d1c97b5dad9a7ada3fe194f717379c62bdd3"
        },
        "date": 1760634566018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8d787871c097c315193fed23204e56fa396003a7",
          "message": "feat: attempt to inline successors in `simplify_cfg` (#9608)",
          "timestamp": "2025-10-16T17:45:01Z",
          "tree_id": "871adceadaec688774cf61cfa435b2b1ab84d997",
          "url": "https://github.com/noir-lang/noir/commit/8d787871c097c315193fed23204e56fa396003a7"
        },
        "date": 1760638861527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "60c6c3c83568928bd792e46b165c4d2cc54b2ebf",
          "message": "chore: redo typo PR by spuradage (#10226)",
          "timestamp": "2025-10-17T12:39:23+01:00",
          "tree_id": "5b7f3402eba48c044fc6f4079d699bc5440bbdb5",
          "url": "https://github.com/noir-lang/noir/commit/60c6c3c83568928bd792e46b165c4d2cc54b2ebf"
        },
        "date": 1760702427926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c0d8cc72785fdbbcad742ee45bc558f5261f474",
          "message": "fix(ACIR): correct brillig parameter slice length for dynamic arrays (#10198)",
          "timestamp": "2025-10-17T11:33:22Z",
          "tree_id": "349398ea9337d649c002a4be0325c629410e7785",
          "url": "https://github.com/noir-lang/noir/commit/0c0d8cc72785fdbbcad742ee45bc558f5261f474"
        },
        "date": 1760703751880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70418,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1365,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2410,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962022,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963382,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263910,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245187,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5700,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f73824e2ce32c7faf36b6d6bfee1fe5a003cb587",
          "message": "chore: simplify `x > 0` to `x != 0` for unsigned types in ACIR (#10220)",
          "timestamp": "2025-10-17T14:25:16Z",
          "tree_id": "015707ad43e2997a9543ba72808f8085630a0f6b",
          "url": "https://github.com/noir-lang/noir/commit/f73824e2ce32c7faf36b6d6bfee1fe5a003cb587"
        },
        "date": 1760713215004,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53568d3b2fee3945ae44b4c42403db4764e398ea",
          "message": "chore(acvm): Optimize logic ops (#10222)",
          "timestamp": "2025-10-17T15:13:46Z",
          "tree_id": "809f4ebdc17be2339a083f6d62f573239dc89c90",
          "url": "https://github.com/noir-lang/noir/commit/53568d3b2fee3945ae44b4c42403db4764e398ea"
        },
        "date": 1760716532842,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7",
          "message": "chore: clone indexed call results (#10140)\n\nCo-authored-by: Ratmir Karabut <rkarabut@users.noreply.github.com>",
          "timestamp": "2025-10-17T16:35:06Z",
          "tree_id": "49ae66499d1050062346e84ed03d114c81bb528a",
          "url": "https://github.com/noir-lang/noir/commit/2fa2ac253280b69ddb63bd3c9cd3056aaa7fbca7"
        },
        "date": 1760721311043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83640e8610ce25be87fe5145020ad6eee08c98df",
          "message": "chore(ACIR): simpler AsSlice implementation (#10214)",
          "timestamp": "2025-10-18T18:31:58Z",
          "tree_id": "4de88f544840e57551d711d75d9e80408bbe432b",
          "url": "https://github.com/noir-lang/noir/commit/83640e8610ce25be87fe5145020ad6eee08c98df"
        },
        "date": 1760814704195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28406cda5dcba5b0a8095f861dae695e9a8b5e3a",
          "message": "chore(frontend): Re-organize frontend tests  (#10221)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-18T19:19:25Z",
          "tree_id": "c7bff4eac0f925df5e35baab90df15e249e58a09",
          "url": "https://github.com/noir-lang/noir/commit/28406cda5dcba5b0a8095f861dae695e9a8b5e3a"
        },
        "date": 1760817460234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "47281315+guipublic@users.noreply.github.com",
            "name": "guipublic",
            "username": "guipublic"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b16e3c6da3f000e3ccd6df0abb80f8487a134c41",
          "message": "fix(mem2reg): Updating referenced value invalidate addresses with unknown aliases (#10175)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T08:19:38Z",
          "tree_id": "2b688b9e790541aa40495a3b69409dd5ff86c520",
          "url": "https://github.com/noir-lang/noir/commit/b16e3c6da3f000e3ccd6df0abb80f8487a134c41"
        },
        "date": 1760951312496,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7be7c101e4b176a20637f898085d6993611fa4e0",
          "message": "chore: Improve compilation time on `rollup-tx-base-public` (#10224)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-20T11:11:39Z",
          "tree_id": "ded678e7e4e8f9eca99d80845a5597523e73cbdd",
          "url": "https://github.com/noir-lang/noir/commit/7be7c101e4b176a20637f898085d6993611fa4e0"
        },
        "date": 1760961584774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "935dc3690a20587a6053046759a9de3db1f6ea42",
          "message": "chore(frontend): Modularize the Elaborator (#10202)",
          "timestamp": "2025-10-20T11:59:24Z",
          "tree_id": "5068079aacc1f290f3511bd04e87133bd276a062",
          "url": "https://github.com/noir-lang/noir/commit/935dc3690a20587a6053046759a9de3db1f6ea42"
        },
        "date": 1760964506803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ec7e299b011ba21c752eba18f11cb1720e05b6b",
          "message": "chore(frontend): HIR printer module for inline macro expansion unit tests  (#10232)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T12:26:53Z",
          "tree_id": "dc400d20337e576878a4cfbf7c96f0d4e8eaa3b1",
          "url": "https://github.com/noir-lang/noir/commit/0ec7e299b011ba21c752eba18f11cb1720e05b6b"
        },
        "date": 1760965925429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6424d4ac90d4a78560d3689066762d5fb6a2640d",
          "message": "chore(frontend): Split up traits tests module into submodules (#10229)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-20T14:58:03Z",
          "tree_id": "3883d248321f0d2ce2fd52e3809cc0f4e61c23da",
          "url": "https://github.com/noir-lang/noir/commit/6424d4ac90d4a78560d3689066762d5fb6a2640d"
        },
        "date": 1760975069834,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "077dd5ebf93b737c363f97491376681e88395bd0",
          "message": "fix(mem2reg): Update array set value alias set and propagate array get result as alias  (#10242)",
          "timestamp": "2025-10-20T19:00:28Z",
          "tree_id": "09fa8aab9dd17a9875f13d58a1265738610686e3",
          "url": "https://github.com/noir-lang/noir/commit/077dd5ebf93b737c363f97491376681e88395bd0"
        },
        "date": 1760989010515,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "870111763153627f1d7d573a3d54ff5c1a60f907",
          "message": "chore(audit): Brillig VM nits (#10237)",
          "timestamp": "2025-10-21T08:32:43Z",
          "tree_id": "ea883f5e77f4c447fab3e551b9a6cf57d3258648",
          "url": "https://github.com/noir-lang/noir/commit/870111763153627f1d7d573a3d54ff5c1a60f907"
        },
        "date": 1761037733418,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4290fa708c149975d0bd64d06eaf435f0dfd5ba",
          "message": "chore: greenlight Elaborator visibility (#10248)",
          "timestamp": "2025-10-21T13:40:46Z",
          "tree_id": "610a1b02ec6641b0f9e2036fbe79e6ace3fc56da",
          "url": "https://github.com/noir-lang/noir/commit/c4290fa708c149975d0bd64d06eaf435f0dfd5ba"
        },
        "date": 1761056425221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65083f9d5ece1b1db152af00ba02cf3709e31750",
          "message": "chore(ACIR): more Circuit, Expression and Opcode parsing (#10250)",
          "timestamp": "2025-10-21T13:57:52Z",
          "tree_id": "c8cc41e85771fbd5280cca77808689d8832d2966",
          "url": "https://github.com/noir-lang/noir/commit/65083f9d5ece1b1db152af00ba02cf3709e31750"
        },
        "date": 1761057262162,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bda3999be86d22cd96ee53f31d6763ddea1f0cc9",
          "message": "chore(frontend): Elaborator module doc comments (#10249)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-10-21T14:58:47Z",
          "tree_id": "f1af83ab02e310f5ec3645efaa4d07af839364f3",
          "url": "https://github.com/noir-lang/noir/commit/bda3999be86d22cd96ee53f31d6763ddea1f0cc9"
        },
        "date": 1761060753455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "334ac7d7a2b1ad637c96400b04b23d41e10a172f",
          "message": "chore(ACIR): turn \"todo\" into \"unreachable\" (#10251)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-21T15:24:48Z",
          "tree_id": "7d75e4e74c2fdbee818eb4891f1d6b1aba85eb72",
          "url": "https://github.com/noir-lang/noir/commit/334ac7d7a2b1ad637c96400b04b23d41e10a172f"
        },
        "date": 1761062421653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "965d61b8a172142f198bb17cf5042815377240f1",
          "message": "chore: typos and some refactors, tests, etc in `acvm/src/compiler` (#10111)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-21T18:08:33Z",
          "tree_id": "609b5d5ee867b9788a6d33cf297262103db191e2",
          "url": "https://github.com/noir-lang/noir/commit/965d61b8a172142f198bb17cf5042815377240f1"
        },
        "date": 1761072191847,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6e50d08787aa8b05d2c9693cad1957b197ec6d1b",
          "message": "chore(frontend): Elaborator function module (#10252)",
          "timestamp": "2025-10-21T19:50:04Z",
          "tree_id": "6f3c531b966b16d9550067ab7d8898fe6ab802b0",
          "url": "https://github.com/noir-lang/noir/commit/6e50d08787aa8b05d2c9693cad1957b197ec6d1b"
        },
        "date": 1761078147417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "acc1cbc2a1f1420d7c22d0753a2a127ba744e545",
          "message": "fix(ssa-interpreter): Add integer modulus to unfit `Field` if the value comes from a subtraction (#10241)",
          "timestamp": "2025-10-22T09:19:38Z",
          "tree_id": "2474d44e2b73746d2f5e7448268215c568fbda96",
          "url": "https://github.com/noir-lang/noir/commit/acc1cbc2a1f1420d7c22d0753a2a127ba744e545"
        },
        "date": 1761126906210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154",
          "message": "chore(ACIR): add a test for OpcodeResolutionError::AcirMainCallAttempted (#10254)",
          "timestamp": "2025-10-22T12:42:07Z",
          "tree_id": "0ba2f2a70153f31c98523e48818a1cfa26284f73",
          "url": "https://github.com/noir-lang/noir/commit/0ef3b9ddc11ade0cba5ceb526a7bf2076c1b4154"
        },
        "date": 1761138986712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0e4ae3017857462a5e626900ed66c9ca82ef166f",
          "message": "chore(frontend): Elaborator struct collection docs (#10266)",
          "timestamp": "2025-10-22T15:33:30Z",
          "tree_id": "e129080781ddeedb411539416a7fe57feb1069df",
          "url": "https://github.com/noir-lang/noir/commit/0e4ae3017857462a5e626900ed66c9ca82ef166f"
        },
        "date": 1761149243205,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8989770e5104f32fa83457f345643a6965134044",
          "message": "chore: Add some detail to the trait documentation (#10273)",
          "timestamp": "2025-10-22T19:20:24Z",
          "tree_id": "10947ddf9125373c701e41591d90ae2b88b7fe37",
          "url": "https://github.com/noir-lang/noir/commit/8989770e5104f32fa83457f345643a6965134044"
        },
        "date": 1761163210167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aakoshh@gmail.com",
            "name": "Akosh Farkash",
            "username": "aakoshh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca04e9cb1a1df5a9378ff380f624adf999d1c8bf",
          "message": "feat(brillig): Automatic register deallocation (#10253)",
          "timestamp": "2025-10-23T09:57:51Z",
          "tree_id": "76523ec7e19c30d7dd07b486ca06b36556d1869c",
          "url": "https://github.com/noir-lang/noir/commit/ca04e9cb1a1df5a9378ff380f624adf999d1c8bf"
        },
        "date": 1761215417752,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "412407c094a82ffab4870ca9198a39aa88e9f7b5",
          "message": "chore(ACIR): handle TODO in radix_decompose (#10272)",
          "timestamp": "2025-10-23T15:39:46Z",
          "tree_id": "3e58fda5698e6e30ae85bf0bc3faca054160e052",
          "url": "https://github.com/noir-lang/noir/commit/412407c094a82ffab4870ca9198a39aa88e9f7b5"
        },
        "date": 1761235989051,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "michaeljklein@users.noreply.github.com",
            "name": "Michael J Klein",
            "username": "michaeljklein"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d6009bd06bb2e05d0caa50c75722a8db65e0ef9b",
          "message": "chore: typos and some refactors, tests, etc in `noirc_evaluator/src/acir` (#10255)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-23T18:08:56Z",
          "tree_id": "8e7a3cd448722e36bec1844bae52637988519c13",
          "url": "https://github.com/noir-lang/noir/commit/d6009bd06bb2e05d0caa50c75722a8db65e0ef9b"
        },
        "date": 1761244937533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e",
          "message": "feat(LSP): show errors on stdlib files (#10283)",
          "timestamp": "2025-10-23T19:41:43Z",
          "tree_id": "ba81e4d89e6fa858feeffaa4d8f179ea454ac077",
          "url": "https://github.com/noir-lang/noir/commit/a43fedf35278de3f4d7b1bd61c955a4ffbed8e8e"
        },
        "date": 1761250454224,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mvezenov@gmail.com",
            "name": "Maxim Vezenov",
            "username": "vezenovm"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f43a65f84b8c1bcba66fa1507abea6d992b2a549",
          "message": "chore(frontend): Split out trait impl setup when defining function meta data  (#10271)",
          "timestamp": "2025-10-23T20:16:36Z",
          "tree_id": "cbe9edf59b2585db1f78548d5f78f18a3c7508e7",
          "url": "https://github.com/noir-lang/noir/commit/f43a65f84b8c1bcba66fa1507abea6d992b2a549"
        },
        "date": 1761252680444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70415,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1364,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1048,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962015,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963375,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263908,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245185,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1486,
            "unit": "opcodes"
          },
          {
            "name": "semaphore-depth-10",
            "value": 5699,
            "unit": "opcodes"
          },
          {
            "name": "sha512-100-bytes",
            "value": 13173,
            "unit": "opcodes"
          }
        ]
      }
    ]
  }
}