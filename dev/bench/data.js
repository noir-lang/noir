window.BENCHMARK_DATA = {
  "lastUpdate": 1763759145494,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762871563278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762872214665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.56,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.61,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762880678372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.54,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762881561149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.61,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889823992,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.56,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762890189317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
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
          "distinct": false,
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890800488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.56,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.53,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.07,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.59,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762895494676,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.86,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.36,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.37,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.57,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.39,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762897150667,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.86,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.36,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.37,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.57,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.39,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.32,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762897253605,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": false,
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762907117535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762956070745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762957041798,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.61,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958816407,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.57,
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762959130882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.61,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762959152393,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762959251144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762964657637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762965147305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.61,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.61,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762975586124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": true,
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762976170101,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.61,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763041012601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763062339738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
            "email": "rkarabut@users.noreply.github.com",
            "name": "Ratmir Karabut",
            "username": "rkarabut"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063967399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.6,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763064462049,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.61,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763064702478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 267.87,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.61,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.12,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.4,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.6,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763139678692,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763149591019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": false,
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149870037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763151016104,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.62,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159787018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.62,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": false,
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763226310458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 334.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 332.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.62,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.59,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.13,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 332.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.6,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763384387891,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.87,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.9,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763385394994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.88,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763389508044,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.87,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763490286131,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763491164409,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.87,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": true,
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491854635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.87,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "distinct": true,
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493783936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.62,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763560051388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.41,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.41,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.89,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.33,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763564742849,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "distinct": true,
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566989769,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763574275942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763574465854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "distinct": true,
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763634607165,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.75,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763651529658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.4,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.77,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763653626922,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763654228352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763654553269,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
          "distinct": true,
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763747138365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 268.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 494.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 336.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 334.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.58,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.52,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11240,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 338.99,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1070,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.36,
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
      }
    ],
    "Compilation Time": [
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762871416898,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.904,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.932,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.778,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.386,
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
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 408,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.807,
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
          "distinct": false,
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879885778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.37,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.822,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 397,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.521,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880806239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.948,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.158,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.702,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 395,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.338,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.809,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.957,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889145343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.1,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.448,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.724,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 384,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 535,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.96,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.777,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.555,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762889379912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.94,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.132,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.766,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 394,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 413,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.921,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890014312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.928,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.616,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.796,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.712,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894760943,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.09,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.842,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.706,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.1,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.828,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.757,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762896387702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.91,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.694,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.742,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 426,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.831,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.584,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762896492453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.978,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.656,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.754,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.356,
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
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 407,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.566,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.16,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.781,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.533,
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762906342376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.034,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.242,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 401,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.762,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.761,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762955311600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.22,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.644,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.726,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 401,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 409,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.526,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762956238695,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.468,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.668,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958075259,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.184,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.664,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.678,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
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
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 461,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.773,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.572,
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762958362631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.886,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762958343542,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.194,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.602,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.682,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 421,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.597,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762958482285,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.96,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.406,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 397,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 401,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.804,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.885,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963936401,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.908,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.276,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 385,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 468,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.16,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.318,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.79,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.62,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762964408043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.19,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.832,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 455,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.26,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.796,
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
          "distinct": false,
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974790987,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.914,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.686,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.718,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 399,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.532,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.67,
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762975450714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.996,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.954,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 484,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 434,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.324,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.804,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.801,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763040282463,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.228,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.316,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.744,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 432,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.3,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.838,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.817,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061573813,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.064,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.98,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.812,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 392,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.822,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.862,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063156737,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.09,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.69,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.694,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.452,
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
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 395,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.304,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763063694572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.876,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.768,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.562,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 426,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.851,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.593,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063945899,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.94,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.808,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 426,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138923599,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.256,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.502,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.766,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 428,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 444,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 24.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.864,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148803479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.992,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.794,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.708,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
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
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.8,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.845,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.626,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149082389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.142,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.854,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 391,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 87.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.606,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763150262227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.048,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.81,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.79,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.34,
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
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 430,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 398,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.546,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159015660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.052,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.576,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.69,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 415,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.536,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.786,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763225464398,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.91,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.684,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.776,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.31,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.686,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.809,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.68,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383640572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.924,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.226,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.722,
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
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 368,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.319,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384669380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.064,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.786,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.882,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 379,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.74,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388757066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.932,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.992,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.546,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 386,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 23.1,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 86.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.83,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.737,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489560823,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.08,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.84,
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
            "value": 387,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 386,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.536,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 24.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.16,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.786,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763490398420,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.022,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.064,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.814,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.31,
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
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 375,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.827,
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491075159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.022,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.846,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.682,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.31,
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
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 378,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.574,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.763,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.625,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493038655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.986,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.774,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.738,
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
            "value": 375,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 389,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.801,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.568,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763559297534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.99,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.696,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.686,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.354,
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
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 369,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 390,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.835,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.627,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563989614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.118,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.594,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.812,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
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
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 26.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.316,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.68,
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566242480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.98,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.744,
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
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 368,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.528,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.778,
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763573514719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.098,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.798,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.69,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.458,
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
            "value": 1.458,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 387,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.106,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573687451,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.064,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.264,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.706,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 399,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 377,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.624,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633903990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.936,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.734,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
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
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 367,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 432,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.822,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650803394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.91,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.908,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 384,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 371,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 25.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.78,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652903010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.744,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.646,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.372,
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
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 390,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.756,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763653513762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.008,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.108,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.642,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.432,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 421,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.96,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 85.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.8,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.615,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653810034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.026,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.808,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.722,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.364,
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
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 403,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 371,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.548,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 83.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.552,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.842,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.703,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763746443760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.146,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.54,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.532,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 382,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 409,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.558,
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
          "id": "0ffef1c383ce73611cad6cdd678bce64a1cbceb1",
          "message": "feat(LSP): semantic tokens for doc comment code blocks (#10565)",
          "timestamp": "2025-11-21T20:34:44Z",
          "tree_id": "300a8b7765cf167d9ac9a0c595ce2eda63ec5f5a",
          "url": "https://github.com/noir-lang/noir/commit/0ffef1c383ce73611cad6cdd678bce64a1cbceb1"
        },
        "date": 1763759118635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.926,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.508,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.706,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 399,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.548,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 21.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 84.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.714,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762870928812,
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
            "value": 0.007,
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
            "value": 23.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.3,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762871423526,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.1,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879885351,
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
            "value": 22.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880804966,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 23.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.7,
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
            "value": 0.072,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889146232,
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
            "value": 22.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 24.5,
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
            "value": 0.07,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762889379898,
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
            "value": 23.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 24,
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
            "value": 0.008,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890012617,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 22.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.338,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.257,
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
            "value": 0.063,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894758997,
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
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.7,
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
            "value": 0.091,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762896384742,
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
            "value": 23.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.262,
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
            "value": 0.092,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762896492158,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 23.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
            "value": 0.047,
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762906347437,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 23,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
            "value": 0.072,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762955310306,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 22.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762956235811,
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
            "value": 22.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.3,
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
            "value": 0.049,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958077230,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 22.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 24.2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.255,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762958342406,
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
            "value": 23,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.337,
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
            "value": 0.008,
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
          "distinct": false,
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762958364917,
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
            "value": 23.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.4,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762958481871,
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
            "value": 23.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.269,
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
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963938889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.195,
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
            "value": 22.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.9,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.074,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762964405668,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 23.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 24.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.338,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974788940,
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
            "value": 0.007,
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
            "value": 23,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.2,
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
          "distinct": true,
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762975453595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.338,
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
            "value": 0.072,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763040290261,
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
            "value": 23,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.338,
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
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.076,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061571179,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.196,
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
            "value": 22.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.337,
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
            "value": 0.074,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063157352,
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
            "value": 0.007,
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
            "value": 23.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.339,
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
            "value": 0.008,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763063695768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.195,
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
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.338,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.258,
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
            "value": 0.074,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063943969,
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
            "value": 22.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.8,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.337,
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
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.069,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138922063,
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
            "value": 22.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.4,
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
            "value": 0.258,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148812646,
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
            "value": 22.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.2,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149081068,
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
            "value": 22.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 23.3,
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
            "value": 0.008,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763150253029,
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
            "value": 28.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 29.7,
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
            "value": 0.08,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159032681,
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
            "value": 28.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 29.1,
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
            "value": 0.008,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763225466131,
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
            "value": 26.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 28.2,
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
            "value": 0.071,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383638722,
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
          "distinct": false,
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384669258,
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
            "value": 24.2,
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
            "value": 0.343,
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
            "value": 0.074,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388758078,
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
            "value": 24.7,
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
            "value": 0.01,
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
          "distinct": false,
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489563046,
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
            "value": 25.9,
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
            "value": 0.084,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763490389861,
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
            "value": 0.076,
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491074791,
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
            "value": 0.346,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493038770,
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
            "value": 24.7,
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
            "value": 0.347,
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
            "value": 0.072,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763559297660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.212,
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
            "value": 0.077,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563991767,
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
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27.6,
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
            "value": 0.347,
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
          "distinct": true,
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566251899,
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
            "value": 25.5,
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
          "distinct": false,
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763573513215,
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
            "value": 24.5,
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
            "value": 0.345,
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
            "value": 0.06,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573736676,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.212,
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
            "value": 0.008,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633905254,
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
            "value": 23.7,
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
            "value": 0.079,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650806002,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652903900,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.214,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763653511885,
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
            "value": 24.7,
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
            "value": 0.088,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653811604,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.212,
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
            "value": 23.9,
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
            "value": 0.343,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763746444400,
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
            "value": 0.054,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Memory": [
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762871561821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762872203936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762880676517,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762881557580,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889816201,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762890176190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890799149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.39,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.43,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.11,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.58,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762895510202,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.43,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.83,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.15,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762897147615,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.43,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.83,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.15,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.08,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762897252443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.71,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.94,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762907117193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762956070446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762957036483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958814059,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762959124156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762959150193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762959252206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762964662147,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762965144547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762975585301,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762976170057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763041025744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.93,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763062339528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.94,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063967685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.94,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763064465464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.94,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763064702299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.48,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.7,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.16,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.64,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.94,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763139676991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763149599216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149870700,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763151015722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159785856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763226293384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 333.73,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 332.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1760,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 333.72,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 521.17,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 467.65,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 331.95,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763384388106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.81,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.31,
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
            "value": 335.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 469.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763385391682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.81,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.31,
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
            "value": 335.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 469.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763389514258,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.81,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.31,
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
            "value": 335.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 469.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763490288270,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 258.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 291.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 238.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.81,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.31,
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
            "value": 335.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.22,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 469.42,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.09,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763491166855,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.83,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.26,
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
            "value": 335.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.92,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.81,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.04,
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491843770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.83,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.26,
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
            "value": 335.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.92,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.81,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.04,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493781850,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.83,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.26,
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
            "value": 335.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.92,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.81,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.04,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763560060408,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.83,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.26,
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
            "value": 335.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.92,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.5,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.81,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.85,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.04,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763564740990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566996866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763574268472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763574459399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763634593919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763651526427,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763653622942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763654230056,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.94,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763654563019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763747140552,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 257.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 290.32,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 237.78,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 335.93,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 334.37,
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
            "value": 335.55,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 524.03,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.6,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 333.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.92,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 72.1,
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
          "id": "1bbaffe34b868226cd95d4282a668b7e57feac9d",
          "message": "fix(acir-gen): Use the side effect variable in `slice_pop_back` (#10455)",
          "timestamp": "2025-11-10T19:34:08Z",
          "tree_id": "e401b06f01a14a5f8b67ba42a8fa92fa086d1159",
          "url": "https://github.com/noir-lang/noir/commit/1bbaffe34b868226cd95d4282a668b7e57feac9d"
        },
        "date": 1762810057389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 603,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 286,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 385,
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
          "id": "2ff4d11df5fecff666e9e60dbfef1255d11c73c9",
          "message": "chore: lock Cargo.lock in cargo-binstall (#10459)",
          "timestamp": "2025-11-10T20:42:33Z",
          "tree_id": "7e8881e080c6aa9f6fa8da098f335f2509612963",
          "url": "https://github.com/noir-lang/noir/commit/2ff4d11df5fecff666e9e60dbfef1255d11c73c9"
        },
        "date": 1762812160674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 409,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 303,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 354,
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
          "id": "c117dfab68bfe7e01465259314eda6abaf7a5a8c",
          "message": "fix(brillig_gen): Switch to iterative variable liveness (#10460)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-10T22:23:08Z",
          "tree_id": "26a1a595da6f10c7e1fd03ea89e99d1ba79f67dc",
          "url": "https://github.com/noir-lang/noir/commit/c117dfab68bfe7e01465259314eda6abaf7a5a8c"
        },
        "date": 1762815896382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 420,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 307,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
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
          "id": "e19c04aa32719ebf1eecd6fcab6620ef199ff4df",
          "message": "chore(fuzzing): fix default artifact for brillig target  (#10465)",
          "timestamp": "2025-11-11T10:44:31Z",
          "tree_id": "85af80327d551fe88bf132ea8c64673ba1b2380e",
          "url": "https://github.com/noir-lang/noir/commit/e19c04aa32719ebf1eecd6fcab6620ef199ff4df"
        },
        "date": 1762859643753,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 295,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 386,
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
          "id": "2a27b186687cab46e6976a68250833d63fd402ec",
          "message": "fix: disallow comptime-only types in non-comptime globals (#10458)",
          "timestamp": "2025-11-11T12:35:15Z",
          "tree_id": "89a3e2290acf0a9d5ab7a5bbd9600ac727f35b31",
          "url": "https://github.com/noir-lang/noir/commit/2a27b186687cab46e6976a68250833d63fd402ec"
        },
        "date": 1762866389010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 403,
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762870724046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 301,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
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
          "distinct": false,
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762871436909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 304,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 334,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879859687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 429,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 293,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
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
          "distinct": true,
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880848287,
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
            "value": 493,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 310,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 344,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762888665498,
        "tool": "customSmallerIsBetter",
        "benches": [
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762889322240,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 342,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890058211,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 434,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 299,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894612270,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 373,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 334,
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
            "value": 0,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762896509060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 415,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 277,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 321,
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
          "distinct": false,
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762906288985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 396,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 289,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 113,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762955220910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 371,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 303,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 344,
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
            "value": 0,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762956272983,
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
            "value": 489,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 306,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 341,
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
          "distinct": false,
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762957635119,
        "tool": "customSmallerIsBetter",
        "benches": [
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762958543665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 474,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 288,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 367,
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
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963674388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 149,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762964372852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 305,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 397,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974702656,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 292,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762975391092,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 376,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 145,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 403,
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
          "distinct": true,
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763040214848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 410,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 105,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 371,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061644827,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 517,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 295,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 407,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763062986030,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 291,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 20,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063995639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 501,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 379,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138935376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 507,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
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
            "value": 16,
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
          "distinct": false,
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148386515,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 0,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149195492,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 509,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 254,
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
          "distinct": true,
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763150257340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 477,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 261,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763158928079,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 393,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 295,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 106,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763225376851,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 368,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 300,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 258,
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
            "email": "163862677+noirwhal@users.noreply.github.com",
            "name": "noirwhal",
            "username": "noirwhal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383525735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 314,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 298,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384529759,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 298,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 144,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388603894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 258,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489430116,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 299,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 292,
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
          "distinct": false,
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763490268035,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 280,
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
          "distinct": true,
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763490941944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 282,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763492936160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 185,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 292,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 271,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763559161459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 291,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563832132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 295,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 274,
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566090491,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 181,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 303,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 257,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573547365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 13,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633736568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 303,
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
          "distinct": true,
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650672280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 280,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652764509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 182,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 308,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 257,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653593611,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 302,
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
            "value": 267,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763746274145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 288,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 256,
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
          "id": "0ffef1c383ce73611cad6cdd678bce64a1cbceb1",
          "message": "feat(LSP): semantic tokens for doc comment code blocks (#10565)",
          "timestamp": "2025-11-21T20:34:44Z",
          "tree_id": "300a8b7765cf167d9ac9a0c595ce2eda63ec5f5a",
          "url": "https://github.com/noir-lang/noir/commit/0ffef1c383ce73611cad6cdd678bce64a1cbceb1"
        },
        "date": 1763758982725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 292,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 264,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762870844450,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260491,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232031,
            "range": "± 7597",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786730,
            "range": "± 1232",
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879308843,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256493,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227413,
            "range": "± 7634",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782742,
            "range": "± 12002",
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880245851,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258591,
            "range": "± 1050",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230769,
            "range": "± 4298",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784260,
            "range": "± 1575",
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762888460186,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259395,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229557,
            "range": "± 1053",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2785807,
            "range": "± 1608",
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762888789670,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259136,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228105,
            "range": "± 2224",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784998,
            "range": "± 4575",
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762889444089,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258460,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228235,
            "range": "± 2261",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783145,
            "range": "± 2865",
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894084365,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 269309,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239943,
            "range": "± 8094",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795166,
            "range": "± 1487",
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762895724295,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257482,
            "range": "± 1019",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226921,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783808,
            "range": "± 2259",
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762895853354,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255057,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231220,
            "range": "± 3966",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259283,
            "range": "± 826",
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762905775277,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257180,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226915,
            "range": "± 3719",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781993,
            "range": "± 5521",
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762954740684,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259361,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235664,
            "range": "± 3720",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2264195,
            "range": "± 1976",
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762955666646,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256709,
            "range": "± 1789",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227030,
            "range": "± 5173",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781892,
            "range": "± 1418",
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762957456553,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261128,
            "range": "± 932",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230983,
            "range": "± 5092",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781869,
            "range": "± 1601",
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762957758906,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258210,
            "range": "± 1623",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228421,
            "range": "± 2444",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784244,
            "range": "± 11616",
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762957799212,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257050,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227426,
            "range": "± 2852",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2751383,
            "range": "± 25010",
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762957923891,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258714,
            "range": "± 1016",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228179,
            "range": "± 18804",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783500,
            "range": "± 1495",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963328067,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254458,
            "range": "± 1875",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230895,
            "range": "± 3430",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2257809,
            "range": "± 1217",
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762963782776,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257981,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228058,
            "range": "± 5759",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782404,
            "range": "± 2000",
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974219553,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255067,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232057,
            "range": "± 2464",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259231,
            "range": "± 2305",
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762974821066,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257241,
            "range": "± 1381",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227390,
            "range": "± 3729",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783048,
            "range": "± 3742",
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763039619216,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257923,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227660,
            "range": "± 4170",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782726,
            "range": "± 1318",
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061001379,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260082,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231629,
            "range": "± 4448",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782518,
            "range": "± 2327",
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763062577538,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265932,
            "range": "± 1423",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234348,
            "range": "± 1290",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2801787,
            "range": "± 4340",
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763063111214,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259651,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230499,
            "range": "± 8627",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784197,
            "range": "± 3206",
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063361821,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254466,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231348,
            "range": "± 2269",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2258991,
            "range": "± 833",
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138321590,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256773,
            "range": "± 617",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227236,
            "range": "± 4101",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783660,
            "range": "± 3259",
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148153486,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258276,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228874,
            "range": "± 3205",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783226,
            "range": "± 1556",
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763148506204,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261203,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227378,
            "range": "± 4044",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783186,
            "range": "± 1539",
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763149674077,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251186,
            "range": "± 1047",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 224863,
            "range": "± 10357",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794447,
            "range": "± 29640",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763158422974,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251946,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 225292,
            "range": "± 3314",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786633,
            "range": "± 2202",
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763224903019,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261515,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232671,
            "range": "± 2248",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794382,
            "range": "± 2771",
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383094395,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260549,
            "range": "± 867",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232480,
            "range": "± 3085",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794568,
            "range": "± 3494",
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384114730,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259479,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229668,
            "range": "± 4804",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790699,
            "range": "± 15366",
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388190091,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 259524,
            "range": "± 1780",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230840,
            "range": "± 1778",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2789703,
            "range": "± 2794",
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489028005,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263566,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235017,
            "range": "± 6583",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796088,
            "range": "± 69035",
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763489836861,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257011,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223650,
            "range": "± 1584",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2260398,
            "range": "± 9819",
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763490516441,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254754,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220678,
            "range": "± 2253",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2258335,
            "range": "± 2678",
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763492483269,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255870,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222018,
            "range": "± 2806",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259476,
            "range": "± 795",
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763558668226,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252181,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223626,
            "range": "± 4499",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2791394,
            "range": "± 1646",
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563423660,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251872,
            "range": "± 1726",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221549,
            "range": "± 3202",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790320,
            "range": "± 10919",
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763565677381,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255962,
            "range": "± 2318",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 226016,
            "range": "± 4236",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784382,
            "range": "± 2722",
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763572892538,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265262,
            "range": "± 4065",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235558,
            "range": "± 1940",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786767,
            "range": "± 8972",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573082882,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251999,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223043,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783028,
            "range": "± 3458",
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633302788,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252748,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223266,
            "range": "± 2160",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781927,
            "range": "± 6671",
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650261238,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253598,
            "range": "± 1801",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 225399,
            "range": "± 3820",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783671,
            "range": "± 2340",
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652333315,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253282,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230164,
            "range": "± 1736",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2261140,
            "range": "± 1612",
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763652912241,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252196,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223281,
            "range": "± 3922",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780794,
            "range": "± 1783",
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653145451,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250359,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228036,
            "range": "± 3421",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259337,
            "range": "± 1929",
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763745863685,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249719,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220788,
            "range": "± 3107",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781115,
            "range": "± 5320",
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
          "id": "0ffef1c383ce73611cad6cdd678bce64a1cbceb1",
          "message": "feat(LSP): semantic tokens for doc comment code blocks (#10565)",
          "timestamp": "2025-11-21T20:34:44Z",
          "tree_id": "300a8b7765cf167d9ac9a0c595ce2eda63ec5f5a",
          "url": "https://github.com/noir-lang/noir/commit/0ffef1c383ce73611cad6cdd678bce64a1cbceb1"
        },
        "date": 1763758559090,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250426,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220531,
            "range": "± 4060",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779186,
            "range": "± 1035",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Artifact Size": [
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762870812042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.9,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 506.4,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762871417532,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879885882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880806989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889145009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762889380344,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890012337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894758758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762896387523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 753.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 439.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.2,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762896491098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762906338878,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762955310861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762956235018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958075209,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762958352920,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762958373636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762958485568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963935660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762964408003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974789568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762975453227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763040283737,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061573544,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063163859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763063697486,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063945340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 754.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2067.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 359.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51679.3,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.5,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138922707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148800533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149080799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763150257500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159028286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763225466566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 200.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 281,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 360,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51639.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51677.9,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 392,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5617.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4881.7,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383639968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49446.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49491.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384678007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49446.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49491.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388756239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49446.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49491.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489559354,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49446.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49491.8,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763490389508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491074867,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493041110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763559305238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563989463,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566244618,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763573510129,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573720446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633902336,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650806130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652903894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763653514366,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653808765,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763746442717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 757.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2068,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 440.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 216.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 297.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 367.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 49445.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 49489,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 397.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5647.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4920.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 179.4,
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
          "id": "bb32d34e16bdcf69fe258cf693dd142095979dec",
          "message": "fix(ssa): Cast to `u64` when inserting OOB checks in DIE (#10463)",
          "timestamp": "2025-11-11T13:49:27Z",
          "tree_id": "d910b38e46210ff9e7ef1c9add82e05f2b68f508",
          "url": "https://github.com/noir-lang/noir/commit/bb32d34e16bdcf69fe258cf693dd142095979dec"
        },
        "date": 1762870798080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "cf8602db79a1b0f940b90c6d20a0375e30043499",
          "message": "fix(brillig): Skip decrementing ref-count in array/vector copy and other refactors (#10335)",
          "timestamp": "2025-11-11T13:59:09Z",
          "tree_id": "dafae5f196b1e391cd5d168e759492c0885e69f6",
          "url": "https://github.com/noir-lang/noir/commit/cf8602db79a1b0f940b90c6d20a0375e30043499"
        },
        "date": 1762871417629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "6512fb9cfd259daa05c4a3740f5ff00345d01f1b",
          "message": "fix: force_substitute bindings during monomorphization for associated constants (#10467)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T16:18:14Z",
          "tree_id": "40495ae392f86d84e08bf90e1e36d643cce2d6d6",
          "url": "https://github.com/noir-lang/noir/commit/6512fb9cfd259daa05c4a3740f5ff00345d01f1b"
        },
        "date": 1762879885879,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "60bfcf2ceb3d717b2c1293b47a8a391db39235ac",
          "message": "fix: handle ambiguous trait methods in assumed traits (#10468)",
          "timestamp": "2025-11-11T16:35:30Z",
          "tree_id": "bfe167b98596e356eb2788883aa0f869b44fe304",
          "url": "https://github.com/noir-lang/noir/commit/60bfcf2ceb3d717b2c1293b47a8a391db39235ac"
        },
        "date": 1762880806807,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "e794b78f57948a0555fdf43b78bc12b90982dc0e",
          "message": "fix: builtin with body now errors instead of crashing (#10474)",
          "timestamp": "2025-11-11T18:49:49Z",
          "tree_id": "1b5235f9f4d41185c5fabc08d6e0877282a32208",
          "url": "https://github.com/noir-lang/noir/commit/e794b78f57948a0555fdf43b78bc12b90982dc0e"
        },
        "date": 1762889144948,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee",
          "message": "chore: refactor codegen_control_flow (#10320)",
          "timestamp": "2025-11-11T18:57:19Z",
          "tree_id": "caca87b21dd9848f953d551fa6ee38a744dfd566",
          "url": "https://github.com/noir-lang/noir/commit/1580d89b1fb9327f566bce8cf7fbd4b5b4e280ee"
        },
        "date": 1762889380917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "fd27764583beb61e7c485cf07b2498ba42d3c386",
          "message": "fix: disallow keywords in attributes (#10473)",
          "timestamp": "2025-11-11T19:07:56Z",
          "tree_id": "f70a78a2546656a74a10cd50ced76d0794be8438",
          "url": "https://github.com/noir-lang/noir/commit/fd27764583beb61e7c485cf07b2498ba42d3c386"
        },
        "date": 1762890015670,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "distinct": false,
          "id": "fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5",
          "message": "chore: bump external pinned commits (#10477)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-11T20:25:50Z",
          "tree_id": "2ba511cd9652c70ae81471a4ae8f0e8efc22b059",
          "url": "https://github.com/noir-lang/noir/commit/fe0bfa09c57dd2dcb53c03f5aa4eb7152edb25c5"
        },
        "date": 1762894759092,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "b392a8dee97633100d69345bb5bcae6145afba7f",
          "message": "chore(frontend): Various tests in elaborator expressions submodule and minor refactors (#10475)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-11T20:45:20Z",
          "tree_id": "7f24d78d29510ce34998584ddf7fb3851bb4e375",
          "url": "https://github.com/noir-lang/noir/commit/b392a8dee97633100d69345bb5bcae6145afba7f"
        },
        "date": 1762896385892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "975ef74029c784e2df96e05fe3bac27593b3d111",
          "message": "fix: check overflow for Pedersen grumpkin scalars (#10462)",
          "timestamp": "2025-11-11T20:49:05Z",
          "tree_id": "434a3b27a058b25b016e463548ce072402f978b9",
          "url": "https://github.com/noir-lang/noir/commit/975ef74029c784e2df96e05fe3bac27593b3d111"
        },
        "date": 1762896493810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "1b1985e6fa77e221a6723006389c1351bc28b2b1",
          "message": "fix(frontend)!: Preserve int type when quoting tokens  (#10330)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-11T23:39:25Z",
          "tree_id": "42892f76705f5928a5655a64ee40c995f4594830",
          "url": "https://github.com/noir-lang/noir/commit/1b1985e6fa77e221a6723006389c1351bc28b2b1"
        },
        "date": 1762906338769,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "62a44328500fea9f76e8eda2e9a777d0d0c722df",
          "message": "chore: green light Brillig for audit (#10376)",
          "timestamp": "2025-11-12T13:16:58Z",
          "tree_id": "c0e0771c5ebe4a4b34215716d5a17bd59e2476b5",
          "url": "https://github.com/noir-lang/noir/commit/62a44328500fea9f76e8eda2e9a777d0d0c722df"
        },
        "date": 1762955312086,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "794b685f77ec3b4c1c885c4131ee7792e949511d",
          "message": "fix(frontend): No negative overflow when quoting signed integer (#10331)\n\nCo-authored-by: Jake Fecher <jfecher11@gmail.com>",
          "timestamp": "2025-11-12T13:32:53Z",
          "tree_id": "f602f3d6c7754cfd69aabe92d6344a69a2f04e3b",
          "url": "https://github.com/noir-lang/noir/commit/794b685f77ec3b4c1c885c4131ee7792e949511d"
        },
        "date": 1762956236073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "c65d5d994bec92b6ed69adca87020bd04234e07d",
          "message": "fix(print): Print enums (#10472)",
          "timestamp": "2025-11-12T14:02:02Z",
          "tree_id": "0a7d2ee6d0ae3a2e145827ceb556b25cf798c851",
          "url": "https://github.com/noir-lang/noir/commit/c65d5d994bec92b6ed69adca87020bd04234e07d"
        },
        "date": 1762958078057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "550b1db5622d55311aab9d886b3c8b59055bc020",
          "message": "fix: remove saturation from loop bound increments (#10479)",
          "timestamp": "2025-11-12T14:05:02Z",
          "tree_id": "6d6de95c0d441efb6909f541db4212ecdd6f2670",
          "url": "https://github.com/noir-lang/noir/commit/550b1db5622d55311aab9d886b3c8b59055bc020"
        },
        "date": 1762958352748,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "6f0c090d3e8412ce0445200d0e720aae5ee6433c",
          "message": "feat(ssa): Limit the number of steps executed by the SSA interpreter during constant folding (#10481)",
          "timestamp": "2025-11-12T14:06:45Z",
          "tree_id": "32203c7b2501c35a49325ac87a0ae56681059653",
          "url": "https://github.com/noir-lang/noir/commit/6f0c090d3e8412ce0445200d0e720aae5ee6433c"
        },
        "date": 1762958364266,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "0c6acb71820d0afdc5514a42742a4c3c6c3aad74",
          "message": "chore: remove a bunch of dummy definitions (#10482)",
          "timestamp": "2025-11-12T14:08:39Z",
          "tree_id": "04d72a5f905f7a3b01c0eefe987ec2d1672820c0",
          "url": "https://github.com/noir-lang/noir/commit/0c6acb71820d0afdc5514a42742a4c3c6c3aad74"
        },
        "date": 1762958489087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
            "email": "52610192+Aristotelis2002@users.noreply.github.com",
            "name": "Aristotelis",
            "username": "Aristotelis2002"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "24de605dd526477ef3e8dc38a6c038f695aeed28",
          "message": "chore: monomorphizer public fields (#9979)",
          "timestamp": "2025-11-12T15:38:48Z",
          "tree_id": "8b6495b1416c65e949951013c181bb7d0a1863d9",
          "url": "https://github.com/noir-lang/noir/commit/24de605dd526477ef3e8dc38a6c038f695aeed28"
        },
        "date": 1762963938256,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "e7253b72e17de37995767f95fc69e43fc44b7f22",
          "message": "chore(frontend): Tuple pattern tests and remove confusing arity error  (#10480)",
          "timestamp": "2025-11-12T15:46:17Z",
          "tree_id": "21b306d8e92343e52bb3795598541db54b8339f9",
          "url": "https://github.com/noir-lang/noir/commit/e7253b72e17de37995767f95fc69e43fc44b7f22"
        },
        "date": 1762964425667,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4",
          "message": "chore: better error recovery for multiple mut in pattern (#10490)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-12T18:41:43Z",
          "tree_id": "7d7a2bde6000230adf26894536e55fe14c2422b6",
          "url": "https://github.com/noir-lang/noir/commit/6289e3cce0a3ffb41c1f41c55b12c28d855ebcc4"
        },
        "date": 1762974788888,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "b34828f17e2b52b3137fca8f7881abaf91b74ad4",
          "message": "chore: remove `local_annotations` from flattening (#10483)",
          "timestamp": "2025-11-12T18:50:05Z",
          "tree_id": "ec581cadf4263c9cd39f9400ef07387694a9db97",
          "url": "https://github.com/noir-lang/noir/commit/b34828f17e2b52b3137fca8f7881abaf91b74ad4"
        },
        "date": 1762975450962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "8da0cc8213c6d9e3c204350f0fc41885b515f07c",
          "message": "chore: improve register moves in brillig return code-gen (#10305)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-11-13T12:44:58Z",
          "tree_id": "68261a34179f7bbd1641a0a7f04b52ccb5f64297",
          "url": "https://github.com/noir-lang/noir/commit/8da0cc8213c6d9e3c204350f0fc41885b515f07c"
        },
        "date": 1763040282484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "ff431be8bf2f50ab02b63d01ac0b8c25af428c08",
          "message": "chore(frontend): Correct type for struct field on type mismatch and extra negative case unit tests  (#10493)",
          "timestamp": "2025-11-13T18:47:32Z",
          "tree_id": "8645daf91e76bb2329149259b7b5bd8a377003ec",
          "url": "https://github.com/noir-lang/noir/commit/ff431be8bf2f50ab02b63d01ac0b8c25af428c08"
        },
        "date": 1763061571457,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "distinct": false,
          "id": "9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36",
          "message": "feat(fuzz): Add support for more functions in comptime_vs_brillig_direct (#10500)",
          "timestamp": "2025-11-13T19:11:42Z",
          "tree_id": "89f700803f366d71de5c1220af96642761a28d64",
          "url": "https://github.com/noir-lang/noir/commit/9fd2cf1aed74715747fab3d4cdc6abdaf6e37b36"
        },
        "date": 1763063157395,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "ae5b83d8978b2364c90e437779a3aafa96678fb6",
          "message": "chore(elaborator): Check that assert message fragments are ABI compatible (#10491)",
          "timestamp": "2025-11-13T19:23:17Z",
          "tree_id": "d78521993b2022460ff8e83383e88208e9467d0a",
          "url": "https://github.com/noir-lang/noir/commit/ae5b83d8978b2364c90e437779a3aafa96678fb6"
        },
        "date": 1763063695848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "5a85979f1b8a05345cf488e7f3f8c400636afa50",
          "message": "fix(fuzzer): Set `in_dynamic` in `gen_match` (#10470)",
          "timestamp": "2025-11-13T19:25:52Z",
          "tree_id": "8fad74b0c4bacff99ba37238d118cd1d569543cf",
          "url": "https://github.com/noir-lang/noir/commit/5a85979f1b8a05345cf488e7f3f8c400636afa50"
        },
        "date": 1763063943998,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18023,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80613,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7209,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "distinct": false,
          "id": "4a87d867d4adc4cbf5eb80e37621de539698d62b",
          "message": "chore: bump external pinned commits (#10507)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-14T16:17:28Z",
          "tree_id": "10239dbf9db0947d5e991bbc60035cc82be20ec5",
          "url": "https://github.com/noir-lang/noir/commit/4a87d867d4adc4cbf5eb80e37621de539698d62b"
        },
        "date": 1763138923678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "2a850d7f7804a9d96a0008f22277ebc8cafa4722",
          "message": "chore: remove Unspecified type, and better wildcard disallowed errors (#10495)",
          "timestamp": "2025-11-14T18:58:49Z",
          "tree_id": "b2251d98ebfcdc0f79e701cff872155d90b4a165",
          "url": "https://github.com/noir-lang/noir/commit/2a850d7f7804a9d96a0008f22277ebc8cafa4722"
        },
        "date": 1763148808200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "9a55cdc745299ad716ee5227541182ddc863e31b",
          "message": "fix(docs): ACIR array flattening (#10509)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-11-14T19:06:13Z",
          "tree_id": "7c724b95c093b8c2003cb280e30f300b60a36042",
          "url": "https://github.com/noir-lang/noir/commit/9a55cdc745299ad716ee5227541182ddc863e31b"
        },
        "date": 1763149081738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "9238b702e944e6a3ac0dffcd92c1ed9027b63e75",
          "message": "chore: green light acir_field for audit (#10360)",
          "timestamp": "2025-11-14T19:23:32Z",
          "tree_id": "4d935215e03a3c9e6c4bef2afbee65489a094baf",
          "url": "https://github.com/noir-lang/noir/commit/9238b702e944e6a3ac0dffcd92c1ed9027b63e75"
        },
        "date": 1763150252674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fce7cac1b6b116b6af98875580b5122eb9fe051",
          "message": "chore(readme): Update Noir logo (#9187)",
          "timestamp": "2025-11-14T21:51:52Z",
          "tree_id": "a97d4fdb16dc7200514818f8955d05db565b53e6",
          "url": "https://github.com/noir-lang/noir/commit/1fce7cac1b6b116b6af98875580b5122eb9fe051"
        },
        "date": 1763159017685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af",
          "message": "fix: missing returned inputs in aes128encrypt black box (#10512)",
          "timestamp": "2025-11-15T16:20:52Z",
          "tree_id": "86655f6d01959110220815a33a7cdc6767524bc1",
          "url": "https://github.com/noir-lang/noir/commit/d2c71c467f2fe0d045d1c1c4ad7d9d0ed961c1af"
        },
        "date": 1763225465775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1100,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 876,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2253,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2132,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919761,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921135,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2607,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306830,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262449,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "distinct": false,
          "id": "83129a48fb0670ea9806568aadf0507dfa0eedb5",
          "message": "chore: bump external pinned commits (#10513)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-11-17T12:14:51Z",
          "tree_id": "e78afeb30005ea004b5983f5b4a5198b42dce561",
          "url": "https://github.com/noir-lang/noir/commit/83129a48fb0670ea9806568aadf0507dfa0eedb5"
        },
        "date": 1763383638203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "50d5c0b4c9ca70f3ca49e13786048d1bb15b155e",
          "message": "fix: don't remove signed min int division overflow in DIE (#10506)",
          "timestamp": "2025-11-17T12:32:38Z",
          "tree_id": "0c830a637f9ffb327e346a72fd08ec4cd60e0ee2",
          "url": "https://github.com/noir-lang/noir/commit/50d5c0b4c9ca70f3ca49e13786048d1bb15b155e"
        },
        "date": 1763384669423,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "881acee6c8df42adb85ff0b9bc9ad144b43bdf6b",
          "message": "chore: remove npm token from CI (#10515)",
          "timestamp": "2025-11-17T14:00:37Z",
          "tree_id": "54ef05d167f03887c2fdd6e47fe5f2d7a4c5476c",
          "url": "https://github.com/noir-lang/noir/commit/881acee6c8df42adb85ff0b9bc9ad144b43bdf6b"
        },
        "date": 1763388766704,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "251d991e582df6cb22aff5d178a04a65dd1f4a6f",
          "message": "fix: error if `Quoted::as_module` finds private module (#10511)",
          "timestamp": "2025-11-18T17:40:25Z",
          "tree_id": "c2de5b745de5708b4dc4f34f80d67fd6d4827deb",
          "url": "https://github.com/noir-lang/noir/commit/251d991e582df6cb22aff5d178a04a65dd1f4a6f"
        },
        "date": 1763489561794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "789455f6bf5a35d21a6398ad85026b05b5779862",
          "message": "fix: evaluate repeated array expr once (#10514)",
          "timestamp": "2025-11-18T17:55:11Z",
          "tree_id": "9ac51d613e42cddb13933a6629a463397f167490",
          "url": "https://github.com/noir-lang/noir/commit/789455f6bf5a35d21a6398ad85026b05b5779862"
        },
        "date": 1763490388486,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "a8736206336afd59b45da4569a55407628c3570f",
          "message": "fix(brillig): Handle the return of multiple vectors from foreign calls (#10505)",
          "timestamp": "2025-11-18T18:05:54Z",
          "tree_id": "7afbc404bfc537499cb229133e352bf0a1cfc9c3",
          "url": "https://github.com/noir-lang/noir/commit/a8736206336afd59b45da4569a55407628c3570f"
        },
        "date": 1763491074870,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "155db34422be717da8636db1d0ecc2ece55ac679",
          "message": "chore: ignore some sha256 failures because of oracles (#10528)",
          "timestamp": "2025-11-18T18:38:52Z",
          "tree_id": "f83c6e7e7ff8834a9110199450c66fd5fb180cca",
          "url": "https://github.com/noir-lang/noir/commit/155db34422be717da8636db1d0ecc2ece55ac679"
        },
        "date": 1763493039263,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "7c50de9f5f0858db8346acd920f6bea1b1abecb4",
          "message": "chore: push noir stdlib docs to gh pages (#10532)",
          "timestamp": "2025-11-19T12:57:45Z",
          "tree_id": "69c9d3d2962d68ebf1dff7d0fa0ccd6f72dd3ff2",
          "url": "https://github.com/noir-lang/noir/commit/7c50de9f5f0858db8346acd920f6bea1b1abecb4"
        },
        "date": 1763559297247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "b131feb017209eda0189ce40986cb96641323fcf",
          "message": "feat: doc comments inter-links (#10527)",
          "timestamp": "2025-11-19T14:21:16Z",
          "tree_id": "bda762d7655ab8cc105f8aec2e86d619b14c6571",
          "url": "https://github.com/noir-lang/noir/commit/b131feb017209eda0189ce40986cb96641323fcf"
        },
        "date": 1763563989309,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "d81a38de4d5963d71f637aed1a6f425119b7ea73",
          "message": "fix(brillig): Prevent wrap-around of the free-memory-pointer (#10526)",
          "timestamp": "2025-11-19T14:58:46Z",
          "tree_id": "d0870a63cabcb27cc80ce0fbd2ecaa3a28b98472",
          "url": "https://github.com/noir-lang/noir/commit/d81a38de4d5963d71f637aed1a6f425119b7ea73"
        },
        "date": 1763566242479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "581c878d9910a361552cc5edc08b4cd440afc656",
          "message": "feat(doc): show deprecated functions (#10536)",
          "timestamp": "2025-11-19T16:58:01Z",
          "tree_id": "925698ac2aceaf758f3069ae49f31f0e43bf7246",
          "url": "https://github.com/noir-lang/noir/commit/581c878d9910a361552cc5edc08b4cd440afc656"
        },
        "date": 1763573524992,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
            "email": "72797635+Savio-Sou@users.noreply.github.com",
            "name": "Savio",
            "username": "Savio-Sou"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9dbad18bf0734de693ef6957fb8eb72e26ddaadc",
          "message": "chore: Remove references to the deprecated grants program (#9253)",
          "timestamp": "2025-11-19T17:01:32Z",
          "tree_id": "2ac5b784bb188091c5912b2d0284c8eb552a95b6",
          "url": "https://github.com/noir-lang/noir/commit/9dbad18bf0734de693ef6957fb8eb72e26ddaadc"
        },
        "date": 1763573688519,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "5e5ec245e8c4966e7a0cc962c7048fa33543212c",
          "message": "fix: only entry-point main is special (#10545)",
          "timestamp": "2025-11-20T09:45:22Z",
          "tree_id": "af16117d6b17fe61ce30dd6077899e920c753de9",
          "url": "https://github.com/noir-lang/noir/commit/5e5ec245e8c4966e7a0cc962c7048fa33543212c"
        },
        "date": 1763633903601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "b37e91f264b92dc64af9096b91c79d1c504ca022",
          "message": "chore: error on match keyword when enums are not enabled (#10549)",
          "timestamp": "2025-11-20T14:20:43Z",
          "tree_id": "509ba9a37169f453e28951278fc8663ad2a478d9",
          "url": "https://github.com/noir-lang/noir/commit/b37e91f264b92dc64af9096b91c79d1c504ca022"
        },
        "date": 1763650802680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "271d332049d25e534dbce77c7f86195407660ae2",
          "message": "feat(doc): colorize code blocks (#10550)",
          "timestamp": "2025-11-20T15:02:15Z",
          "tree_id": "d6098bf0b90d7d5ba76b68caa98740ff9df2dd00",
          "url": "https://github.com/noir-lang/noir/commit/271d332049d25e534dbce77c7f86195407660ae2"
        },
        "date": 1763652903970,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "55e961e9e1f9e4e5211ac0c9bd1561b13b736c76",
          "message": "chore: we don't warn anymore when a single trait method is not in scope (#10551)",
          "timestamp": "2025-11-20T15:11:36Z",
          "tree_id": "d2962f0a54aa9d4b7b55b2694da9cedca5d167a4",
          "url": "https://github.com/noir-lang/noir/commit/55e961e9e1f9e4e5211ac0c9bd1561b13b736c76"
        },
        "date": 1763653513225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "ed5853fd1204c91bff3eca421c7f3b11159a3dc7",
          "message": "chore: add permission to add label to PR (#10552)",
          "timestamp": "2025-11-20T15:36:36Z",
          "tree_id": "d8884ab4119d2a520d86b8f9597a771bce36034a",
          "url": "https://github.com/noir-lang/noir/commit/ed5853fd1204c91bff3eca421c7f3b11159a3dc7"
        },
        "date": 1763653807956,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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
          "id": "173400205f7fec10951d2d52d641dc646f6b6b5d",
          "message": "chore: clippy fixes (#10560)",
          "timestamp": "2025-11-21T17:21:56Z",
          "tree_id": "06aec7c5c3f895d2f61ecc432c5ad3fc745a281b",
          "url": "https://github.com/noir-lang/noir/commit/173400205f7fec10951d2d52d641dc646f6b6b5d"
        },
        "date": 1763746443302,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18033,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80614,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 7208,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1049,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2430,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2127,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1821341,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1822715,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2589,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 306635,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262389,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1485,
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