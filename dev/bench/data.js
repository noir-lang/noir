window.BENCHMARK_DATA = {
  "lastUpdate": 1764863861297,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
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
          "id": "ab71130b55537fde2baca8ec7545e55c657bfcbe",
          "message": "fix(stdlib): Fix visibility of ecdsa foreign function calls (#10658)",
          "timestamp": "2025-11-27T16:55:58Z",
          "tree_id": "63db4962dfccdddbefe350cbde9252fda7d0563b",
          "url": "https://github.com/noir-lang/noir/commit/ab71130b55537fde2baca8ec7545e55c657bfcbe"
        },
        "date": 1764265700278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.34,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.58,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.64,
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764267613673,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.34,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.58,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764284264859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764323630222,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.76,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329850243,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.73,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333822329,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764335030152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.69,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764343565003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.33,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764343593989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764346723238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.78,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764350431113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.64,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361830063,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.69,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764374101454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.76,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.72,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764375500550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.36,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585798765,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.6,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764586292503,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.72,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764588608407,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.65,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595793619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.35,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764602386030,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604920587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764606491713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.78,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.64,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764607626766,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.74,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764610078812,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.69,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764615410699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764616319177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.67,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621795324,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764678640420,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764681632600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681782898,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764682265242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.63,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764683089105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.74,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764683293457,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.72,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764693410759,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764696109089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697875961,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764698692719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764699413816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764699606867,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764707639020,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709891256,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766868516,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.65,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764771217668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764811549246,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.7,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764847546138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764855204051,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.71,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764855395615,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855770588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855810797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764859322389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.66,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764863543715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 496.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 335.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.89,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11290,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 339.28,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3020,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.57,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.68,
            "unit": "MB"
          }
        ]
      }
    ],
    "Compilation Time": [
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266915732,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.974,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.016,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.928,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 390,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 367,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.781,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.595,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764283563166,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.976,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.884,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.794,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 450,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 387,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.514,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.16,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 88.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.889,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322920074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.158,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.806,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.438,
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
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 380,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.768,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329179636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.964,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.698,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.816,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 408,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 91.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.695,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333107306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.006,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.872,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 2.182,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.432,
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
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 387,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.66,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.92,
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
            "value": 1.636,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764334299841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.942,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.606,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.916,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 374,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.616,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764342862663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.832,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.828,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.39,
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
            "value": 1.432,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 377,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.16,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.733,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342875018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.92,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.922,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
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
            "name": "rollup-checkpoint-root-single-block",
            "value": 371,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.678,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.71,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345975945,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.062,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.612,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.83,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 379,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.843,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349651904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.12,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.932,
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
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 363,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.612,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361100044,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.934,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.204,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.866,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.372,
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
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 397,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 408,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.805,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.888,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764373339833,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.858,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.302,
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
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 364,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.837,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374765658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.194,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.662,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.842,
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
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.781,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.69,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585069835,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.914,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.85,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.552,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 410,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 395,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.534,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764585522713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 9.326,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.788,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 381,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.766,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587878806,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.768,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 380,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 397,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.778,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.705,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595074052,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.198,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.798,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 379,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.534,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.825,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.563,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601662014,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.022,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.622,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.942,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.388,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.64,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 390,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 376,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.804,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.563,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604192096,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.36,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.842,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 388,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.638,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 88.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.697,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605769131,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.972,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.596,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.854,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 397,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.838,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.63,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606846068,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.118,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.864,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.516,
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
            "value": 1.432,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 390,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 378,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.612,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.816,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.97,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764609329983,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.186,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.618,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 2.022,
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
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 402,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.783,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.86,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614700539,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.99,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.762,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.87,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 375,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764615561749,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.992,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.48,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.84,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 366,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.628,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.823,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.928,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621058132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.11,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.12,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.928,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.534,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.634,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677912399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.866,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.876,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
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
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 401,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 386,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764680899759,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.144,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.024,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.802,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 375,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 385,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.803,
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681027854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.046,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.572,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.87,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.446,
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
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 366,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.562,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764681515470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.066,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.052,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.846,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 376,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.548,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764682366940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.07,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.12,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.842,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 379,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.566,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682587010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.95,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.832,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.892,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 395,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 394,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 88.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.647,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692661414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.25,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.888,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 409,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 407,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.328,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.77,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.744,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764695377454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.136,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.09,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.836,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.538,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 375,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 388,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.669,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697176550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.938,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.08,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.87,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 428,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.821,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.641,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697957032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.108,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.01,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.906,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.526,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.66,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 381,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 385,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764698682174,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.994,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.91,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.894,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
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
            "value": 381,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.79,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.609,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698809642,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.914,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.834,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.948,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 380,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.618,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.819,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.975,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706868360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.264,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.982,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.886,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 387,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.654,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709177755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.138,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.044,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.802,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.384,
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
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 369,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 376,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.628,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.721,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766167238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.094,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.978,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.854,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.456,
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
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.797,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.631,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764770409679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.548,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.822,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 372,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.699,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810835579,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.998,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.67,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.908,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.584,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 365,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 386,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.74,
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
            "value": 1.773,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846809065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.048,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.996,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.988,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 366,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.794,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.793,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764854449532,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.07,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.6,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.93,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 400,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.544,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.678,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854699345,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.184,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.992,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.854,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.58,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.518,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 425,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 88.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.786,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.646,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855067226,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.942,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.296,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.844,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.426,
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
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 401,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 366,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.582,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.571,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855115396,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.562,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.01,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.882,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 383,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.08,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858601320,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.032,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.868,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 386,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 420,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.556,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.08,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.801,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.869,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862907424,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.09,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.676,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.938,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.328,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 505,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 483,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.686,
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
          "id": "80c1f3997b2db3b04869938903bbdabb5ed929e2",
          "message": "chore: do lazy cloning of `Instruction`s (#10800)",
          "timestamp": "2025-12-04T15:25:10Z",
          "tree_id": "1237a35f392c9cdd3b16a85ed83c0037e2b355de",
          "url": "https://github.com/noir-lang/noir/commit/80c1f3997b2db3b04869938903bbdabb5ed929e2"
        },
        "date": 1764863828749,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.82,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.974,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.532,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.783,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.626,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
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
          "id": "ab71130b55537fde2baca8ec7545e55c657bfcbe",
          "message": "fix(stdlib): Fix visibility of ecdsa foreign function calls (#10658)",
          "timestamp": "2025-11-27T16:55:58Z",
          "tree_id": "63db4962dfccdddbefe350cbde9252fda7d0563b",
          "url": "https://github.com/noir-lang/noir/commit/ab71130b55537fde2baca8ec7545e55c657bfcbe"
        },
        "date": 1764264974565,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.265,
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
            "value": 0.055,
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266907227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764283565161,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322919261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.27,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329179154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.341,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.264,
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
            "value": 0.087,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333106821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
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
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764334294153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
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
            "value": 24.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764342862002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.265,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342875919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.345,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.264,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345975975,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.346,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.268,
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
            "value": 0.077,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349652802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.27,
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
            "value": 0.059,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361099278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.347,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.265,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764373349516,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 24.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.261,
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
            "value": 0.079,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374776871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585075400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.349,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764585531917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.204,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.265,
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
            "value": 0.073,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587874201,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.348,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595071336,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.202,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.346,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.264,
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
            "value": 0.048,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601661414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.2,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.347,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.271,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604191724,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
            "value": 0.059,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605770574,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606846317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 24.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.341,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
            "value": 0.073,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764609331037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.261,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614702203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764615570946,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.206,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621059848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.267,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677906939,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
            "value": 0.081,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764680902141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
          "distinct": false,
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681029893,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 23.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.345,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.267,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764681513213,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
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
            "value": 24,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.259,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764682366911,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.211,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682589404,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.198,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.264,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692659787,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.271,
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
            "value": 0.059,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764695377486,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.271,
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
            "value": 0.054,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697175012,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
            "value": 0.054,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697954745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.201,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.266,
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
          "distinct": false,
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764698684596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698810721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.346,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
            "value": 0.081,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706869028,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
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
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709167181,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.197,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766157613,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764770397781,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.208,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.268,
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
            "value": 0.085,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810837331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 23.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.347,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.261,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846803862,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 24.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.265,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764854447868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.199,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.341,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
            "value": 0.054,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854699556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.261,
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
            "value": 0.05,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855063909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
            "value": 0.056,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855113955,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.016,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.209,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 25.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.342,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
          "distinct": false,
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858600632,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.198,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 24.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.263,
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
            "value": 0.054,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862907425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.207,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.008,
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
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 26.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.26,
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
      }
    ],
    "Execution Memory": [
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
          "id": "ab71130b55537fde2baca8ec7545e55c657bfcbe",
          "message": "fix(stdlib): Fix visibility of ecdsa foreign function calls (#10658)",
          "timestamp": "2025-11-27T16:55:58Z",
          "tree_id": "63db4962dfccdddbefe350cbde9252fda7d0563b",
          "url": "https://github.com/noir-lang/noir/commit/ab71130b55537fde2baca8ec7545e55c657bfcbe"
        },
        "date": 1764265680824,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764267608835,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764284265478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764323630074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329860094,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333825568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764335033569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764343567832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.68,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.02,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.61,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764343584820,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764346720634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764350425679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361828369,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764374094530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764375503812,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585801227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764586291245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764588608547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595793453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764602389692,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604920981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764606491090,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764607628908,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764610075882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764615411781,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764616320151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621779037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764678647322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764681630748,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681787856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764682259426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764683092687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764683303214,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764693398972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764696111556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697875715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764698696030,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764699408594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764699609885,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764707640158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709889612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766864416,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764771213256,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764811545925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764847550511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764855199757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764855398318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855773300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855825502,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764859309235,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764863547274,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 259.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 241.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1750,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 335.69,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.04,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.91,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.07,
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
          "id": "55fd80a80fa4771a75c1b90ffb3bd3dd3f3aeeea",
          "message": "chore(ssa): Validate that the `return_data` matches the `return` values (#10622)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-26T14:42:37Z",
          "tree_id": "0bcca4a53166689ab1eda5bf19f532fef7de6a37",
          "url": "https://github.com/noir-lang/noir/commit/55fd80a80fa4771a75c1b90ffb3bd3dd3f3aeeea"
        },
        "date": 1764169963267,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 266,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "6c92a1a9b57a5c1737d45b0474b0e2d299e057a2",
          "message": "chore: fix documentation around `current_witness_index` (#10591)",
          "timestamp": "2025-11-26T15:59:15Z",
          "tree_id": "e9c1ce6c698229c1abf72d0d372c79829b60eede",
          "url": "https://github.com/noir-lang/noir/commit/6c92a1a9b57a5c1737d45b0474b0e2d299e057a2"
        },
        "date": 1764174672768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 309,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 282,
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
          "id": "1633ddc366fd54a6a4296e0b912400deb269305d",
          "message": "chore(ssa)!: Validate that no ACIR-to-Brillig call contains a reference in SSA (#10497)",
          "timestamp": "2025-11-26T18:22:32Z",
          "tree_id": "58cb4c5cd56def7a405cbbb7cce2224f1dc8dbf6",
          "url": "https://github.com/noir-lang/noir/commit/1633ddc366fd54a6a4296e0b912400deb269305d"
        },
        "date": 1764183163118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 323,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 276,
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
          "id": "b47e62a186971a47540ff9e961da12332a5f5ec2",
          "message": "fix: Error when slice_insert is OOB during comptime (#10645)",
          "timestamp": "2025-11-26T19:11:06Z",
          "tree_id": "7178d1483c1007e9f015ecdf5c68a69209410eed",
          "url": "https://github.com/noir-lang/noir/commit/b47e62a186971a47540ff9e961da12332a5f5ec2"
        },
        "date": 1764186052238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 322,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 267,
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
          "id": "520dd4fa8afd54c3fbf7b64225a6f06e06f3691f",
          "message": "fix: several comptime interpreter fixes (#10641)",
          "timestamp": "2025-11-26T20:07:01Z",
          "tree_id": "b0c9d9339aacc3864749ce1ac1668bbae46a7e3d",
          "url": "https://github.com/noir-lang/noir/commit/520dd4fa8afd54c3fbf7b64225a6f06e06f3691f"
        },
        "date": 1764189404120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 307,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 268,
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
          "id": "ab71130b55537fde2baca8ec7545e55c657bfcbe",
          "message": "fix(stdlib): Fix visibility of ecdsa foreign function calls (#10658)",
          "timestamp": "2025-11-27T16:55:58Z",
          "tree_id": "63db4962dfccdddbefe350cbde9252fda7d0563b",
          "url": "https://github.com/noir-lang/noir/commit/ab71130b55537fde2baca8ec7545e55c657bfcbe"
        },
        "date": 1764264799780,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 286,
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
          "distinct": false,
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266751004,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 310,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764283369952,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 258,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322763848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 313,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 18,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329008796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 317,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 283,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 17,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764332953110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 304,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 265,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764334187067,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 318,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 285,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342742909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 270,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345840774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 307,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 261,
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
          "distinct": false,
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349547277,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 332,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 272,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "distinct": false,
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764360956673,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 264,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764373221531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 319,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 279,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374650565,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 183,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 304,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 262,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764584835937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764585463536,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 183,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 306,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 267,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587732711,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 305,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 139,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 269,
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
          "distinct": true,
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764594930862,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 310,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 272,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 18,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601521236,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 305,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
            "value": 0,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604031521,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 324,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 18,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605678987,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 181,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 360,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 147,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 278,
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
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606713254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 147,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 277,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764609205560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 261,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614574808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 321,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 274,
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
            "value": 1,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764615418731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 323,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 263,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764620948724,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 176,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 329,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 270,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677776636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 318,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 258,
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
          "distinct": false,
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764680845111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "distinct": false,
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764681469802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 325,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 266,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682488140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 179,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 318,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 259,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 18,
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
          "distinct": false,
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692533107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 324,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 280,
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
          "distinct": false,
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764695238380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 176,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 16,
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
            "email": "jfecher11@gmail.com",
            "name": "jfecher",
            "username": "jfecher"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697031840,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 333,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 278,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "test_report_zkpassport_noir-ecdsa_",
            "value": 4,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697810988,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 231,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 313,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 290,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698744698,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 260,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706756355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 327,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 262,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 16,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709052127,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 334,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 276,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766026903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 334,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764770296814,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 332,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 273,
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
            "value": 0,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810698865,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 265,
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
          "distinct": false,
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846654713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 327,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 278,
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
            "value": 0,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854373240,
        "tool": "customSmallerIsBetter",
        "benches": [
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764854997929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 319,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 268,
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
          "distinct": false,
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858430981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 311,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 288,
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
          "distinct": false,
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862681534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 314,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 266,
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
          "id": "80c1f3997b2db3b04869938903bbdabb5ed929e2",
          "message": "chore: do lazy cloning of `Instruction`s (#10800)",
          "timestamp": "2025-12-04T15:25:10Z",
          "tree_id": "1237a35f392c9cdd3b16a85ed83c0037e2b355de",
          "url": "https://github.com/noir-lang/noir/commit/80c1f3997b2db3b04869938903bbdabb5ed929e2"
        },
        "date": 1764863671528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 318,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 269,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 17,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 16,
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
      }
    ],
    "ACVM Benchmarks": [
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266324394,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251104,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226100,
            "range": "± 3318",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2253982,
            "range": "± 5289",
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764282954566,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257216,
            "range": "± 830",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230941,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792383,
            "range": "± 1559",
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322349312,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258881,
            "range": "± 811",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232528,
            "range": "± 3191",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795855,
            "range": "± 2659",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764328563050,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257637,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230973,
            "range": "± 7234",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797279,
            "range": "± 10807",
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764332531356,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256061,
            "range": "± 1062",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229897,
            "range": "± 4163",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792904,
            "range": "± 15388",
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764333743732,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256331,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229888,
            "range": "± 2155",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795036,
            "range": "± 1442",
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764342292070,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257271,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230466,
            "range": "± 3504",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798858,
            "range": "± 2257",
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342305023,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258204,
            "range": "± 2066",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230883,
            "range": "± 5509",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796713,
            "range": "± 7976",
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345424695,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259337,
            "range": "± 1166",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232010,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2800393,
            "range": "± 5044",
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349088704,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257066,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229683,
            "range": "± 1573",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790907,
            "range": "± 1195",
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764360532989,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257030,
            "range": "± 2549",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230407,
            "range": "± 1496",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790170,
            "range": "± 11927",
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764372787772,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257636,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231481,
            "range": "± 3205",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797892,
            "range": "± 5384",
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374230667,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256156,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229543,
            "range": "± 4892",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793236,
            "range": "± 1996",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764584478787,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257701,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232296,
            "range": "± 3603",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798180,
            "range": "± 46008",
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764584963768,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256288,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230697,
            "range": "± 1603",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793193,
            "range": "± 10450",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587322788,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249990,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220934,
            "range": "± 3235",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2253375,
            "range": "± 8611",
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764594490208,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256414,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231168,
            "range": "± 5381",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791145,
            "range": "± 7791",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601107535,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257907,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229855,
            "range": "± 1803",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793624,
            "range": "± 1687",
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764603596847,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257396,
            "range": "± 1180",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230631,
            "range": "± 1595",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797154,
            "range": "± 28188",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605201457,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256261,
            "range": "± 4236",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230730,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796565,
            "range": "± 2362",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606286978,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257019,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231220,
            "range": "± 9755",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797085,
            "range": "± 1875",
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764608775719,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258828,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231491,
            "range": "± 3908",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798761,
            "range": "± 2353",
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614132883,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257179,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229547,
            "range": "± 1558",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792978,
            "range": "± 18921",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764614979262,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257234,
            "range": "± 876",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227372,
            "range": "± 19145",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791016,
            "range": "± 37402",
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764620508233,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251934,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221509,
            "range": "± 2754",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2255925,
            "range": "± 13710",
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677336506,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260439,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230431,
            "range": "± 4503",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792730,
            "range": "± 3216",
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764680354914,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261390,
            "range": "± 1185",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229522,
            "range": "± 7306",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791177,
            "range": "± 2348",
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764680463522,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258489,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227978,
            "range": "± 4578",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2788746,
            "range": "± 4201",
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764680968940,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259673,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228169,
            "range": "± 3107",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793204,
            "range": "± 4729",
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764681790824,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264299,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236756,
            "range": "± 3519",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794618,
            "range": "± 2861",
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682024086,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259684,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228248,
            "range": "± 3640",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791257,
            "range": "± 1190",
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692078939,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257586,
            "range": "± 911",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226897,
            "range": "± 7107",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792955,
            "range": "± 5594",
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764694801452,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257489,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226348,
            "range": "± 2474",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791726,
            "range": "± 5607",
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764696582934,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258008,
            "range": "± 1053",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227392,
            "range": "± 2919",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790281,
            "range": "± 8466",
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697386097,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260353,
            "range": "± 1260",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230625,
            "range": "± 1487",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791885,
            "range": "± 9180",
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764698141232,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252074,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221716,
            "range": "± 2234",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2255846,
            "range": "± 39843",
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698247871,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252308,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221493,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2254795,
            "range": "± 1752",
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706307781,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257441,
            "range": "± 1513",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227274,
            "range": "± 5373",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793908,
            "range": "± 2457",
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764708601744,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258949,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228598,
            "range": "± 2429",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796887,
            "range": "± 46791",
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764765583688,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260382,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229119,
            "range": "± 6260",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792652,
            "range": "± 7966",
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764769853055,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261061,
            "range": "± 764",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228783,
            "range": "± 1715",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791394,
            "range": "± 3301",
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810256692,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253609,
            "range": "± 1111",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222629,
            "range": "± 3096",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2256078,
            "range": "± 4690",
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846215895,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260636,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229604,
            "range": "± 4127",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792785,
            "range": "± 7011",
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764853886142,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259160,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226007,
            "range": "± 3841",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791134,
            "range": "± 18678",
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854132433,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252912,
            "range": "± 814",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222790,
            "range": "± 2990",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2261350,
            "range": "± 2541",
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764854499036,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259517,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226268,
            "range": "± 5456",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2789714,
            "range": "± 1631",
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764854548251,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254822,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226296,
            "range": "± 3291",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790330,
            "range": "± 1628",
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858005565,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252332,
            "range": "± 721",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222381,
            "range": "± 15149",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2789382,
            "range": "± 29585",
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862248786,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255523,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 225595,
            "range": "± 3820",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790413,
            "range": "± 3590",
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
          "id": "80c1f3997b2db3b04869938903bbdabb5ed929e2",
          "message": "chore: do lazy cloning of `Instruction`s (#10800)",
          "timestamp": "2025-12-04T15:25:10Z",
          "tree_id": "1237a35f392c9cdd3b16a85ed83c0037e2b355de",
          "url": "https://github.com/noir-lang/noir/commit/80c1f3997b2db3b04869938903bbdabb5ed929e2"
        },
        "date": 1764863250102,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254893,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 225861,
            "range": "± 1409",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791811,
            "range": "± 6275",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Artifact Size": [
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266905934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764283564042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322921942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329178780,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333108612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764334294294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764342860212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 761.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2054.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 366.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.2,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49497.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.7,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342865300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345977278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349653398,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361109612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764373344513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374765808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585069871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764585522113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587874972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595072753,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601658165,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604192350,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605772415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606842787,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764609331500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614702192,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764615559679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621057714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677905293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764680899810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681031583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764681512405,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764682369187,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682590683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692662568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764695374624,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697177685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697955975,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764698685141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698811442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706866437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709167107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766158354,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764770395265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810839990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846803800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764854448417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854702854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855063959,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855116962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858601913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862908971,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "80c1f3997b2db3b04869938903bbdabb5ed929e2",
          "message": "chore: do lazy cloning of `Instruction`s (#10800)",
          "timestamp": "2025-12-04T15:25:10Z",
          "tree_id": "1237a35f392c9cdd3b16a85ed83c0037e2b355de",
          "url": "https://github.com/noir-lang/noir/commit/80c1f3997b2db3b04869938903bbdabb5ed929e2"
        },
        "date": 1764863829101,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 762.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2055.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 453.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 213.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 296.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49453.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49496.5,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4921.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.8,
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
          "id": "ab71130b55537fde2baca8ec7545e55c657bfcbe",
          "message": "fix(stdlib): Fix visibility of ecdsa foreign function calls (#10658)",
          "timestamp": "2025-11-27T16:55:58Z",
          "tree_id": "63db4962dfccdddbefe350cbde9252fda7d0563b",
          "url": "https://github.com/noir-lang/noir/commit/ab71130b55537fde2baca8ec7545e55c657bfcbe"
        },
        "date": 1764264975746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "6ea935ed11ad2f142703db8c8bdee95e67232db4",
          "message": "chore: check stdout of comptime interpret tests (#10667)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-11-27T17:35:46Z",
          "tree_id": "d7990a9e6d8da26eb5c4a8fda536f291d476ede8",
          "url": "https://github.com/noir-lang/noir/commit/6ea935ed11ad2f142703db8c8bdee95e67232db4"
        },
        "date": 1764266915031,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "b378d13b75433adb895987f9f4d1898895380bd3",
          "message": "chore(stdlib): Fix `__get_shuffle_indices` to use `break` (#10673)",
          "timestamp": "2025-11-27T22:14:42Z",
          "tree_id": "cf23915cbfcf14698c97cbd4697efd6219d38736",
          "url": "https://github.com/noir-lang/noir/commit/b378d13b75433adb895987f9f4d1898895380bd3"
        },
        "date": 1764283560839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "cba5f56a58fcbc627020ad72c0611c3393f95ce4",
          "message": "fix: do not deduplicate ifelse for Brillig arrays (#10668)",
          "timestamp": "2025-11-28T09:09:30Z",
          "tree_id": "31891ad18d9b2eca561de91f96728b2fed7b61f6",
          "url": "https://github.com/noir-lang/noir/commit/cba5f56a58fcbc627020ad72c0611c3393f95ce4"
        },
        "date": 1764322919614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3ae1278cf11fbc4cbe2a789b2434cb82069d1be0",
          "message": "chore(deps): bump node-forge from 1.3.1 to 1.3.2 (#10674)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-28T10:53:58Z",
          "tree_id": "0fc223e38503a0b24c6824a45605d53a1bbac580",
          "url": "https://github.com/noir-lang/noir/commit/3ae1278cf11fbc4cbe2a789b2434cb82069d1be0"
        },
        "date": 1764329180919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "d5f1d15992c399fc7650b42bd469bbe312b16a58",
          "message": "fix: subtraction operator for witness (#10675)",
          "timestamp": "2025-11-28T12:00:41Z",
          "tree_id": "6eae890e7701712c8f50d6d4b4dccef55734bf7d",
          "url": "https://github.com/noir-lang/noir/commit/d5f1d15992c399fc7650b42bd469bbe312b16a58"
        },
        "date": 1764333105645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "6a42a5489037666ce762e2a3f1e83b2e0c54489c",
          "message": "chore!: Do not allow returning functions from unconstrained to constrained (#10666)",
          "timestamp": "2025-11-28T12:09:50Z",
          "tree_id": "c1adb209fd22b93fc5471d7e2ec23d1adc041f3d",
          "url": "https://github.com/noir-lang/noir/commit/6a42a5489037666ce762e2a3f1e83b2e0c54489c"
        },
        "date": 1764334297461,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "4d286678bfc1168c9dae7f65cc6fbabb4c9b9707",
          "message": "chore(comptime): Remove redundant overflow check when shifting (#10650)",
          "timestamp": "2025-11-28T14:42:16Z",
          "tree_id": "bf509688916ddd2afcd6e84906ec8e41e0282dd8",
          "url": "https://github.com/noir-lang/noir/commit/4d286678bfc1168c9dae7f65cc6fbabb4c9b9707"
        },
        "date": 1764342860122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "6f2deba980ded9d0c917bfb8c0200c2129e81dd3",
          "message": "chore: Infer that a lambda given to an unconstrained function can only be unconstrained (#10661)",
          "timestamp": "2025-11-28T14:43:09Z",
          "tree_id": "23c24788d3023a5481257306718b2f5d831154a7",
          "url": "https://github.com/noir-lang/noir/commit/6f2deba980ded9d0c917bfb8c0200c2129e81dd3"
        },
        "date": 1764342865636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "8eca323daa078b5aa207fbb4133fd5b4f23ae1e3",
          "message": "fix: avoid adding default entry (#10679)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-28T15:34:20Z",
          "tree_id": "2b9537f46c5ac3b39a7b6ae463ef47777d63fa57",
          "url": "https://github.com/noir-lang/noir/commit/8eca323daa078b5aa207fbb4133fd5b4f23ae1e3"
        },
        "date": 1764345976981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "5a8e87e87d01fd1a3e08484132abf0d21a7e102d",
          "message": "chore: update comment (#10676)",
          "timestamp": "2025-11-28T16:36:04Z",
          "tree_id": "17ac595404d6aa2978c4436a6cbe9c667bce7f4d",
          "url": "https://github.com/noir-lang/noir/commit/5a8e87e87d01fd1a3e08484132abf0d21a7e102d"
        },
        "date": 1764349672655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "4ed0303f50d0a73bac518f38c71f555381a3c170",
          "message": "chore: remove unused `Ord` implementation on `Expression` (#10685)",
          "timestamp": "2025-11-28T19:47:05Z",
          "tree_id": "c81279d6535ee84664a500aada2a1a31070252ad",
          "url": "https://github.com/noir-lang/noir/commit/4ed0303f50d0a73bac518f38c71f555381a3c170"
        },
        "date": 1764361098804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "100a0cac7ea25eaf69567f95deb426ba58ac54b4",
          "message": "chore: disallow databus outside of main() (#10682)",
          "timestamp": "2025-11-28T23:11:53Z",
          "tree_id": "94ae4ba5eb4c3cdfe8ed731dcd550ab553d48803",
          "url": "https://github.com/noir-lang/noir/commit/100a0cac7ea25eaf69567f95deb426ba58ac54b4"
        },
        "date": 1764373339663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "5afaaeba5756755939511890872232daf68d9c16",
          "message": "chore: simplify function signature of `range_constrain_var` (#10677)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-28T23:35:12Z",
          "tree_id": "14c5c832a3860a32763d28886d482f2326fac439",
          "url": "https://github.com/noir-lang/noir/commit/5afaaeba5756755939511890872232daf68d9c16"
        },
        "date": 1764374765133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ab3836c51c999c32fb799f4d66d9ca0d0d98626d",
          "message": "chore(deps-dev): bump typedoc from 0.28.14 to 0.28.15 in the typedoc group (#10701)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T09:58:33Z",
          "tree_id": "a95bb6099a55f8deed493c881feec2246c50a8dd",
          "url": "https://github.com/noir-lang/noir/commit/ab3836c51c999c32fb799f4d66d9ca0d0d98626d"
        },
        "date": 1764585071027,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "316aef679a9cd8a1faabec66c7052f7ee4319bfc",
          "message": "feat: remove bit shifts by small known amounts in DIE (#10680)",
          "timestamp": "2025-12-01T10:04:14Z",
          "tree_id": "981dabddefba00396abeeb83927ff4dac5fe25ac",
          "url": "https://github.com/noir-lang/noir/commit/316aef679a9cd8a1faabec66c7052f7ee4319bfc"
        },
        "date": 1764585533813,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3261da3549c220ecb938b0fd46f4bfa1bcec0d9",
          "message": "chore(deps): bump the linter group with 2 updates (#10700)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T10:46:31Z",
          "tree_id": "c9f783536409e472169864f177e5803675cd057a",
          "url": "https://github.com/noir-lang/noir/commit/c3261da3549c220ecb938b0fd46f4bfa1bcec0d9"
        },
        "date": 1764587875347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "e5587f8509fbc92ebd15ad8c13a6e44b96d1154b",
          "message": "chore: bump webpack deps (#10708)",
          "timestamp": "2025-12-01T13:05:43Z",
          "tree_id": "7003ea85cfed7def46054b580e0e8f4143a3a75d",
          "url": "https://github.com/noir-lang/noir/commit/e5587f8509fbc92ebd15ad8c13a6e44b96d1154b"
        },
        "date": 1764595074687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dc62efd79cc1173302234e27e079d5471ec8cef",
          "message": "chore(deps): bump tslog from 4.9.3 to 4.10.2 (#10714)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T14:34:58Z",
          "tree_id": "524eb588bd4d8e724b777309b0715d90415d4558",
          "url": "https://github.com/noir-lang/noir/commit/7dc62efd79cc1173302234e27e079d5471ec8cef"
        },
        "date": 1764601661870,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "2aa30d22e83ce2b3b642b1758f57acc0dc303cf2",
          "message": "chore: check for nested slices during monomorphization (#10610)",
          "timestamp": "2025-12-01T15:17:37Z",
          "tree_id": "6f8b3d06509c44d0643471f74aa9c38f3715dca3",
          "url": "https://github.com/noir-lang/noir/commit/2aa30d22e83ce2b3b642b1758f57acc0dc303cf2"
        },
        "date": 1764604194634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11a6a3b472158a5c4474cab52ab6c056a6b1b98f",
          "message": "chore(deps): bump @easyops-cn/docusaurus-search-local from 0.35.0 to 0.52.2 (#10713)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-01T15:44:51Z",
          "tree_id": "de0207a4cb43700db9bae4abdf5ce51924c1b7ed",
          "url": "https://github.com/noir-lang/noir/commit/11a6a3b472158a5c4474cab52ab6c056a6b1b98f"
        },
        "date": 1764605770340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6741e02edf075d1e1e1296542ee0c80de4b5e970",
          "message": "fix(ssa): Fix cast/truncate handling with lookback (#10646)",
          "timestamp": "2025-12-01T16:02:31Z",
          "tree_id": "335d6ab38740f6ef8aacd80a85f81baecebe6d2f",
          "url": "https://github.com/noir-lang/noir/commit/6741e02edf075d1e1e1296542ee0c80de4b5e970"
        },
        "date": 1764606844684,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "df9c20a655a741a1201b8764aa06200883a99cca",
          "message": "fix(comptime): Validate that radix decomposition fits in the specified limbs (#10656)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-12-01T16:44:33Z",
          "tree_id": "5293a81e03364ad9e2b2274f34650a45fed4b87a",
          "url": "https://github.com/noir-lang/noir/commit/df9c20a655a741a1201b8764aa06200883a99cca"
        },
        "date": 1764609332617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "a0f05179e5db8b6bc82fbc256655bf24a577c1a0",
          "message": "fix: address off-by-one error when calculating bitsize of remainder (#10721)",
          "timestamp": "2025-12-01T18:32:51Z",
          "tree_id": "6e78176f48f1295a98b7a4641394d3226f506b64",
          "url": "https://github.com/noir-lang/noir/commit/a0f05179e5db8b6bc82fbc256655bf24a577c1a0"
        },
        "date": 1764614711809,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d46fca7203545cbbfb31a0d0328de6c10a8db95",
          "message": "chore: Release Noir(1.0.0-beta.16) (#10486)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-12-01T18:47:07Z",
          "tree_id": "b50b98c27e9075dfa699c9bb7d314541bdd36f39",
          "url": "https://github.com/noir-lang/noir/commit/2d46fca7203545cbbfb31a0d0328de6c10a8db95"
        },
        "date": 1764615562152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "e47304d210c6d9510d1412d14436871446287d61",
          "message": "chore(comptime): Additional cast test cases (#10649)",
          "timestamp": "2025-12-01T19:58:39Z",
          "tree_id": "c594fe09e558a96c8ba3be6c1048e6f5a9dd6ce1",
          "url": "https://github.com/noir-lang/noir/commit/e47304d210c6d9510d1412d14436871446287d61"
        },
        "date": 1764621058418,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "131f9bc7bdd53b7b529ea7868bb5b5605baccf8d",
          "message": "fix: Fix no numeric generic given leading to panic (#10725)",
          "timestamp": "2025-12-02T11:46:46Z",
          "tree_id": "3c1fbe8cead4eae925b00b2022356191c4af7640",
          "url": "https://github.com/noir-lang/noir/commit/131f9bc7bdd53b7b529ea7868bb5b5605baccf8d"
        },
        "date": 1764677903764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "2c47c5ab47c230cfa186c2eb27383cc8f5b533ce",
          "message": "fix: apply_range_constraint off-by-one error (#10692)",
          "timestamp": "2025-12-02T12:36:21Z",
          "tree_id": "945cec80233a123fbc019fe91a50f6b3e6c2fb0b",
          "url": "https://github.com/noir-lang/noir/commit/2c47c5ab47c230cfa186c2eb27383cc8f5b533ce"
        },
        "date": 1764680900384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "99f503dc97f45d9fbc48c89324d1d92274e16b3c",
          "message": "chore: no need to get all fields to fetch one (#10687)",
          "timestamp": "2025-12-02T12:37:43Z",
          "tree_id": "aacbe2321e324c3391be63aea912e9044d2f6c64",
          "url": "https://github.com/noir-lang/noir/commit/99f503dc97f45d9fbc48c89324d1d92274e16b3c"
        },
        "date": 1764681030411,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a",
          "message": "chore: simplify evaluate_ordering (#10681)",
          "timestamp": "2025-12-02T12:48:22Z",
          "tree_id": "84d2d7fd04b4528e7ebab51111044d8f67fe8da3",
          "url": "https://github.com/noir-lang/noir/commit/ffa50afedcd00e3c00fd245d1128c1fd9c7f0e0a"
        },
        "date": 1764681514958,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9",
          "message": "fix: implement `checked_transmute` in the comptime interpreter (#10732)",
          "timestamp": "2025-12-02T12:56:54Z",
          "tree_id": "2348dfff3fd4bfda292faaf6df3631cd7981a29b",
          "url": "https://github.com/noir-lang/noir/commit/a4391f8c4dd5dfb5b8e559527bfc3cfbed5ad6b9"
        },
        "date": 1764682366757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "bc40e892bdc179077cb08a30965aad8d4c2247bd",
          "message": "fix: Capture variables in lamdba by copy (#10683)",
          "timestamp": "2025-12-02T13:04:17Z",
          "tree_id": "4dd4418e5c4014a5ac7b346a6620947340ab7dc1",
          "url": "https://github.com/noir-lang/noir/commit/bc40e892bdc179077cb08a30965aad8d4c2247bd"
        },
        "date": 1764682589768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "a15b88f0dcac3961ccf95b722a317f9257b431ed",
          "message": "chore: simplify `evaluate_integer` (#10665)",
          "timestamp": "2025-12-02T15:50:46Z",
          "tree_id": "32932909a3262f14c22473e3e139cab6f98ee82f",
          "url": "https://github.com/noir-lang/noir/commit/a15b88f0dcac3961ccf95b722a317f9257b431ed"
        },
        "date": 1764692662919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "a3b5f9752ce0141df75db30edaeb121e5e010e3d",
          "message": "feat: remove `bounded-codegen` feature from ACIRgen (#10693)",
          "timestamp": "2025-12-02T16:37:12Z",
          "tree_id": "60d49741a148e7f9945367d37a6a7c31ff5d7e4f",
          "url": "https://github.com/noir-lang/noir/commit/a3b5f9752ce0141df75db30edaeb121e5e010e3d"
        },
        "date": 1764695376092,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "distinct": false,
          "id": "9493100ea745f2bea79632f4132220b773494a01",
          "message": "fix: Fix calling type variables of kind `Any` (#10724)",
          "timestamp": "2025-12-02T17:05:29Z",
          "tree_id": "d5fe3133c9d0854a9fbc4440dcf6273fbfe19f8d",
          "url": "https://github.com/noir-lang/noir/commit/9493100ea745f2bea79632f4132220b773494a01"
        },
        "date": 1764697199631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "e76a83247cd3e880b286e9805a3e83a0a29cb575",
          "message": "fix: slice with zero size elements (#10716)",
          "timestamp": "2025-12-02T17:12:56Z",
          "tree_id": "7e3b6c40f514bc4cdfa793646fc665b8dc5db453",
          "url": "https://github.com/noir-lang/noir/commit/e76a83247cd3e880b286e9805a3e83a0a29cb575"
        },
        "date": 1764697953836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "14552bddf7089998d29af9f066d109c114c6c343",
          "message": "chore: remove catch-all branch in `array_set` + add missing panic (#10586)",
          "timestamp": "2025-12-02T17:21:00Z",
          "tree_id": "1e287c63cefaa5722689fa446b3ce48bba6eb6e7",
          "url": "https://github.com/noir-lang/noir/commit/14552bddf7089998d29af9f066d109c114c6c343"
        },
        "date": 1764698684429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "919cbce9620d3e7bff55f1497de537350333c794",
          "message": "chore(audit): `path_resolution`  (#10717)",
          "timestamp": "2025-12-02T17:35:36Z",
          "tree_id": "c7c51eb6b56a6f013db6fcc73fdd32cccc1f678b",
          "url": "https://github.com/noir-lang/noir/commit/919cbce9620d3e7bff55f1497de537350333c794"
        },
        "date": 1764698813389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "00eadc5b94f900fac3eede72d9024d61086329f4",
          "message": "fix: do not crash on invalid strings (#10739)",
          "timestamp": "2025-12-02T19:45:07Z",
          "tree_id": "2aff3cd1a0eafcd52e69426ecccef6465d2cfd42",
          "url": "https://github.com/noir-lang/noir/commit/00eadc5b94f900fac3eede72d9024d61086329f4"
        },
        "date": 1764706865964,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "a5edaa61161198bd1f0f66952c671bc879366dfd",
          "message": "fix: Error on duplicate field in set_fields (#10726)",
          "timestamp": "2025-12-02T20:26:44Z",
          "tree_id": "51ee42dc7d57be7a2e701ddb40bbebdc37a10fbe",
          "url": "https://github.com/noir-lang/noir/commit/a5edaa61161198bd1f0f66952c671bc879366dfd"
        },
        "date": 1764709170174,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "e948361e67bec5ed45196879c3619067d561718e",
          "message": "chore(ssa_verification): retest acir relations (#10729)",
          "timestamp": "2025-12-03T12:17:34Z",
          "tree_id": "03735ed4539ee255f170913bfbccec9c1112a8c3",
          "url": "https://github.com/noir-lang/noir/commit/e948361e67bec5ed45196879c3619067d561718e"
        },
        "date": 1764766159054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "2a925b031d6480def78e2722451e6ed9d4f87fca",
          "message": "chore: use `NOIR_REPO_TOKEN` for triggering binary builds for release (#10744)",
          "timestamp": "2025-12-03T13:48:21Z",
          "tree_id": "ccaf1d96ce40a8aef41c29f02064ba7cf5ab360a",
          "url": "https://github.com/noir-lang/noir/commit/2a925b031d6480def78e2722451e6ed9d4f87fca"
        },
        "date": 1764770408101,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "7615632df856497750d6c0f856643a93df7dc40f",
          "message": "fix: keep track of comptime closure callstack (#10735)",
          "timestamp": "2025-12-04T00:42:17Z",
          "tree_id": "96015982954a251e327c0d6d3429b940ed7d767a",
          "url": "https://github.com/noir-lang/noir/commit/7615632df856497750d6c0f856643a93df7dc40f"
        },
        "date": 1764810836320,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "6fa1a4bd35006b292e21b26b217f20101d84c8e1",
          "message": "feat(doc): mobile style (#10760)",
          "timestamp": "2025-12-04T10:41:49Z",
          "tree_id": "2069789883be06ed9e4588b5a061737f19e57647",
          "url": "https://github.com/noir-lang/noir/commit/6fa1a4bd35006b292e21b26b217f20101d84c8e1"
        },
        "date": 1764846804525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "40146b93c0ee38bdf55dd13058ac966c77a83118",
          "message": "fix(LSP): correct link range for doc comment references (#10769)",
          "timestamp": "2025-12-04T12:49:02Z",
          "tree_id": "ec436fb8c3a5c32a984cd2ebbd583cd4f31c5b90",
          "url": "https://github.com/noir-lang/noir/commit/40146b93c0ee38bdf55dd13058ac966c77a83118"
        },
        "date": 1764854447477,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "20473e1c85f835cf8d25ac1efd64fff292b78ec4",
          "message": "fix(lexer): don't create broken span on broken interpolation (#10722)",
          "timestamp": "2025-12-04T12:53:19Z",
          "tree_id": "e462e2174c33e65f20ba9e372ccd7ad4eaf4cc81",
          "url": "https://github.com/noir-lang/noir/commit/20473e1c85f835cf8d25ac1efd64fff292b78ec4"
        },
        "date": 1764854699564,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "929790438ed336cf246072b4c3e13df1a5199bdd",
          "message": "feat: always perform pedantic checks on embedded curve operations (#10776)",
          "timestamp": "2025-12-04T12:59:12Z",
          "tree_id": "dc76457b7dbdd7ac57e8bd5acbcf4cefbb65d385",
          "url": "https://github.com/noir-lang/noir/commit/929790438ed336cf246072b4c3e13df1a5199bdd"
        },
        "date": 1764855064567,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "6d9dcae76a4cd9e1d756b14e27604970e01098e0",
          "message": "feat: always check bitsize of logical operation inputs (#10750)",
          "timestamp": "2025-12-04T12:59:44Z",
          "tree_id": "ce934dc31c70456f3b3200e4d7d44742f5425cca",
          "url": "https://github.com/noir-lang/noir/commit/6d9dcae76a4cd9e1d756b14e27604970e01098e0"
        },
        "date": 1764855116952,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "caaf7c3d601533e546119994a2798a6f5454083d",
          "message": "fix: SignedField Eq and Hash implementations (#10671)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-12-04T13:57:26Z",
          "tree_id": "9db6f7e74d7220e5bf7e4ee7bf3aef401c767717",
          "url": "https://github.com/noir-lang/noir/commit/caaf7c3d601533e546119994a2798a6f5454083d"
        },
        "date": 1764858610795,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18083,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80482,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8549,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262386,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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
          "id": "b5f559fd7aff6449a13a8687f3f0c1706bbf7549",
          "message": "fix: element type sizes array has extra room for slice_insert (#10742)",
          "timestamp": "2025-12-04T15:06:39Z",
          "tree_id": "948901478f12bd97d5424ce4380c6b03e8a2c009",
          "url": "https://github.com/noir-lang/noir/commit/b5f559fd7aff6449a13a8687f3f0c1706bbf7549"
        },
        "date": 1764862914985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18088,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80496,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 8554,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1083,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 968,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2346,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1823214,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306632,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262388,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1478,
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