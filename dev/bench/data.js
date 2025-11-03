window.BENCHMARK_DATA = {
  "lastUpdate": 1762182445713,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
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
        "date": 1761252957379,
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
            "value": 339.39,
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
          "distinct": false,
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253983674,
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
            "value": 339.39,
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
            "value": 185.47,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309874684,
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
            "value": 339.39,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761316038472,
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
            "value": 339.39,
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
            "value": 185.43,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329389501,
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
          "distinct": false,
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333627764,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338756814,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761556127862,
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
            "value": 185.51,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579666251,
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
            "value": 185.45,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761583190539,
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
      },
      {
        "commit": {
          "author": {
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596660189,
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
            "value": 185.5,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598408445,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.51,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761600125206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.39,
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
            "value": 185.47,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669964470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
          "distinct": false,
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687321845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.44,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692274904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.4,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765611023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.41,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769802826,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.45,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761838120596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.43,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845267252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.19,
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
            "value": 185.5,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854812380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.73,
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
            "value": 341.28,
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
            "value": 336.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858262743,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.35,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.73,
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
            "value": 341.27,
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
            "value": 336.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.4,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907952740,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.73,
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
            "value": 341.27,
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
            "value": 336.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926774454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.36,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.73,
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
            "value": 341.27,
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
            "value": 336.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929694872,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.35,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.73,
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
            "value": 341.27,
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
            "value": 336.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761945013241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.35,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.74,
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
            "value": 341.28,
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
            "value": 336.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164468871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.61,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.35,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.74,
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
            "value": 341.28,
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
            "value": 336.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762166081719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 493.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 244.64,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 337.38,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.4,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 339.74,
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
            "value": 341.28,
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
            "value": 336.56,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
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
          "distinct": true,
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762175225348,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 498.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 339.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.19,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 340.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 340.95,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11250,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11260,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.49,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1080,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 3030,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 92.16,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 185.44,
            "unit": "MB"
          }
        ]
      }
    ],
    "Compilation Time": [
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
        "date": 1761252683398,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.792,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.75,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.392,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.66,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.853,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.734,
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253758752,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.778,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.846,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.434,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.78,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.77,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309847500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.892,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.444,
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
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.839,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.596,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315813008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.942,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.848,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.366,
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
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.837,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.527,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329171357,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.776,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.94,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.777,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.607,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333425701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.32,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.08,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.907,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338548875,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.388,
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
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.63,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.837,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.513,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555927323,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.752,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.188,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 194,
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
            "value": 18.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579531107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.86,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.816,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.517,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582972797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.812,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.378,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
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
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.552,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.008,
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596498963,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.724,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.228,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.404,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.448,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.526,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.806,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.929,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598149176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.764,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.582,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.785,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599902017,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.772,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.25,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.78,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.523,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669741549,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.832,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.904,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.432,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.46,
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
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.675,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687083446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.864,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.256,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.42,
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
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.535,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692083367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.668,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.326,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.1,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.826,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.588,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765236728,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.936,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.28,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.797,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769603678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.868,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.928,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
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
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.783,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837836163,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.692,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.634,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.396,
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
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.554,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.805,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845029678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.184,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
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
            "value": 213,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.797,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.525,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854575138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.858,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.62,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.26,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.809,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.683,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858022557,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.982,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.614,
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
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.515,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907529241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.4,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.372,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.48,
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
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.89,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926463989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.704,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.324,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 222,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77,
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
            "value": 1.678,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929444598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.708,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.804,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.442,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.538,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.796,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.575,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944825203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.682,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.87,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.86,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 79.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.825,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164177594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.828,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.866,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.352,
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
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.512,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.684,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.807,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.549,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165823093,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.702,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.858,
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
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.48,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.44,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.544,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762174477409,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.986,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.06,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.59,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.458,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 419,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.518,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.805,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.71,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
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
      },
      {
        "commit": {
          "author": {
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253758669,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309819728,
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
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.299,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315812450,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.296,
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
            "value": 0.047,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329168546,
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
            "value": 11.1,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333430078,
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
            "value": 0.298,
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
            "value": 0.076,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338592450,
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
            "value": 11.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555924501,
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
            "value": 11.5,
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
            "value": 0.055,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579592501,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582972646,
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596477600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.143,
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
            "value": 10.9,
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
            "value": 0.048,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598186761,
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
            "value": 11.1,
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
            "value": 0.05,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599903461,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.142,
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
            "value": 12.1,
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
            "value": 0.297,
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
            "value": 0.008,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669786371,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687084237,
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
            "value": 0.298,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692081288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.15,
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
            "value": 0.299,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765236553,
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
            "value": 0.004,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769592800,
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
            "value": 0.047,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837840445,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845033140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.142,
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
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.5,
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
            "value": 0.233,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854787136,
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
            "value": 11,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858025697,
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
            "value": 0.298,
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
            "value": 0.045,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907531931,
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
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 11.1,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926462339,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929449667,
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
            "value": 11.1,
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
            "value": 0.299,
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
            "value": 0.048,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944833969,
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
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 11.5,
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
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.086,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164395744,
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
            "value": 11,
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
            "value": 0.048,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165818057,
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
            "value": 11.5,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762174477433,
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
            "value": 0.012,
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
            "value": 0.321,
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
            "value": 0.057,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Memory": [
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
        "date": 1761252959141,
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253985719,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309883261,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761316039206,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329387469,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333627528,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338755357,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761556127052,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579617710,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761583188847,
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596671207,
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
            "value": 451.41,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598408716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761600111453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669956346,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687319731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692277144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765612266,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769804742,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761838129599,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845263852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.23,
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
            "value": 451.41,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854829651,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.09,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858264582,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.09,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907958086,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.09,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926781517,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.09,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929694966,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.47,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.45,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.09,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761945015159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.48,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.85,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.39,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.46,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.1,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164502628,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.48,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.85,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.39,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.46,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.1,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762166082087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 253.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 287.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.51,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.85,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.64,
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
            "value": 337.84,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 451.39,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 466.46,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 336.1,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762175225125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 260.4,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.14,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 250.82,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 339.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 337.85,
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
            "value": 339.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 523.33,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 472.93,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.31,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.68,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.94,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
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
      },
      {
        "commit": {
          "author": {
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253680136,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309720523,
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
            "value": 259,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 443,
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
          "distinct": true,
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315788896,
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
            "value": 292,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 395,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329120575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 291,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 240,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 126,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333260289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 287,
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
            "value": 164,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338461771,
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
            "value": 301,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 236,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 140,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 330,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555802526,
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
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 329,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579314509,
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
            "value": 290,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 231,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 144,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 368,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582929488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
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
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 378,
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
          "distinct": false,
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596493551,
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
            "value": 286,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 231,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598010373,
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
            "value": 270,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 331,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599783796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 269,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 320,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669702046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687012262,
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
            "value": 266,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 358,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761691994312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 257,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 324,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765288349,
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
            "value": 263,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
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
            "value": 329,
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
          "distinct": true,
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769435462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 325,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837925611,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
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
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 154,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 332,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845045275,
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
            "value": 262,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 225,
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
            "value": 354,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854658239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 272,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 340,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761857916414,
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
            "value": 261,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 231,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 325,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907514740,
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
            "value": 251,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 232,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 421,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926822392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 249,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 228,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 338,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929357431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 293,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 333,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944684592,
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
            "value": 324,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 329,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164215494,
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
            "value": 254,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 227,
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
            "value": 359,
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
          "distinct": true,
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165735274,
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
            "value": 315,
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
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 340,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762174497347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 433,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 241,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 139,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 146,
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
      },
      {
        "commit": {
          "author": {
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253245948,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263883,
            "range": "± 920",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231800,
            "range": "± 6233",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794354,
            "range": "± 13663",
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761308919878,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263894,
            "range": "± 752",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233255,
            "range": "± 3035",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795429,
            "range": "± 1492",
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315287634,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264331,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233450,
            "range": "± 3200",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795173,
            "range": "± 2012",
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761328643380,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266740,
            "range": "± 671",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237080,
            "range": "± 1496",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3010376,
            "range": "± 32452",
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761332843356,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263115,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232833,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796347,
            "range": "± 10387",
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338018399,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268453,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 240416,
            "range": "± 1866",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795260,
            "range": "± 20055",
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555297850,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252479,
            "range": "± 2266",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221254,
            "range": "± 2790",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259833,
            "range": "± 14138",
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761578822863,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265829,
            "range": "± 995",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235049,
            "range": "± 1526",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797092,
            "range": "± 1961",
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582435048,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263529,
            "range": "± 508",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233286,
            "range": "± 2788",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797292,
            "range": "± 14705",
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761595838391,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252482,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221283,
            "range": "± 3708",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2258017,
            "range": "± 2684",
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761597183471,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266940,
            "range": "± 1861",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237726,
            "range": "± 3902",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798884,
            "range": "± 2435",
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599365444,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262389,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231761,
            "range": "± 5867",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795176,
            "range": "± 3004",
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761668892012,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251933,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220640,
            "range": "± 1924",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2259734,
            "range": "± 10347",
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761686452728,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265045,
            "range": "± 3380",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234265,
            "range": "± 6303",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2800745,
            "range": "± 13944",
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761691540806,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265266,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234281,
            "range": "± 2934",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799529,
            "range": "± 5663",
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761764731955,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265225,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236465,
            "range": "± 2008",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2793999,
            "range": "± 3017",
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769001796,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262406,
            "range": "± 1141",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232820,
            "range": "± 2204",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795128,
            "range": "± 7751",
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837068763,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263360,
            "range": "± 2266",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232649,
            "range": "± 5970",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2797384,
            "range": "± 16695",
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761844494745,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267542,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236461,
            "range": "± 7796",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795766,
            "range": "± 1660",
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761853339758,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262303,
            "range": "± 1728",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232013,
            "range": "± 4294",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794579,
            "range": "± 6877",
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761857485879,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266590,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236243,
            "range": "± 1777",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796386,
            "range": "± 926",
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761906980877,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263144,
            "range": "± 1124",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233844,
            "range": "± 10977",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2799210,
            "range": "± 5794",
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761925880890,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253059,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221576,
            "range": "± 2796",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2260662,
            "range": "± 1161",
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761928910527,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261180,
            "range": "± 690",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231454,
            "range": "± 5566",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794189,
            "range": "± 8658",
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944213960,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263824,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233415,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794988,
            "range": "± 1655",
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762162889831,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263030,
            "range": "± 758",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232956,
            "range": "± 1932",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2795695,
            "range": "± 14708",
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165085490,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262620,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232088,
            "range": "± 6770",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794305,
            "range": "± 22896",
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762173946280,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265776,
            "range": "± 1628",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234948,
            "range": "± 3594",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796921,
            "range": "± 3144",
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
          "id": "ce4b6bc8d9cf98c4081382b9076626be9b4a3204",
          "message": "fix: check value of `ArraySet` during `array_set_optimization` (#10325)",
          "timestamp": "2025-11-03T14:45:46Z",
          "tree_id": "69ca99743c2d5000928c3736dfbd745739bfea96",
          "url": "https://github.com/noir-lang/noir/commit/ce4b6bc8d9cf98c4081382b9076626be9b4a3204"
        },
        "date": 1762182420157,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265607,
            "range": "± 671",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233701,
            "range": "± 4197",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794809,
            "range": "± 1848",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Artifact Size": [
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
        "date": 1761252691577,
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253760678,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309854813,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315811577,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329170115,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333421749,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338548357,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555924017,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579577662,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582975062,
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596496630,
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
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.4,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598166153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599903113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669753072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687086611,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692079175,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765239297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769594017,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837872299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845031907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854563331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858033406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907531137,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926483571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929446006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944824250,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164394687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 546.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27638.6,
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
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165829405,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 714.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 1863.8,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 548.5,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 179.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 177.6,
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
            "value": 27640.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27687.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 411.4,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4906.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4554.5,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762174478583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 770.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2048.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 627.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 199.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 280.3,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 363.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 51640.7,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 51678.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 395.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5168.2,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4878.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 178.9,
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
      },
      {
        "commit": {
          "author": {
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
          "id": "2f73a86cb76fb656e920e73463e8956d6aad82fe",
          "message": "chore(Brillig): no need to handle ArrayLen intrinsic (#10280)",
          "timestamp": "2025-10-23T20:34:54Z",
          "tree_id": "7727d0d62967dc2055d8295b849bc993070bde40",
          "url": "https://github.com/noir-lang/noir/commit/2f73a86cb76fb656e920e73463e8956d6aad82fe"
        },
        "date": 1761253758020,
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
          "id": "9b37c69344d24087ee48068dc6d5e029a5e2bf73",
          "message": "chore(ACIR): document AcirValue (#10276)",
          "timestamp": "2025-10-24T12:01:58Z",
          "tree_id": "ae2fefa6d6dd375f9610b3153bf4db07f27ff09f",
          "url": "https://github.com/noir-lang/noir/commit/9b37c69344d24087ee48068dc6d5e029a5e2bf73"
        },
        "date": 1761309842604,
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
          "id": "4c0cf516986173087f996a98c0cc618b1461e8e9",
          "message": "chore(ACIR): handle TODO in `more_than_eq_var` (#10274)",
          "timestamp": "2025-10-24T13:49:48Z",
          "tree_id": "bdd512008526a341defcca3feaca7a0f16a85c88",
          "url": "https://github.com/noir-lang/noir/commit/4c0cf516986173087f996a98c0cc618b1461e8e9"
        },
        "date": 1761315811746,
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
          "id": "42bab876cd4e1edec45fd3a09217709e35eaa56c",
          "message": "feat(github): Add Security Policy (#10262)",
          "timestamp": "2025-10-24T17:27:47Z",
          "tree_id": "4177c4f6c11e8c47638872eaa322246a5c416ed4",
          "url": "https://github.com/noir-lang/noir/commit/42bab876cd4e1edec45fd3a09217709e35eaa56c"
        },
        "date": 1761329171284,
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
          "id": "e34f4ee59422830b75f23c1bbe8aa558f4c3fe13",
          "message": "chore(ACIR): use u32::MAX for PLACEHOLDER_BRILLIG_INDEX (#10287)",
          "timestamp": "2025-10-24T18:34:00Z",
          "tree_id": "f42ed029cc1794c8a0f72fde6e104898907e98cd",
          "url": "https://github.com/noir-lang/noir/commit/e34f4ee59422830b75f23c1bbe8aa558f4c3fe13"
        },
        "date": 1761333431881,
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
          "id": "41b22babe5776b8cb3fa0f7f67eac562c24557fd",
          "message": "chore: Move variable elaboration to its own file (#10285)",
          "timestamp": "2025-10-24T20:09:40Z",
          "tree_id": "1db94d386fb851a6a5a7886535b2ea7446f61383",
          "url": "https://github.com/noir-lang/noir/commit/41b22babe5776b8cb3fa0f7f67eac562c24557fd"
        },
        "date": 1761338548035,
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
          "id": "52b341d71984273eb5cabe9edbd290b1f34a6a6e",
          "message": "chore(audit): Refactors in `BrilligGlobals`, `ConstantAllocation` and `VariableLiveness` (#10265)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-27T08:24:57Z",
          "tree_id": "6b481b97f3498b5b80516c3995fe3027496fb039",
          "url": "https://github.com/noir-lang/noir/commit/52b341d71984273eb5cabe9edbd290b1f34a6a6e"
        },
        "date": 1761555924585,
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
          "id": "3f27b25875ecd349691dcce90f6007774a9ac067",
          "message": "chore: avoid unrolling loop headers twice in unrolling passes (#10284)",
          "timestamp": "2025-10-27T15:01:06Z",
          "tree_id": "2740a06b91f6722a1f212f97ba679fdd998d5713",
          "url": "https://github.com/noir-lang/noir/commit/3f27b25875ecd349691dcce90f6007774a9ac067"
        },
        "date": 1761579570202,
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
          "id": "b43153bd34f52ea835f6b7715bcf2c1f931a0fc8",
          "message": "chore(frontend): Elaborator lazy globals and documentation (#10260)",
          "timestamp": "2025-10-27T15:58:07Z",
          "tree_id": "79ab81fb1132b126e874107c418502ed94d5549b",
          "url": "https://github.com/noir-lang/noir/commit/b43153bd34f52ea835f6b7715bcf2c1f931a0fc8"
        },
        "date": 1761582974217,
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
          "id": "8220860e565e1fa8afad96dd30a1e1a32683a562",
          "message": "feat(SSA): simplify array_get from param (#10300)",
          "timestamp": "2025-10-27T19:44:22Z",
          "tree_id": "3904bca1b93533bef7319a3866943add9918f5b2",
          "url": "https://github.com/noir-lang/noir/commit/8220860e565e1fa8afad96dd30a1e1a32683a562"
        },
        "date": 1761596505260,
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
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245168,
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
          "id": "7fb2f1a26b9c04860e24f018eb528a9a84e0f055",
          "message": "feat(ACIR): reuse element_type_sizes blocks with the same structure (#10231)",
          "timestamp": "2025-10-27T20:03:26Z",
          "tree_id": "3d56bce3614b26e1560a27e7dfad1a6efd0ec650",
          "url": "https://github.com/noir-lang/noir/commit/7fb2f1a26b9c04860e24f018eb528a9a84e0f055"
        },
        "date": 1761598173878,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "dd8b7e81c3249cbaef557b734220aac44e361090",
          "message": "chore(frontend): Elaborator impls documentation and additional tests  (#10302)",
          "timestamp": "2025-10-27T20:47:12Z",
          "tree_id": "93a3efac70941d22d7089c158e0262698bef5ae3",
          "url": "https://github.com/noir-lang/noir/commit/dd8b7e81c3249cbaef557b734220aac44e361090"
        },
        "date": 1761599899762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "6826de8470367a7a732ca5731eee3162717e0e37",
          "message": "chore: Document each elaborator trait function (#10303)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T16:06:15Z",
          "tree_id": "726145edee057362275b914452f7afffa3350517",
          "url": "https://github.com/noir-lang/noir/commit/6826de8470367a7a732ca5731eee3162717e0e37"
        },
        "date": 1761669794587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "9cc81909aa6d1197198935de9f423ee8313ebcde",
          "message": "chore(audit): Refactors and tests for reg-to-reg movements (#10293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-28T20:51:55Z",
          "tree_id": "816763bf96a41f48d544c30cd4a8079805943065",
          "url": "https://github.com/noir-lang/noir/commit/9cc81909aa6d1197198935de9f423ee8313ebcde"
        },
        "date": 1761687094661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "fcafe4e506fe1c2a5d4fc61fb923429db185e312",
          "message": "chore(audit): Fix vector items offset and other refactors (#10294)",
          "timestamp": "2025-10-28T22:21:48Z",
          "tree_id": "52f8cb86eb3e40f03e609974969f1269604a7319",
          "url": "https://github.com/noir-lang/noir/commit/fcafe4e506fe1c2a5d4fc61fb923429db185e312"
        },
        "date": 1761692081653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "900d3c21ec03d4b73b1c126045dee1fc69c2901e",
          "message": "fix: \"No size for slice\" when using black_box (#10312)",
          "timestamp": "2025-10-29T18:39:52Z",
          "tree_id": "823df1b8189b10bb5a0aa822754b047071a604c2",
          "url": "https://github.com/noir-lang/noir/commit/900d3c21ec03d4b73b1c126045dee1fc69c2901e"
        },
        "date": 1761765236942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "0de2ac86d6cd969e2774f1017b4f104c64bfc039",
          "message": "chore(frontend): Elaborator comptime module docs (#10318)",
          "timestamp": "2025-10-29T19:50:26Z",
          "tree_id": "d59821de5a4e91155eb399e07a14b5de57d1d9af",
          "url": "https://github.com/noir-lang/noir/commit/0de2ac86d6cd969e2774f1017b4f104c64bfc039"
        },
        "date": 1761769595541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "9808525b4f420397ea30ba91293ffb6539668e3f",
          "message": "chore: Fix typo in defunctionalize docs (#10321)",
          "timestamp": "2025-10-30T14:39:52Z",
          "tree_id": "75b50b8aa7da14c3d61a30d8391319757a58ada0",
          "url": "https://github.com/noir-lang/noir/commit/9808525b4f420397ea30ba91293ffb6539668e3f"
        },
        "date": 1761837917198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "084629edea663fc6813478488e5bb2cfa9ee73a2",
          "message": "fix: slice push_back when length is not known (#10206)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-30T16:51:43Z",
          "tree_id": "b41b0748cf8e4029a4a5b1b5037d9f3e9efe41d2",
          "url": "https://github.com/noir-lang/noir/commit/084629edea663fc6813478488e5bb2cfa9ee73a2"
        },
        "date": 1761845042190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "53da0afe66ed9946c6dc071f7e24b2d937b46092",
          "message": "fix: remove leading stars from block doc comments (#10316)",
          "timestamp": "2025-10-30T19:17:15Z",
          "tree_id": "a4c1c39843cb15f5cd0530ebaef48ff8de12601e",
          "url": "https://github.com/noir-lang/noir/commit/53da0afe66ed9946c6dc071f7e24b2d937b46092"
        },
        "date": 1761854569910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "8b0f07f6876846407daaada493b94229c122d204",
          "message": "chore(frontend): Quoting/unquoting roundtrip testing  (#10327)",
          "timestamp": "2025-10-30T20:23:12Z",
          "tree_id": "4e46b835300c5bfdbfddf4667810acdbf8b6b83f",
          "url": "https://github.com/noir-lang/noir/commit/8b0f07f6876846407daaada493b94229c122d204"
        },
        "date": 1761858026579,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "577cbdc068800166f543e350d2f5d0dca75a1292",
          "message": "fix(print): Convert `HirType::Function` into `PrintableType::Tuple`  (#10189)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T10:12:01Z",
          "tree_id": "1c4aa0b15fc03fa94a7ed1f725e6a3c1073ea2ee",
          "url": "https://github.com/noir-lang/noir/commit/577cbdc068800166f543e350d2f5d0dca75a1292"
        },
        "date": 1761907531052,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae",
          "message": "chore: add unit tests to show some features of the analysis (#10286)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-31T15:21:14Z",
          "tree_id": "070c436573f622dbc5a3e1eb9c45251b70b30401",
          "url": "https://github.com/noir-lang/noir/commit/4ae0fbdc473ae9d66823c9b4049b8fc71a9518ae"
        },
        "date": 1761926484597,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "1324e732b92ee9624307ee226ffeed01610287a6",
          "message": "chore: green light for basic_conditional audit (#10134)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-31T16:16:32Z",
          "tree_id": "9b8badeeed4fc0c3dde31d0b8df5ad1ed491d742",
          "url": "https://github.com/noir-lang/noir/commit/1324e732b92ee9624307ee226ffeed01610287a6"
        },
        "date": 1761929447621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "303dd2130d39aaee03b5d2bdf8af75f36eb39796",
          "message": "feat: Add `#[must_use]` attribute to promote unused warning to an error (#10313)",
          "timestamp": "2025-10-31T20:30:59Z",
          "tree_id": "19c201694ca4b5f4b77ccd62b60a75399329fe69",
          "url": "https://github.com/noir-lang/noir/commit/303dd2130d39aaee03b5d2bdf8af75f36eb39796"
        },
        "date": 1761944823487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "4a19333381f35b7381e4f7c6d490c2cd66ca8726",
          "message": "chore: update directory name (#10348)",
          "timestamp": "2025-11-03T09:39:02Z",
          "tree_id": "a627fbdc8f6c3c42bdd077555f337445f33981e3",
          "url": "https://github.com/noir-lang/noir/commit/4a19333381f35b7381e4f7c6d490c2cd66ca8726"
        },
        "date": 1762164286813,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962003,
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
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "9b0813041eae25642d2e4629625e53578ea3a9f8",
          "message": "fix: do not simplify call-data values (#10032)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-11-03T09:41:50Z",
          "tree_id": "6bd34f077becae9a935f8c77a8f3739d4dae0259",
          "url": "https://github.com/noir-lang/noir/commit/9b0813041eae25642d2e4629625e53578ea3a9f8"
        },
        "date": 1762165827710,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14544,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 70414,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11680,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1348,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1033,
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
            "value": 962068,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 963443,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2630,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263892,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245167,
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
          "id": "f74bd7c1212c548d3e63ea83ceff20ea6740d2dc",
          "message": "chore(frontend): Comptime item generation unit tests (#10319)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-11-03T12:20:44Z",
          "tree_id": "021c4e04892adf0a0c5a33eedd2306203732afda",
          "url": "https://github.com/noir-lang/noir/commit/f74bd7c1212c548d3e63ea83ceff20ea6740d2dc"
        },
        "date": 1762174477200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 18027,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 80804,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 17176,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1098,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 873,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2249,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2133,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1919757,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1921130,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2608,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 278025,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 262435,
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